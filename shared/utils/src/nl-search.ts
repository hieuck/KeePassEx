/**
 * Natural Language Search — TypeScript client-side parser
 * Mirrors the Rust implementation in packages/core/src/search/
 * Used by browser extension and web contexts
 */

export type NlIntent =
  | { type: 'all' }
  | { type: 'expired' }
  | { type: 'expiringSoon'; days: number }
  | { type: 'weak' }
  | { type: 'reused' }
  | { type: 'noPassword' }
  | { type: 'breached' }
  | { type: 'favorites' }
  | { type: 'recent' }
  | { type: 'withFeature'; feature: EntryFeature }
  | { type: 'createdIn'; timeFilter: TimeFilter }
  | { type: 'modifiedIn'; timeFilter: TimeFilter }
  | { type: 'notUsedIn'; timeFilter: TimeFilter }
  | { type: 'search'; text: string };

export type EntryFeature = 'otp' | 'passkey' | 'ssh' | 'attachment';

export type TimeFilter =
  | 'today'
  | 'yesterday'
  | 'thisWeek'
  | 'lastWeek'
  | 'thisMonth'
  | 'lastMonth'
  | { days: number }
  | { months: number }
  | { years: number };

export interface NlQuery {
  intent: NlIntent;
  group?: string;
  tags: string[];
  timeFilter?: TimeFilter;
  features: EntryFeature[];
  text?: string;
  lang: 'en' | 'vi' | 'unknown';
}

export interface SearchFilter {
  text?: string;
  group?: string;
  tags: string[];
  expiredOnly: boolean;
  expiringWithinDays?: number;
  weakOnly: boolean;
  reusedOnly: boolean;
  noPasswordOnly: boolean;
  breachedOnly: boolean;
  favoritesOnly: boolean;
  recentlyUsedDays?: number;
  createdAfter?: Date;
  createdBefore?: Date;
  modifiedAfter?: Date;
  modifiedBefore?: Date;
  notAccessedSince?: Date;
  hasOtp?: boolean;
  hasPasskey?: boolean;
  hasSshKey?: boolean;
  hasAttachment?: boolean;
}

// ─── NL Keywords ─────────────────────────────────────────────────────────────

const EN_KEYWORDS = [
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

const VI_KEYWORDS = [
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

/**
 * Check if a query string looks like a natural language query
 * (as opposed to a simple keyword search)
 */
export function isNaturalLanguageQuery(query: string): boolean {
  const lower = query.toLowerCase();
  return EN_KEYWORDS.some(kw => lower.includes(kw)) || VI_KEYWORDS.some(kw => lower.includes(kw));
}

/**
 * Parse a natural language query string into a structured NlQuery.
 * Supports English and Vietnamese.
 */
export function parseNlQuery(input: string): NlQuery {
  const normalized = input.trim().toLowerCase();
  const lang = detectLanguage(normalized);

  const query: NlQuery = {
    intent: { type: 'all' },
    tags: [],
    features: [],
    lang,
  };

  // Extract group
  query.group = extractGroup(normalized);

  // Extract features
  query.features = extractFeatures(normalized);

  // Extract time filter
  query.timeFilter = extractTimeFilter(normalized);

  // Determine intent
  query.intent = determineIntent(normalized, query);

  // Remaining text
  const remaining = extractRemainingText(normalized);
  if (remaining) query.text = remaining;

  return query;
}

/**
 * Build a SearchFilter from a parsed NlQuery.
 */
export function buildSearchFilter(query: NlQuery): SearchFilter {
  const now = new Date();
  const filter: SearchFilter = {
    tags: query.tags,
    expiredOnly: false,
    weakOnly: false,
    reusedOnly: false,
    noPasswordOnly: false,
    breachedOnly: false,
    favoritesOnly: false,
  };

  // Group
  filter.group = query.group;

  // Features
  for (const feature of query.features) {
    if (feature === 'otp') filter.hasOtp = true;
    if (feature === 'passkey') filter.hasPasskey = true;
    if (feature === 'ssh') filter.hasSshKey = true;
    if (feature === 'attachment') filter.hasAttachment = true;
  }

  // Intent
  switch (query.intent.type) {
    case 'expired':
      filter.expiredOnly = true;
      break;
    case 'expiringSoon':
      filter.expiringWithinDays = query.intent.days;
      break;
    case 'weak':
      filter.weakOnly = true;
      break;
    case 'reused':
      filter.reusedOnly = true;
      break;
    case 'noPassword':
      filter.noPasswordOnly = true;
      break;
    case 'breached':
      filter.breachedOnly = true;
      break;
    case 'favorites':
      filter.favoritesOnly = true;
      break;
    case 'recent':
      filter.recentlyUsedDays = 7;
      break;
    case 'createdIn':
      const [ca, cb] = timeFilterToRange(query.intent.timeFilter, now);
      filter.createdAfter = ca;
      filter.createdBefore = cb;
      break;
    case 'modifiedIn':
      const [ma, mb] = timeFilterToRange(query.intent.timeFilter, now);
      filter.modifiedAfter = ma;
      filter.modifiedBefore = mb;
      break;
    case 'notUsedIn':
      filter.notAccessedSince = timeFilterToCutoff(query.intent.timeFilter, now);
      break;
    case 'search':
      filter.text = query.intent.text;
      break;
  }

  // Free text
  if (!filter.text && query.text) {
    filter.text = query.text;
  }

  return filter;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function detectLanguage(input: string): 'en' | 'vi' | 'unknown' {
  const viScore = VI_KEYWORDS.filter(kw => input.includes(kw)).length;
  const enScore = EN_KEYWORDS.filter(kw => input.includes(kw)).length;
  if (viScore > enScore) return 'vi';
  if (enScore > 0) return 'en';
  return 'unknown';
}

function determineIntent(input: string, query: NlQuery): NlIntent {
  if (matchesAny(input, ['expired', 'hết hạn', 'quá hạn'])) return { type: 'expired' };
  if (matchesAny(input, ['expiring', 'sắp hết hạn'])) {
    const days = extractNumber(input) ?? 7;
    return { type: 'expiringSoon', days };
  }
  if (matchesAny(input, ['weak password', 'mật khẩu yếu', 'yếu'])) return { type: 'weak' };
  if (matchesAny(input, ['reused', 'duplicate', 'trùng lặp'])) return { type: 'reused' };
  if (matchesAny(input, ['no password', 'without password', 'không mật khẩu']))
    return { type: 'noPassword' };
  if (matchesAny(input, ['breached', 'breach', 'rò rỉ', 'bị lộ'])) return { type: 'breached' };
  if (matchesAny(input, ['favorite', 'starred', 'yêu thích'])) return { type: 'favorites' };
  if (matchesAny(input, ['recent', 'recently', 'gần đây'])) return { type: 'recent' };

  if (query.features.length > 0) return { type: 'withFeature', feature: query.features[0] };

  if (query.timeFilter) {
    if (matchesAny(input, ['created', 'added', 'tạo', 'thêm'])) {
      return { type: 'createdIn', timeFilter: query.timeFilter };
    }
    if (matchesAny(input, ['modified', 'changed', 'sửa', 'thay đổi'])) {
      return { type: 'modifiedIn', timeFilter: query.timeFilter };
    }
    if (matchesAny(input, ['not used', 'unused', 'chưa dùng'])) {
      return { type: 'notUsedIn', timeFilter: query.timeFilter };
    }
    return { type: 'createdIn', timeFilter: query.timeFilter };
  }

  if (query.text) return { type: 'search', text: query.text };
  return { type: 'all' };
}

function extractGroup(input: string): string | undefined {
  const patterns = ['in group ', 'in the group ', 'trong nhóm ', 'nhóm '];
  for (const pattern of patterns) {
    const idx = input.indexOf(pattern);
    if (idx !== -1) {
      const rest = input.slice(idx + pattern.length);
      const group = rest.split(/\s+/).slice(0, 3).join(' ').trim();
      if (group) return group;
    }
  }
  return undefined;
}

function extractFeatures(input: string): EntryFeature[] {
  const features: EntryFeature[] = [];
  if (matchesAny(input, ['otp', 'totp', '2fa', 'two-factor', 'xác thực 2 bước']))
    features.push('otp');
  if (matchesAny(input, ['passkey', 'webauthn', 'fido2'])) features.push('passkey');
  if (matchesAny(input, ['ssh', 'ssh key', 'khóa ssh'])) features.push('ssh');
  if (matchesAny(input, ['attachment', 'file', 'tệp đính kèm'])) features.push('attachment');
  return features;
}

function extractTimeFilter(input: string): TimeFilter | undefined {
  if (matchesAny(input, ['today', 'hôm nay'])) return 'today';
  if (matchesAny(input, ['yesterday', 'hôm qua'])) return 'yesterday';
  if (matchesAny(input, ['this week', 'tuần này'])) return 'thisWeek';
  if (matchesAny(input, ['last week', 'tuần trước'])) return 'lastWeek';
  if (matchesAny(input, ['this month', 'tháng này'])) return 'thisMonth';
  if (matchesAny(input, ['last month', 'tháng trước'])) return 'lastMonth';

  const dayMatch = input.match(/(\d+)\s+(?:day|days|ngày)/);
  if (dayMatch) return { days: parseInt(dayMatch[1], 10) };

  const monthMatch = input.match(/(\d+)\s+(?:month|months|tháng)/);
  if (monthMatch) return { months: parseInt(monthMatch[1], 10) };

  const yearMatch = input.match(/(\d+)\s+(?:year|years|năm)/);
  if (yearMatch) return { years: parseInt(yearMatch[1], 10) };

  return undefined;
}

function extractRemainingText(input: string): string | undefined {
  const stopWords = [
    'show',
    'find',
    'list',
    'get',
    'all',
    'entries',
    'entry',
    'passwords',
    'password',
    'with',
    'without',
    'in',
    'the',
    'a',
    'an',
    'that',
    'are',
    'is',
    'have',
    'has',
    'tìm',
    'hiển thị',
    'tất cả',
    'mục',
    'mật khẩu',
    'có',
    'không',
    'trong',
  ];
  const words = input.split(/\s+/).filter(w => !stopWords.includes(w));
  const text = words.join(' ').trim();
  return text || undefined;
}

function matchesAny(input: string, patterns: string[]): boolean {
  return patterns.some(p => input.includes(p));
}

function extractNumber(input: string): number | undefined {
  const match = input.match(/\d+/);
  return match ? parseInt(match[0], 10) : undefined;
}

function timeFilterToRange(tf: TimeFilter, now: Date): [Date | undefined, Date | undefined] {
  const d = new Date(now);
  switch (tf) {
    case 'today': {
      const start = new Date(d);
      start.setHours(0, 0, 0, 0);
      return [start, undefined];
    }
    case 'yesterday': {
      const start = new Date(d);
      start.setDate(start.getDate() - 1);
      start.setHours(0, 0, 0, 0);
      const end = new Date(start);
      end.setHours(23, 59, 59, 999);
      return [start, end];
    }
    case 'thisWeek': {
      const start = new Date(d);
      start.setDate(start.getDate() - start.getDay());
      start.setHours(0, 0, 0, 0);
      return [start, undefined];
    }
    case 'lastWeek': {
      const end = new Date(d);
      end.setDate(end.getDate() - end.getDay() - 1);
      end.setHours(23, 59, 59, 999);
      const start = new Date(end);
      start.setDate(start.getDate() - 6);
      start.setHours(0, 0, 0, 0);
      return [start, end];
    }
    case 'thisMonth': {
      const start = new Date(d.getFullYear(), d.getMonth(), 1);
      return [start, undefined];
    }
    case 'lastMonth': {
      const start = new Date(d.getFullYear(), d.getMonth() - 1, 1);
      const end = new Date(d.getFullYear(), d.getMonth(), 0, 23, 59, 59, 999);
      return [start, end];
    }
    default:
      if (typeof tf === 'object' && 'days' in tf) {
        const start = new Date(d);
        start.setDate(start.getDate() - tf.days);
        return [start, undefined];
      }
      if (typeof tf === 'object' && 'months' in tf) {
        const start = new Date(d);
        start.setMonth(start.getMonth() - tf.months);
        return [start, undefined];
      }
      if (typeof tf === 'object' && 'years' in tf) {
        const start = new Date(d);
        start.setFullYear(start.getFullYear() - tf.years);
        return [start, undefined];
      }
      return [undefined, undefined];
  }
}

function timeFilterToCutoff(tf: TimeFilter, now: Date): Date {
  const d = new Date(now);
  if (typeof tf === 'object' && 'days' in tf) {
    d.setDate(d.getDate() - tf.days);
  } else if (typeof tf === 'object' && 'months' in tf) {
    d.setMonth(d.getMonth() - tf.months);
  } else if (tf === 'lastMonth') {
    d.setMonth(d.getMonth() - 1);
  } else if (tf === 'lastWeek') {
    d.setDate(d.getDate() - 7);
  }
  return d;
}
