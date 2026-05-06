//! Password Policy Tauri commands

use crate::state::AppState;
use keepassex_core::password_policy::{
    built_in_policies, evaluate_all_policies, PasswordPolicy, PolicyEvaluation,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct PolicyDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub scope: String,
}

impl From<&PasswordPolicy> for PolicyDto {
    fn from(p: &PasswordPolicy) -> Self {
        let scope = match &p.scope {
            keepassex_core::password_policy::PolicyScope::Global => "Global".to_string(),
            keepassex_core::password_policy::PolicyScope::Groups(_) => "Groups".to_string(),
            keepassex_core::password_policy::PolicyScope::Tags(_) => "Tags".to_string(),
        };
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            enabled: p.enabled,
            scope,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ViolationDto {
    pub rule: String,
    pub message: String,
    pub message_vi: String,
}

#[derive(Debug, Serialize)]
pub struct EvaluationDto {
    pub policy_id: String,
    pub policy_name: String,
    pub passed: bool,
    pub violations: Vec<ViolationDto>,
    pub warnings: Vec<ViolationDto>,
}

impl From<PolicyEvaluation> for EvaluationDto {
    fn from(e: PolicyEvaluation) -> Self {
        Self {
            policy_id: e.policy_id,
            policy_name: e.policy_name,
            passed: e.passed,
            violations: e
                .violations
                .into_iter()
                .map(|v| ViolationDto {
                    rule: v.rule,
                    message: v.message,
                    message_vi: v.message_vi,
                })
                .collect(),
            warnings: e
                .warnings
                .into_iter()
                .map(|w| ViolationDto {
                    rule: w.rule,
                    message: w.message,
                    message_vi: w.message_vi,
                })
                .collect(),
        }
    }
}

/// Get all password policies
#[tauri::command]
pub fn get_password_policies(state: State<'_, AppState>) -> Result<Vec<PolicyDto>, String> {
    let settings = state.settings.read().unwrap();
    let policies = settings
        .password_policies
        .as_deref()
        .map(|_| built_in_policies())
        .unwrap_or_else(built_in_policies);
    Ok(policies.iter().map(PolicyDto::from).collect())
}

/// Enable or disable a policy
#[tauri::command]
pub fn set_policy_enabled(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    // Store enabled state in settings
    let enabled_ids = settings.enabled_policy_ids.get_or_insert_with(Vec::new);
    if enabled {
        if !enabled_ids.contains(&id) {
            enabled_ids.push(id);
        }
    } else {
        enabled_ids.retain(|i| i != &id);
    }
    Ok(())
}

/// Evaluate a password against all enabled policies
#[tauri::command]
pub fn evaluate_password_policies(
    password: String,
    username: Option<String>,
    title: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<EvaluationDto>, String> {
    let settings = state.settings.read().unwrap();
    let enabled_ids = settings.enabled_policy_ids.as_deref().unwrap_or(&[]);

    let mut policies = built_in_policies();
    // Apply enabled state from settings
    for policy in &mut policies {
        policy.enabled = enabled_ids.contains(&policy.id);
    }

    let results = evaluate_all_policies(
        &policies,
        &password,
        username.as_deref(),
        title.as_deref(),
        None,
    );

    Ok(results.into_iter().map(EvaluationDto::from).collect())
}

/// Check password strength
#[tauri::command]
pub fn check_password_strength(password: String) -> Result<serde_json::Value, String> {
    use keepassex_core::password_policy::{calculate_entropy, classifyPasswordStrength};

    let entropy = calculate_entropy(&password);
    let score = classifyPasswordStrength(entropy);

    let label = match score {
        0 => "Very Weak",
        1 => "Weak",
        2 => "Fair",
        3 => "Strong",
        _ => "Very Strong",
    };

    Ok(serde_json::json!({
        "score": score,
        "entropy": entropy,
        "strengthLabel": label,
    }))
}
