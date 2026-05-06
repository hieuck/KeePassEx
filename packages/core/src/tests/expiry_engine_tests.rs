//! Expiry engine tests — uses the public analyze_rotation() and analyze_vault_rotations() APIs

use crate::expiry_engine::{
    analyze_rotation, analyze_vault_rotations, ExpiryInput, RotationUrgency,
};
use chrono::{Duration, Utc};

fn make_input(title: &str, age_days: i64, category: Option<&str>) -> ExpiryInput {
    ExpiryInput {
        uuid: uuid::Uuid::new_v4().to_string(),
        title: title.to_string(),
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
    assert_eq!(RotationUrgency::Expired.label_en(), "Expired");
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

#[test]
fn test_recommendation_contains_entry_info() {
    let input = make_input("Chase Bank", 95, Some("banking"));
    let rec = analyze_rotation(&input).unwrap();
    assert_eq!(rec.entry_title, "Chase Bank");
    assert!(rec.age_days > 0);
    assert!(rec.recommended_max_days > 0);
}

#[test]
fn test_urgency_color_hex() {
    assert_eq!(RotationUrgency::Fresh.color_hex(), "#16a34a");
    assert_eq!(RotationUrgency::Overdue.color_hex(), "#dc2626");
}
