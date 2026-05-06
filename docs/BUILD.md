# KeePassEx — Build Guide

## Prerequisites

### All Platforms
- **Rust** 1.75+ — `rustup install stable`
- **Node.js** 20+ — https://nodejs.org
- **pnpm** 9+ — `npm install -g pnpm`

### Desktop (Tauri)
- **Windows**: Visual Studio Build Tools 2022, WebView2
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Mobile (React Native)
- **iOS**: macOS + Xcode 15+ + CocoaPods (`gem install cocoapods`)
- **Android**: Android Studio + NDK 26 + JDK 17

### CLI
- Rust toolchain only

---

## Quick Start

```bash
# Clone
git clone https://github.com/keepassex/keepassex
cd keepassex

# Install JS dependencies
pnpm install

# Build Rust core
cargo build -p keepassex-core

# Run desktop (dev)
pnpm desktop

# Run mobile (dev)
pnpm mobile

# Run CLI
pnpm cli -- --help
```

---

## Desktop Build

```bash
# Development
pnpm --filter @keepassex/desktop tauri:dev

# Production build
pnpm --filter @keepassex/desktop tauri:build

# Output locations:
# Windows: apps/desktop/src-tauri/target/release/bundle/msi/
# macOS:   apps/desktop/src-tauri/target/release/bundle/dmg/
# Linux:   apps/desktop/src-tauri/target/release/bundle/deb/
#          apps/desktop/src-tauri/target/release/bundle/appimage/
```

---

## Mobile Build

### iOS

```bash
# Install CocoaPods dependencies
cd apps/mobile/ios && pod install && cd ../../..

# Run on simulator
pnpm --filter @keepassex/mobile ios

# Build for release
pnpm --filter @keepassex/mobile build:ios
```

### Android

```bash
# Run on emulator/device
pnpm --filter @keepassex/mobile android

# Build release APK
pnpm --filter @keepassex/mobile build:android

# Output: apps/mobile/android/app/build/outputs/apk/release/
```

---

## CLI Build

```bash
# Debug build
cargo build -p keepassex-cli

# Release build
cargo build --release -p keepassex-cli

# Cross-compile for all platforms
cargo build --release -p keepassex-cli --target x86_64-unknown-linux-gnu
cargo build --release -p keepassex-cli --target x86_64-apple-darwin
cargo build --release -p keepassex-cli --target aarch64-apple-darwin
cargo build --release -p keepassex-cli --target x86_64-pc-windows-msvc
```

---

## Browser Extension Build

```bash
# Chrome/Edge (Manifest V3)
pnpm --filter @keepassex/browser-extension build:chrome

# Firefox (Manifest V2)
pnpm --filter @keepassex/browser-extension build:firefox

# Output: apps/browser-extension/dist/
```

---

## Watch Apps

### watchOS
Open `apps/watch/watchos/KeePassExWatch.xcodeproj` in Xcode and build.

### WearOS
Open `apps/watch/wearos/` in Android Studio and build.

---

## Testing

```bash
# Rust tests
cargo test --all --all-features

# TypeScript tests
pnpm test:ts

# All tests
pnpm test:rust && pnpm test:ts

# With coverage
pnpm test:ts -- --coverage
```

---

## Linting & Formatting

```bash
# Rust
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# TypeScript
pnpm lint
pnpm --write .  # format with prettier

# All checks
pnpm check
```

---

## Environment Variables

```bash
# CLI
KPX_VAULT=/path/to/vault.kdbx
KPX_PASSWORD=your-master-password  # Use with caution
KPX_KEY_FILE=/path/to/keyfile.keyx

# Desktop (Tauri)
TAURI_SIGNING_PRIVATE_KEY=...
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=...

# macOS signing
APPLE_CERTIFICATE=...
APPLE_CERTIFICATE_PASSWORD=...
APPLE_SIGNING_IDENTITY=...
APPLE_ID=...
APPLE_PASSWORD=...
APPLE_TEAM_ID=...
```
