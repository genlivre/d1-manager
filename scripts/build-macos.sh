#!/bin/bash
# Build macOS app bundle for D1 Manager (Universal Binary)

set -e

APP_NAME="D1 Manager"
BUNDLE_ID="com.i-kikata.d1-manager"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo "Building D1 Manager v${VERSION} for macOS (Universal Binary)..."

# Build for both architectures
echo "Building for arm64..."
cargo build --release --target aarch64-apple-darwin

echo "Building for x86_64..."
cargo build --release --target x86_64-apple-darwin

# Create universal binary
echo "Creating universal binary..."
UNIVERSAL_DIR="target/universal-apple-darwin/release"
mkdir -p "${UNIVERSAL_DIR}"

lipo -create \
    "target/aarch64-apple-darwin/release/d1-manager" \
    "target/x86_64-apple-darwin/release/d1-manager" \
    -output "${UNIVERSAL_DIR}/d1-manager"

echo "Universal binary created: $(file ${UNIVERSAL_DIR}/d1-manager)"

# Create app bundle structure
BUNDLE_DIR="${UNIVERSAL_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp "${UNIVERSAL_DIR}/d1-manager" "${MACOS_DIR}/"

# Copy Info.plist and update version
sed "s/1.0.0/${VERSION}/g" resources/macos/Info.plist > "${CONTENTS_DIR}/Info.plist"

# Copy icon if exists
if [ -f "resources/macos/icon.icns" ]; then
    cp "resources/macos/icon.icns" "${RESOURCES_DIR}/"
fi

echo "App bundle created at: ${BUNDLE_DIR}"

# Code signing (optional - requires signing identity)
if [ -n "$SIGNING_IDENTITY" ]; then
    echo "Code signing with: ${SIGNING_IDENTITY}"
    codesign --force --options runtime --timestamp \
        --entitlements resources/macos/App.entitlements \
        --sign "${SIGNING_IDENTITY}" \
        "${BUNDLE_DIR}"
    echo "Code signing complete"
fi

# Create PKG for App Store (optional - requires installer identity)
if [ -n "$INSTALLER_IDENTITY" ]; then
    echo "Creating PKG..."
    PKG_NAME="D1Manager-${VERSION}.pkg"
    productbuild --component "${BUNDLE_DIR}" /Applications \
        --sign "${INSTALLER_IDENTITY}" \
        "${UNIVERSAL_DIR}/${PKG_NAME}"
    echo "PKG created at: ${UNIVERSAL_DIR}/${PKG_NAME}"
fi

# Create DMG (optional, requires create-dmg)
if command -v create-dmg &> /dev/null && [ -z "$INSTALLER_IDENTITY" ]; then
    echo "Creating DMG..."
    DMG_NAME="D1-Manager-${VERSION}-universal.dmg"
    create-dmg \
        --volname "D1 Manager" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --app-drop-link 450 185 \
        "${UNIVERSAL_DIR}/${DMG_NAME}" \
        "${BUNDLE_DIR}"
    echo "DMG created at: ${UNIVERSAL_DIR}/${DMG_NAME}"
else
    # Create zip instead
    echo "Creating zip archive..."
    cd "${UNIVERSAL_DIR}"
    ZIP_NAME="d1-manager-${VERSION}-universal.zip"
    zip -r "${ZIP_NAME}" "${APP_NAME}.app"
    echo "Zip created at: ${UNIVERSAL_DIR}/${ZIP_NAME}"
fi

echo ""
echo "Build complete!"
echo ""
echo "For App Store submission, run with signing identities:"
echo "  SIGNING_IDENTITY='3rd Party Mac Developer Application: ...' \\"
echo "  INSTALLER_IDENTITY='3rd Party Mac Developer Installer: ...' \\"
echo "  ./scripts/build-macos.sh"
