//! Windows registry operations for credential provider registration
//!
//! Registers/unregisters the KeePassEx credential provider DLL with Windows.
//! Must be run as Administrator.

#![cfg(windows)]

use windows::core::{HRESULT, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegSetValueExW, HKEY_CLASSES_ROOT,
    HKEY_LOCAL_MACHINE, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

/// CLSID for the KeePassEx credential provider
pub const KEEPASSEX_CLSID: &str = "{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}";
pub const PROVIDER_NAME: &str = "KeePassEx";

/// Register the credential provider DLL with Windows.
pub fn register() -> HRESULT {
    let dll_path = match get_dll_path() {
        Ok(p) => p,
        Err(e) => return e,
    };

    if let Err(e) = register_clsid(&dll_path) {
        return e;
    }
    if let Err(e) = register_credential_provider() {
        return e;
    }

    tracing::info!("KeePassEx Credential Provider registered");
    HRESULT(0)
}

/// Unregister the credential provider DLL from Windows.
pub fn unregister() -> HRESULT {
    let _ = unregister_credential_provider();
    let _ = unregister_clsid();
    tracing::info!("KeePassEx Credential Provider unregistered");
    HRESULT(0)
}

fn get_dll_path() -> Result<Vec<u16>, HRESULT> {
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
    let mut path = vec![0u16; 260];
    let len = unsafe { GetModuleFileNameW(None, &mut path) };
    if len == 0 {
        return Err(HRESULT(-1i32));
    }
    path.truncate(len as usize + 1);
    Ok(path)
}

fn register_clsid(dll_path: &[u16]) -> Result<(), HRESULT> {
    let clsid_key = format!("CLSID\\{}", KEEPASSEX_CLSID);
    let inproc_key = format!("CLSID\\{}\\InprocServer32", KEEPASSEX_CLSID);

    set_registry_string(HKEY_CLASSES_ROOT, &clsid_key, "", PROVIDER_NAME)?;

    let dll_path_str = String::from_utf16_lossy(dll_path)
        .trim_end_matches('\0')
        .to_string();
    set_registry_string(HKEY_CLASSES_ROOT, &inproc_key, "", &dll_path_str)?;
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
    set_registry_string(HKEY_LOCAL_MACHINE, &key_path, "", PROVIDER_NAME)
}

fn unregister_clsid() -> Result<(), HRESULT> {
    let clsid_key = format!("CLSID\\{}", KEEPASSEX_CLSID);
    let wide = to_wide(&clsid_key);
    let result = unsafe { RegDeleteKeyW(HKEY_CLASSES_ROOT, PCWSTR(wide.as_ptr())) };
    if result.is_err() {
        return Err(HRESULT(-1i32));
    }
    Ok(())
}

fn unregister_credential_provider() -> Result<(), HRESULT> {
    let key_path = format!(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\Credential Providers\\{}",
        KEEPASSEX_CLSID
    );
    let wide = to_wide(&key_path);
    let result = unsafe { RegDeleteKeyW(HKEY_LOCAL_MACHINE, PCWSTR(wide.as_ptr())) };
    if result.is_err() {
        return Err(HRESULT(-1i32));
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

    let create_result = unsafe {
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
    };
    if create_result.is_err() {
        return Err(HRESULT(-1i32));
    }

    let value_bytes = unsafe {
        std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2)
    };

    let set_result = unsafe {
        RegSetValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            0,
            REG_SZ,
            Some(value_bytes),
        )
    };

    unsafe {
        let _ = RegCloseKey(hkey);
    }

    if set_result.is_err() {
        return Err(HRESULT(-1i32));
    }

    Ok(())
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
