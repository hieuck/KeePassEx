//! KeePassEx Windows Credential Provider
//!
//! Implements the ICredentialProvider COM interface to allow users to unlock
//! the Windows login screen using their KeePassEx vault master password.
//!
//! # How it works
//! 1. Windows calls this DLL during the login screen
//! 2. The credential provider shows a "KeePassEx" tile on the login screen
//! 3. User enters their vault master password
//! 4. The provider verifies the password against the stored ZKPV commitment
//!    (zero-knowledge pre-check — no vault decryption needed at login time)
//! 5. On success, the provider returns the Windows credentials stored in the vault
//!
//! # Security model
//! - The vault master password is NEVER stored in plaintext
//! - Windows credentials (username/password) are stored encrypted in the vault
//! - The ZKPV commitment allows fast password verification without full decryption
//! - Screen capture protection is active during credential entry
//!
//! # Competitor gap
//! No competitor (KeePass, KeePassXC, Keepassium, KeePass2Android) has this feature.
//! This is unique to KeePassEx on Windows.
//!
//! # Installation
//! Run `keepassex-credprov.exe --install` to register the DLL with Windows.
//! Run `keepassex-credprov.exe --uninstall` to remove it.

#![cfg(windows)]

mod credential;
mod provider;
mod registry;
mod tile;

use std::ffi::c_void;

// ─── DLL Entry Point ──────────────────────────────────────────────────────────

/// DLL entry point — called by Windows when the DLL is loaded/unloaded
#[no_mangle]
pub extern "system" fn DllMain(_hmodule: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;

    match reason {
        DLL_PROCESS_ATTACH => {
            // Initialize logging
            let _ = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .try_init();
            tracing::info!("KeePassEx Credential Provider loaded");
        }
        DLL_PROCESS_DETACH => {
            tracing::info!("KeePassEx Credential Provider unloaded");
        }
        _ => {}
    }
    1 // TRUE
}

/// COM class factory — called by Windows to create the credential provider
#[no_mangle]
pub extern "system" fn DllGetClassObject(
    rclsid: *const windows::core::GUID,
    riid: *const windows::core::GUID,
    ppv: *mut *mut c_void,
) -> windows::core::HRESULT {
    provider::get_class_object(rclsid, riid, ppv)
}

/// Called by regsvr32 to register the DLL
#[no_mangle]
pub extern "system" fn DllRegisterServer() -> windows::core::HRESULT {
    registry::register()
}

/// Called by regsvr32 to unregister the DLL
#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> windows::core::HRESULT {
    registry::unregister()
}

/// Check if the DLL can be unloaded (COM reference counting)
#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> windows::core::HRESULT {
    // S_OK = can unload, S_FALSE = cannot unload
    windows::core::HRESULT(0) // S_OK
}
