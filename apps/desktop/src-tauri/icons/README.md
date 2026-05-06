# KeePassEx Desktop Icons

Place the following icon files in this directory before building:

| File | Size | Platform |
|------|------|----------|
| `32x32.png` | 32×32 | Linux |
| `128x128.png` | 128×128 | Linux, Windows |
| `128x128@2x.png` | 256×256 | macOS Retina |
| `icon.icns` | Multi-size | macOS |
| `icon.ico` | Multi-size | Windows |
| `icon.png` | 512×512 | System tray |

## Generating Icons

Use the Tauri CLI to generate icons from a single source image:

```bash
# From the apps/desktop directory:
pnpm tauri icon path/to/icon-1024x1024.png
```

This will generate all required sizes automatically.

## Design Guidelines

- Use the KeePassEx lock/key logo
- Primary color: #2563EB (blue)
- Background: transparent (PNG) or white (ICO)
- Minimum size for source image: 1024×1024 pixels
