//! Windows registry operations for credential provider registration
//!
//! Registers/unregisters the KeePassEx credential provider DLL with Windows.
//! Must be run as Administrator.
//!
//! Registry path:
//! HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\
//!   Credential Providers\{KEEPASSEX_CLSID}
//!
//! CLSID path:
//! HKCR\CLSID\{KEEPASSEX_CLSID}
//! HKCR\CLSID\{KEEPASSEX_CLSID}\InprocServer32

#![cfg(windows)]

use windows::core::{HRESULT, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegSetValueExW, HKEY_CLASSES_ROOT,
    HKEY_LOCAL_MACHINE, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

/// CLSID for the KeePassEx credential provider
/// {A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
pub const KEEPASSEX_CLSID: &str = "{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}";
pub const PROVIDER_NAME: &str = "KeePassEx";
pub const PROVIDER_DESC: &str = "Unlock Windows with your KeePassEx vault master password";

/// Register the credential provider DLL with Windows.
/// Called by `regsvr32 keepassex_credprov.dll` or the installer.
pub fn register() -> HRESULT {
    let dll_path = match get_dll_path() {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Register CLSID in HKCR
    if let Err(e) = register_clsid(&dll_path) {
        return e;
    }

    // Register as credential provider in HKLM
    if let Err(e) = register_credential_provider() {
        return e;
    }

    tracing::info!("KeePassEx Credential Provider registered successfully");
    HRESULT(0) // S_OK
}

/// Unregister the credential provider DLL from Windows.
/// Called by `regsvr32 /u keepassex_credprov.dll` or the uninstaller.
pub fn unregister() -> HRESULT {
    // Remove from credential providers list
    let _ = unregister_credential_provider();

    // Remove CLSID registration
    let _ = unregister_clsid();

    tracing::info!("KeePassEx Credential Provider unregistered");
    HRESULT(0) // S_OK
}

fn get_dll_path() -> Result<Vec<u16>, HRESULT> {
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
    let mut path = vec![0u16; 260];
    let len = unsafe { GetModuleFileNameW(None, &mut path) };
    if len == 0 {
        return Err(HRESULT(-1));
    }
    path.truncate(len as usize + 1); // Include null terminator
    Ok(path)
}

fn register_clsid(dll_path: &[u16]) -> Result<(), HRESULT> {
    let clsid_key = format!("CLSID\\{}", KEEPASSEX_CLSID);
    let inproc_key = format!("CLSID\\{}\\InprocServer32", KEEPASSEX_CLSID);

    // HKCR\CLSID\{CLSID} = "KeePassEx Credential Provider"
    set_registry_string(HKEY_CLASSES_ROOT, &clsid_key, "", PROVIDER_NAME)?;

    // HKCR\CLSID\{CLSID}\InprocServer32 = "path\to\dll"
    let dll_path_str: String = String::from_utf16_lossy(dll_path)
        .trim_end_matches('\0')
        .to_string();
    set_registry_string(HKEY_CLASSES_ROOT, &inproc_key, "", &dll_path_str)?;

    // ThreadingModel = "Apartment"
    set_registry_string(
        HKEY_CLASSES_ROOT,
        &inproc_key,
        "ThreadingModel",
        "Apartment",
    )?;

    Ok(())
}

fn register_credential_provider() -> Result<(), HRESULT> {
    let key_path = format!(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\Credential Providers\\{}",
        KEEPASSEX_CLSID
    );
    set_registry_string(HKEY_LOCAL_MACHINE, &key_path, "", PROVIDER_NAME)?;
    Ok(())
}

fn unregister_clsid() -> Result<(), HRESULT> {
    let clsid_key = format!("CLSID\\{}", KEEPASSEX_CLSID);
    unsafe {
        RegDeleteKeyW(HKEY_CLASSES_ROOT, PCWSTR(to_wide(&clsid_key).as_ptr()))
            .map_err(|e| e.code())?;
    }
    Ok(())
}

fn unregister_credential_provider() -> Result<(), HRESULT> {
    let key_path = format!(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\Credential Providers\\{}",
        KEEPASSEX_CLSID
    );
    unsafe {
        RegDeleteKeyW(HKEY_LOCAL_MACHINE, PCWSTR(to_wide(&key_path).as_ptr()))
            .map_err(|e| e.code())?;
    }
    Ok(())
}

fn set_registry_string(
    root: windows::Win32::System::Registry::HKEY,
    key_path: &str,
    value_name: &str,
    value: &str,
) -> Result<(), HRESULT> {
    let key_path_wide = to_wide(key_path);
    let value_name_wide = to_wide(value_name);
    let value_wide = to_wide(value);

    let mut hkey = windows::Win32::System::Registry::HKEY::default();
    unsafe {
        RegCreateKeyExW(
            root,
            PCWSTR(key_path_wide.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
        .map_err(|e| e.code())?;

        let value_bytes =
            std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2);

        RegSetValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            0,
            REG_SZ,
            Some(value_bytes),
        )
        .map_err(|e| e.code())?;

        RegCloseKey(hkey).map_err(|e| e.code())?;
    }
    Ok(())
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
