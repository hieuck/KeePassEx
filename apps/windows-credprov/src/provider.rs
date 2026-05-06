//! ICredentialProvider COM interface implementation
//!
//! This is the main entry point for the Windows credential provider.
//! Windows calls GetClassObject() → creates KeePassExCredentialProvider
//! → calls GetCredentialCount() → calls GetCredentialAt() → shows tile

#![cfg(windows)]

use std::ffi::c_void;
use windows::core::{implement, IUnknown, GUID, HRESULT};
use windows::Win32::Foundation::E_NOINTERFACE;

use crate::registry::KEEPASSEX_CLSID;

/// Called by Windows to create the credential provider factory
pub fn get_class_object(rclsid: *const GUID, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    if ppv.is_null() {
        return HRESULT(-2147024809i32); // E_POINTER
    }

    unsafe {
        *ppv = std::ptr::null_mut();

        let clsid = &*rclsid;
        let expected_clsid = parse_guid(KEEPASSEX_CLSID);

        if *clsid != expected_clsid {
            return HRESULT(-2147221231i32); // CLASS_E_CLASSNOTAVAILABLE
        }

        // Create the class factory
        let factory = KeePassExClassFactory;
        let factory_ptr = Box::into_raw(Box::new(factory)) as *mut c_void;
        *ppv = factory_ptr;
    }

    HRESULT(0) // S_OK
}

/// Parse a GUID string like "{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}"
fn parse_guid(s: &str) -> GUID {
    // Strip braces
    let s = s.trim_matches(|c| c == '{' || c == '}');
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return GUID::zeroed();
    }

    let data1 = u32::from_str_radix(parts[0], 16).unwrap_or(0);
    let data2 = u16::from_str_radix(parts[1], 16).unwrap_or(0);
    let data3 = u16::from_str_radix(parts[2], 16).unwrap_or(0);

    let data4_hex = format!("{}{}", parts[3], parts[4]);
    let mut data4 = [0u8; 8];
    for i in 0..8 {
        data4[i] = u8::from_str_radix(&data4_hex[i * 2..i * 2 + 2], 16).unwrap_or(0);
    }

    GUID {
        data1,
        data2,
        data3,
        data4,
    }
}

/// COM class factory for KeePassExCredentialProvider
struct KeePassExClassFactory;

/// The credential provider — implements ICredentialProvider
///
/// Windows calls these methods in order:
/// 1. SetUsageScenario() — tells us why we're being shown (logon, unlock, etc.)
/// 2. SetSerialization() — provides any pre-filled credentials
/// 3. Advise() — gives us a callback interface for updates
/// 4. GetFieldDescriptorCount() — how many fields does our tile have?
/// 5. GetFieldDescriptorAt() — describe each field (label, type, etc.)
/// 6. GetCredentialCount() — how many credential tiles to show?
/// 7. GetCredentialAt() — return each credential tile
pub struct KeePassExCredentialProvider {
    /// Whether we're in a usage scenario we support
    supported: bool,
    /// The credential tile
    credential: Option<crate::tile::KeePassExCredentialTile>,
}

impl KeePassExCredentialProvider {
    pub fn new() -> Self {
        Self {
            supported: false,
            credential: None,
        }
    }

    /// Called by Windows to set the usage scenario
    /// We support: logon, unlock workstation, change password
    pub fn set_usage_scenario(&mut self, scenario: u32) -> HRESULT {
        // CPUS_LOGON = 1, CPUS_UNLOCK_WORKSTATION = 2, CPUS_CHANGE_PASSWORD = 3
        self.supported = matches!(scenario, 1 | 2);
        if self.supported {
            self.credential = Some(crate::tile::KeePassExCredentialTile::new());
            HRESULT(0) // S_OK
        } else {
            HRESULT(-2147467259i32) // E_FAIL — not supported for this scenario
        }
    }

    /// How many credential tiles to show
    pub fn get_credential_count(&self) -> (u32, u32, bool) {
        if self.supported && self.credential.is_some() {
            (1, 0, false) // count=1, default=0, auto_logon=false
        } else {
            (0, u32::MAX, false)
        }
    }
}
