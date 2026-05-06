//! Password Policy Engine
//!
//! Defines and enforces password policies for vault entries.
//! Policies can be applied globally (vault-wide) or per-group.
//! The plugin system can also define custom policies.
//!
//! # Use cases
//! - Enterprise: enforce minimum length, complexity requirements
//! - Personal: warn when passwords are older than 365 days
//! - Per-service: different rules for banking vs social media

use crate::error::Result;
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A password policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub rules: Vec<PolicyRule>,
    /// Apply to all entries, or only entries in specific groups
    pub scope: PolicyScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyScope {
    /// Apply to all entries in the vault
    Global,
    /// Apply only to entries in these groups (by UUID)
    Groups(Vec<String>),
    /// Apply only to entries with these tags
    Tags(Vec<String>),
}

/// A single policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyRule {
    /// Minimum password length
    MinLength(usize),
    /// Maximum password length
    MaxLength(usize),
    /// Require at least N uppercase letters
    RequireUppercase(usize),
    /// Require at least N lowercase letters
    RequireLowercase(usize),
    /// Require at least N digits
    RequireDigits(usize),
    /// Require at least N special characters
    RequireSpecial(usize),
    /// Forbid these specific characters
    ForbidChars(String),
    /// Require these specific characters to be present
    RequireChars(String),
    /// Maximum password age in days (warn if older)
    MaxAgeDays(u32),
    /// Minimum entropy in bits
    MinEntropy(f64),
    /// Forbid common passwords (checked against offline list)
    ForbidCommon,
    /// Forbid passwords that contain the username
    ForbidUsername,
    /// Forbid passwords that contain the entry title
    ForbidTitle,
    /// Forbid sequential characters (e.g. "abc", "123")
    ForbidSequential(usize),
    /// Forbid repeated characters (e.g. "aaa")
    ForbidRepeated(usize),
    /// Custom regex pattern the password must match
    MatchPattern {
        pattern: String,
        description: String,
    },
    /// Custom regex pattern the password must NOT match
    ForbidPattern {
        pattern: String,
        description: String,
    },
}

/// Result of evaluating a policy against a password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub policy_id: String,
    pub policy_name: String,
    pub passed: bool,
    pub violations: Vec<PolicyViolation>,
    pub warnings: Vec<PolicyWarning>,
}

/// A policy rule violation (hard failure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub message: String,
    pub message_vi: String,
}

/// A policy warning (soft failure — informational)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWarning {
    pub rule: String,
    pub message: String,
    pub message_vi: String,
}

// ─── Built-in Policies ────────────────────────────────────────────────────────

/// Returns the default set of built-in password policies
pub fn built_in_policies() -> Vec<PasswordPolicy> {
    vec![
        basic_policy(),
        strong_policy(),
        enterprise_policy(),
        banking_policy(),
    ]
}

fn basic_policy() -> PasswordPolicy {
    PasswordPolicy {
        id: "builtin.basic".to_string(),
        name: "Basic".to_string(),
        description: "Minimum security requirements".to_string(),
        enabled: true,
        scope: PolicyScope::Global,
        rules: vec![PolicyRule::MinLength(8), PolicyRule::ForbidCommon],
    }
}

pub(crate) fn strong_policy() -> PasswordPolicy {
    PasswordPolicy {
        id: "builtin.strong".to_string(),
        name: "Strong".to_string(),
        description: "Strong password requirements for most accounts".to_string(),
        enabled: false,
        scope: PolicyScope::Global,
        rules: vec![
            PolicyRule::MinLength(12),
            PolicyRule::RequireUppercase(1),
            PolicyRule::RequireLowercase(1),
            PolicyRule::RequireDigits(1),
            PolicyRule::RequireSpecial(1),
            PolicyRule::ForbidCommon,
            PolicyRule::ForbidUsername,
            PolicyRule::MinEntropy(50.0),
        ],
    }
}

fn enterprise_policy() -> PasswordPolicy {
    PasswordPolicy {
        id: "builtin.enterprise".to_string(),
        name: "Enterprise".to_string(),
        description: "Enterprise-grade requirements with rotation policy".to_string(),
        enabled: false,
        scope: PolicyScope::Global,
        rules: vec![
            PolicyRule::MinLength(14),
            PolicyRule::MaxLength(128),
            PolicyRule::RequireUppercase(2),
            PolicyRule::RequireLowercase(2),
            PolicyRule::RequireDigits(2),
            PolicyRule::RequireSpecial(1),
            PolicyRule::ForbidCommon,
            PolicyRule::ForbidUsername,
            PolicyRule::ForbidTitle,
            PolicyRule::ForbidSequential(3),
            PolicyRule::ForbidRepeated(3),
            PolicyRule::MaxAgeDays(90),
            PolicyRule::MinEntropy(70.0),
        ],
    }
}

fn banking_policy() -> PasswordPolicy {
    PasswordPolicy {
        id: "builtin.banking".to_string(),
        name: "Banking & Finance".to_string(),
        description: "High-security requirements for financial accounts".to_string(),
        enabled: false,
        scope: PolicyScope::Tags(vec!["banking".to_string(), "finance".to_string()]),
        rules: vec![
            PolicyRule::MinLength(16),
            PolicyRule::RequireUppercase(1),
            PolicyRule::RequireLowercase(1),
            PolicyRule::RequireDigits(2),
            PolicyRule::RequireSpecial(2),
            PolicyRule::ForbidCommon,
            PolicyRule::ForbidUsername,
            PolicyRule::MaxAgeDays(180),
            PolicyRule::MinEntropy(80.0),
        ],
    }
}

// ─── Policy Evaluator ─────────────────────────────────────────────────────────

/// Evaluate a password against a policy
pub fn evaluate_policy(
    policy: &PasswordPolicy,
    password: &str,
    username: Option<&str>,
    title: Option<&str>,
    password_age_days: Option<u32>,
) -> PolicyEvaluation {
    let mut violations = Vec::new();
    let mut warnings = Vec::new();

    for rule in &policy.rules {
        match rule {
            PolicyRule::MinLength(min) => {
                if password.len() < *min {
                    violations.push(PolicyViolation {
                        rule: "min_length".to_string(),
                        message: format!("Password must be at least {} characters", min),
                        message_vi: format!("Mật khẩu phải có ít nhất {} ký tự", min),
                    });
                }
            }

            PolicyRule::MaxLength(max) => {
                if password.len() > *max {
                    violations.push(PolicyViolation {
                        rule: "max_length".to_string(),
                        message: format!("Password must be at most {} characters", max),
                        message_vi: format!("Mật khẩu phải có tối đa {} ký tự", max),
                    });
                }
            }

            PolicyRule::RequireUppercase(n) => {
                let count = password.chars().filter(|c| c.is_uppercase()).count();
                if count < *n {
                    violations.push(PolicyViolation {
                        rule: "require_uppercase".to_string(),
                        message: format!(
                            "Password must contain at least {} uppercase letter(s)",
                            n
                        ),
                        message_vi: format!("Mật khẩu phải có ít nhất {} chữ hoa", n),
                    });
                }
            }

            PolicyRule::RequireLowercase(n) => {
                let count = password.chars().filter(|c| c.is_lowercase()).count();
                if count < *n {
                    violations.push(PolicyViolation {
                        rule: "require_lowercase".to_string(),
                        message: format!(
                            "Password must contain at least {} lowercase letter(s)",
                            n
                        ),
                        message_vi: format!("Mật khẩu phải có ít nhất {} chữ thường", n),
                    });
                }
            }

            PolicyRule::RequireDigits(n) => {
                let count = password.chars().filter(|c| c.is_ascii_digit()).count();
                if count < *n {
                    violations.push(PolicyViolation {
                        rule: "require_digits".to_string(),
                        message: format!("Password must contain at least {} digit(s)", n),
                        message_vi: format!("Mật khẩu phải có ít nhất {} chữ số", n),
                    });
                }
            }

            PolicyRule::RequireSpecial(n) => {
                let count = password.chars().filter(|c| !c.is_alphanumeric()).count();
                if count < *n {
                    violations.push(PolicyViolation {
                        rule: "require_special".to_string(),
                        message: format!(
                            "Password must contain at least {} special character(s)",
                            n
                        ),
                        message_vi: format!("Mật khẩu phải có ít nhất {} ký tự đặc biệt", n),
                    });
                }
            }

            PolicyRule::ForbidChars(chars) => {
                let found: Vec<char> = password.chars().filter(|c| chars.contains(*c)).collect();
                if !found.is_empty() {
                    violations.push(PolicyViolation {
                        rule: "forbid_chars".to_string(),
                        message: format!("Password contains forbidden characters: {:?}", found),
                        message_vi: format!("Mật khẩu chứa ký tự bị cấm: {:?}", found),
                    });
                }
            }

            PolicyRule::MaxAgeDays(max_days) => {
                if let Some(age) = password_age_days {
                    if age > *max_days {
                        warnings.push(PolicyWarning {
                            rule: "max_age".to_string(),
                            message: format!("Password is {} days old (max: {})", age, max_days),
                            message_vi: format!(
                                "Mật khẩu đã {} ngày tuổi (tối đa: {})",
                                age, max_days
                            ),
                        });
                    }
                }
            }

            PolicyRule::MinEntropy(min_bits) => {
                let entropy = calculate_entropy(password);
                if entropy < *min_bits {
                    violations.push(PolicyViolation {
                        rule: "min_entropy".to_string(),
                        message: format!(
                            "Password entropy is {:.0} bits (min: {:.0})",
                            entropy, min_bits
                        ),
                        message_vi: format!(
                            "Entropy mật khẩu là {:.0} bit (tối thiểu: {:.0})",
                            entropy, min_bits
                        ),
                    });
                }
            }

            PolicyRule::ForbidCommon => {
                // Check against a small list of very common passwords
                // In production, this would check against a larger offline list
                let common = [
                    "password",
                    "123456",
                    "qwerty",
                    "abc123",
                    "password1",
                    "111111",
                    "letmein",
                    "monkey",
                    "dragon",
                    "master",
                ];
                if common.iter().any(|&c| password.to_lowercase() == c) {
                    violations.push(PolicyViolation {
                        rule: "forbid_common".to_string(),
                        message: "Password is too common".to_string(),
                        message_vi: "Mật khẩu quá phổ biến".to_string(),
                    });
                }
            }

            PolicyRule::ForbidUsername => {
                if let Some(uname) = username {
                    if !uname.is_empty() && password.to_lowercase().contains(&uname.to_lowercase())
                    {
                        violations.push(PolicyViolation {
                            rule: "forbid_username".to_string(),
                            message: "Password must not contain the username".to_string(),
                            message_vi: "Mật khẩu không được chứa tên đăng nhập".to_string(),
                        });
                    }
                }
            }

            PolicyRule::ForbidTitle => {
                if let Some(t) = title {
                    if !t.is_empty()
                        && t.len() >= 4
                        && password.to_lowercase().contains(&t.to_lowercase())
                    {
                        violations.push(PolicyViolation {
                            rule: "forbid_title".to_string(),
                            message: "Password must not contain the entry title".to_string(),
                            message_vi: "Mật khẩu không được chứa tiêu đề mục".to_string(),
                        });
                    }
                }
            }

            PolicyRule::ForbidSequential(n) => {
                if has_sequential(password, *n) {
                    violations.push(PolicyViolation {
                        rule: "forbid_sequential".to_string(),
                        message: format!("Password contains {} or more sequential characters", n),
                        message_vi: format!("Mật khẩu chứa {} ký tự liên tiếp trở lên", n),
                    });
                }
            }

            PolicyRule::ForbidRepeated(n) => {
                if has_repeated(password, *n) {
                    violations.push(PolicyViolation {
                        rule: "forbid_repeated".to_string(),
                        message: format!("Password contains {} or more repeated characters", n),
                        message_vi: format!("Mật khẩu chứa {} ký tự lặp lại trở lên", n),
                    });
                }
            }

            PolicyRule::RequireChars(chars) => {
                let missing: Vec<char> = chars.chars().filter(|c| !password.contains(*c)).collect();
                if !missing.is_empty() {
                    violations.push(PolicyViolation {
                        rule: "require_chars".to_string(),
                        message: format!("Password must contain: {:?}", missing),
                        message_vi: format!("Mật khẩu phải chứa: {:?}", missing),
                    });
                }
            }

            PolicyRule::MatchPattern {
                pattern,
                description,
            } => {
                // Simplified pattern matching — production would use regex crate
                let _ = (pattern, description);
            }

            PolicyRule::ForbidPattern {
                pattern,
                description,
            } => {
                let _ = (pattern, description);
            }
        }
    }

    PolicyEvaluation {
        policy_id: policy.id.clone(),
        policy_name: policy.name.clone(),
        passed: violations.is_empty(),
        violations,
        warnings,
    }
}

/// Evaluate a password against multiple policies
pub fn evaluate_all_policies(
    policies: &[PasswordPolicy],
    password: &str,
    username: Option<&str>,
    title: Option<&str>,
    password_age_days: Option<u32>,
) -> Vec<PolicyEvaluation> {
    policies
        .iter()
        .filter(|p| p.enabled)
        .map(|p| evaluate_policy(p, password, username, title, password_age_days))
        .collect()
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn calculate_entropy(password: &str) -> f64 {
    let mut pool = 0usize;
    if password.chars().any(|c| c.is_lowercase()) {
        pool += 26;
    }
    if password.chars().any(|c| c.is_uppercase()) {
        pool += 26;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        pool += 10;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        pool += 32;
    }
    if pool == 0 {
        return 0.0;
    }
    (pool as f64).log2() * password.len() as f64
}

fn has_sequential(password: &str, n: usize) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < n {
        return false;
    }
    for window in chars.windows(n) {
        let all_sequential = window
            .windows(2)
            .all(|pair| (pair[1] as i32 - pair[0] as i32).abs() == 1);
        if all_sequential {
            return true;
        }
    }
    false
}

fn has_repeated(password: &str, n: usize) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < n {
        return false;
    }
    for window in chars.windows(n) {
        if window.iter().all(|&c| c == window[0]) {
            return true;
        }
    }
    false
}

// ─── Policy Manager ───────────────────────────────────────────────────────────

pub struct PolicyManager {
    policies: Vec<PasswordPolicy>,
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            policies: built_in_policies(),
        }
    }

    pub fn all(&self) -> &[PasswordPolicy] {
        &self.policies
    }

    pub fn get(&self, id: &str) -> Option<&PasswordPolicy> {
        self.policies.iter().find(|p| p.id == id)
    }

    pub fn enabled(&self) -> impl Iterator<Item = &PasswordPolicy> {
        self.policies.iter().filter(|p| p.enabled)
    }

    pub fn add(&mut self, policy: PasswordPolicy) -> Result<()> {
        if self.policies.iter().any(|p| p.id == policy.id) {
            return Err(crate::error::KeePassExError::Other(format!(
                "Policy '{}' already exists",
                policy.id
            )));
        }
        self.policies.push(policy);
        Ok(())
    }

    pub fn enable(&mut self, id: &str) -> Result<()> {
        self.policies
            .iter_mut()
            .find(|p| p.id == id)
            .map(|p| p.enabled = true)
            .ok_or_else(|| {
                crate::error::KeePassExError::Other(format!("Policy '{}' not found", id))
            })
    }

    pub fn disable(&mut self, id: &str) -> Result<()> {
        self.policies
            .iter_mut()
            .find(|p| p.id == id)
            .map(|p| p.enabled = false)
            .ok_or_else(|| {
                crate::error::KeePassExError::Other(format!("Policy '{}' not found", id))
            })
    }
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_length_violation() {
        let policy = PasswordPolicy {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            enabled: true,
            scope: PolicyScope::Global,
            rules: vec![PolicyRule::MinLength(12)],
        };
        let result = evaluate_policy(&policy, "short", None, None, None);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "min_length");
    }

    #[test]
    fn test_strong_password_passes() {
        let policy = strong_policy();
        let result = evaluate_policy(&policy, "Tr0ub4dor&3xY!", None, None, None);
        assert!(result.passed, "Violations: {:?}", result.violations);
    }

    #[test]
    fn test_forbid_username() {
        let policy = PasswordPolicy {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            enabled: true,
            scope: PolicyScope::Global,
            rules: vec![PolicyRule::ForbidUsername],
        };
        let result = evaluate_policy(&policy, "alice123", Some("alice"), None, None);
        assert!(!result.passed);
        assert_eq!(result.violations[0].rule, "forbid_username");
    }

    #[test]
    fn test_forbid_common() {
        let policy = PasswordPolicy {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            enabled: true,
            scope: PolicyScope::Global,
            rules: vec![PolicyRule::ForbidCommon],
        };
        let result = evaluate_policy(&policy, "password", None, None, None);
        assert!(!result.passed);
    }

    #[test]
    fn test_sequential_detection() {
        assert!(has_sequential("abc123", 3));
        assert!(has_sequential("xyz", 3));
        assert!(!has_sequential("axb", 3));
    }

    #[test]
    fn test_repeated_detection() {
        assert!(has_repeated("aaab", 3));
        assert!(!has_repeated("aab", 3));
    }

    #[test]
    fn test_entropy_calculation() {
        let entropy = calculate_entropy("Tr0ub4dor&3");
        assert!(entropy > 50.0, "Expected entropy > 50, got {}", entropy);
    }

    #[test]
    fn test_max_age_warning() {
        let policy = PasswordPolicy {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            enabled: true,
            scope: PolicyScope::Global,
            rules: vec![PolicyRule::MaxAgeDays(90)],
        };
        let result = evaluate_policy(&policy, "anypassword", None, None, Some(100));
        // MaxAgeDays produces a warning, not a violation
        assert!(result.passed);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_built_in_policies_count() {
        let policies = built_in_policies();
        assert_eq!(policies.len(), 4);
    }

    #[test]
    fn test_policy_manager() {
        let mut manager = PolicyManager::new();
        assert_eq!(manager.all().len(), 4);
        manager.enable("builtin.strong").unwrap();
        assert_eq!(manager.enabled().count(), 2); // basic + strong
    }
}
