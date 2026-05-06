/**
 * KeePassEx iOS Native Module
 * Bridges React Native to the Rust core library via FFI
 */
import Foundation
import React

@objc(KeePassExCore)
class KeePassExCore: NSObject {

  // MARK: - Vault Operations

  @objc
  func openVault(
    _ path: String,
    password: String,
    keyFileData: [UInt8]?,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      // Call Rust FFI
      let result = keepassex_open_vault(
        path,
        password,
        keyFileData.map { Data($0) }
      )

      DispatchQueue.main.async {
        if result.success {
          resolve([
            "name": result.vault_name ?? "",
            "description": result.vault_description ?? "",
            "entryCount": result.entry_count,
            "groupCount": result.group_count,
            "path": path,
          ])
        } else {
          reject("VAULT_ERROR", result.error ?? "Failed to open vault", nil)
        }
      }
    }
  }

  @objc
  func createVault(
    _ path: String,
    name: String,
    password: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_create_vault(path, name, password)
      DispatchQueue.main.async {
        if result.success {
          resolve([
            "name": name,
            "description": "",
            "entryCount": 0,
            "groupCount": 1,
            "path": path,
          ])
        } else {
          reject("VAULT_ERROR", result.error ?? "Failed to create vault", nil)
        }
      }
    }
  }

  @objc
  func closeVault(
    _ resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    keepassex_close_vault()
    resolve(nil)
  }

  @objc
  func lockVault() {
    keepassex_lock_vault()
  }

  // MARK: - Entry Operations

  @objc
  func getEntries(
    _ groupUuid: String?,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_get_entries(groupUuid)
      DispatchQueue.main.async {
        resolve(result.entries ?? [])
      }
    }
  }

  @objc
  func getEntry(
    _ uuid: String,
    includePassword: Bool,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_get_entry(uuid, includePassword)
      DispatchQueue.main.async {
        if let entry = result.entry {
          resolve(entry)
        } else {
          reject("NOT_FOUND", "Entry not found", nil)
        }
      }
    }
  }

  @objc
  func getEntryPassword(
    _ uuid: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_get_entry_password(uuid)
      DispatchQueue.main.async {
        if result.success {
          resolve(result.password ?? "")
        } else {
          reject("ERROR", result.error ?? "Failed to get password", nil)
        }
      }
    }
  }

  @objc
  func createEntry(
    _ args: NSDictionary,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_create_entry(args as! [String: Any])
      DispatchQueue.main.async {
        if result.success {
          resolve(result.uuid ?? "")
        } else {
          reject("ERROR", result.error ?? "Failed to create entry", nil)
        }
      }
    }
  }

  @objc
  func updateEntry(
    _ args: NSDictionary,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_update_entry(args as! [String: Any])
      DispatchQueue.main.async {
        if result.success {
          resolve(nil)
        } else {
          reject("ERROR", result.error ?? "Failed to update entry", nil)
        }
      }
    }
  }

  @objc
  func deleteEntry(
    _ uuid: String,
    permanent: Bool,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_delete_entry(uuid, permanent)
      DispatchQueue.main.async {
        if result.success {
          resolve(nil)
        } else {
          reject("ERROR", result.error ?? "Failed to delete entry", nil)
        }
      }
    }
  }

  @objc
  func searchEntries(
    _ query: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_search_entries(query)
      DispatchQueue.main.async {
        resolve(result.entries ?? [])
      }
    }
  }

  // MARK: - OTP

  @objc
  func generateTotp(
    _ entryUuid: String,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    let result = keepassex_generate_totp(entryUuid)
    if result.success {
      resolve([
        "code": result.code ?? "",
        "remainingSeconds": result.remaining_seconds,
        "period": result.period,
        "progress": result.progress,
      ])
    } else {
      reject("OTP_ERROR", result.error ?? "Failed to generate OTP", nil)
    }
  }

  // MARK: - Generator

  @objc
  func generatePassword(
    _ args: NSDictionary,
    resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_generate_password(args as! [String: Any])
      DispatchQueue.main.async {
        if result.success {
          resolve([
            "password": result.password ?? "",
            "entropy": result.entropy,
            "strengthScore": result.strength_score,
            "strengthLabel": result.strength_label ?? "",
          ])
        } else {
          reject("ERROR", result.error ?? "Failed to generate password", nil)
        }
      }
    }
  }

  // MARK: - Health

  @objc
  func auditVault(
    _ resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    DispatchQueue.global(qos: .userInitiated).async {
      let result = keepassex_audit_vault()
      DispatchQueue.main.async {
        resolve(result)
      }
    }
  }

  // MARK: - Groups

  @objc
  func getGroups(
    _ resolve: @escaping RCTPromiseResolveBlock,
    reject: @escaping RCTPromiseRejectBlock
  ) {
    let result = keepassex_get_groups()
    resolve(result.groups ?? [])
  }

  // MARK: - React Native Module Registration

  @objc
  static func requiresMainQueueSetup() -> Bool {
    return false
  }
}

// MARK: - FFI Stubs (replaced by actual Rust FFI in production)
// These would be generated by cbindgen from the Rust core library

private func keepassex_open_vault(_ path: String, _ password: String, _ keyFile: Data?) -> VaultResult {
  // Stub — real impl calls Rust via C FFI
  return VaultResult(success: false, error: "Native module not linked")
}

private func keepassex_create_vault(_ path: String, _ name: String, _ password: String) -> VaultResult {
  return VaultResult(success: false, error: "Native module not linked")
}

private func keepassex_close_vault() {}
private func keepassex_lock_vault() {}

private func keepassex_get_entries(_ groupUuid: String?) -> EntriesResult {
  return EntriesResult(entries: [])
}

private func keepassex_get_entry(_ uuid: String, _ includePassword: Bool) -> EntryResult {
  return EntryResult(entry: nil)
}

private func keepassex_get_entry_password(_ uuid: String) -> PasswordResult {
  return PasswordResult(success: false, password: nil, error: "Not implemented")
}

private func keepassex_create_entry(_ args: [String: Any]) -> UuidResult {
  return UuidResult(success: false, uuid: nil, error: "Not implemented")
}

private func keepassex_update_entry(_ args: [String: Any]) -> SimpleResult {
  return SimpleResult(success: false, error: "Not implemented")
}

private func keepassex_delete_entry(_ uuid: String, _ permanent: Bool) -> SimpleResult {
  return SimpleResult(success: false, error: "Not implemented")
}

private func keepassex_search_entries(_ query: String) -> EntriesResult {
  return EntriesResult(entries: [])
}

private func keepassex_generate_totp(_ uuid: String) -> OtpResult {
  return OtpResult(success: false, code: nil, remaining_seconds: 0, period: 30, progress: 0, error: "Not implemented")
}

private func keepassex_generate_password(_ args: [String: Any]) -> PasswordGenResult {
  return PasswordGenResult(success: false, password: nil, entropy: 0, strength_score: 0, strength_label: nil, error: "Not implemented")
}

private func keepassex_audit_vault() -> [String: Any] { return [:] }
private func keepassex_get_groups() -> GroupsResult { return GroupsResult(groups: []) }

// MARK: - Result Types

private struct VaultResult {
  let success: Bool
  var vault_name: String? = nil
  var vault_description: String? = nil
  var entry_count: Int = 0
  var group_count: Int = 0
  var error: String? = nil
}

private struct EntriesResult {
  let entries: [[String: Any]]?
}

private struct EntryResult {
  let entry: [String: Any]?
}

private struct PasswordResult {
  let success: Bool
  let password: String?
  let error: String?
}

private struct UuidResult {
  let success: Bool
  let uuid: String?
  let error: String?
}

private struct SimpleResult {
  let success: Bool
  let error: String?
}

private struct OtpResult {
  let success: Bool
  let code: String?
  let remaining_seconds: Int
  let period: Int
  let progress: Float
  let error: String?
}

private struct PasswordGenResult {
  let success: Bool
  let password: String?
  let entropy: Double
  let strength_score: Int
  let strength_label: String?
  let error: String?
}

private struct GroupsResult {
  let groups: [[String: Any]]?
}
