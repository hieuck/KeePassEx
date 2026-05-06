/**
 * Group tree sidebar component
 */
import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { useVaultStore } from '../store/vault';

interface GroupDto {
  uuid: string;
  parentUuid?: string;
  name: string;
  iconId: number;
  isExpanded: boolean;
  entryCount: number;
  childGroupCount: number;
}

export function GroupTree() {
  const { selectedGroupUuid, setSelectedGroup } = useVaultStore();

  const { data: groups = [] } = useQuery({
    queryKey: ['groups'],
    queryFn: () => invoke<GroupDto[]>('get_groups'),
  });

  // Build tree structure
  const rootGroups = groups.filter(g => !g.parentUuid);

  return (
    <div className="group-tree" role="tree" aria-label="Vault groups">
      {/* All entries */}
      <GroupItem
        label="All Entries"
        icon="🔑"
        count={groups.reduce((sum, g) => sum + g.entryCount, 0)}
        selected={selectedGroupUuid === null}
        onSelect={() => setSelectedGroup(null)}
        depth={0}
      />

      {/* Group tree */}
      {rootGroups.map(group => (
        <GroupNode
          key={group.uuid}
          group={group}
          allGroups={groups}
          selectedUuid={selectedGroupUuid}
          onSelect={setSelectedGroup}
          depth={0}
        />
      ))}

      <style>{`
        .group-tree {
          padding: 4px 0;
        }
      `}</style>
    </div>
  );
}

function GroupNode({
  group,
  allGroups,
  selectedUuid,
  onSelect,
  depth,
}: {
  group: GroupDto;
  allGroups: GroupDto[];
  selectedUuid: string | null;
  onSelect: (uuid: string) => void;
  depth: number;
}) {
  const [expanded, setExpanded] = useState(group.isExpanded);
  const children = allGroups.filter(g => g.parentUuid === group.uuid);
  const hasChildren = children.length > 0;

  return (
    <div role="treeitem" aria-expanded={hasChildren ? expanded : undefined}>
      <GroupItem
        label={group.name}
        icon={getGroupIcon(group.iconId)}
        count={group.entryCount}
        selected={selectedUuid === group.uuid}
        onSelect={() => onSelect(group.uuid)}
        onToggle={hasChildren ? () => setExpanded(e => !e) : undefined}
        expanded={expanded}
        hasChildren={hasChildren}
        depth={depth}
      />

      {expanded && hasChildren && (
        <div role="group">
          {children.map(child => (
            <GroupNode
              key={child.uuid}
              group={child}
              allGroups={allGroups}
              selectedUuid={selectedUuid}
              onSelect={onSelect}
              depth={depth + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function GroupItem({
  label,
  icon,
  count,
  selected,
  onSelect,
  onToggle,
  expanded,
  hasChildren,
  depth,
}: {
  label: string;
  icon: string;
  count: number;
  selected: boolean;
  onSelect: () => void;
  onToggle?: () => void;
  expanded?: boolean;
  hasChildren?: boolean;
  depth: number;
}) {
  return (
    <div
      className={`group-item ${selected ? 'selected' : ''}`}
      style={{ paddingLeft: 12 + depth * 16 }}
      onClick={onSelect}
      onKeyDown={e => e.key === 'Enter' && onSelect()}
      tabIndex={0}
      role="button"
      aria-pressed={selected}
      aria-label={`${label}, ${count} entries`}
    >
      {hasChildren && (
        <button
          className="group-expand"
          onClick={e => { e.stopPropagation(); onToggle?.(); }}
          aria-label={expanded ? 'Collapse' : 'Expand'}
          tabIndex={-1}
        >
          {expanded ? '▾' : '▸'}
        </button>
      )}
      {!hasChildren && <span className="group-expand-placeholder" />}

      <span className="group-icon" aria-hidden="true">{icon}</span>
      <span className="group-name">{label}</span>
      {count > 0 && (
        <span className="group-count" aria-label={`${count} entries`}>{count}</span>
      )}

      <style>{`
        .group-item {
          display: flex;
          align-items: center;
          gap: 6px;
          padding-top: 5px;
          padding-bottom: 5px;
          padding-right: 12px;
          cursor: pointer;
          border-radius: 6px;
          margin: 1px 6px;
          outline: none;
          transition: background 0.1s;
        }
        .group-item:hover { background: var(--color-bg-tertiary); }
        .group-item.selected { background: var(--color-primary); }
        .group-item.selected .group-name { color: white; }
        .group-item.selected .group-count { background: rgba(255,255,255,0.2); color: white; }
        .group-item:focus-visible { box-shadow: 0 0 0 2px var(--color-primary); }
        .group-expand {
          width: 16px;
          height: 16px;
          border: none;
          background: none;
          cursor: pointer;
          font-size: 10px;
          color: var(--color-text-secondary);
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
          padding: 0;
        }
        .group-expand-placeholder { width: 16px; flex-shrink: 0; }
        .group-icon { font-size: 14px; flex-shrink: 0; }
        .group-name {
          flex: 1;
          font-size: 13px;
          color: var(--color-text);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .group-count {
          font-size: 11px;
          color: var(--color-text-tertiary);
          background: var(--color-bg-tertiary);
          padding: 1px 5px;
          border-radius: 10px;
          flex-shrink: 0;
        }
      `}</style>
    </div>
  );
}

function getGroupIcon(iconId: number): string {
  const map: Record<number, string> = {
    48: '📁', 49: '📂', 43: '🗑️',
  };
  return map[iconId] ?? '📁';
}
