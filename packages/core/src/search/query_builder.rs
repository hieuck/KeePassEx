//! Search Filter Builder — Convert NlQuery into executable vault search filters

use super::nl_parser::{EntryFeature, NlIntent, NlQuery, StrengthFilter, TimeFilter};
use chrono::{DateTime, Duration, Utc};

/// Executable search filter for vault entries
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    /// Free text search (title, username, URL, notes)
    pub text: Option<String>,
    /// Filter by group name/UUID
    pub group: Option<String>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Only expired entries
    pub expired_only: bool,
    /// Only entries expiring within N days
    pub expiring_within_days: Option<u32>,
    /// Only weak passwords (score < threshold)
    pub weak_only: bool,
    /// Only reused passwords
    pub reused_only: bool,
    /// Only entries with no password
    pub no_password_only: bool,
    /// Only breached entries
    pub breached_only: bool,
    /// Only favorites
    pub favorites_only: bool,
    /// Only recently used (last N days)
    pub recently_used_days: Option<u32>,
    /// Created after this date
    pub created_after: Option<DateTime<Utc>>,
    /// Created before this date
    pub created_before: Option<DateTime<Utc>>,
    /// Modified after this date
    pub modified_after: Option<DateTime<Utc>>,
    /// Modified before this date
    pub modified_before: Option<DateTime<Utc>>,
    /// Not accessed since this date
    pub not_accessed_since: Option<DateTime<Utc>>,
    /// Required features
    pub has_otp: Option<bool>,
    pub has_passkey: Option<bool>,
    pub has_ssh_key: Option<bool>,
    pub has_attachment: Option<bool>,
    /// Minimum password strength (0-4)
    pub min_strength: Option<u8>,
    /// Maximum password strength (0-4)
    pub max_strength: Option<u8>,
    /// Sort order
    pub sort: SortOrder,
    /// Maximum results (0 = unlimited)
    pub limit: usize,
}

/// Sort order for search results
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SortOrder {
    #[default]
    Relevance,
    TitleAsc,
    TitleDesc,
    ModifiedDesc,
    ModifiedAsc,
    CreatedDesc,
    CreatedAsc,
    StrengthAsc,
    StrengthDesc,
}

/// Build a `SearchFilter` from a parsed `NlQuery`.
pub fn build_search_filter(query: &NlQuery) -> SearchFilter {
    let now = Utc::now();
    let mut filter = SearchFilter::default();

    // Apply group filter
    filter.group = query.group.clone();

    // Apply tag filters
    filter.tags = query.tags.clone();

    // Apply feature filters
    for feature in &query.features {
        match feature {
            EntryFeature::Otp => filter.has_otp = Some(true),
            EntryFeature::Passkey => filter.has_passkey = Some(true),
            EntryFeature::SshKey => filter.has_ssh_key = Some(true),
            EntryFeature::Attachment => filter.has_attachment = Some(true),
            EntryFeature::CustomField => {}
        }
    }

    // Apply strength filter
    if let Some(strength) = &query.strength_filter {
        match strength {
            StrengthFilter::VeryWeak => {
                filter.max_strength = Some(0);
            }
            StrengthFilter::Weak => {
                filter.min_strength = Some(0);
                filter.max_strength = Some(1);
            }
            StrengthFilter::Fair => {
                filter.min_strength = Some(2);
                filter.max_strength = Some(2);
            }
            StrengthFilter::Strong => {
                filter.min_strength = Some(3);
                filter.max_strength = Some(3);
            }
            StrengthFilter::VeryStrong => {
                filter.min_strength = Some(4);
            }
        }
    }

    // Apply intent-specific filters
    match &query.intent {
        NlIntent::All => {}

        NlIntent::Expired => {
            filter.expired_only = true;
            filter.sort = SortOrder::ModifiedDesc;
        }

        NlIntent::ExpiringSoon { days } => {
            filter.expiring_within_days = Some(*days);
            filter.sort = SortOrder::ModifiedAsc;
        }

        NlIntent::Weak => {
            filter.weak_only = true;
            filter.sort = SortOrder::StrengthAsc;
        }

        NlIntent::Reused => {
            filter.reused_only = true;
        }

        NlIntent::NoPassword => {
            filter.no_password_only = true;
        }

        NlIntent::Breached => {
            filter.breached_only = true;
        }

        NlIntent::Favorites => {
            filter.favorites_only = true;
            filter.sort = SortOrder::TitleAsc;
        }

        NlIntent::Recent => {
            filter.recently_used_days = Some(7);
            filter.sort = SortOrder::ModifiedDesc;
        }

        NlIntent::WithFeature(_) => {
            // Already handled above via features loop
            filter.sort = SortOrder::TitleAsc;
        }

        NlIntent::CreatedIn(tf) => {
            let (after, before) = time_filter_to_range(tf, now);
            filter.created_after = after;
            filter.created_before = before;
            filter.sort = SortOrder::CreatedDesc;
        }

        NlIntent::ModifiedIn(tf) => {
            let (after, before) = time_filter_to_range(tf, now);
            filter.modified_after = after;
            filter.modified_before = before;
            filter.sort = SortOrder::ModifiedDesc;
        }

        NlIntent::NotUsedIn(tf) => {
            // "not used in N months" = not accessed since N months ago
            let cutoff = match tf {
                TimeFilter::LastNDays(n) => now - Duration::days(*n as i64),
                TimeFilter::LastNMonths(n) => now - Duration::days(*n as i64 * 30),
                TimeFilter::LastNYears(n) => now - Duration::days(*n as i64 * 365),
                TimeFilter::LastMonth => now - Duration::days(30),
                TimeFilter::LastWeek => now - Duration::days(7),
                _ => now - Duration::days(90),
            };
            filter.not_accessed_since = Some(cutoff);
            filter.sort = SortOrder::ModifiedAsc;
        }

        NlIntent::Search(text) => {
            filter.text = Some(text.clone());
            filter.sort = SortOrder::Relevance;
        }
    }

    // Apply free text if present and not already set
    if filter.text.is_none() {
        if let Some(text) = &query.text {
            if !text.is_empty() {
                filter.text = Some(text.clone());
            }
        }
    }

    filter
}

/// Convert a TimeFilter to (after, before) DateTime range
fn time_filter_to_range(
    tf: &TimeFilter,
    now: DateTime<Utc>,
) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    match tf {
        TimeFilter::Today => {
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let start = DateTime::from_naive_utc_and_offset(start, Utc);
            (Some(start), None)
        }
        TimeFilter::Yesterday => {
            let yesterday = now - Duration::days(1);
            let start = yesterday.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let end = yesterday.date_naive().and_hms_opt(23, 59, 59).unwrap();
            (
                Some(DateTime::from_naive_utc_and_offset(start, Utc)),
                Some(DateTime::from_naive_utc_and_offset(end, Utc)),
            )
        }
        TimeFilter::ThisWeek => {
            let start = now - Duration::days(now.weekday().num_days_from_monday() as i64);
            let start = start.date_naive().and_hms_opt(0, 0, 0).unwrap();
            (Some(DateTime::from_naive_utc_and_offset(start, Utc)), None)
        }
        TimeFilter::LastWeek => {
            let end = now - Duration::days(now.weekday().num_days_from_monday() as i64);
            let start = end - Duration::days(7);
            (
                Some(DateTime::from_naive_utc_and_offset(
                    start.date_naive().and_hms_opt(0, 0, 0).unwrap(),
                    Utc,
                )),
                Some(DateTime::from_naive_utc_and_offset(
                    end.date_naive().and_hms_opt(23, 59, 59).unwrap(),
                    Utc,
                )),
            )
        }
        TimeFilter::ThisMonth => {
            let start = now
                .date_naive()
                .with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            (Some(DateTime::from_naive_utc_and_offset(start, Utc)), None)
        }
        TimeFilter::LastMonth => {
            let first_this_month = now.date_naive().with_day(1).unwrap();
            let last_month_end = first_this_month - Duration::days(1);
            let last_month_start = last_month_end.with_day(1).unwrap();
            (
                Some(DateTime::from_naive_utc_and_offset(
                    last_month_start.and_hms_opt(0, 0, 0).unwrap(),
                    Utc,
                )),
                Some(DateTime::from_naive_utc_and_offset(
                    last_month_end.and_hms_opt(23, 59, 59).unwrap(),
                    Utc,
                )),
            )
        }
        TimeFilter::LastNDays(n) => {
            let start = now - Duration::days(*n as i64);
            (Some(start), None)
        }
        TimeFilter::LastNMonths(n) => {
            let start = now - Duration::days(*n as i64 * 30);
            (Some(start), None)
        }
        TimeFilter::LastNYears(n) => {
            let start = now - Duration::days(*n as i64 * 365);
            (Some(start), None)
        }
    }
}

// Need chrono::Datelike for with_day
use chrono::Datelike;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::nl_parser::parse_nl_query;

    #[test]
    fn test_build_expired_filter() {
        let q = parse_nl_query("show expired entries");
        let f = build_search_filter(&q);
        assert!(f.expired_only);
        assert_eq!(f.sort, SortOrder::ModifiedDesc);
    }

    #[test]
    fn test_build_weak_filter() {
        let q = parse_nl_query("find weak passwords");
        let f = build_search_filter(&q);
        assert!(f.weak_only);
        assert_eq!(f.sort, SortOrder::StrengthAsc);
    }

    #[test]
    fn test_build_otp_filter() {
        let q = parse_nl_query("entries with OTP");
        let f = build_search_filter(&q);
        assert_eq!(f.has_otp, Some(true));
    }

    #[test]
    fn test_build_group_filter() {
        let q = parse_nl_query("show entries in group Banking");
        let f = build_search_filter(&q);
        assert_eq!(f.group, Some("banking".to_string()));
    }

    #[test]
    fn test_build_favorites_filter() {
        let q = parse_nl_query("show favorites");
        let f = build_search_filter(&q);
        assert!(f.favorites_only);
    }

    #[test]
    fn test_build_text_search() {
        let q = parse_nl_query("github");
        let f = build_search_filter(&q);
        assert!(f.text.is_some());
        assert_eq!(f.sort, SortOrder::Relevance);
    }

    #[test]
    fn test_build_created_last_month() {
        let q = parse_nl_query("entries created last month");
        let f = build_search_filter(&q);
        assert!(f.created_after.is_some());
        assert_eq!(f.sort, SortOrder::CreatedDesc);
    }

    #[test]
    fn test_build_not_used_filter() {
        let q = parse_nl_query("passwords not used in 6 months");
        let f = build_search_filter(&q);
        assert!(f.not_accessed_since.is_some());
    }
}
