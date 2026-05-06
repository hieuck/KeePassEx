/**
 * Tag Input — add/remove tags with keyboard support
 */
import React, { useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';

interface TagInputProps {
  tags: string[];
  onChange: (tags: string[]) => void;
  placeholder?: string;
  maxTags?: number;
  readOnly?: boolean;
}

export function TagInput({
  tags,
  onChange,
  placeholder,
  maxTags = 20,
  readOnly = false,
}: TagInputProps) {
  const { t } = useTranslation();
  const [input, setInput] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const addTag = (value: string) => {
    const tag = value.trim().toLowerCase();
    if (!tag || tags.includes(tag) || tags.length >= maxTags) return;
    onChange([...tags, tag]);
    setInput('');
  };

  const removeTag = (tag: string) => onChange(tags.filter(t => t !== tag));

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(input);
    } else if (e.key === 'Backspace' && !input && tags.length > 0) removeTag(tags[tags.length - 1]);
  };

  return (
    <div
      className="tag-input-container"
      onClick={() => inputRef.current?.focus()}
      role="group"
      aria-label={t('entry.tags')}
    >
      {tags.map(tag => (
        <span key={tag} className="tag-chip">
          {tag}
          {!readOnly && (
            <button
              className="tag-chip-remove"
              onClick={e => {
                e.stopPropagation();
                removeTag(tag);
              }}
              aria-label={`Remove tag ${tag}`}
              type="button"
            >
              ✕
            </button>
          )}
        </span>
      ))}
      {!readOnly && tags.length < maxTags && (
        <input
          ref={inputRef}
          type="text"
          className="tag-input-field"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={() => {
            if (input.trim()) addTag(input);
          }}
          placeholder={tags.length === 0 ? (placeholder ?? t('entry.tags')) : ''}
          aria-label={t('entry.tags')}
        />
      )}
      <style>{`
        .tag-input-container { display:flex; flex-wrap:wrap; gap:4px; align-items:center; min-height:36px; padding:4px 8px; border:1px solid var(--color-border); border-radius:var(--radius-md); background:var(--color-bg); cursor:text; transition:border-color .15s; }
        .tag-input-container:focus-within { border-color:var(--color-primary); }
        .tag-chip { display:inline-flex; align-items:center; gap:4px; background:var(--color-bg-tertiary); color:var(--color-text-secondary); padding:2px 8px; border-radius:var(--radius-full); font-size:12px; font-weight:500; }
        .tag-chip-remove { background:none; border:none; cursor:pointer; padding:0; color:var(--color-text-tertiary); font-size:10px; line-height:1; display:flex; align-items:center; }
        .tag-chip-remove:hover { color:var(--color-danger); }
        .tag-input-field { border:none; outline:none; background:none; font-size:13px; color:var(--color-text); min-width:80px; flex:1; }
        .tag-input-field::placeholder { color:var(--color-text-tertiary); }
      `}</style>
    </div>
  );
}
