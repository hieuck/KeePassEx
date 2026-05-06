//! Password Policy tests

use crate::password_policy::*;

fn make_policy(rules: Vec<PolicyRule>) -> PasswordPolicy {
    PasswordPolicy {
        id: "test".to_string(),
        name: "Test Policy".to_string(),
        description: String::new(),
        enabled: true,
        scope: PolicyScope::Global,
        rules,
    }
}

#[test]
fn test_min_length_pass() {
    let policy = make_policy(vec![PolicyRule::MinLength(8)]);
    let result = evaluate_policy(&policy, "password1", None, None, None);
    assert!(result.passed);
}

#[test]
fn test_min_length_fail() {
    let policy = make_policy(vec![PolicyRule::MinLength(12)]);
    let result = evaluate_policy(&policy, "short", None, None, None);
    assert!(!result.passed);
    assert_eq!(result.violations[0].rule, "min_length");
}

#[test]
fn test_require_uppercase_pass() {
    let policy = make_policy(vec![PolicyRule::RequireUppercase(1)]);
    let result = evaluate_policy(&policy, "Password1", None, None, None);
    assert!(result.passed);
}

#[test]
fn test_require_uppercase_fail() {
    let policy = make_policy(vec![PolicyRule::RequireUppercase(1)]);
    let result = evaluate_policy(&policy, "password1", None, None, None);
    assert!(!result.passed);
}

#[test]
fn test_require_digits_fail() {
    let policy = make_policy(vec![PolicyRule::RequireDigits(2)]);
    let result = evaluate_policy(&policy, "Password1", None, None, None);
    assert!(!result.passed);
    assert_eq!(result.violations[0].rule, "require_digits");
}

#[test]
fn test_require_special_pass() {
    let policy = make_policy(vec![PolicyRule::RequireSpecial(1)]);
    let result = evaluate_policy(&policy, "Password1!", None, None, None);
    assert!(result.passed);
}

#[test]
fn test_forbid_username_fail() {
    let policy = make_policy(vec![PolicyRule::ForbidUsername]);
    let result = evaluate_policy(&policy, "alice123", Some("alice"), None, None);
    assert!(!result.passed);
    assert_eq!(result.violations[0].rule, "forbid_username");
}

#[test]
fn test_forbid_username_pass() {
    let policy = make_policy(vec![PolicyRule::ForbidUsername]);
    let result = evaluate_policy(&policy, "Tr0ub4dor&3", Some("alice"), None, None);
    assert!(result.passed);
}

#[test]
fn test_forbid_common_fail() {
    let policy = make_policy(vec![PolicyRule::ForbidCommon]);
    let result = evaluate_policy(&policy, "password", None, None, None);
    assert!(!result.passed);
}

#[test]
fn test_forbid_common_pass() {
    let policy = make_policy(vec![PolicyRule::ForbidCommon]);
    let result = evaluate_policy(&policy, "Tr0ub4dor&3xY!", None, None, None);
    assert!(result.passed);
}

#[test]
fn test_max_age_warning() {
    let policy = make_policy(vec![PolicyRule::MaxAgeDays(90)]);
    let result = evaluate_policy(&policy, "anypassword", None, None, Some(100));
    // MaxAgeDays produces a warning, not a violation
    assert!(result.passed);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0].rule, "max_age");
}

#[test]
fn test_max_age_not_triggered() {
    let policy = make_policy(vec![PolicyRule::MaxAgeDays(90)]);
    let result = evaluate_policy(&policy, "anypassword", None, None, Some(30));
    assert!(result.passed);
    assert_eq!(result.warnings.len(), 0);
}

#[test]
fn test_min_entropy_fail() {
    let policy = make_policy(vec![PolicyRule::MinEntropy(60.0)]);
    let result = evaluate_policy(&policy, "abc", None, None, None);
    assert!(!result.passed);
    assert_eq!(result.violations[0].rule, "min_entropy");
}

#[test]
fn test_min_entropy_pass() {
    let policy = make_policy(vec![PolicyRule::MinEntropy(50.0)]);
    let result = evaluate_policy(&policy, "Tr0ub4dor&3xY!", None, None, None);
    assert!(result.passed);
}

#[test]
fn test_sequential_detection_fail() {
    let policy = make_policy(vec![PolicyRule::ForbidSequential(3)]);
    let result = evaluate_policy(&policy, "abc123", None, None, None);
    assert!(!result.passed);
}

#[test]
fn test_sequential_detection_pass() {
    let policy = make_policy(vec![PolicyRule::ForbidSequential(3)]);
    let result = evaluate_policy(&policy, "axb2cy", None, None, None);
    assert!(result.passed);
}

#[test]
fn test_repeated_detection_fail() {
    let policy = make_policy(vec![PolicyRule::ForbidRepeated(3)]);
    let result = evaluate_policy(&policy, "aaab", None, None, None);
    assert!(!result.passed);
}

#[test]
fn test_multiple_violations() {
    let policy = make_policy(vec![
        PolicyRule::MinLength(12),
        PolicyRule::RequireUppercase(1),
        PolicyRule::RequireDigits(1),
    ]);
    let result = evaluate_policy(&policy, "short", None, None, None);
    assert!(!result.passed);
    assert!(result.violations.len() >= 2);
}

#[test]
fn test_strong_policy_passes_strong_password() {
    let policy = strong_policy();
    let result = evaluate_policy(&policy, "Tr0ub4dor&3xY!", None, None, None);
    assert!(
        result.passed,
        "Strong password should pass strong policy. Violations: {:?}",
        result.violations
    );
}

#[test]
fn test_evaluate_all_policies_only_enabled() {
    let mut policies = built_in_policies();
    // Only enable basic policy
    for p in &mut policies {
        p.enabled = p.id == "builtin.basic";
    }
    let results = evaluate_all_policies(&policies, "password", None, None, None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].policy_id, "builtin.basic");
}

#[test]
fn test_policy_manager_enable_disable() {
    let mut manager = PolicyManager::new();
    assert_eq!(manager.enabled().count(), 1); // Only basic enabled by default

    manager.enable("builtin.strong").unwrap();
    assert_eq!(manager.enabled().count(), 2);

    manager.disable("builtin.basic").unwrap();
    assert_eq!(manager.enabled().count(), 1);
}

#[test]
fn test_policy_manager_add_custom() {
    let mut manager = PolicyManager::new();
    let custom = PasswordPolicy {
        id: "custom.test".to_string(),
        name: "Custom Test".to_string(),
        description: String::new(),
        enabled: true,
        scope: PolicyScope::Global,
        rules: vec![PolicyRule::MinLength(16)],
    };
    manager.add(custom).unwrap();
    assert!(manager.get("custom.test").is_some());
}

#[test]
fn test_forbid_title_fail() {
    let policy = make_policy(vec![PolicyRule::ForbidTitle]);
    let result = evaluate_policy(&policy, "github123", None, Some("GitHub"), None);
    assert!(!result.passed);
}

#[test]
fn test_forbid_chars() {
    let policy = make_policy(vec![PolicyRule::ForbidChars("@#".to_string())]);
    let result = evaluate_policy(&policy, "pass@word", None, None, None);
    assert!(!result.passed);
    assert_eq!(result.violations[0].rule, "forbid_chars");
}

#[test]
fn test_violations_have_vi_messages() {
    let policy = make_policy(vec![PolicyRule::MinLength(20)]);
    let result = evaluate_policy(&policy, "short", None, None, None);
    assert!(!result.violations[0].message_vi.is_empty());
}
