/**
 * Search bar component
 */
import { useState, useRef, useCallback } from 'react';

interface SearchBarProps {
  placeholder?: string;
  onChange: (value: string) => void;
  debounceMs?: number;
}

export function SearchBar({ placeholder = 'Search...', onChange, debounceMs = 200 }: SearchBarProps) {
  const [value, setValue] = useState('');
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    setValue(v);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => onChange(v), debounceMs);
  }, [onChange, debounceMs]);

  const handleClear = () => {
    setValue('');
    onChange('');
  };

  return (
    <div className="search-bar" role="search">
      <span className="search-icon" aria-hidden="true">🔍</span>
      <input
        type="search"
        className="search-input"
        value={value}
        onChange={handleChange}
        placeholder={placeholder}
        aria-label={placeholder}
      />
      {value && (
        <button
          className="search-clear"
          onClick={handleClear}
          aria-label="Clear search"
        >
          ✕
        </button>
      )}

      <style>{`
        .search-bar {
          display: flex;
          align-items: center;
          gap: 6px;
          background: var(--color-bg);
          border: 1px solid var(--color-border);
          border-radius: var(--radius-full);
          padding: 5px 10px;
          transition: border-color 0.15s;
        }
        .search-bar:focus-within { border-color: var(--color-primary); }
        .search-icon { font-size: 12px; flex-shrink: 0; }
        .search-input {
          flex: 1;
          border: none;
          background: none;
          outline: none;
          font-size: 13px;
          color: var(--color-text);
          min-width: 0;
        }
        .search-input::placeholder { color: var(--color-text-tertiary); }
        .search-input::-webkit-search-cancel-button { display: none; }
        .search-clear {
          border: none;
          background: none;
          cursor: pointer;
          font-size: 11px;
          color: var(--color-text-tertiary);
          padding: 0;
          line-height: 1;
          flex-shrink: 0;
        }
        .search-clear:hover { color: var(--color-text); }
      `}</style>
    </div>
  );
}
