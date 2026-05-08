"""
Generate KeePassEx app icons for Tauri build.
Creates minimal valid PNG files at required sizes.
Run: python scripts/gen_icons.py
"""
import struct
import zlib
import os

ICONS_DIR = os.path.join(os.path.dirname(__file__), '..', 'apps', 'desktop', 'src-tauri', 'icons')
os.makedirs(ICONS_DIR, exist_ok=True)

# KeePassEx brand color: #2563EB (blue)
# Lock icon rendered as simple pixel art

def make_png(width: int, height: int, color_rgb=(37, 99, 235)) -> bytes:
    """Create a minimal valid PNG with a solid color + simple lock shape."""
    r, g, b = color_rgb

    # Build RGBA pixel data
    pixels = []
    cx, cy = width // 2, height // 2
    lock_w = max(2, width // 3)
    lock_h = max(2, height // 3)
    shackle_r = lock_w // 2

    for y in range(height):
        row = []
        for x in range(width):
            dx = x - cx
            dy = y - cy

            # Background gradient (dark blue)
            bg_r = max(0, r - 40)
            bg_g = max(0, g - 40)
            bg_b = min(255, b + 20)

            # Lock body (white rectangle in center-bottom)
            in_body = (
                abs(dx) <= lock_w // 2 and
                dy >= -lock_h // 4 and
                dy <= lock_h // 2
            )

            # Shackle (white arc on top)
            dist = (dx * dx + (dy + lock_h // 4) * (dy + lock_h // 4)) ** 0.5
            in_shackle = (
                shackle_r - 1 <= dist <= shackle_r + 1 and
                dy <= -lock_h // 4
            )

            if in_body or in_shackle:
                row.extend([255, 255, 255, 255])  # white, opaque
            else:
                row.extend([bg_r, bg_g, bg_b, 255])  # brand color, opaque

        pixels.append(bytes(row))

    # PNG encoding
    def png_chunk(chunk_type: bytes, data: bytes) -> bytes:
        c = chunk_type + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xFFFFFFFF)

    # IHDR
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)  # 8-bit RGB... wait, RGBA=6
    ihdr_data = struct.pack('>II', width, height) + bytes([8, 6, 0, 0, 0])  # RGBA

    # IDAT
    raw = b''
    for row in pixels:
        raw += b'\x00' + row  # filter type 0 (None) per row
    compressed = zlib.compress(raw, 9)

    png = b'\x89PNG\r\n\x1a\n'
    png += png_chunk(b'IHDR', ihdr_data)
    png += png_chunk(b'IDAT', compressed)
    png += png_chunk(b'IEND', b'')
    return png


def make_ico(sizes=((16,16),(32,32),(48,48),(256,256))) -> bytes:
    """Create a minimal ICO file containing multiple PNG images."""
    images = []
    for w, h in sizes:
        png_data = make_png(w, h)
        images.append((w, h, png_data))

    # ICO header
    header = struct.pack('<HHH', 0, 1, len(images))  # reserved, type=1 (ICO), count

    # Directory entries (each 16 bytes)
    offset = 6 + len(images) * 16
    directory = b''
    for w, h, data in images:
        size = len(data)
        directory += struct.pack('<BBBBHHII',
            w if w < 256 else 0,   # width (0 = 256)
            h if h < 256 else 0,   # height (0 = 256)
            0,    # color count
            0,    # reserved
            1,    # planes
            32,   # bit count
            size, # size of image data
            offset
        )
        offset += size

    ico = header + directory
    for _, _, data in images:
        ico += data
    return ico


def make_icns() -> bytes:
    """Create a minimal ICNS file (macOS icon)."""
    # ICNS format: 'icns' magic + total size + icon entries
    # We'll embed a 512x512 PNG as 'ic09' (512x512 PNG)
    png_512 = make_png(512, 512)
    png_1024 = make_png(1024, 1024)

    def icns_entry(type_code: bytes, data: bytes) -> bytes:
        return type_code + struct.pack('>I', len(data) + 8) + data

    entries = icns_entry(b'ic09', png_512)   # 512x512
    entries += icns_entry(b'ic10', png_1024) # 1024x1024 (Retina)

    total_size = 8 + len(entries)
    return b'icns' + struct.pack('>I', total_size) + entries


# Generate all required Tauri icon files
print("Generating KeePassEx icons...")

# PNG icons
for size in [32, 128]:
    path = os.path.join(ICONS_DIR, f'{size}x{size}.png')
    with open(path, 'wb') as f:
        f.write(make_png(size, size))
    print(f"  ✓ {size}x{size}.png")

# 128x128@2x.png (256x256 content)
path = os.path.join(ICONS_DIR, '128x128@2x.png')
with open(path, 'wb') as f:
    f.write(make_png(256, 256))
print("  ✓ 128x128@2x.png")

# icon.png (512x512)
path = os.path.join(ICONS_DIR, 'icon.png')
with open(path, 'wb') as f:
    f.write(make_png(512, 512))
print("  ✓ icon.png")

# icon.ico (Windows)
path = os.path.join(ICONS_DIR, 'icon.ico')
with open(path, 'wb') as f:
    f.write(make_ico())
print("  ✓ icon.ico")

# icon.icns (macOS)
path = os.path.join(ICONS_DIR, 'icon.icns')
with open(path, 'wb') as f:
    f.write(make_icns())
print("  ✓ icon.icns")

print(f"\nAll icons generated in: {os.path.abspath(ICONS_DIR)}")
print("Run 'cargo check -p keepassex-desktop' to verify build.")
