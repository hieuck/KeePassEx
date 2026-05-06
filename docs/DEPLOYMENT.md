# KeePassEx — Deployment Guide

## Desktop (Tauri)

### Windows

**Requirements:**
- Windows 10+ (64-bit)
- WebView2 Runtime (bundled in installer)

**Build:**
```bash
pnpm --filter @keepassex/desktop tauri:build
# Output: apps/desktop/src-tauri/target/release/bundle/msi/KeePassEx_1.0.0_x64_en-US.msi
```

**Code Signing:**
```bash
# Set environment variables
export TAURI_SIGNING_PRIVATE_KEY="..."
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="..."

pnpm --filter @keepassex/desktop tauri:build
```

**Distribution:**
- Microsoft Store (MSIX package)
- Direct download (MSI installer)
- Winget: `winget install KeePassEx.KeePassEx`

---

### macOS

**Requirements:**
- macOS 12.0+ (Monterey)
- Apple Developer account for distribution

**Build:**
```bash
# Intel
pnpm --filter @keepassex/desktop tauri:build --target x86_64-apple-darwin

# Apple Silicon
pnpm --filter @keepassex/desktop tauri:build --target aarch64-apple-darwin

# Universal binary
pnpm --filter @keepassex/desktop tauri:build --target universal-apple-darwin
```

**Code Signing & Notarization:**
```bash
export APPLE_CERTIFICATE="..."
export APPLE_CERTIFICATE_PASSWORD="..."
export APPLE_SIGNING_IDENTITY="Developer ID Application: KeePassEx Team (TEAMID)"
export APPLE_ID="developer@keepassex.app"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="YOURTEAMID"

pnpm --filter @keepassex/desktop tauri:build
```

**Distribution:**
- Mac App Store (requires additional entitlements)
- Direct download (DMG)
- Homebrew: `brew install --cask keepassex`

---

### Linux

**Requirements:**
- Ubuntu 20.04+ / Debian 11+ / Fedora 35+
- libwebkit2gtk-4.1

**Build:**
```bash
# DEB package (Ubuntu/Debian)
pnpm --filter @keepassex/desktop tauri:build
# Output: apps/desktop/src-tauri/target/release/bundle/deb/keepassex_1.0.0_amd64.deb

# RPM package (Fedora/RHEL)
# Output: apps/desktop/src-tauri/target/release/bundle/rpm/keepassex-1.0.0-1.x86_64.rpm

# AppImage (universal)
# Output: apps/desktop/src-tauri/target/release/bundle/appimage/keepassex_1.0.0_amd64.AppImage
```

**Distribution:**
- Flathub: `flatpak install flathub app.keepassex.KeePassEx`
- Snap Store: `snap install keepassex`
- AUR (Arch): `yay -S keepassex`
- Direct download (DEB, RPM, AppImage)

---

## Mobile

### iOS App Store

**Requirements:**
- macOS with Xcode 15+
- Apple Developer Program membership ($99/year)
- App Store Connect account

**Build:**
```bash
cd apps/mobile/ios
pod install

# Build for release
cd ..
npx react-native build-ios --mode Release
```

**Xcode Steps:**
1. Open `apps/mobile/ios/KeePassEx.xcworkspace`
2. Select "Any iOS Device" as target
3. Product → Archive
4. Distribute App → App Store Connect

**App Store Requirements:**
- Privacy policy URL
- App screenshots (6.7", 6.1", 5.5", iPad)
- App description (EN + VI)
- Keywords
- Age rating: 4+
- Category: Utilities

**AutoFill Extension:**
The AutoFill extension is bundled with the main app. No separate submission needed.

**Widget:**
The WidgetKit extension is bundled with the main app.

---

### Google Play Store

**Requirements:**
- Google Play Developer account ($25 one-time)
- Android Studio for signing

**Build:**
```bash
cd apps/mobile/android

# Generate release keystore (first time only)
keytool -genkey -v -keystore keepassex-release.jks \
  -alias keepassex -keyalg RSA -keysize 4096 -validity 10000

# Build release APK
./gradlew assembleRelease

# Build release AAB (recommended for Play Store)
./gradlew bundleRelease
```

**Signing:**
```bash
# gradle.properties
KEEPASSEX_UPLOAD_STORE_FILE=keepassex-release.jks
KEEPASSEX_UPLOAD_STORE_PASSWORD=your-store-password
KEEPASSEX_UPLOAD_KEY_ALIAS=keepassex
KEEPASSEX_UPLOAD_KEY_PASSWORD=your-key-password
```

**Play Store Requirements:**
- Privacy policy URL
- Screenshots (phone, 7" tablet, 10" tablet)
- Feature graphic (1024×500)
- App description (EN + VI)
- Content rating questionnaire
- Target API level: 34+

**AutoFill Service:**
Automatically available after installation. Users enable via:
Settings → System → Languages & Input → Autofill service → KeePassEx

---

## Watch Apps

### watchOS (Apple Watch)

The watchOS app is bundled with the iOS app. No separate submission.

**Requirements:**
- Paired iPhone with KeePassEx installed
- watchOS 8.0+

---

### WearOS

**Build:**
```bash
cd apps/watch/wearos
./gradlew assembleRelease
```

**Distribution:**
- Google Play Store (separate listing or companion app)
- Requires Wear OS 2.0+

---

## Browser Extension

### Chrome Web Store

**Build:**
```bash
pnpm --filter @keepassex/browser-extension build:chrome
# Output: apps/browser-extension/dist/
```

**Package:**
```bash
cd apps/browser-extension/dist
zip -r keepassex-chrome-1.0.0.zip .
```

**Submission:**
1. Chrome Web Store Developer Dashboard
2. Upload ZIP file
3. Fill in store listing (EN + VI descriptions)
4. Submit for review (1-3 business days)

**Requirements:**
- $5 one-time developer registration
- Privacy policy
- Screenshots (1280×800 or 640×400)
- Promotional tile (440×280)

---

### Firefox Add-ons (AMO)

**Build:**
```bash
pnpm --filter @keepassex/browser-extension build:firefox
```

**Submission:**
1. addons.mozilla.org Developer Hub
2. Upload XPI file
3. Source code submission (required for review)
4. Submit for review (1-7 business days)

---

## CLI

### GitHub Releases

CLI binaries are automatically built and published by the release workflow:

```yaml
# .github/workflows/release.yml
# Builds for:
# - x86_64-unknown-linux-gnu (Linux x64)
# - aarch64-unknown-linux-gnu (Linux ARM64)
# - x86_64-apple-darwin (macOS Intel)
# - aarch64-apple-darwin (macOS Apple Silicon)
# - x86_64-pc-windows-msvc (Windows x64)
```

**Manual release:**
```bash
# Tag a release
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will automatically:
# 1. Build all platform binaries
# 2. Create GitHub Release
# 3. Upload binaries as release assets
```

### Package Managers

**Homebrew (macOS/Linux):**
```ruby
# Formula: Formula/keepassex.rb
class Keepassex < Formula
  desc "Native cross-platform password manager"
  homepage "https://keepassex.app"
  url "https://github.com/keepassex/keepassex/releases/download/v1.0.0/kpx-macos-arm64"
  sha256 "..."
  version "1.0.0"

  def install
    bin.install "kpx-macos-arm64" => "kpx"
  end
end
```

**Scoop (Windows):**
```json
{
  "version": "1.0.0",
  "description": "KeePassEx CLI",
  "homepage": "https://keepassex.app",
  "url": "https://github.com/keepassex/keepassex/releases/download/v1.0.0/kpx-windows-x64.exe",
  "bin": "kpx-windows-x64.exe"
}
```

---

## Auto-Updates

KeePassEx uses Tauri's built-in updater for desktop apps.

**Update server:**
```
https://releases.keepassex.app/{target}/{arch}/latest.json
```

**Update manifest format:**
```json
{
  "version": "1.1.0",
  "notes": "Bug fixes and improvements",
  "pub_date": "2025-06-01T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://releases.keepassex.app/darwin-aarch64/KeePassEx_1.1.0_aarch64.dmg"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://releases.keepassex.app/windows-x86_64/KeePassEx_1.1.0_x64_en-US.msi"
    }
  }
}
```

---

## Version Numbering

KeePassEx follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes (new KDBX format, incompatible API)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes and security patches

**Release cadence:**
- Patch releases: As needed (security fixes within 48 hours)
- Minor releases: Monthly
- Major releases: Annually
