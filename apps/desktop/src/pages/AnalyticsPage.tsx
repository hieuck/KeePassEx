/**
 * AnalyticsPage — Vault Analytics Dashboard
 *
 * Exclusive KeePassEx feature: No competitor has this.
 * Shows password strength distribution, entry timeline, most accessed entries,
 * breach history, OTP usage, and security summary.
 */
import React, { useEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';

interface StrengthDistribution {
  very_weak: number;
  weak: number;
  fair: number;
  strong: number;
  very_strong: number;
  no_password: number;
}

interface TimelinePoint {
  year: number;
  month: number;
  label: string;
  count: number;
}

interface AccessedEntry {
  uuid: string;
  title: string;
  access_count: number;
  last_accessed: string | null;
}

interface FeatureUsage {
  with_otp: number;
  with_passkey: number;
  with_ssh_key: number;
  with_attachment: number;
  with_custom_fields: number;
  with_expiry: number;
  favorites: number;
}

interface SecuritySummary {
  health_score: number;
  weak_count: number;
  reused_count: number;
  expired_count: number;
  expiring_soon_count: number;
  breached_count: number;
  no_password_count: number;
}

interface VaultAnalytics {
  generated_at: string;
  total_entries: number;
  strength_distribution: StrengthDistribution;
  creation_timeline: TimelinePoint[];
  modification_timeline: TimelinePoint[];
  most_accessed: AccessedEntry[];
  feature_usage: FeatureUsage;
  security_summary: SecuritySummary;
  password_age: {
    average_days: number;
    oldest_days: number;
    newest_days: number;
    older_than_1_year: number;
    older_than_6_months: number;
    changed_last_30_days: number;
  };
}

type TabId = 'overview' | 'strength' | 'timeline' | 'access' | 'features';

export function AnalyticsPage() {
  const { t } = useTranslation();
  const [analytics, setAnalytics] = useState<VaultAnalytics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabId>('overview');
  const [exporting, setExporting] = useState(false);

  const loadAnalytics = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<VaultAnalytics>('get_vault_analytics');
      setAnalytics(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAnalytics();
  }, [loadAnalytics]);

  const handleExportReport = useCallback(async () => {
    setExporting(true);
    try {
      const path = await invoke<string>('export_analytics_report');
      // Show success toast
    } catch {
      // Show error
    } finally {
      setExporting(false);
    }
  }, []);

  if (loading) {
    return (
      <div className="page-loading" role="status" aria-live="polite">
        <span className="spinner" aria-hidden="true" />
        {t('common.loading')}
      </div>
    );
  }

  if (error || !analytics) {
    return (
      <div className="page-error" role="alert">
        <p>{error ?? t('common.error')}</p>
        <button className="btn btn-primary" onClick={loadAnalytics}>
          {t('common.retry', 'Retry')}
        </button>
      </div>
    );
  }

  const { strength_distribution: sd, security_summary: ss } = analytics;
  const totalWithPwd = sd.very_weak + sd.weak + sd.fair + sd.strong + sd.very_strong;
  const pctStrong =
    totalWithPwd > 0 ? Math.round(((sd.strong + sd.very_strong) / totalWithPwd) * 100) : 0;

  const healthColor =
    ss.health_score >= 80 ? 'health-good' : ss.health_score >= 60 ? 'health-fair' : 'health-poor';

  return (
    <div className="page-container" role="main" aria-label={t('analytics.title')}>
      {/* Header */}
      <div className="page-header">
        <div className="page-header-row">
          <div>
            <h1 className="page-title">📊 {t('analytics.title')}</h1>
            <p className="page-subtitle">{t('analytics.subtitle')}</p>
          </div>
          <div className="page-header-actions">
            <button
              className="btn btn-secondary"
              onClick={loadAnalytics}
              aria-label={t('common.refresh', 'Refresh')}
            >
              🔄
            </button>
            <button
              className="btn btn-primary"
              onClick={handleExportReport}
              disabled={exporting}
              aria-busy={exporting}
            >
              {exporting ? t('analytics.exportingReport') : `📄 ${t('analytics.exportReport')}`}
            </button>
          </div>
        </div>
        <span className="badge badge-exclusive">✨ {t('analytics.uniqueFeature')}</span>
      </div>

      {/* Tabs */}
      <div className="tab-bar" role="tablist">
        {(['overview', 'strength', 'timeline', 'access', 'features'] as TabId[]).map(tab => (
          <button
            key={tab}
            role="tab"
            aria-selected={activeTab === tab}
            className={`tab ${activeTab === tab ? 'tab-active' : ''}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab === 'overview' && '🏠 Overview'}
            {tab === 'strength' && '💪 Strength'}
            {tab === 'timeline' && '📅 Timeline'}
            {tab === 'access' && '🔑 Access'}
            {tab === 'features' && '⚙️ Features'}
          </button>
        ))}
      </div>

      <div className="page-content">
        {/* OVERVIEW TAB */}
        {activeTab === 'overview' && (
          <div className="analytics-overview">
            {/* Health Score */}
            <div
              className={`health-score-card ${healthColor}`}
              aria-label={`Health score: ${ss.health_score}`}
            >
              <div className="health-score-circle">
                <span className="health-score-number">{ss.health_score}</span>
                <span className="health-score-label">/100</span>
              </div>
              <div className="health-score-info">
                <h3>Vault Health Score</h3>
                <p>{analytics.total_entries} entries analyzed</p>
              </div>
            </div>

            {/* Summary Cards */}
            <div className="summary-grid" role="list">
              <SummaryCard
                icon="🔑"
                label={t('statistics.totalEntries')}
                value={analytics.total_entries}
                color="blue"
              />
              <SummaryCard
                icon="💪"
                label="Strong Passwords"
                value={`${pctStrong}%`}
                color={pctStrong >= 70 ? 'green' : pctStrong >= 40 ? 'yellow' : 'red'}
              />
              <SummaryCard
                icon="⚠️"
                label={t('health.weakPasswords')}
                value={ss.weak_count}
                color={ss.weak_count === 0 ? 'green' : 'red'}
              />
              <SummaryCard
                icon="🔄"
                label={t('health.reusedPasswords')}
                value={ss.reused_count}
                color={ss.reused_count === 0 ? 'green' : 'orange'}
              />
              <SummaryCard
                icon="⏰"
                label={t('health.expiredEntries')}
                value={ss.expired_count}
                color={ss.expired_count === 0 ? 'green' : 'red'}
              />
              <SummaryCard
                icon="🚨"
                label={t('health.breachCheck')}
                value={ss.breached_count}
                color={ss.breached_count === 0 ? 'green' : 'red'}
              />
            </div>

            {/* Password Age */}
            <section className="analytics-section" aria-labelledby="age-label">
              <h3 id="age-label">🕐 Password Age</h3>
              <div className="age-stats">
                <div className="age-stat">
                  <span className="age-stat-value">
                    {Math.round(analytics.password_age.average_days)}
                  </span>
                  <span className="age-stat-label">days average</span>
                </div>
                <div className="age-stat">
                  <span className="age-stat-value">{analytics.password_age.older_than_1_year}</span>
                  <span className="age-stat-label">older than 1 year</span>
                </div>
                <div className="age-stat">
                  <span className="age-stat-value">
                    {analytics.password_age.changed_last_30_days}
                  </span>
                  <span className="age-stat-label">changed last 30 days</span>
                </div>
              </div>
            </section>
          </div>
        )}

        {/* STRENGTH TAB */}
        {activeTab === 'strength' && (
          <section className="analytics-section" aria-labelledby="strength-label">
            <h3 id="strength-label">💪 {t('analytics.passwordStrength')}</h3>
            <div className="strength-bars">
              <StrengthBar
                label={t('analytics.strengthVeryWeak')}
                count={sd.very_weak}
                total={analytics.total_entries}
                color="#ef4444"
              />
              <StrengthBar
                label={t('analytics.strengthWeak')}
                count={sd.weak}
                total={analytics.total_entries}
                color="#f97316"
              />
              <StrengthBar
                label={t('analytics.strengthFair')}
                count={sd.fair}
                total={analytics.total_entries}
                color="#eab308"
              />
              <StrengthBar
                label={t('analytics.strengthStrong')}
                count={sd.strong}
                total={analytics.total_entries}
                color="#22c55e"
              />
              <StrengthBar
                label={t('analytics.strengthVeryStrong')}
                count={sd.very_strong}
                total={analytics.total_entries}
                color="#16a34a"
              />
              <StrengthBar
                label="No Password"
                count={sd.no_password}
                total={analytics.total_entries}
                color="#6b7280"
              />
            </div>
          </section>
        )}

        {/* TIMELINE TAB */}
        {activeTab === 'timeline' && (
          <section className="analytics-section" aria-labelledby="timeline-label">
            <h3 id="timeline-label">📅 {t('analytics.entryTimeline')}</h3>
            <div className="timeline-chart" role="img" aria-label="Entry creation timeline">
              {analytics.creation_timeline.map(point => (
                <div key={`${point.year}-${point.month}`} className="timeline-bar-wrapper">
                  <div
                    className="timeline-bar"
                    style={{
                      height: `${Math.max(4, (point.count / Math.max(...analytics.creation_timeline.map(p => p.count), 1)) * 120)}px`,
                    }}
                    aria-label={`${point.label}: ${point.count} entries`}
                    title={`${point.label}: ${point.count}`}
                  />
                  <span className="timeline-label">{point.label}</span>
                  <span className="timeline-count">{point.count}</span>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* ACCESS TAB */}
        {activeTab === 'access' && (
          <section className="analytics-section" aria-labelledby="access-label">
            <h3 id="access-label">🔑 {t('analytics.mostAccessed')}</h3>
            {analytics.most_accessed.length === 0 ? (
              <p className="empty-state">{t('analytics.noData')}</p>
            ) : (
              <ol className="most-accessed-list">
                {analytics.most_accessed.map((entry, i) => (
                  <li key={entry.uuid} className="most-accessed-item">
                    <span className="rank">#{i + 1}</span>
                    <span className="entry-title">{entry.title}</span>
                    <span className="access-count">
                      {entry.access_count}{' '}
                      {t('analytics.totalCopies', { count: '' }).replace('{{count}} ', '')}
                    </span>
                  </li>
                ))}
              </ol>
            )}
          </section>
        )}

        {/* FEATURES TAB */}
        {activeTab === 'features' && (
          <section className="analytics-section" aria-labelledby="features-label">
            <h3 id="features-label">⚙️ Feature Usage</h3>
            <div className="feature-grid">
              <FeatureCard
                icon="🔐"
                label="OTP / 2FA"
                count={analytics.feature_usage.with_otp}
                total={analytics.total_entries}
              />
              <FeatureCard
                icon="🗝️"
                label="Passkeys"
                count={analytics.feature_usage.with_passkey}
                total={analytics.total_entries}
              />
              <FeatureCard
                icon="🖥️"
                label="SSH Keys"
                count={analytics.feature_usage.with_ssh_key}
                total={analytics.total_entries}
              />
              <FeatureCard
                icon="📎"
                label="Attachments"
                count={analytics.feature_usage.with_attachment}
                total={analytics.total_entries}
              />
              <FeatureCard
                icon="⭐"
                label="Favorites"
                count={analytics.feature_usage.favorites}
                total={analytics.total_entries}
              />
              <FeatureCard
                icon="⏰"
                label="With Expiry"
                count={analytics.feature_usage.with_expiry}
                total={analytics.total_entries}
              />
            </div>
          </section>
        )}
      </div>
    </div>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function SummaryCard({
  icon,
  label,
  value,
  color,
}: {
  icon: string;
  label: string;
  value: number | string;
  color: string;
}) {
  return (
    <div className={`summary-card summary-card-${color}`} role="listitem">
      <span className="summary-icon" aria-hidden="true">
        {icon}
      </span>
      <div className="summary-content">
        <span className="summary-value">{value}</span>
        <span className="summary-label">{label}</span>
      </div>
    </div>
  );
}

function StrengthBar({
  label,
  count,
  total,
  color,
}: {
  label: string;
  count: number;
  total: number;
  color: string;
}) {
  const pct = total > 0 ? Math.round((count / total) * 100) : 0;
  return (
    <div
      className="strength-bar-row"
      role="meter"
      aria-valuenow={pct}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={`${label}: ${count} (${pct}%)`}
    >
      <span className="strength-bar-label">{label}</span>
      <div className="strength-bar-track">
        <div className="strength-bar-fill" style={{ width: `${pct}%`, backgroundColor: color }} />
      </div>
      <span className="strength-bar-count">
        {count} ({pct}%)
      </span>
    </div>
  );
}

function FeatureCard({
  icon,
  label,
  count,
  total,
}: {
  icon: string;
  label: string;
  count: number;
  total: number;
}) {
  const pct = total > 0 ? Math.round((count / total) * 100) : 0;
  return (
    <div className="feature-card">
      <span className="feature-icon" aria-hidden="true">
        {icon}
      </span>
      <span className="feature-label">{label}</span>
      <span className="feature-count">{count}</span>
      <span className="feature-pct">{pct}%</span>
    </div>
  );
}
