//! KeePassEx Desktop — Tauri v2 application

mod autotype;
mod commands;
mod native_messaging;
mod ssh_agent_server;
mod state;
mod tray;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("keepassex=debug,warn")),
        )
        .init();

    tauri::Builder::default()
        // Plugins
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        // App state
        .manage(state::AppState::new())
        // Commands
        .invoke_handler(tauri::generate_handler![
            // Vault commands
            commands::vault::open_vault,
            commands::vault::create_vault,
            commands::vault::close_vault,
            commands::vault::save_vault,
            commands::vault::lock_vault,
            commands::vault::change_credentials,
            commands::vault::get_vault_meta,
            commands::vault::update_vault_meta,
            commands::vault::open_vault_tab,
            commands::vault::close_vault_tab,
            commands::vault::lock_vault_tab,
            // Entry commands
            commands::entries::get_entries,
            commands::entries::get_entry,
            commands::entries::get_entry_password,
            commands::entries::create_entry,
            commands::entries::update_entry,
            commands::entries::delete_entry,
            commands::entries::move_entry,
            commands::entries::duplicate_entry,
            commands::entries::search_entries,
            commands::entries::get_entry_history,
            commands::entries::restore_entry_from_history,
            commands::entries::clear_entry_history,
            // Group commands
            commands::groups::get_groups,
            commands::groups::create_group,
            commands::groups::update_group,
            commands::groups::delete_group,
            commands::groups::move_group,
            // Generator commands
            commands::generator::generate_password,
            commands::generator::estimate_entropy,
            commands::generator::score_strength,
            // OTP commands
            commands::otp::generate_totp,
            commands::otp::parse_otp_uri,
            commands::otp::set_entry_otp,
            commands::otp::remove_entry_otp,
            // Health commands
            commands::health::audit_vault,
            commands::health::get_rotation_recommendations,
            commands::health::find_duplicate_entries,
            // Clipboard commands
            commands::clipboard::copy_to_clipboard,
            commands::clipboard::clear_clipboard,
            commands::clipboard::read_clipboard_text,
            // SSH Agent commands
            commands::ssh::start_ssh_agent,
            commands::ssh::stop_ssh_agent,
            commands::ssh::add_ssh_key,
            commands::ssh::list_ssh_keys,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::save_settings,
            // Import/Export commands
            commands::import_export::import_vault,
            commands::import_export::export_vault_cmd,
            commands::import_export::detect_import_format,
            // Breach check commands
            commands::breach::check_vault_breaches,
            commands::breach::check_password_breach,
            // Sync commands
            commands::sync_cmd::get_sync_status,
            commands::sync_cmd::configure_sync,
            commands::sync_cmd::sync_now,
            commands::sync_cmd::test_sync_connection,
            // Attachment commands
            commands::attachments::read_file_bytes,
            commands::attachments::save_attachment,
            commands::attachments::add_attachment,
            commands::attachments::remove_attachment,
            // Hardware key commands
            commands::hardware_key::list_hardware_keys_cmd,
            commands::hardware_key::test_hardware_key_cmd,
            commands::hardware_key::configure_hardware_key,
            commands::hardware_key::remove_hardware_key,
            commands::hardware_key::get_hardware_key_config,
            // Backup commands
            commands::backup::get_backup_config,
            commands::backup::save_backup_config,
            commands::backup::backup_now,
            commands::backup::list_backups_cmd,
            commands::backup::restore_from_backup_cmd,
            commands::backup::delete_backup_cmd,
            // Audit log commands
            commands::audit_log_cmd::get_audit_log,
            commands::audit_log_cmd::clear_audit_log,
            commands::audit_log_cmd::export_audit_log,
            commands::audit_log_cmd::record_audit_event,
            // Password policy commands
            commands::policy_cmd::get_password_policies,
            commands::policy_cmd::set_policy_enabled,
            commands::policy_cmd::evaluate_password_policies,
            commands::policy_cmd::check_password_strength,
            // Vault compare commands
            commands::vault_compare_cmd::compare_vaults_cmd,
            commands::vault_compare_cmd::merge_vaults_cmd,
            // Analytics commands
            commands::analytics_cmd::get_vault_analytics,
            commands::analytics_cmd::export_analytics_report,
            // Natural language search commands
            commands::search_cmd::nl_search,
            commands::search_cmd::parse_search_query,
            // Steganography commands
            commands::steg_cmd::detect_steg_carrier,
            commands::steg_cmd::steg_embed_vault,
            commands::steg_cmd::steg_extract_vault,
            // Team vault commands
            commands::team_cmd::get_team_vault,
            commands::team_cmd::invite_team_member,
            commands::team_cmd::change_team_member_role,
            commands::team_cmd::remove_team_member,
            commands::team_cmd::set_entry_permission,
            // Rotation commands
            commands::rotation_cmd::get_rotation_summary,
            commands::rotation_cmd::bulk_rotate_passwords,
            // AI password suggestion commands
            commands::ai_cmd::suggest_passwords_cmd,
            // Field reference commands
            commands::field_references::resolve_entry_refs,
            commands::field_references::resolve_ref_string,
            commands::field_references::build_field_ref,
            commands::field_references::check_has_refs,
            // Favicon commands
            commands::favicon::fetch_entry_favicon,
            commands::favicon::fetch_all_favicons,
            commands::favicon::get_domain_from_url,
            // PQC commands
            commands::pqc_cmd::migrate_to_pqc,
            commands::pqc_cmd::downgrade_from_pqc,
            commands::pqc_cmd::check_pqc_status,
            // Updater commands
            commands::updater_cmd::check_for_updates,
            commands::updater_cmd::get_app_version,
            // Auto-type command
            commands::autotype_cmd::auto_type_entry,
            // Emergency Access commands
            commands::emergency_access_cmd::get_emergency_grants,
            commands::emergency_access_cmd::add_emergency_grant,
            commands::emergency_access_cmd::revoke_emergency_grant,
            commands::emergency_access_cmd::approve_emergency_request,
            commands::emergency_access_cmd::delete_emergency_grant,
            // Plugin commands
            commands::plugins_cmd::list_plugins,
            commands::plugins_cmd::toggle_plugin,
            commands::plugins_cmd::uninstall_plugin,
            commands::plugins_cmd::install_plugin_from_file,
            // Passkey commands
            commands::passkey_cmd::get_entry_passkeys,
            commands::passkey_cmd::add_entry_passkey,
            commands::passkey_cmd::remove_entry_passkey,
            commands::passkey_cmd::get_passkey_registration_options,
            // SSH entry commands
            commands::ssh_entry_cmd::get_entry_ssh_key,
            commands::ssh_entry_cmd::set_entry_ssh_key,
            commands::ssh_entry_cmd::remove_entry_ssh_key,
            commands::ssh_entry_cmd::get_entry_ssh_private_key,
            commands::ssh_entry_cmd::load_ssh_key_to_agent,
        ])
        .setup(|app| {
            // Load persisted settings
            let app_handle = app.handle().clone();
            let state = app.state::<state::AppState>();
            let state_clone = state.inner().vault.clone();
            let settings_arc = state.inner().settings.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(saved) = commands::settings::load_settings_from_disk(&app_handle).await
                {
                    *settings_arc.write().unwrap() = saved;
                    tracing::info!("Settings loaded from disk");
                }
            });

            // Setup system tray
            tray::setup_tray(app)?;

            // Setup global shortcuts — ignore if hotkey already registered
            if let Err(e) = setup_shortcuts(app) {
                tracing::warn!("Could not register global shortcuts: {}", e);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to tray instead of closing (if configured)
                let state = window.state::<state::AppState>();
                if state.settings.read().unwrap().minimize_to_tray {
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running KeePassEx");
}

fn setup_shortcuts(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    // Ctrl+Alt+K — Show/hide window
    let shortcut = "CommandOrControl+Alt+K".parse::<Shortcut>()?;

    // Ignore if already registered (e.g. another instance running)
    let _ = app
        .global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });

    Ok(())
}
