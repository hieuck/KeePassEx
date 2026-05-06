/**
 * KeePassEx watchOS — SwiftUI native app
 * Shows TOTP codes and quick-access entries on Apple Watch
 *
 * Features:
 * - Entry list with search (Digital Crown scroll)
 * - Favorites / pinned entries
 * - OTP countdown ring with haptic feedback
 * - Copy password via WatchConnectivity
 * - Complications (circular, rectangular, corner)
 * - Accessibility: VoiceOver labels, Dynamic Type
 */
import SwiftUI
import WatchConnectivity
import WatchKit

// ─── Root View ────────────────────────────────────────────────────────────────

struct ContentView: View {
    @StateObject private var vaultManager = WatchVaultManager()
    @StateObject private var connectivity = WatchConnectivityManager.shared

    var body: some View {
        NavigationStack {
            if vaultManager.isLocked {
                LockedView(vaultManager: vaultManager)
            } else {
                EntryListView(vaultManager: vaultManager)
            }
        }
        .onAppear {
            connectivity.vaultManager = vaultManager
        }
    }
}

// ─── Locked View ──────────────────────────────────────────────────────────────

struct LockedView: View {
    @ObservedObject var vaultManager: WatchVaultManager
    @State private var isUnlocking = false

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "lock.fill")
                .font(.system(size: 36))
                .foregroundColor(.blue)
                .accessibilityHidden(true)

            Text("KeePassEx")
                .font(.headline)
                .accessibilityAddTraits(.isHeader)

            if isUnlocking {
                ProgressView()
                    .accessibilityLabel("Unlocking vault")
            } else {
                Button {
                    isUnlocking = true
                    vaultManager.unlock { success in
                        isUnlocking = false
                        if success {
                            WKInterfaceDevice.current().play(.success)
                        } else {
                            WKInterfaceDevice.current().play(.failure)
                        }
                    }
                } label: {
                    Label("Unlock", systemImage: "lock.open")
                }
                .buttonStyle(.borderedProminent)
                .accessibilityLabel("Unlock vault on iPhone")
            }
        }
        .navigationTitle("KeePassEx")
    }
}

// ─── Entry List ───────────────────────────────────────────────────────────────

struct EntryListView: View {
    @ObservedObject var vaultManager: WatchVaultManager
    @State private var searchText = ""
    @State private var showFavoritesOnly = false

    var filteredEntries: [WatchEntry] {
        var entries = vaultManager.entries
        if showFavoritesOnly {
            entries = entries.filter { $0.isFavorite }
        }
        if !searchText.isEmpty {
            entries = entries.filter {
                $0.title.localizedCaseInsensitiveContains(searchText) ||
                $0.username.localizedCaseInsensitiveContains(searchText)
            }
        }
        return entries
    }

    var body: some View {
        List {
            // Favorites section
            if !showFavoritesOnly && vaultManager.entries.contains(where: { $0.isFavorite }) {
                Section {
                    ForEach(vaultManager.entries.filter { $0.isFavorite }) { entry in
                        EntryRowView(entry: entry)
                    }
                } header: {
                    Label("Favorites", systemImage: "star.fill")
                        .foregroundColor(.yellow)
                        .font(.caption2)
                }
            }

            // All entries section
            Section {
                if filteredEntries.isEmpty {
                    Text(searchText.isEmpty ? "No entries" : "No results")
                        .foregroundColor(.secondary)
                        .font(.caption)
                } else {
                    ForEach(filteredEntries) { entry in
                        EntryRowView(entry: entry)
                    }
                }
            } header: {
                if !showFavoritesOnly {
                    Text("\(filteredEntries.count) entries")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
        }
        .searchable(text: $searchText, prompt: "Search")
        .navigationTitle("Vault")
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                Button {
                    showFavoritesOnly.toggle()
                    WKInterfaceDevice.current().play(.click)
                } label: {
                    Image(systemName: showFavoritesOnly ? "star.fill" : "star")
                        .foregroundColor(showFavoritesOnly ? .yellow : .secondary)
                }
                .accessibilityLabel(showFavoritesOnly ? "Show all entries" : "Show favorites only")

                Button {
                    vaultManager.lock()
                    WKInterfaceDevice.current().play(.click)
                } label: {
                    Image(systemName: "lock")
                }
                .accessibilityLabel("Lock vault")
            }
        }
        .refreshable {
            await vaultManager.refreshEntries()
        }
    }
}

// ─── Entry Row ────────────────────────────────────────────────────────────────

struct EntryRowView: View {
    let entry: WatchEntry

    var body: some View {
        NavigationLink(destination: EntryDetailView(entry: entry)) {
            HStack(spacing: 8) {
                // Icon
                ZStack {
                    Circle()
                        .fill(Color.blue.opacity(0.15))
                        .frame(width: 28, height: 28)
                    Text(entry.title.prefix(1).uppercased())
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(.blue)
                }
                .accessibilityHidden(true)

                VStack(alignment: .leading, spacing: 1) {
                    Text(entry.title)
                        .font(.system(size: 14, weight: .medium))
                        .lineLimit(1)
                    if !entry.username.isEmpty {
                        Text(entry.username)
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                    }
                }

                Spacer(minLength: 0)

                HStack(spacing: 4) {
                    if entry.isFavorite {
                        Image(systemName: "star.fill")
                            .font(.system(size: 9))
                            .foregroundColor(.yellow)
                    }
                    if entry.hasOtp {
                        Image(systemName: "clock.fill")
                            .font(.system(size: 10))
                            .foregroundColor(.blue)
                    }
                }
            }
        }
        .accessibilityLabel("\(entry.title), \(entry.username)")
        .accessibilityHint(entry.hasOtp ? "Has OTP code" : "")
    }
}

// ─── Entry Detail ─────────────────────────────────────────────────────────────

struct EntryDetailView: View {
    let entry: WatchEntry
    @State private var otpCode: String = "------"
    @State private var otpRemaining: Int = 30
    @State private var otpPeriod: Int = 30
    @State private var timer: Timer?
    @State private var copyFeedback: String? = nil
    @State private var isCopying = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 10) {

                // Username
                if !entry.username.isEmpty {
                    InfoRow(
                        label: "Username",
                        value: entry.username,
                        icon: "person",
                        onCopy: { copyField("username") }
                    )
                }

                // URL
                if let url = entry.url, !url.isEmpty {
                    InfoRow(
                        label: "URL",
                        value: url,
                        icon: "link",
                        onCopy: nil
                    )
                }

                // OTP
                if entry.hasOtp {
                    OtpRow(
                        code: otpCode,
                        remaining: otpRemaining,
                        period: otpPeriod,
                        onCopy: { copyOtp() }
                    )
                }

                // Copy password button
                Button {
                    copyField("password")
                } label: {
                    HStack {
                        if isCopying {
                            ProgressView()
                                .scaleEffect(0.7)
                        } else {
                            Image(systemName: copyFeedback != nil ? "checkmark" : "doc.on.doc")
                        }
                        Text(copyFeedback ?? "Copy Password")
                            .font(.system(size: 13))
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(isCopying)
                .accessibilityLabel("Copy password to iPhone clipboard")

                // Notes indicator
                if entry.hasNotes {
                    HStack {
                        Image(systemName: "note.text")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                        Text("Has notes")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .padding(.horizontal, 4)
        }
        .navigationTitle(entry.title)
        .onAppear {
            if entry.hasOtp { startOtpTimer() }
        }
        .onDisappear {
            timer?.invalidate()
            timer = nil
        }
    }

    private func startOtpTimer() {
        refreshOtp()
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { _ in
            refreshOtp()
            // Haptic warning when OTP is about to expire
            if otpRemaining == 5 {
                WKInterfaceDevice.current().play(.notification)
            }
        }
    }

    private func refreshOtp() {
        WatchConnectivityManager.shared.requestOtp(entryUuid: entry.uuid) { code, remaining, period in
            DispatchQueue.main.async {
                otpCode = code
                otpRemaining = remaining
                otpPeriod = period
            }
        }
    }

    private func copyOtp() {
        WatchConnectivityManager.shared.requestOtp(entryUuid: entry.uuid) { code, _, _ in
            DispatchQueue.main.async {
                copyFeedback = "Copied!"
                WKInterfaceDevice.current().play(.success)
                DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                    copyFeedback = nil
                }
            }
        }
    }

    private func copyField(_ field: String) {
        isCopying = true
        WatchConnectivityManager.shared.copyField(entryUuid: entry.uuid, field: field) { success in
            DispatchQueue.main.async {
                isCopying = false
                if success {
                    copyFeedback = "Copied!"
                    WKInterfaceDevice.current().play(.success)
                    DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                        copyFeedback = nil
                    }
                } else {
                    WKInterfaceDevice.current().play(.failure)
                }
            }
        }
    }
}

// ─── OTP Row ─────────────────────────────────────────────────────────────────

struct OtpRow: View {
    let code: String
    let remaining: Int
    let period: Int
    let onCopy: () -> Void

    private var progress: Double {
        Double(remaining) / Double(max(period, 1))
    }

    private var isUrgent: Bool { remaining <= 5 }
    private var codeColor: Color { isUrgent ? .red : .blue }

    var body: some View {
        Button(action: onCopy) {
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Label("OTP", systemImage: "clock")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                    Spacer()
                    // Countdown ring
                    ZStack {
                        Circle()
                            .stroke(Color.secondary.opacity(0.25), lineWidth: 2.5)
                        Circle()
                            .trim(from: 0, to: progress)
                            .stroke(codeColor, style: StrokeStyle(lineWidth: 2.5, lineCap: .round))
                            .rotationEffect(.degrees(-90))
                            .animation(.linear(duration: 1), value: remaining)
                        Text("\(remaining)")
                            .font(.system(size: 9, weight: .bold, design: .rounded))
                            .foregroundColor(codeColor)
                    }
                    .frame(width: 24, height: 24)
                }

                Text(formattedCode)
                    .font(.system(.title3, design: .monospaced))
                    .fontWeight(.bold)
                    .foregroundColor(codeColor)
                    .accessibilityLabel("OTP code: \(code.map { String($0) }.joined(separator: " "))")
            }
            .padding(8)
            .background(codeColor.opacity(0.08))
            .cornerRadius(8)
        }
        .buttonStyle(.plain)
        .accessibilityHint("Tap to copy OTP code")
    }

    var formattedCode: String {
        guard code.count == 6 else { return code }
        return "\(code.prefix(3)) \(code.suffix(3))"
    }
}

// ─── Info Row ─────────────────────────────────────────────────────────────────

struct InfoRow: View {
    let label: String
    let value: String
    let icon: String
    let onCopy: (() -> Void)?

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            Label(label, systemImage: icon)
                .font(.caption2)
                .foregroundColor(.secondary)
            Text(value)
                .font(.system(size: 13))
                .lineLimit(2)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(8)
        .background(Color.secondary.opacity(0.08))
        .cornerRadius(8)
        .onTapGesture {
            onCopy?()
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(label): \(value)")
        .accessibilityHint(onCopy != nil ? "Tap to copy" : "")
    }
}

// ─── Models ───────────────────────────────────────────────────────────────────

struct WatchEntry: Identifiable {
    let id = UUID()
    let uuid: String
    let title: String
    let username: String
    let url: String?
    let hasOtp: Bool
    let hasNotes: Bool
    let isFavorite: Bool
}

// ─── View Model ───────────────────────────────────────────────────────────────

@MainActor
class WatchVaultManager: ObservableObject {
    @Published var isLocked = true
    @Published var entries: [WatchEntry] = []
    @Published var isLoading = false

    func unlock(completion: @escaping (Bool) -> Void) {
        isLoading = true
        WatchConnectivityManager.shared.requestUnlock { [weak self] success in
            Task { @MainActor in
                self?.isLoading = false
                if success {
                    self?.isLocked = false
                    await self?.refreshEntries()
                }
                completion(success)
            }
        }
    }

    func lock() {
        isLocked = true
        entries = []
    }

    func refreshEntries() async {
        await withCheckedContinuation { continuation in
            WatchConnectivityManager.shared.requestEntries { [weak self] entries in
                Task { @MainActor in
                    self?.entries = entries
                    continuation.resume()
                }
            }
        }
    }
}

// ─── Watch Connectivity ───────────────────────────────────────────────────────

class WatchConnectivityManager: NSObject, WCSessionDelegate {
    static let shared = WatchConnectivityManager()

    weak var vaultManager: WatchVaultManager?
    private var session: WCSession?

    override init() {
        super.init()
        if WCSession.isSupported() {
            session = WCSession.default
            session?.delegate = self
            session?.activate()
        }
    }

    func requestUnlock(completion: @escaping (Bool) -> Void) {
        session?.sendMessage(
            ["action": "unlock"],
            replyHandler: { reply in completion(reply["success"] as? Bool ?? false) },
            errorHandler: { _ in completion(false) }
        )
    }

    func requestEntries(completion: @escaping ([WatchEntry]) -> Void) {
        session?.sendMessage(
            ["action": "getEntries"],
            replyHandler: { reply in
                guard let raw = reply["entries"] as? [[String: Any]] else {
                    completion([])
                    return
                }
                let entries = raw.compactMap { dict -> WatchEntry? in
                    guard let uuid = dict["uuid"] as? String,
                          let title = dict["title"] as? String else { return nil }
                    return WatchEntry(
                        uuid: uuid,
                        title: title,
                        username: dict["username"] as? String ?? "",
                        url: dict["url"] as? String,
                        hasOtp: dict["hasOtp"] as? Bool ?? false,
                        hasNotes: dict["hasNotes"] as? Bool ?? false,
                        isFavorite: dict["isFavorite"] as? Bool ?? false
                    )
                }
                completion(entries)
            },
            errorHandler: { _ in completion([]) }
        )
    }

    func requestOtp(entryUuid: String, completion: @escaping (String, Int, Int) -> Void) {
        session?.sendMessage(
            ["action": "getOtp", "uuid": entryUuid],
            replyHandler: { reply in
                let code = reply["code"] as? String ?? "------"
                let remaining = reply["remaining"] as? Int ?? 30
                let period = reply["period"] as? Int ?? 30
                completion(code, remaining, period)
            },
            errorHandler: { _ in completion("------", 30, 30) }
        )
    }

    func copyField(entryUuid: String, field: String, completion: @escaping (Bool) -> Void) {
        session?.sendMessage(
            ["action": "copyField", "uuid": entryUuid, "field": field],
            replyHandler: { reply in completion(reply["success"] as? Bool ?? false) },
            errorHandler: { _ in completion(false) }
        )
    }

    // WCSessionDelegate
    func session(
        _ session: WCSession,
        activationDidCompleteWith activationState: WCSessionActivationState,
        error: Error?
    ) {}

    func session(_ session: WCSession, didReceiveMessage message: [String: Any]) {
        // Handle push messages from iPhone (e.g., vault locked remotely)
        if let action = message["action"] as? String, action == "vaultLocked" {
            Task { @MainActor in
                vaultManager?.lock()
            }
        }
    }
}
