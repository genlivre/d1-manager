#!/bin/bash
# Build macOS app bundle for D1 Manager

set -e

APP_NAME="D1 Manager"
BUNDLE_ID="com.d1manager.app"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo "Building D1 Manager v${VERSION} for macOS..."

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
else
    TARGET="x86_64-apple-darwin"
fi

echo "Target: ${TARGET}"

# Build release
cargo build --release --target ${TARGET}

# Create app bundle structure
BUNDLE_DIR="target/${TARGET}/release/${APP_NAME}.app"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp "target/${TARGET}/release/d1-manager" "${MACOS_DIR}/"

# Copy Info.plist and update version
sed "s/1.0.0/${VERSION}/g" resources/macos/Info.plist > "${CONTENTS_DIR}/Info.plist"

# Copy icon if exists
if [ -f "resources/macos/icon.icns" ]; then
    cp "resources/macos/icon.icns" "${RESOURCES_DIR}/"
fi

echo "App bundle created at: ${BUNDLE_DIR}"

# Create DMG (optional, requires create-dmg)
if command -v create-dmg &> /dev/null; then
    echo "Creating DMG..."
    DMG_NAME="D1-Manager-${VERSION}-${ARCH}.dmg"
    create-dmg \
        --volname "D1 Manager" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --app-drop-link 450 185 \
        "target/${TARGET}/release/${DMG_NAME}" \
        "${BUNDLE_DIR}"
    echo "DMG created at: target/${TARGET}/release/${DMG_NAME}"
else
    # Create zip instead
    echo "Creating zip archive..."
    cd "target/${TARGET}/release"
    ZIP_NAME="d1-manager-${VERSION}-${TARGET}.zip"
    zip -r "${ZIP_NAME}" "${APP_NAME}.app"
    echo "Zip created at: target/${TARGET}/release/${ZIP_NAME}"
fi

echo "Build complete!"
