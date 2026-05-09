"""
Create a minimal KDBX 4.x vault for testing.
Uses only Python standard library + hashlib + hmac.
Password: test123
"""
import struct
import hashlib
import hmac as hmac_lib
import os
import zlib

# ─── Constants ────────────────────────────────────────────────────────────────
KDBX_SIG1 = 0x9AA2D903
KDBX_SIG2 = 0xB54BFB67
KDBX_VERSION_4_1 = 0x00040001

# Cipher UUIDs
AES256_CBC_UUID = bytes.fromhex('31C1F2E6BF714350BE5805216AFC5AFF')
CHACHA20_UUID   = bytes.fromhex('D6038A2B8B6F4CB5A524339A31DBB59A')

# KDF UUIDs
ARGON2ID_UUID = bytes.fromhex('EF636DDF8C29444B91F7A9A403E30A0C')

# ─── Argon2id KDF ─────────────────────────────────────────────────────────────
def argon2id_derive(password_hash: bytes, salt: bytes, iterations: int, memory_kb: int, parallelism: int) -> bytes:
    """Simple Argon2id using argon2-cffi if available, else fallback."""
    try:
        from argon2.low_level import hash_secret_raw, Type
        return hash_secret_raw(
            secret=password_hash,
            salt=salt,
            time_cost=iterations,
            memory_cost=memory_kb,
            parallelism=parallelism,
            hash_len=32,
            type=Type.ID,
        )
    except ImportError:
        # Fallback: use PBKDF2 (not real Argon2, just for structure testing)
        print("WARNING: argon2-cffi not available, using PBKDF2 fallback")
        return hashlib.pbkdf2_hmac('sha256', password_hash, salt, iterations * 1000, 32)

# ─── Key derivation ───────────────────────────────────────────────────────────
def composite_key(password: str) -> bytes:
    """SHA256(SHA256(password))"""
    return hashlib.sha256(hashlib.sha256(password.encode('utf-8')).digest()).digest()

def derive_keys(master_seed: bytes, transformed_key: bytes):
    """
    KDBX 4.x:
    enc_key  = SHA256(master_seed || transformed_key || 0x01)
    hmac_key = SHA512(master_seed || transformed_key || 0x01)
    """
    enc_key = hashlib.sha256(master_seed + transformed_key + b'\x01').digest()
    hmac_key = hashlib.sha512(master_seed + transformed_key + b'\x01').digest()
    return enc_key, hmac_key

def get_hmac_key(block_index: int, hmac_key: bytes) -> bytes:
    """Block HMAC key = SHA512(block_index_le64 || hmac_key) — full 64 bytes"""
    return hashlib.sha512(struct.pack('<Q', block_index) + hmac_key).digest()

def compute_block_hmac(hmac_key: bytes, block_index: int, data: bytes) -> bytes:
    block_key = get_hmac_key(block_index, hmac_key)
    msg = struct.pack('<Q', block_index) + struct.pack('<I', len(data)) + data
    return hmac_lib.new(block_key, msg, hashlib.sha256).digest()

def compute_header_hmac(hmac_key: bytes, header_data: bytes) -> bytes:
    """Header HMAC key = SHA512(UINT64_MAX_le64 || hmac_key) — full 64 bytes"""
    header_key = hashlib.sha512(struct.pack('<Q', 0xFFFFFFFFFFFFFFFF) + hmac_key).digest()
    return hmac_lib.new(header_key, header_data, hashlib.sha256).digest()

# ─── Variant map (KDF params) ─────────────────────────────────────────────────
def write_variant_map(entries: list) -> bytes:
    """Write KDBX 4.x VariantMap"""
    buf = struct.pack('<H', 0x0100)  # version
    for type_id, key, value in entries:
        key_bytes = key.encode('utf-8')
        buf += bytes([type_id])
        buf += struct.pack('<I', len(key_bytes)) + key_bytes
        buf += struct.pack('<I', len(value)) + value
    buf += b'\x00'  # end marker
    return buf

# ─── Header field ─────────────────────────────────────────────────────────────
def header_field(field_id: int, data: bytes) -> bytes:
    return bytes([field_id]) + struct.pack('<I', len(data)) + data

# ─── AES-256-CBC encrypt ──────────────────────────────────────────────────────
def aes_cbc_encrypt(key: bytes, iv: bytes, data: bytes) -> bytes:
    from Crypto.Cipher import AES
    cipher = AES.new(key, AES.MODE_CBC, iv)
    # Pad to block size
    pad_len = 16 - (len(data) % 16)
    data += bytes([pad_len] * pad_len)
    return cipher.encrypt(data)

# ─── ChaCha20-Poly1305 encrypt ────────────────────────────────────────────────
def chacha20_encrypt(key: bytes, nonce: bytes, data: bytes) -> bytes:
    try:
        from Crypto.Cipher import ChaCha20_Poly1305
        cipher = ChaCha20_Poly1305.new(key=key, nonce=nonce)
        ct, tag = cipher.encrypt_and_digest(data)
        return ct + tag
    except ImportError:
        # Fallback: XOR with keystream (not real ChaCha20, just for structure)
        print("WARNING: pycryptodome not available")
        return data

# ─── Main ─────────────────────────────────────────────────────────────────────
def create_kdbx4(output_path: str, password: str):
    print(f"Creating KDBX 4.x vault: {output_path}")

    # Random values
    master_seed = os.urandom(32)
    encryption_iv = os.urandom(12)  # ChaCha20 nonce
    argon2_salt = os.urandom(32)
    inner_stream_key = os.urandom(64)

    # KDF params
    argon2_iterations = 2
    argon2_memory = 65536 * 1024  # 64 MB in bytes
    argon2_parallelism = 2

    # Build KDF variant map
    kdf_map = write_variant_map([
        (0x42, '$UUID', ARGON2ID_UUID),
        (0x42, 'S', argon2_salt),
        (0x05, 'I', struct.pack('<Q', argon2_iterations)),
        (0x05, 'M', struct.pack('<Q', argon2_memory)),
        (0x05, 'P', struct.pack('<Q', argon2_parallelism)),
        (0x04, 'V', struct.pack('<I', 19)),  # Argon2 version 1.3
    ])

    # Build header
    header_fields = b''
    header_fields += header_field(2, CHACHA20_UUID)           # CipherId
    header_fields += header_field(3, struct.pack('<I', 1))    # Compression: GZip
    header_fields += header_field(4, master_seed)             # MasterSeed
    header_fields += header_field(7, encryption_iv)           # EncryptionIV
    header_fields += header_field(11, kdf_map)                # KdfParameters
    header_fields += header_field(0, b'\r\n\r\n')             # EndOfHeader

    # Full header = sig + version + fields
    header_data = (
        struct.pack('<I', KDBX_SIG1) +
        struct.pack('<I', KDBX_SIG2) +
        struct.pack('<I', KDBX_VERSION_4_1) +
        header_fields
    )

    # Derive keys
    ck = composite_key(password)
    print(f"composite_key: {ck.hex()}")

    transformed_key = argon2id_derive(ck, argon2_salt, argon2_iterations, 65536, argon2_parallelism)
    print(f"transformed_key: {transformed_key.hex()}")

    enc_key, hmac_key = derive_keys(master_seed, transformed_key)
    print(f"enc_key: {enc_key.hex()}")
    print(f"hmac_key[:8]: {hmac_key[:8].hex()}")

    # Header checksums
    header_sha256 = hashlib.sha256(header_data).digest()
    header_hmac = compute_header_hmac(hmac_key, header_data)
    print(f"header_sha256: {header_sha256.hex()}")
    print(f"header_hmac: {header_hmac.hex()}")

    # Minimal XML payload
    xml = b'''<?xml version="1.0" encoding="utf-8"?>
<KeePassFile>
<Meta><Generator>KeePassEx-Test</Generator><DatabaseName>Test</DatabaseName></Meta>
<Root><Group><UUID>AAAAAAAAAAAAAAAAAAAAAA==</UUID><Name>Root</Name></Group></Root>
</KeePassFile>'''

    # Inner header
    inner_header = b''
    inner_header += bytes([1]) + struct.pack('<I', 4) + struct.pack('<I', 3)  # ChaCha20 stream
    inner_header += bytes([2]) + struct.pack('<I', len(inner_stream_key)) + inner_stream_key
    inner_header += bytes([0]) + struct.pack('<I', 0)  # EndOfHeader

    payload = inner_header + xml

    # Compress
    compressed = zlib.compress(payload, 6)
    # Add gzip header/footer manually
    import io
    buf = io.BytesIO()
    with __import__('gzip').GzipFile(fileobj=buf, mode='wb') as f:
        f.write(payload)
    gzipped = buf.getvalue()

    # Encrypt with ChaCha20-Poly1305
    try:
        from Crypto.Cipher import ChaCha20_Poly1305
        cipher = ChaCha20_Poly1305.new(key=enc_key, nonce=encryption_iv)
        ct, tag = cipher.encrypt_and_digest(gzipped)
        encrypted = ct + tag
    except ImportError:
        print("ERROR: pycryptodome required. Install: pip install pycryptodome")
        return

    # Build HMAC blocks
    block_size = 1024 * 1024
    blocks = b''
    block_index = 0
    for i in range(0, len(encrypted), block_size):
        chunk = encrypted[i:i+block_size]
        block_hmac = compute_block_hmac(hmac_key, block_index, chunk)
        blocks += block_hmac + struct.pack('<I', len(chunk)) + chunk
        block_index += 1
    # Terminal block
    terminal_hmac = compute_block_hmac(hmac_key, block_index, b'')
    blocks += terminal_hmac + struct.pack('<I', 0)

    # Assemble file
    output = header_data + header_sha256 + header_hmac + blocks

    with open(output_path, 'wb') as f:
        f.write(output)

    print(f"Created {output_path} ({len(output)} bytes)")
    print("Done!")

if __name__ == '__main__':
    create_kdbx4('packages/core/src/tests/test_kdbx4_py.kdbx', 'test123')
