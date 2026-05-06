//! Password Expiry Engine — proactive rotation reminders
//!
//! Tracks password age and generates rotation recommendations based on
//! entry category, site sensitivity, and user-defined policies.
//!
//! # Rotation recommendations (defaults)
//! - Banking/Crypto: 90 days
//! - Email/Work: 180 days
//! - Social/Shopping: 365 days
//! - Other: 365 days
//!
//! # Features
//! - Per-category rotation schedules
//! - Risk-based urgency scoring
//! - Batch expiry report for vault health dashboard
//! - EN/VI localized messages

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Rotation urgency level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RotationUrgency {
    /// Password is fresh — no action needed
    Fresh,
    /// Password is aging — consider rotating soon
    Aging,
    /// Password should be rotated soon (within 30 days of recommendation)
    Soon,
    /// Password is overdue for rotation
    Overdue,
    /// Password has been explicitly marked as expired
    Expired,
}

impl RotationUrgency {
    pub fn label_en(&self) -> &'static str {
        match self {
            Self::Fresh => "Fresh",
            Self::Aging => "Aging",
            Self::Soon => "Rotate Soon",
            Self::Overdue => "Overdue",
            Self::Expired => "Expired",
        }
    }

    pub fn label_vi(&self) -> &'static str {
        match self {
            Self::Fresh => "Còn mới",
            Self::Aging => "Đang cũ",
            Self::Soon => "Sắp cần đổi",
            Self::Overdue => "Quá hạn đổi",
            Self::Expired => "Đã hết hạn",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::Fresh => "#16a34a",   // green
            Self::Aging => "#d97706",   // amber
            Self::Soon => "#ea580c",    // orange
            Self::Overdue => "#dc2626", // red
            Self::Expired => "#7f1d1d", // dark red
        }
    }
}

/// Rotation recommendation for a single entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationRecommendation {
    pub entry_uuid: String,
    pub entry_title: String,
    pub urgency: RotationUrgency,
    /// Days since last password change
    pub age_days: i64,
    /// Recommended max age in days for this entry type
    pub recommended_max_days: i64,
    /// Days until/since recommendation threshold
    pub days_until_overdue: i64,
    /// Localized message (EN)
    pub message_en: String,
    /// Localized message (VI)
    pub message_vi: String,
}

/// Per-category rotation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationSchedule {
    /// Max age in days before "Aging" status
    pub aging_days: i64,
    /// Max age in days before "Soon" status
    pub soon_days: i64,
    /// Max age in days before "Overdue" status
    pub overdue_days: i64,
}

impl RotationSchedule {
    pub fn banking() -> Self {
        Self {
            aging_days: 60,
            soon_days: 75,
            overdue_days: 90,
        }
    }
    pub fn email() -> Self {
        Self {
            aging_days: 120,
            soon_days: 150,
            overdue_days: 180,
        }
    }
    pub fn social() -> Self {
        Self {
            aging_days: 270,
            soon_days: 330,
            overdue_days: 365,
        }
    }
    pub fn default_schedule() -> Self {
        Self {
            aging_days: 270,
            soon_days: 330,
            overdue_days: 365,
        }
    }
}

/// Input data for expiry analysis
#[derive(Debug, Clone)]
pub struct ExpiryInput {
    pub uuid: String,
    pub title: String,
    pub password_modified_at: DateTime<Utc>,
    pub explicit_expiry: Option<DateTime<Utc>>,
    pub category_hint: Option<String>, // "banking", "email", "social", etc.
    pub has_password: bool,
}

/// Analyze a single entry's rotation status
pub fn analyze_rotation(input: &ExpiryInput) -> Option<RotationRecommendation> {
    if !input.has_password {
        return None; // No password to rotate
    }

    let now = Utc::now();

    // Check explicit expiry first
    if let Some(expiry) = input.explicit_expiry {
        if expiry <= now {
            let age = (now - input.password_modified_at).num_days();
            return Some(RotationRecommendation {
                entry_uuid: input.uuid.clone(),
                entry_title: input.title.clone(),
                urgency: RotationUrgency::Expired,
                age_days: age,
                recommended_max_days: 0,
                days_until_overdue: (expiry - now).num_days(),
                message_en: format!(
                    "\"{}\" has expired. Change password immediately.",
                    input.title
                ),
                message_vi: format!("\"{}\" đã hết hạn. Đổi mật khẩu ngay.", input.title),
            });
        }
    }

    // Get rotation schedule based on category
    let schedule = get_schedule(input.category_hint.as_deref());
    let age_days = (now - input.password_modified_at).num_days();
    let days_until_overdue = schedule.overdue_days - age_days;

    let urgency = if age_days >= schedule.overdue_days {
        RotationUrgency::Overdue
    } else if age_days >= schedule.soon_days {
        RotationUrgency::Soon
    } else if age_days >= schedule.aging_days {
        RotationUrgency::Aging
    } else {
        RotationUrgency::Fresh
    };

    // Only return recommendation if action is needed
    if urgency == RotationUrgency::Fresh {
        return None;
    }

    let (message_en, message_vi) =
        build_messages(&input.title, &urgency, age_days, days_until_overdue);

    Some(RotationRecommendation {
        entry_uuid: input.uuid.clone(),
        entry_title: input.title.clone(),
        urgency,
        age_days,
        recommended_max_days: schedule.overdue_days,
        days_until_overdue,
        message_en,
        message_vi,
    })
}

/// Analyze all entries and return sorted recommendations
pub fn analyze_vault_rotations(entries: &[ExpiryInput]) -> Vec<RotationRecommendation> {
    let mut recommendations: Vec<RotationRecommendation> =
        entries.iter().filter_map(analyze_rotation).collect();

    // Sort by urgency (most urgent first), then by age (oldest first)
    recommendations.sort_by(|a, b| b.urgency.cmp(&a.urgency).then(b.age_days.cmp(&a.age_days)));

    recommendations
}

fn get_schedule(category: Option<&str>) -> RotationSchedule {
    match category {
        Some("banking") | Some("crypto") => RotationSchedule::banking(),
        Some("email") | Some("work") => RotationSchedule::email(),
        Some("social") | Some("shopping") | Some("entertainment") => RotationSchedule::social(),
        _ => RotationSchedule::default_schedule(),
    }
}

fn build_messages(
    title: &str,
    urgency: &RotationUrgency,
    age_days: i64,
    days_until_overdue: i64,
) -> (String, String) {
    match urgency {
        RotationUrgency::Overdue => (
            format!(
                "\"{}\" password is {} days old — overdue for rotation.",
                title, age_days
            ),
            format!(
                "Mật khẩu \"{}\" đã {} ngày — quá hạn cần đổi.",
                title, age_days
            ),
        ),
        RotationUrgency::Soon => (
            format!(
                "\"{}\" password should be rotated within {} days.",
                title,
                days_until_overdue.max(0)
            ),
            format!(
                "Mật khẩu \"{}\" nên đổi trong {} ngày tới.",
                title,
                days_until_overdue.max(0)
            ),
        ),
        RotationUrgency::Aging => (
            format!(
                "\"{}\" password is {} days old — consider rotating.",
                title, age_days
            ),
            format!(
                "Mật khẩu \"{}\" đã {} ngày — nên cân nhắc đổi.",
                title, age_days
            ),
        ),
        _ => (String::new(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(title: &str, age_days: i64, category: Option<&str>) -> ExpiryInput {
        ExpiryInput {
            uuid: "test-uuid".into(),
            title: title.into(),
            password_modified_at: Utc::now() - Duration::days(age_days),
            explicit_expiry: None,
            category_hint: category.map(|s| s.to_string()),
            has_password: true,
        }
    }

    #[test]
    fn test_fresh_password_no_recommendation() {
        let input = make_input("GitHub", 10, Some("development"));
        assert!(analyze_rotation(&input).is_none());
    }

    #[test]
    fn test_banking_overdue_at_90_days() {
        let input = make_input("Chase Bank", 95, Some("banking"));
        let rec = analyze_rotation(&input).unwrap();
        assert_eq!(rec.urgency, RotationUrgency::Overdue);
        assert_eq!(rec.age_days, 95);
    }

    #[test]
    fn test_banking_soon_at_80_days() {
        let input = make_input("Chase Bank", 80, Some("banking"));
        let rec = analyze_rotation(&input).unwrap();
        assert_eq!(rec.urgency, RotationUrgency::Soon);
    }

    #[test]
    fn test_banking_aging_at_65_days() {
        let input = make_input("Chase Bank", 65, Some("banking"));
        let rec = analyze_rotation(&input).unwrap();
        assert_eq!(rec.urgency, RotationUrgency::Aging);
    }

    #[test]
    fn test_social_fresh_at_200_days() {
        let input = make_input("Facebook", 200, Some("social"));
        assert!(analyze_rotation(&input).is_none());
    }

    #[test]
    fn test_social_overdue_at_400_days() {
        let input = make_input("Facebook", 400, Some("social"));
        let rec = analyze_rotation(&input).unwrap();
        assert_eq!(rec.urgency, RotationUrgency::Overdue);
    }

    #[test]
    fn test_explicit_expiry_expired() {
        let mut input = make_input("Test", 10, None);
        input.explicit_expiry = Some(Utc::now() - Duration::days(5));
        let rec = analyze_rotation(&input).unwrap();
        assert_eq!(rec.urgency, RotationUrgency::Expired);
    }

    #[test]
    fn test_no_password_no_recommendation() {
        let mut input = make_input("Note", 400, None);
        input.has_password = false;
        assert!(analyze_rotation(&input).is_none());
    }

    #[test]
    fn test_batch_sorted_by_urgency() {
        let entries = vec![
            make_input("Fresh", 10, Some("banking")),
            make_input("Overdue", 100, Some("banking")),
            make_input("Soon", 80, Some("banking")),
        ];
        let recs = analyze_vault_rotations(&entries);
        assert_eq!(recs.len(), 2); // Fresh excluded
        assert_eq!(recs[0].urgency, RotationUrgency::Overdue);
        assert_eq!(recs[1].urgency, RotationUrgency::Soon);
    }

    #[test]
    fn test_urgency_labels_en() {
        assert_eq!(RotationUrgency::Fresh.label_en(), "Fresh");
        assert_eq!(RotationUrgency::Overdue.label_en(), "Overdue");
    }

    #[test]
    fn test_urgency_labels_vi() {
        assert_eq!(RotationUrgency::Fresh.label_vi(), "Còn mới");
        assert_eq!(RotationUrgency::Overdue.label_vi(), "Quá hạn đổi");
    }

    #[test]
    fn test_messages_are_localized() {
        let input = make_input("Gmail", 200, Some("email"));
        let rec = analyze_rotation(&input).unwrap();
        assert!(!rec.message_en.is_empty());
        assert!(!rec.message_vi.is_empty());
        assert!(rec.message_vi.contains("Gmail"));
    }
}
