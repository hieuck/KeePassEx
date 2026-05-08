"""
Sync missing i18n keys into all locale files.
Adds the 10 missing 'advanced.*' keys to zh, ja, ko, es, fr, de, pt, ru.
Run: python scripts/sync_i18n.py
"""
import os
import re

LOCALES_DIR = os.path.join(os.path.dirname(__file__), '..', 'packages', 'i18n', 'src', 'locales')

# Translations for the 10 missing keys in each language
MISSING_KEYS = {
    'zh': {
        'database': '数据库',
        'historyMaxItems': '每条目最大历史记录数',
        'compactMode': '紧凑条目列表',
        'compactModeDesc': '以减少间距显示更多条目',
        'autoTypeConfirm': '自动输入前确认',
        'autoTypeConfirmDesc': '填写凭据前显示确认对话框',
        'clearClipboardOnLock': '锁定时清除剪贴板',
        'clearClipboardOnLockDesc': '锁定密码库时自动清除剪贴板',
        'debugLogs': '调试日志',
        'exportDebugLog': '导出调试日志',
    },
    'ja': {
        'database': 'データベース',
        'historyMaxItems': 'エントリごとの最大履歴数',
        'compactMode': 'コンパクトエントリリスト',
        'compactModeDesc': '間隔を縮小してより多くのエントリを表示',
        'autoTypeConfirm': '自動入力前に確認',
        'autoTypeConfirmDesc': '認証情報を入力する前に確認ダイアログを表示',
        'clearClipboardOnLock': 'ロック時にクリップボードをクリア',
        'clearClipboardOnLockDesc': 'ボルトがロックされたときにクリップボードを自動的にクリア',
        'debugLogs': 'デバッグログ',
        'exportDebugLog': 'デバッグログをエクスポート',
    },
    'ko': {
        'database': '데이터베이스',
        'historyMaxItems': '항목당 최대 기록 수',
        'compactMode': '컴팩트 항목 목록',
        'compactModeDesc': '간격을 줄여 더 많은 항목 표시',
        'autoTypeConfirm': '자동 입력 전 확인',
        'autoTypeConfirmDesc': '자격 증명 입력 전 확인 대화 상자 표시',
        'clearClipboardOnLock': '잠금 시 클립보드 지우기',
        'clearClipboardOnLockDesc': '볼트가 잠길 때 클립보드 자동 지우기',
        'debugLogs': '디버그 로그',
        'exportDebugLog': '디버그 로그 내보내기',
    },
    'es': {
        'database': 'Base de datos',
        'historyMaxItems': 'Máximo de elementos de historial por entrada',
        'compactMode': 'Lista de entradas compacta',
        'compactModeDesc': 'Mostrar más entradas con espaciado reducido',
        'autoTypeConfirm': 'Confirmar antes de autoescribir',
        'autoTypeConfirmDesc': 'Mostrar diálogo de confirmación antes de rellenar credenciales',
        'clearClipboardOnLock': 'Limpiar portapapeles al bloquear',
        'clearClipboardOnLockDesc': 'Limpiar automáticamente el portapapeles al bloquear el cofre',
        'debugLogs': 'Registros de depuración',
        'exportDebugLog': 'Exportar registro de depuración',
    },
    'fr': {
        'database': 'Base de données',
        'historyMaxItems': "Nombre maximum d'éléments d'historique par entrée",
        'compactMode': 'Liste d\'entrées compacte',
        'compactModeDesc': 'Afficher plus d\'entrées avec un espacement réduit',
        'autoTypeConfirm': 'Confirmer avant la saisie automatique',
        'autoTypeConfirmDesc': 'Afficher une boîte de dialogue de confirmation avant de remplir les identifiants',
        'clearClipboardOnLock': 'Effacer le presse-papiers au verrouillage',
        'clearClipboardOnLockDesc': 'Effacer automatiquement le presse-papiers lors du verrouillage du coffre',
        'debugLogs': 'Journaux de débogage',
        'exportDebugLog': 'Exporter le journal de débogage',
    },
    'de': {
        'database': 'Datenbank',
        'historyMaxItems': 'Maximale Verlaufseinträge pro Eintrag',
        'compactMode': 'Kompakte Eintrags-Liste',
        'compactModeDesc': 'Mehr Einträge mit reduziertem Abstand anzeigen',
        'autoTypeConfirm': 'Vor Auto-Eingabe bestätigen',
        'autoTypeConfirmDesc': 'Bestätigungsdialog vor dem Ausfüllen von Anmeldedaten anzeigen',
        'clearClipboardOnLock': 'Zwischenablage beim Sperren leeren',
        'clearClipboardOnLockDesc': 'Zwischenablage automatisch leeren, wenn der Tresor gesperrt wird',
        'debugLogs': 'Debug-Protokolle',
        'exportDebugLog': 'Debug-Protokoll exportieren',
    },
    'pt': {
        'database': 'Banco de dados',
        'historyMaxItems': 'Máximo de itens de histórico por entrada',
        'compactMode': 'Lista de entradas compacta',
        'compactModeDesc': 'Mostrar mais entradas com espaçamento reduzido',
        'autoTypeConfirm': 'Confirmar antes do preenchimento automático',
        'autoTypeConfirmDesc': 'Mostrar diálogo de confirmação antes de preencher credenciais',
        'clearClipboardOnLock': 'Limpar área de transferência ao bloquear',
        'clearClipboardOnLockDesc': 'Limpar automaticamente a área de transferência ao bloquear o cofre',
        'debugLogs': 'Logs de depuração',
        'exportDebugLog': 'Exportar log de depuração',
    },
    'ru': {
        'database': 'База данных',
        'historyMaxItems': 'Максимум записей истории на запись',
        'compactMode': 'Компактный список записей',
        'compactModeDesc': 'Показывать больше записей с уменьшенными отступами',
        'autoTypeConfirm': 'Подтверждать перед автовводом',
        'autoTypeConfirmDesc': 'Показывать диалог подтверждения перед заполнением учётных данных',
        'clearClipboardOnLock': 'Очищать буфер при блокировке',
        'clearClipboardOnLockDesc': 'Автоматически очищать буфер обмена при блокировке хранилища',
        'debugLogs': 'Журналы отладки',
        'exportDebugLog': 'Экспортировать журнал отладки',
    },
}

# Keys to insert after 'historyMaxSize' in the advanced section
KEYS_ORDER = [
    'database',
    'historyMaxItems',
    'compactMode',
    'compactModeDesc',
    'autoTypeConfirm',
    'autoTypeConfirmDesc',
    'clearClipboardOnLock',
    'clearClipboardOnLockDesc',
    'debugLogs',
    'exportDebugLog',
]

def add_missing_keys(locale_code: str, translations: dict) -> None:
    path = os.path.join(LOCALES_DIR, f'{locale_code}.ts')
    if not os.path.exists(path):
        print(f'  ⚠ File not found: {path}')
        return

    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Check which keys are already present
    missing = []
    for key in KEYS_ORDER:
        # Check if key exists in advanced section
        pattern = rf"advanced:\s*\{{[^}}]*\b{re.escape(key)}\b"
        if not re.search(pattern, content, re.DOTALL):
            missing.append(key)

    if not missing:
        print(f'  ✓ {locale_code}: all keys present')
        return

    # Find the advanced section and add missing keys before the closing brace
    # Strategy: find 'advanced: {' and its closing '},' then insert before it

    # Find the advanced section
    adv_match = re.search(r'(  // ─── Advanced.*?\n  advanced:\s*\{)', content, re.DOTALL)
    if not adv_match:
        adv_match = re.search(r'(  advanced:\s*\{)', content)

    if not adv_match:
        print(f'  ⚠ {locale_code}: could not find advanced section')
        return

    # Find the closing of the advanced section
    # Count braces from the opening
    start_pos = adv_match.end()
    depth = 1
    pos = start_pos
    while pos < len(content) and depth > 0:
        if content[pos] == '{':
            depth += 1
        elif content[pos] == '}':
            depth -= 1
        pos += 1

    # pos is now just after the closing '}'
    close_pos = pos - 1  # position of '}'

    # Build the new keys string
    new_keys = ''
    for key in missing:
        value = translations.get(key, f'[{key}]')
        # Escape single quotes in value
        value_escaped = value.replace("'", "\\'")
        new_keys += f"    {key}: '{value_escaped}',\n"

    # Insert before the closing brace
    # Find the last newline before close_pos
    insert_pos = close_pos

    new_content = content[:insert_pos] + new_keys + '  ' + content[insert_pos:]

    with open(path, 'w', encoding='utf-8') as f:
        f.write(new_content)

    print(f'  ✓ {locale_code}: added {len(missing)} keys: {", ".join(missing)}')


print('Syncing missing i18n keys...')
for code, translations in MISSING_KEYS.items():
    add_missing_keys(code, translations)

print('\nDone! Run: pnpm test:ts to verify')
