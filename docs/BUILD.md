# KeePassEx — Build Guide (v0.1.0)

This guide covers building KeePassEx v0.1.0 from source on Windows, macOS, and Linux.

---

## Prerequisites

### All Platforms

- **Node.js** ≥ 20.0.0 — [Download](https://nodejs.org/)
- **pnpm** ≥ 9.0.0 — `npm install -g pnpm@9`
- **Rust** ≥ 1.82 — [Install via rustup](https://rustup.rs/)
- **Git** — [Download](https://git-scm.com/)

### Platform-Specific

#### Windows

- **Visual Studio 2022** with "Desktop development with C++" workload
- **WebView2 Runtime** (usually pre-installed on Windows 11)

#### macOS

- **Xcode Command Line Tools** — `xcode-select --install`
- **Homebrew** (optional, for dependencies) — [Install](https://brew.sh/)

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.0-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsqlite3-dev
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install -y \
  gcc \
  gcc-c++ \
  openssl-devel \
  gtk3-devel \
  webkit2gtk4.0-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  sqlite-devel
```

---

## Clone Repository

```bash
git clone https://github.com/keepassex/keepassex.git
cd keepassex
```

---

## Install Dependencies

```bash
make install
```

This runs:

- `pnpm install` — installs all JavaScript dependencies
- `cargo build --release -p keepassex-core` — builds Rust core library

**Expected output:**

```
✓ pnpm install completed
✓ Rust core built successfully
```

---

## Build Desktop App

### Development Mode

```bash
make desktop
```

This starts:

- Vite dev server on `http://localhost:1420`
- Tauri dev window with hot-reload

**Expected output:**

```
VITE v6.0.7  ready in 1234 ms
➜  Local:   http://localhost:1420/
➜  Network: use --host to expose

Tauri window opened
```

### Production Build

```bash
make build-desktop
```

**Output locations:**

- **Windows**: `apps/desktop/src-tauri/target/release/bundle/msi/KeePassEx_0.1.0_x64_en-US.msi`
- **macOS**: `apps/desktop/src-tauri/target/release/bundle/dmg/KeePassEx_0.1.0_x64.dmg`
- **Linux**: `apps/desktop/src-tauri/target/release/bundle/deb/keepassex_0.1.0_amd64.deb`

---

## Build CLI

```bash
make build-cli
```

**Output**: `apps/cli/target/release/kpx` (or `kpx.exe` on Windows)

**Test it:**

```bash
./apps/cli/target/release/kpx --version
# KeePassEx CLI 0.1.0
```

---

## Build Browser Extension

### Chrome/Edge

```bash
make build-extension-chrome
```

**Output**: `apps/browser-extension/dist-chrome/`

**Load in Chrome:**

1. Open `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select `apps/browser-extension/dist-chrome/`

### Firefox

```bash
make build-extension-firefox
```

**Output**: `apps/browser-extension/dist-firefox/`

**Load in Firefox:**

1. Open `about:debugging#/runtime/this-firefox`
2. Click "Load Temporary Add-on"
3. Select `apps/browser-extension/dist-firefox/manifest.json`

---

## Build Server

```bash
make build-server
```

**Output**: `apps/server/target/release/keepassex-server`

**Run it:**

```bash
./apps/server/target/release/keepassex-server --port 8080
```

**Docker:**

```bash
cd apps/server
docker build -t keepassex-server:0.1.0 .
docker run -p 8080:8080 -v $(pwd)/data:/data keepassex-server:0.1.0
```

---

## Run Tests

### All Tests

```bash
make test
```

**Expected output:**

```
Running Rust tests...
test result: ok. 709 passed; 0 failed; 0 ignored

Running TypeScript tests...
Test Files  4 passed (4)
     Tests  156 passed (156)
```

### Rust Only

```bash
make test-rust
```

### TypeScript Only

```bash
make test-ts
```

---

## Troubleshooting

### Windows: "WebView2 not found"

**Solution**: Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### macOS: "xcrun: error: invalid active developer path"

**Solution**: Install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Linux: "webkit2gtk-4.0 not found"

**Solution**: Install WebKit2GTK:

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.0-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel
```

### Rust: "linker `cc` not found"

**Solution**: Install build tools:

```bash
# Ubuntu/Debian
sudo apt install build-essential

# Fedora
sudo dnf install gcc gcc-c++

# macOS
xcode-select --install
```

### pnpm: "EACCES: permission denied"

**Solution**: Fix npm permissions:

```bash
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
export PATH=~/.npm-global/bin:$PATH
```

---

## Build Artifacts

After successful builds, you'll have:

```
keepassex/
├── apps/desktop/src-tauri/target/release/bundle/
│   ├── msi/KeePassEx_0.1.0_x64_en-US.msi       # Windows installer
│   ├── dmg/KeePassEx_0.1.0_x64.dmg             # macOS disk image
│   └── deb/keepassex_0.1.0_amd64.deb           # Linux Debian package
├── apps/cli/target/release/kpx                 # CLI binary
├── apps/server/target/release/keepassex-server # Server binary
├── apps/browser-extension/dist-chrome/         # Chrome extension
└── apps/browser-extension/dist-firefox/        # Firefox extension
```

---

## Next Steps

- **Desktop**: Run `make desktop` to start development
- **CLI**: Run `./apps/cli/target/release/kpx --help` to see commands
- **Server**: Run `./apps/server/target/release/keepassex-server --help` for options
- **Tests**: Run `make test` to verify everything works

---

## CI/CD

GitHub Actions workflows:

- `.github/workflows/ci.yml` — Runs tests on every push
- `.github/workflows/release.yml` — Builds release artifacts on tags

**Trigger a release:**

```bash
git tag v0.1.0
git push origin v0.1.0
```

---

## Support

- **Issues**: https://github.com/keepassex/keepassex/issues
- **Discussions**: https://github.com/keepassex/keepassex/discussions
- **Security**: security@keepassex.app
