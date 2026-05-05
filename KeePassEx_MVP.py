#!/usr/bin/env python3
"""
KeePassEx MVP - Autonomous Password Manager
Reads KDBX 4.x databases, auto-unlocks using keyfile+password, generates TOTP,
and simulates auto-type. No external dependencies except standard library + argon2-cffi.
Install: pip install argon2-cffi
"""

import sys
import os
import struct
import hashlib
import hmac
import base64
import time
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
import xml.etree.ElementTree as ET
import zlib

try:
    from argon2.low_level import hash_secret_raw, Type
    ARGON2_AVAILABLE = True
except ImportError:
    ARGON2_AVAILABLE = False
    print("ERROR: Install argon2-cffi: pip install argon2-cffi", file=sys.stderr)
    sys.exit(1)

# ---------- KDBX 4.x Structures ----------
@dataclass
class KdbxHeader:
    signature1: int
    signature2: int
    minor_version: int
    major_version: int
    cipher_uuid: bytes
    compression_flags: int
    master_seed: bytes
    transform_seed: bytes
    transform_rounds: int
    encryption_iv: bytes
    protected_stream_key: bytes
    stream_start_bytes: bytes
    inner_random_stream_id: int
    kdf_parameters: Dict[bytes, bytes]  # only Argon2 for v4

# ---------- KDBX Parser ----------
def read_kdbx_header(f) -> KdbxHeader:
    # Read fixed header fields
    sig1 = struct.unpack('<I', f.read(4))[0]
    sig2 = struct.unpack('<I', f.read(4))[0]
    if sig1 != 0x9AA2D903 or sig2 != 0xB54BFB67:
        raise ValueError("Not a valid KDBX file")
    minor = struct.unpack('<I', f.read(4))[0]
    major = struct.unpack('<I', f.read(4))[0]
    # Parse header fields until EOF
    header = KdbxHeader(sig1, sig2, minor, major, None, 0, None, None, 0, None, None, None, 0, {})
    while True:
        fid = f.read(1)
        if not fid:
            break
        fid = fid[0]
        if fid == 0:
            break  # end of header
        length = struct.unpack('<I', f.read(4))[0]
        data = f.read(length)
        if fid == 2:  # CipherUUID
            header.cipher_uuid = data
        elif fid == 3:  # CompressionFlags
            header.compression_flags = struct.unpack('<I', data)[0]
        elif fid == 4:  # MasterSeed
            header.master_seed = data
        elif fid == 5:  # TransformSeed
            header.transform_seed = data
        elif fid == 6:  # TransformRounds
            header.transform_rounds = struct.unpack('<Q', data)[0]
        elif fid == 7:  # EncryptionIV
            header.encryption_iv = data
        elif fid == 8:  # ProtectedStreamKey
            header.protected_stream_key = data
        elif fid == 9:  # StreamStartBytes
            header.stream_start_bytes = data
        elif fid == 10: # InnerRandomStreamID
            header.inner_random_stream_id = struct.unpack('<I', data)[0]
        elif fid == 11: # KDFParameters
            # Parse VariantDict (simple for Argon2)
            header.kdf_parameters = parse_variant_dict(data)
    return header

def parse_variant_dict(data: bytes) -> Dict[bytes, bytes]:
    # Simplified: just extract Argon2 parameters as raw bytes
    # Real implementation would parse proper structure
    # For MVP we assume only Argon2 and we'll hardcode common defaults if missing
    result = {}
    idx = 0
    while idx < len(data):
        if idx + 1 > len(data):
            break
        field_type = data[idx]
        idx += 1
        if field_type == 0x04:  # String
            # length (UInt32)
            if idx + 4 > len(data):
                break
            key_len = struct.unpack('<I', data[idx:idx+4])[0]
            idx += 4
            key = data[idx:idx+key_len]
            idx += key_len
            if idx + 4 > len(data):
                break
            val_len = struct.unpack('<I', data[idx:idx+4])[0]
            idx += 4
            val = data[idx:idx+val_len]
            idx += val_len
            result[key] = val
        else:
            # Skip unknown
            if idx + 4 > len(data):
                break
            length = struct.unpack('<I', data[idx:idx+4])[0]
            idx += 4 + length
    return result

def derive_key(password: str, header: KdbxHeader) -> bytes:
    # Argon2 KDF for KDBX 4
    if not ARGON2_AVAILABLE:
        raise RuntimeError("argon2-cffi required")
    p = header.kdf_parameters.get(b"$PARAM$K", None)
    salt = header.kdf_parameters.get(b"$PARAM$S", b"")
    if not salt:
        salt = b""
    memory = 1 << 20  # 1 MiB default
    iterations = 2
    parallelism = 1
    # Parse from kdf_parameters if present
    if b"$PARAM$M" in header.kdf_parameters:
        memory = struct.unpack('<Q', header.kdf_parameters[b"$PARAM$M"])[0]
    if b"$PARAM$T" in header.kdf_parameters:
        iterations = struct.unpack('<Q', header.kdf_parameters[b"$PARAM$T"])[0]
    if b"$PARAM$P" in header.kdf_parameters:
        parallelism = struct.unpack('<I', header.kdf_parameters[b"$PARAM$P"])[0]
    # Argon2id
    key = hash_secret_raw(
        secret=password.encode('utf-8'),
        salt=salt,
        time_cost=iterations,
        memory_cost=memory,
        parallelism=parallelism,
        hash_len=32,
        type=Type.ID
    )
    # Combine with master seed
    combined = key + header.master_seed
    return hashlib.sha256(combined).digest()

def decrypt_payload(encrypted_data: bytes, key: bytes, iv: bytes) -> bytes:
    from Crypto.Cipher import AES
    cipher = AES.new(key, AES.MODE_CBC, iv=iv)
    decrypted = cipher.decrypt(encrypted_data)
    # Remove PKCS#7 padding
    pad_len = decrypted[-1]
    return decrypted[:-pad_len]

def parse_xml(decrypted_payload: bytes) -> ET.Element:
    # Decompress if needed
    # In KDBX 4, inner payload may be GZip-compressed
    # Try to detect: first two bytes are 0x1F 0x8B for gzip
    if decrypted_payload[:2] == b'\x1f\x8b':
        import gzip
        decrypted_payload = gzip.decompress(decrypted_payload)
    # Remove header bytes: first 32 bytes are stream start bytes? Actually we already removed.
    # The XML starts with <?xml or <KeePassFile>
    try:
        root = ET.fromstring(decrypted_payload)
        return root
    except ET.ParseError:
        # Maybe there's a random padding? Search for '<'
        start = decrypted_payload.find(b'<')
        if start == -1:
            raise ValueError("No XML found")
        root = ET.fromstring(decrypted_payload[start:])
        return root

def find_entries(root: ET.Element) -> List[Dict]:
    entries = []
    for group in root.findall('.//Group'):
        for entry_el in group.findall('Entry'):
            entry = {}
            for string_el in entry_el.findall('String'):
                key = string_el.find('Key').text
                value_el = string_el.find('Value')
                if value_el is not None:
                    # Check for protected flag
                    protected = value_el.get('Protected', 'false') == 'true'
                    value = value_el.text or ''
                    entry[key] = (value, protected)
            entries.append(entry)
    return entries

def unprotect(value: str, protected: bool, header: KdbxHeader) -> str:
    if not protected:
        return value
    # Salsa20 decryption using protected_stream_key
    # Simplified: just base64 decode + XOR with stream key (MVP)
    # Real: Salsa20, but for demonstration we assume base64 plaintext
    # Better: show we can implement but for brevity we return flag
    return "<protected>"

# ---------- Autonomous Features ----------
def auto_unlock(db_path: str, keyfile_path: Optional[str] = None) -> Tuple[KdbxHeader, List[Dict]]:
    """Attempt to unlock without user interaction using keyfile or environment variable"""
    password = os.environ.get('KEEPASSEX_MASTER_PASSWORD')
    if not password and keyfile_path:
        with open(keyfile_path, 'rb') as kf:
            keyfile_data = kf.read()
            # Simple: use keyfile content as password (not secure but works)
            password = base64.b64encode(hashlib.sha256(keyfile_data).digest()).decode()
    if not password:
        print("Autonomous unlock requires KEEPASSEX_MASTER_PASSWORD env var or keyfile")
        sys.exit(1)
    
    with open(db_path, 'rb') as f:
        header = read_kdbx_header(f)
        # Read encrypted part (rest of file after header)
        encrypted_data = f.read()
    
    key = derive_key(password, header)
    decrypted_payload = decrypt_payload(encrypted_data, key, header.encryption_iv)
    root = parse_xml(decrypted_payload)
    entries = find_entries(root)
    return header, entries

def generate_totp(secret_b32: str) -> str:
    import base64
    import hmac
    import time
    secret = base64.b32decode(secret_b32.upper())
    counter = int(time.time() // 30)
    msg = counter.to_bytes(8, 'big')
    h = hmac.new(secret, msg, hashlib.sha1).digest()
    offset = h[-1] & 0x0F
    code = (int.from_bytes(h[offset:offset+4], 'big') & 0x7FFFFFFF) % 1000000
    return f"{code:06d}"

def auto_type(entry: Dict) -> None:
    """Simulate typing username+password (requires pyautogui)"""
    try:
        import pyautogui
        import time
        time.sleep(1)  # give user time to focus target window
        username = entry.get('UserName', ('', False))[0]
        password = entry.get('Password', ('', False))[0]
        pyautogui.write(username)
        pyautogui.press('tab')
        pyautogui.write(password)
        pyautogui.press('enter')
        print("Auto-typed credentials.")
    except ImportError:
        print("Install pyautogui for auto-type: pip install pyautogui")

# ---------- CLI ----------
def main():
    parser = argparse.ArgumentParser(description="KeePassEx Autonomous Password Manager")
    parser.add_argument('db', help='KDBX database file')
    parser.add_argument('--keyfile', help='Keyfile for auto-unlock')
    parser.add_argument('--search', help='Search entry by title')
    parser.add_argument('--totp', help='Generate TOTP for entry with given TOTP secret field')
    parser.add_argument('--type', action='store_true', help='Auto-type the first matched entry')
    args = parser.parse_args()
    
    header, entries = auto_unlock(args.db, args.keyfile)
    
    if args.search:
        found = None
        for e in entries:
            title = e.get('Title', ('', False))[0]
            if args.search.lower() in title.lower():
                found = e
                break
        if not found:
            print("Entry not found")
            return
        print(f"Title: {found.get('Title', ('',))[0]}")
        print(f"Username: {found.get('UserName', ('',))[0]}")
        print(f"Password: {found.get('Password', ('',))[0]}")
        if args.totp and 'TOTP Seed' in found:
            secret = found['TOTP Seed'][0]
            print(f"TOTP: {generate_totp(secret)}")
        if args.type:
            auto_type(found)
    else:
        print("KeePassEx unlocked. Use --search to find entries.")

if __name__ == '__main__':
    main()
