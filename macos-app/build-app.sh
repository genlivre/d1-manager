#!/bin/bash
set -e

# Configuration
APP_NAME="D1 Manager"
BUNDLE_ID="com.i-kikata.d1-manager"
VERSION="1.0.0"
BUILD_NUMBER="4"
TEAM_ID="BLZ29BVB77"

# Paths
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/target/release"
APP_DIR="$SCRIPT_DIR/$APP_NAME.app"
BINARY_NAME="d1-manager"
UNIVERSAL_BINARY="d1-manager-universal"

echo "=== Building D1 Manager for App Store ==="
echo "Team ID: $TEAM_ID"
echo "Bundle ID: $BUNDLE_ID"
echo ""

# Step 1: Clean previous build
echo "[1/6] Cleaning previous build..."
rm -rf "$APP_DIR"
rm -f "$SCRIPT_DIR"/*.pkg

# Step 2: Create .app bundle structure
echo "[2/6] Creating .app bundle structure..."
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Step 3: Copy files
echo "[3/6] Copying files..."
cp "$BUILD_DIR/$UNIVERSAL_BINARY" "$APP_DIR/Contents/MacOS/$BINARY_NAME"
cp "$SCRIPT_DIR/Info.plist" "$APP_DIR/Contents/"

# Copy icon if exists
if [ -f "$SCRIPT_DIR/AppIcon.icns" ]; then
    cp "$SCRIPT_DIR/AppIcon.icns" "$APP_DIR/Contents/Resources/"
    echo "    Icon copied."
else
    echo "    WARNING: AppIcon.icns not found. Please create an icon."
    echo "    You can use: https://www.appicon.co/ to generate .icns from PNG"
fi

# Copy provisioning profile (required for TestFlight/App Store)
if [ -f "$SCRIPT_DIR/D1_Manager_App_Store.provisionprofile" ]; then
    cp "$SCRIPT_DIR/D1_Manager_App_Store.provisionprofile" "$APP_DIR/Contents/embedded.provisionprofile"
    echo "    Provisioning profile embedded."
else
    echo "    WARNING: Provisioning profile not found!"
fi

# Remove quarantine attributes (required for App Store)
echo "    Removing quarantine attributes..."
xattr -cr "$APP_DIR"

# Fix permissions (must be readable by all users)
echo "    Fixing permissions..."
chmod -R a+r "$APP_DIR"
chmod a+x "$APP_DIR/Contents/MacOS/$BINARY_NAME"
find "$APP_DIR" -type d -exec chmod a+x {} \;

# Step 4: Sign the app
echo "[4/6] Signing the application..."
echo "    Looking for signing identities..."

# List available signing identities
SIGNING_IDENTITY=$(security find-identity -v -p codesigning | grep "Apple Distribution: " | grep "$TEAM_ID" | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$SIGNING_IDENTITY" ]; then
    # Try 3rd Party Mac Developer Application
    SIGNING_IDENTITY=$(security find-identity -v -p codesigning | grep "3rd Party Mac Developer Application:" | grep "$TEAM_ID" | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

if [ -z "$SIGNING_IDENTITY" ]; then
    echo "    ERROR: No valid signing identity found for Team ID: $TEAM_ID"
    echo ""
    echo "    Available identities:"
    security find-identity -v -p codesigning
    echo ""
    echo "    Please install your Mac App Distribution certificate from:"
    echo "    https://developer.apple.com/account/resources/certificates/list"
    exit 1
fi

echo "    Using identity: $SIGNING_IDENTITY"

codesign --deep --force --verify --verbose \
    --sign "$SIGNING_IDENTITY" \
    --options runtime \
    --entitlements "$SCRIPT_DIR/entitlements.plist" \
    "$APP_DIR"

echo "    Signature verification:"
codesign --verify --verbose "$APP_DIR"

# Step 5: Create .pkg for App Store
echo "[5/6] Creating .pkg installer..."

INSTALLER_IDENTITY=$(security find-identity -v -p basic | grep "3rd Party Mac Developer Installer:" | grep "$TEAM_ID" | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$INSTALLER_IDENTITY" ]; then
    echo "    ERROR: No installer signing identity found for Team ID: $TEAM_ID"
    echo ""
    echo "    Please install your Mac Installer Distribution certificate from:"
    echo "    https://developer.apple.com/account/resources/certificates/list"
    exit 1
fi

echo "    Using installer identity: $INSTALLER_IDENTITY"

productbuild --component "$APP_DIR" /Applications \
    --sign "$INSTALLER_IDENTITY" \
    "$SCRIPT_DIR/D1Manager.pkg"

echo "[6/6] Build complete!"
echo ""
echo "=== Output ==="
echo "App:       $APP_DIR"
echo "Installer: $SCRIPT_DIR/D1Manager.pkg"
echo ""
echo "=== Next Steps ==="
echo "1. Open Transporter app (download from Mac App Store if needed)"
echo "2. Sign in with your Apple ID"
echo "3. Drag D1Manager.pkg into Transporter"
echo "4. Click 'Deliver' to upload to App Store Connect"
echo ""
echo "Or use command line:"
echo "xcrun altool --upload-app -f \"$SCRIPT_DIR/D1Manager.pkg\" -t macos -u YOUR_APPLE_ID -p APP_SPECIFIC_PASSWORD"
