// MenuBarViewModel — State management for the menu bar popover
// Communicates with the main KeePassEx desktop app via XPC / named pipe

import Foundation
import Combine

/// Lightweight entry representation for menu bar display
struct MenuBarEntry: Identifiable, Hashable {
    let id: String
    let title: String
    let username: String
    let url: String
    let hasOtp: Bool
    let iconId: Int
}

/// OTP code with countdown
struct OtpCode {
    let code: String
    let remainingSeconds: Int
    let progress: Double // 0.0 – 1.0
}

@MainActor
class MenuBarViewModel: ObservableObject {
    @Published var isConnected = false
    @Published var isVaultLocked = true
    @Published var vaultName = ""
    @Published var entries: [MenuBarEntry] = []
    @Published var searchQuery = ""
    @Published var otpCodes: [String: OtpCode] = [:] // entryId → OtpCode
    @Published var recentEntries: [MenuBarEntry] = []
    @Published var errorMessage: String?

    private var ipcClient: IPCClient?
    private var otpTimer: Timer?
    private var cancellables = Set<AnyCancellable>()

    init() {
        connectToMainApp()
        startOtpTimer()

        // Debounce search
        $searchQuery
            .debounce(for: .milliseconds(200), scheduler: RunLoop.main)
            .sink { [weak self] query in
                Task { await self?.search(query: query) }
            }
            .store(in: &cancellables)
    }

    // ─── Connection ───────────────────────────────────────────────────────────

    func connectToMainApp() {
        ipcClient = IPCClient(socketName: "com.keepassex.app.menubar")
        Task {
            do {
                try await ipcClient?.connect()
                isConnected = true
                await refreshVaultStatus()
            } catch {
                isConnected = false
                errorMessage = "KeePassEx not running"
            }
        }
    }

    // ─── Vault operations ─────────────────────────────────────────────────────

    func refreshVaultStatus() async {
        guard let client = ipcClient else { return }
        do {
            let status = try await client.request(action: "get_vault_status")
            isVaultLocked = status["locked"] as? Bool ?? true
            vaultName = status["name"] as? String ?? ""
            if !isVaultLocked {
                await loadRecentEntries()
            }
        } catch {
            isConnected = false
        }
    }

    func lockVault() {
        Task {
            try? await ipcClient?.request(action: "lock_vault")
            isVaultLocked = true
            entries = []
            recentEntries = []
        }
    }

    func openMainApp() {
        NSWorkspace.shared.launchApplication("KeePassEx")
    }

    // ─── Entry operations ─────────────────────────────────────────────────────

    func search(query: String) async {
        guard !isVaultLocked, let client = ipcClient else { return }
        if query.isEmpty {
            await loadRecentEntries()
            return
        }
        do {
            let result = try await client.request(action: "search_entries", params: ["query": query])
            if let rawEntries = result["entries"] as? [[String: Any]] {
                entries = rawEntries.compactMap { parseEntry($0) }
            }
        } catch {
            entries = []
        }
    }

    func loadRecentEntries() async {
        guard let client = ipcClient else { return }
        do {
            let result = try await client.request(action: "get_recent_entries", params: ["limit": 5])
            if let rawEntries = result["entries"] as? [[String: Any]] {
                recentEntries = rawEntries.compactMap { parseEntry($0) }
            }
        } catch {}
    }

    func copyPassword(entryId: String) async {
        guard let client = ipcClient else { return }
        do {
            let result = try await client.request(action: "get_entry_password", params: ["uuid": entryId])
            if let password = result["password"] as? String {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(password, forType: .string)
                // Auto-clear after 10 seconds
                DispatchQueue.main.asyncAfter(deadline: .now() + 10) {
                    if NSPasteboard.general.string(forType: .string) == password {
                        NSPasteboard.general.clearContents()
                    }
                }
            }
        } catch {}
    }

    func copyUsername(entryId: String) async {
        guard let client = ipcClient else { return }
        do {
            let result = try await client.request(action: "get_entry_username", params: ["uuid": entryId])
            if let username = result["username"] as? String {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(username, forType: .string)
            }
        } catch {}
    }

    func copyOtp(entryId: String) async {
        guard let client = ipcClient else { return }
        do {
            let result = try await client.request(action: "generate_totp", params: ["uuid": entryId])
            if let code = result["code"] as? String {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(code, forType: .string)
            }
        } catch {}
    }

    // ─── OTP Timer ────────────────────────────────────────────────────────────

    private func startOtpTimer() {
        otpTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor [weak self] in
                await self?.refreshOtpCodes()
            }
        }
    }

    private func refreshOtpCodes() async {
        let otpEntries = (searchQuery.isEmpty ? recentEntries : entries).filter { $0.hasOtp }
        guard !otpEntries.isEmpty, let client = ipcClient else { return }

        for entry in otpEntries {
            do {
                let result = try await client.request(action: "generate_totp", params: ["uuid": entry.id])
                if let code = result["code"] as? String,
                   let remaining = result["remaining_seconds"] as? Int {
                    let period = result["period"] as? Int ?? 30
                    otpCodes[entry.id] = OtpCode(
                        code: code,
                        remainingSeconds: remaining,
                        progress: Double(remaining) / Double(period)
                    )
                }
            } catch {}
        }
    }

    // ─── Helpers ──────────────────────────────────────────────────────────────

    private func parseEntry(_ dict: [String: Any]) -> MenuBarEntry? {
        guard let id = dict["uuid"] as? String,
              let title = dict["title"] as? String else { return nil }
        return MenuBarEntry(
            id: id,
            title: title,
            username: dict["username"] as? String ?? "",
            url: dict["url"] as? String ?? "",
            hasOtp: dict["has_otp"] as? Bool ?? false,
            iconId: dict["icon_id"] as? Int ?? 0
        )
    }

    var filteredEntries: [MenuBarEntry] {
        searchQuery.isEmpty ? recentEntries : entries
    }
}

// ─── IPC Client ───────────────────────────────────────────────────────────────

/// Lightweight IPC client that communicates with the main KeePassEx app
/// via a Unix domain socket (same mechanism as the browser extension).
class IPCClient {
    private let socketName: String
    private var connection: URLSessionWebSocketTask?

    init(socketName: String) {
        self.socketName = socketName
    }

    func connect() async throws {
        // Connect to the KeePassEx IPC socket
        // The main app exposes a local WebSocket on a fixed port for menu bar IPC
        let url = URL(string: "ws://127.0.0.1:27015/menubar")!
        let session = URLSession(configuration: .default)
        connection = session.webSocketTask(with: url)
        connection?.resume()
    }

    func request(action: String, params: [String: Any] = [:]) async throws -> [String: Any] {
        guard let connection = connection else {
            throw NSError(domain: "IPCClient", code: -1, userInfo: [NSLocalizedDescriptionKey: "Not connected"])
        }

        let requestId = UUID().uuidString
        var payload: [String: Any] = ["id": requestId, "action": action]
        payload.merge(params) { _, new in new }

        let data = try JSONSerialization.data(withJSONObject: payload)
        let message = URLSessionWebSocketTask.Message.data(data)
        try await connection.send(message)

        let response = try await connection.receive()
        switch response {
        case .data(let data):
            let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]
            return json
        case .string(let str):
            let data = str.data(using: .utf8) ?? Data()
            let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]
            return json
        @unknown default:
            return [:]
        }
    }
}
