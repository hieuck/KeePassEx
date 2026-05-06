/**
 * NaturalLanguageSearch — Smart search bar with NL query support
 *
 * Exclusive KeePassEx feature: No competitor has this.
 * Supports English and Vietnamese natural language queries.
 *
 * Examples:
 * - "show expired entries"
 * - "find weak passwords in Banking"
 * - "entries with OTP created last month"
 * - "tìm mật khẩu yếu" (Vietnamese)
 */
import React, { useState, useCallback, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';

interface NlSearchProps {
  onSearch: (query: string, isNl: boolean) => void;
  placeholder?: string;
  className?: string;
}

// Suggestion chips for quick NL queries
const EN_SUGGESTIONS = [
  { label: '⏰ Expired', query: 'show expired entries' },
  { label: '💪 Weak', query: 'find weak passwords' },
  { label: '🔄 Reused', query: 'show reused passwords' },
  { label: '🔐 OTP', query: 'entries with OTP' },
  { label: '⭐ Favorites', query: 'show favorites' },
  { label: '🚨 Breached', query: 'find breached passwords' },
  { label: '📅 This month', query: 'entries created this month' },
  { label: '🔑 No password', query: 'entries without password' },
];

const VI_SUGGESTIONS = [
  { label: '⏰ Hết hạn', query: 'tìm mục hết hạn' },
  { label: '💪 Yếu', query: 'mật khẩu yếu' },
  { label: '🔐 OTP', query: 'mục có OTP' },
  { label: '⭐ Yêu thích', query: 'yêu thích' },
  { label: '🚨 Rò rỉ', query: 'mật khẩu bị rò rỉ' },
  { label: '📅 Tháng này', query: 'mục tạo tháng này' },
];

// NL query keywords that trigger smart mode
const NL_KEYWORDS_EN = [
  'show',
  'find',
  'list',
  'get',
  'expired',
  'weak',
  'reused',
  'breached',
  'with otp',
  'with passkey',
  'with ssh',
  'favorites',
  'recent',
  'created',
  'modified',
  'not used',
  'without password',
  'no password',
  'in group',
  'last month',
  'last week',
  'this month',
  'this week',
  'today',
];

const NL_KEYWORDS_VI = [
  'tìm',
  'hiển thị',
  'hết hạn',
  'yếu',
  'trùng',
  'rò rỉ',
  'có otp',
  'yêu thích',
  'gần đây',
  'tạo',
  'sửa',
  'chưa dùng',
  'không mật khẩu',
  'tháng trước',
  'tuần trước',
  'tháng này',
  'hôm nay',
];

function isNlQuery(query: string): boolean {
  const lower = query.toLowerCase();
  return (
    NL_KEYWORDS_EN.some(kw => lower.includes(kw)) || NL_KEYWORDS_VI.some(kw => lower.includes(kw))
  );
}

export function NaturalLanguageSearch({ onSearch, placeholder, className = '' }: NlSearchProps) {
  const { t, i18n } = useTranslation();
  const [query, setQuery] = useState('');
  const [isNlMode, setIsNlMode] = useState(false);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const isVi = i18n.language === 'vi';
  const suggestions = isVi ? VI_SUGGESTIONS : EN_SUGGESTIONS;

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setQuery(val);
    setIsNlMode(val.length > 2 && isNlQuery(val));
    setShowSuggestions(val.length === 0);
  }, []);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (query.trim()) {
        onSearch(query.trim(), isNlMode);
        setShowSuggestions(false);
      }
    },
    [query, isNlMode, onSearch]
  );

  const handleSuggestion = useCallback(
    (suggestionQuery: string) => {
      setQuery(suggestionQuery);
      setIsNlMode(true);
      setShowSuggestions(false);
      onSearch(suggestionQuery, true);
      inputRef.current?.focus();
    },
    [onSearch]
  );

  const handleClear = useCallback(() => {
    setQuery('');
    setIsNlMode(false);
    setShowSuggestions(false);
    onSearch('', false);
    inputRef.current?.focus();
  }, [onSearch]);

  const handleFocus = useCallback(() => {
    if (query.length === 0) {
      setShowSuggestions(true);
    }
  }, [query]);

  const handleBlur = useCallback(() => {
    // Delay to allow suggestion click
    setTimeout(() => setShowSuggestions(false), 150);
  }, []);

  // Keyboard shortcut: Ctrl+F or / to focus
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey && e.key === 'f') || e.key === '/') {
        if (document.activeElement !== inputRef.current) {
          e.preventDefault();
          inputRef.current?.focus();
        }
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  return (
    <div className={`nl-search-wrapper ${className}`}>
      <form
        className={`nl-search-form ${isNlMode ? 'nl-mode' : ''}`}
        onSubmit={handleSubmit}
        role="search"
        aria-label={t('search.naturalLanguage', 'Natural language search')}
      >
        {/* Search icon / NL indicator */}
        <span
          className={`nl-search-icon ${isNlMode ? 'nl-active' : ''}`}
          aria-hidden="true"
          title={isNlMode ? 'Natural language mode' : 'Search'}
        >
          {isNlMode ? '🧠' : '🔍'}
        </span>

        <input
          ref={inputRef}
          type="search"
          className="nl-search-input"
          value={query}
          onChange={handleChange}
          onFocus={handleFocus}
          onBlur={handleBlur}
          placeholder={
            placeholder ??
            (isVi ? 'Tìm kiếm... hoặc "tìm mật khẩu yếu"' : 'Search... or "find weak passwords"')
          }
          aria-label={t('entry.searchPlaceholder')}
          autoComplete="off"
          spellCheck={false}
        />

        {/* NL mode badge */}
        {isNlMode && (
          <span className="nl-badge" aria-label="Natural language mode active">
            NL
          </span>
        )}

        {/* Clear button */}
        {query && (
          <button
            type="button"
            className="nl-clear-btn"
            onClick={handleClear}
            aria-label={t('common.clear')}
          >
            ✕
          </button>
        )}
      </form>

      {/* Suggestion chips */}
      {showSuggestions && (
        <div className="nl-suggestions" role="listbox" aria-label="Search suggestions">
          <span className="nl-suggestions-label">
            {isVi ? 'Gợi ý tìm kiếm:' : 'Quick searches:'}
          </span>
          <div className="nl-chips">
            {suggestions.map(s => (
              <button
                key={s.query}
                role="option"
                className="nl-chip"
                onClick={() => handleSuggestion(s.query)}
                aria-selected={false}
              >
                {s.label}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
