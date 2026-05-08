"""
Generate complete locale files for all 8 languages by merging existing translations
with missing keys (using EN as fallback for untranslated keys).

This ensures 100% parity with en.ts for all locales.
Run: python scripts/gen_locales.py
"""
import os
import re

LOCALES_DIR = os.path.join(os.path.dirname(__file__), '..', 'packages', 'i18n', 'src', 'locales')

def read_file(path):
    with open(path, 'r', encoding='utf-8') as f:
        return f.read()

def write_file(path, content):
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)

def extract_flat_keys(content):
    """Extract all key: 'value' pairs from a TS object, returning flat dict."""
    keys = {}

    def parse_obj(s, prefix):
        i = 0
        while i < len(s):
            # Skip whitespace
            while i < len(s) and s[i] in ' \t\n\r':
                i += 1
            if i >= len(s):
                break
            # Skip comment lines
            if s[i:i+2] == '//':
                end = s.find('\n', i)
                i = (end + 1) if end != -1 else len(s)
                continue
            # Match key
            km = re.match(r'(\w+)\s*:', s[i:])
            if not km:
                i += 1
                continue
            key = km.group(1)
            i += km.end()
            while i < len(s) and s[i] in ' \t':
                i += 1
            full_key = f"{prefix}.{key}" if prefix else key
            if i < len(s) and s[i] == '{':
                depth = 1
                j = i + 1
                while j < len(s) and depth > 0:
                    if s[j] == '{': depth += 1
                    elif s[j] == '}': depth -= 1
                    j += 1
                parse_obj(s[i+1:j-1], full_key)
                i = j
            elif i < len(s) and s[i] == "'":
                j = i + 1
                while j < len(s):
                    if s[j] == '\\': j += 2; continue
                    if s[j] == "'": break
                    j += 1
                keys[full_key] = s[i+1:j]
                i = j + 1
            else:
                i += 1
            while i < len(s) and s[i] in ' \t\n\r,':
                i += 1

    start = content.find('= {')
    if start == -1: return keys
    start = content.find('{', start)
    depth = 1; i = start + 1
    while i < len(content) and depth > 0:
        if content[i] == '{': depth += 1
        elif content[i] == '}': depth -= 1
        i += 1
    parse_obj(content[start+1:i-1], '')
    return keys

def rebuild_ts_from_en(en_content, locale_keys, locale_code, locale_name, locale_comment):
    """Rebuild a locale TS file using EN structure + existing translations."""

    def replace_values(s, prefix, locale_keys, en_keys):
        """Recursively replace EN values with locale values."""
        result = []
        i = 0
        lines = s.split('\n')

        for line in lines:
            # Check if this line has a key: 'value' pattern
            m = re.match(r'^(\s+)(\w+):\s*\'(.*?)\'(,?)(\s*(?://.*)?)?$', line)
            if m:
                indent, key, en_val, comma, comment = m.groups()
                full_key = f"{prefix}.{key}" if prefix else key
                # Use locale translation if available, else EN value
                locale_val = locale_keys.get(full_key, en_val)
                # Escape single quotes
                locale_val_escaped = locale_val.replace("'", "\\'")
                result.append(f"{indent}{key}: '{locale_val_escaped}'{comma}{comment}")
            else:
                result.append(line)

        return '\n'.join(result)

    # Get EN flat keys for reference
    en_keys = extract_flat_keys(en_content)

    # Extract the object body from en.ts
    start = en_content.find('export const en = {')
    if start == -1:
        start = en_content.find('export const en={')

    # Replace all string values with locale equivalents
    # We'll do a line-by-line replacement tracking the key path

    lines = en_content.split('\n')
    result_lines = []
    key_stack = []  # Track current object path

    for line in lines:
        # Track object nesting
        stripped = line.strip()

        # Opening of a section: key: {
        section_m = re.match(r'^(\s+)(\w+):\s*\{', line)
        if section_m and not stripped.startswith('//'):
            key_stack.append(section_m.group(2))
            result_lines.append(line)
            continue

        # Closing brace
        if re.match(r'^\s+\},?', line) and not stripped.startswith('//'):
            if key_stack:
                key_stack.pop()
            result_lines.append(line)
            continue

        # Key: 'value' line
        val_m = re.match(r'^(\s+)(\w+):\s*\'((?:[^\'\\]|\\.)*)\'(,?)(\s*(?://.*)?)?$', line)
        if val_m:
            indent, key, en_val, comma, comment = val_m.groups()
            full_key = '.'.join(key_stack + [key]) if key_stack else key
            locale_val = locale_keys.get(full_key, en_val)
            locale_val_escaped = locale_val.replace("'", "\\'")
            result_lines.append(f"{indent}{key}: '{locale_val_escaped}'{comma}{comment}")
            continue

        result_lines.append(line)

    # Replace the export declaration
    result = '\n'.join(result_lines)
    result = result.replace(
        f'export const en = {{',
        f'export const {locale_code} = {{'
    )
    result = result.replace(
        f'export type TranslationEn = typeof en;',
        ''
    )

    # Fix the header comment
    header_end = result.find('export const')
    old_header = result[:header_end]
    new_header = f"/**\n * {locale_name} translations — KeePassEx\n * {locale_comment}\n */\n"
    result = new_header + result[header_end:]

    return result

# Language metadata
LANG_META = {
    'zh': ('Chinese (Simplified)', '简体中文翻译'),
    'ja': ('Japanese', '日本語翻訳'),
    'ko': ('Korean', '한국어 번역'),
    'es': ('Spanish', 'Traducciones al español'),
    'fr': ('French', 'Traductions françaises'),
    'de': ('German', 'Deutsche Übersetzungen'),
    'pt': ('Portuguese', 'Traduções em português'),
    'ru': ('Russian', 'Переводы на русский'),
}

# Read en.ts
en_path = os.path.join(LOCALES_DIR, 'en.ts')
en_content = read_file(en_path)
en_keys = extract_flat_keys(en_content)
print(f"EN: {len(en_keys)} keys")

# Process each locale
for locale_code, (lang_name, lang_comment) in LANG_META.items():
    locale_path = os.path.join(LOCALES_DIR, f'{locale_code}.ts')

    # Read existing translations
    if os.path.exists(locale_path):
        existing_content = read_file(locale_path)
        existing_keys = extract_flat_keys(existing_content)
    else:
        existing_keys = {}

    print(f"\n{locale_code} ({lang_name}): {len(existing_keys)} existing / {len(en_keys)} total")

    # Rebuild the file
    new_content = rebuild_ts_from_en(en_content, existing_keys, locale_code, lang_name, lang_comment)

    write_file(locale_path, new_content)

    # Verify
    new_keys = extract_flat_keys(new_content)
    missing = [k for k in en_keys if k not in new_keys]
    extra = [k for k in new_keys if k not in en_keys]
    print(f"  → {len(new_keys)} keys, {len(missing)} missing, {len(extra)} extra")
    if missing:
        print(f"  ⚠ Still missing: {missing[:3]}")

print("\n✓ All locale files rebuilt!")
print("Run: pnpm test:ts to verify parity")
