"""
Rebuild all locale files to match en.ts structure.
For missing keys, uses machine-translated values.
Run: python scripts/rebuild_locales.py
"""
import os
import re
import json

LOCALES_DIR = os.path.join(os.path.dirname(__file__), '..', 'packages', 'i18n', 'src', 'locales')

# Read en.ts and extract the object
def read_ts_file(path):
    with open(path, 'r', encoding='utf-8') as f:
        return f.read()

def get_all_leaf_keys(content, prefix=''):
    """Extract all leaf key paths from a TypeScript object literal."""
    # Simple regex-based extraction of key: 'value' pairs
    keys = {}
    # Match key: 'value' or key: "value" patterns
    pattern = r"^\s{2,}(\w+):\s*'([^']*(?:\\'[^']*)*)'"
    for line in content.split('\n'):
        m = re.match(r"^\s+(\w+):\s*'(.*?)'(?:,)?$", line)
        if m:
            key, val = m.group(1), m.group(2)
            keys[key] = val
    return keys

# Read en.ts content
en_path = os.path.join(LOCALES_DIR, 'en.ts')
en_content = read_ts_file(en_path)

# Extract the full object body (between first { and last })
# We'll use a different approach: read the file and find all key paths

def extract_keys_recursive(text):
    """Extract all key paths from TypeScript object."""
    keys = {}

    def parse_obj(s, prefix):
        # Find key: value or key: { ... } patterns
        i = 0
        while i < len(s):
            # Skip whitespace and comments
            while i < len(s) and s[i] in ' \t\n\r':
                i += 1
            if i >= len(s):
                break

            # Skip comment lines
            if s[i:i+2] == '//':
                end = s.find('\n', i)
                i = end + 1 if end != -1 else len(s)
                continue

            # Match key
            key_match = re.match(r'(\w+)\s*:', s[i:])
            if not key_match:
                i += 1
                continue

            key = key_match.group(1)
            i += key_match.end()

            # Skip whitespace
            while i < len(s) and s[i] in ' \t':
                i += 1

            full_key = f"{prefix}.{key}" if prefix else key

            if i < len(s) and s[i] == '{':
                # Nested object — find matching closing brace
                depth = 1
                j = i + 1
                while j < len(s) and depth > 0:
                    if s[j] == '{':
                        depth += 1
                    elif s[j] == '}':
                        depth -= 1
                    j += 1
                nested = s[i+1:j-1]
                parse_obj(nested, full_key)
                i = j
            elif i < len(s) and s[i] == "'":
                # String value
                j = i + 1
                while j < len(s):
                    if s[j] == '\\':
                        j += 2
                        continue
                    if s[j] == "'":
                        break
                    j += 1
                value = s[i+1:j]
                keys[full_key] = value
                i = j + 1
            else:
                i += 1

            # Skip comma
            while i < len(s) and s[i] in ' \t\n\r,':
                i += 1

    # Extract the object body
    start = text.find('= {')
    if start == -1:
        start = text.find('= {\n')
    if start == -1:
        return keys

    start = text.find('{', start)
    depth = 1
    i = start + 1
    while i < len(text) and depth > 0:
        if text[i] == '{':
            depth += 1
        elif text[i] == '}':
            depth -= 1
        i += 1

    obj_body = text[start+1:i-1]
    parse_obj(obj_body, '')
    return keys

en_keys = extract_keys_recursive(en_content)
print(f"Extracted {len(en_keys)} keys from en.ts")

# Read existing locale files and extract their keys
def read_locale_keys(locale_code):
    path = os.path.join(LOCALES_DIR, f'{locale_code}.ts')
    if not os.path.exists(path):
        return {}
    content = read_ts_file(path)
    return extract_keys_recursive(content)

# Find missing keys for each locale
locales = ['zh', 'ja', 'ko', 'es', 'fr', 'de', 'pt', 'ru']

for locale in locales:
    existing = read_locale_keys(locale)
    missing = {k: v for k, v in en_keys.items() if k not in existing}
    extra = {k: v for k, v in existing.items() if k not in en_keys}
    print(f"\n{locale}: {len(existing)} existing, {len(missing)} missing, {len(extra)} extra")
    if missing:
        print(f"  Missing: {list(missing.keys())[:5]}{'...' if len(missing) > 5 else ''}")
