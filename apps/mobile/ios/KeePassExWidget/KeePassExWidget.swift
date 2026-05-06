/**
 * KeePassEx iOS Widget — WidgetKit
 * Shows vault lock status and quick OTP access on the home screen
 */
import WidgetKit
import SwiftUI
import AppIntents

// ─── Widget Entry ─────────────────────────────────────────────────────────────

struct VaultEntry: TimelineEntry {
    let date: Date
    let isLocked: Bool
    let vaultName: String
    let otpEntries: [OtpWidgetEntry]
    let recentEntries: [RecentWidgetEntry]
}

struct OtpWidgetEntry: Identifiable {
    let id: String
    let title: String
    let issuer: String
    let code: String
    let remainingSeconds: Int
}

struct RecentWidgetEntry: Identifiable {
    let id: String
    let title: String
    let username: String
}

// ─── Timeline Provider ────────────────────────────────────────────────────────

struct VaultProvider: TimelineProvider {
    func placeholder(in context: Context) -> VaultEntry {
        VaultEntry(
            date: Date(),
            isLocked: false,
            vaultName: "My Vault",
            otpEntries: [
                OtpWidgetEntry(id: "1", title: "GitHub", issuer: "GitHub", code: "123 456", remainingSeconds: 20),
            ],
            recentEntries: [
                RecentWidgetEntry(id: "1", title: "GitHub", username: "user@example.com"),
                RecentWidgetEntry(id: "2", title: "Gmail", username: "user@gmail.com"),
            ]
        )
    }

    func getSnapshot(in context: Context, completion: @escaping (VaultEntry) -> Void) {
        completion(loadEntry())
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<VaultEntry>) -> Void) {
        let entry = loadEntry()
        // Refresh every 30 seconds for OTP countdown
        let nextUpdate = Calendar.current.date(byAdding: .second, value: 30, to: Date())!
        let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))
        completion(timeline)
    }

    private func loadEntry() -> VaultEntry {
        let defaults = UserDefaults(suiteName: "group.com.keepassex.app")
        let isLocked = defaults?.bool(forKey: "vault_locked") ?? true
        let vaultName = defaults?.string(forKey: "vault_name") ?? "KeePassEx"

        // Load OTP entries from shared container
        var otpEntries: [OtpWidgetEntry] = []
        if let data = defaults?.data(forKey: "widget_otp_entries"),
           let json = try? JSONDecoder().decode([OtpWidgetData].self, from: data) {
            otpEntries = json.map { item in
                OtpWidgetEntry(
                    id: item.uuid,
                    title: item.title,
                    issuer: item.issuer,
                    code: generateOtpCode(secret: item.secret),
                    remainingSeconds: otpRemainingSeconds()
                )
            }
        }

        // Load recent entries
        var recentEntries: [RecentWidgetEntry] = []
        if let data = defaults?.data(forKey: "widget_recent_entries"),
           let json = try? JSONDecoder().decode([RecentWidgetData].self, from: data) {
            recentEntries = json.prefix(4).map { item in
                RecentWidgetEntry(id: item.uuid, title: item.title, username: item.username)
            }
        }

        return VaultEntry(
            date: Date(),
            isLocked: isLocked,
            vaultName: vaultName,
            otpEntries: otpEntries,
            recentEntries: recentEntries
        )
    }

    private func otpRemainingSeconds() -> Int {
        let seconds = Int(Date().timeIntervalSince1970)
        return 30 - (seconds % 30)
    }

    private func generateOtpCode(secret: String) -> String {
        // In production: use TOTP algorithm
        // Placeholder for widget display
        return "--- ---"
    }
}

// ─── Widget Views ─────────────────────────────────────────────────────────────

struct KeePassExWidgetEntryView: View {
    var entry: VaultEntry
    @Environment(\.widgetFamily) var family

    var body: some View {
        switch family {
        case .systemSmall:
            SmallWidgetView(entry: entry)
        case .systemMedium:
            MediumWidgetView(entry: entry)
        case .systemLarge:
            LargeWidgetView(entry: entry)
        case .accessoryCircular:
            AccessoryCircularView(entry: entry)
        case .accessoryRectangular:
            AccessoryRectangularView(entry: entry)
        default:
            SmallWidgetView(entry: entry)
        }
    }
}

// Small widget — lock status
struct SmallWidgetView: View {
    let entry: VaultEntry

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: entry.isLocked ? "lock.fill" : "lock.open.fill")
                .font(.system(size: 28))
                .foregroundColor(entry.isLocked ? .orange : .green)

            Text("KeePassEx")
                .font(.caption2)
                .fontWeight(.semibold)

            Text(entry.isLocked ? "Locked" : "Open")
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .containerBackground(.background, for: .widget)
        .widgetURL(URL(string: "keepassex://unlock"))
    }
}

// Medium widget — OTP codes
struct MediumWidgetView: View {
    let entry: VaultEntry

    var body: some View {
        HStack(spacing: 0) {
            // Left: status
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Image(systemName: "lock.shield.fill")
                        .foregroundColor(.blue)
                    Text("KeePassEx")
                        .font(.caption)
                        .fontWeight(.bold)
                }
                Text(entry.vaultName)
                    .font(.caption2)
                    .foregroundColor(.secondary)
                    .lineLimit(1)

                Spacer()

                Label(
                    entry.isLocked ? "Locked" : "Open",
                    systemImage: entry.isLocked ? "lock.fill" : "lock.open.fill"
                )
                .font(.caption2)
                .foregroundColor(entry.isLocked ? .orange : .green)
            }
            .padding()
            .frame(maxWidth: .infinity)

            Divider()

            // Right: OTP entries
            VStack(alignment: .leading, spacing: 6) {
                if entry.isLocked {
                    VStack {
                        Image(systemName: "lock.fill")
                            .foregroundColor(.secondary)
                        Text("Unlock to view OTP")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if entry.otpEntries.isEmpty {
                    Text("No OTP entries")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    ForEach(entry.otpEntries.prefix(2)) { otp in
                        Link(destination: URL(string: "keepassex://otp/\(otp.id)")!) {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(otp.title)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                                    .lineLimit(1)
                                HStack {
                                    Text(otp.code)
                                        .font(.system(.body, design: .monospaced))
                                        .fontWeight(.bold)
                                        .foregroundColor(otp.remainingSeconds <= 5 ? .red : .primary)
                                    Spacer()
                                    Text("\(otp.remainingSeconds)s")
                                        .font(.caption2)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                    }
                }
            }
            .padding()
            .frame(maxWidth: .infinity)
        }
        .containerBackground(.background, for: .widget)
    }
}

// Large widget — recent entries + OTP
struct LargeWidgetView: View {
    let entry: VaultEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header
            HStack {
                Image(systemName: "lock.shield.fill")
                    .foregroundColor(.blue)
                Text("KeePassEx")
                    .font(.headline)
                Spacer()
                Label(
                    entry.isLocked ? "Locked" : "Open",
                    systemImage: entry.isLocked ? "lock.fill" : "lock.open.fill"
                )
                .font(.caption)
                .foregroundColor(entry.isLocked ? .orange : .green)
            }

            if entry.isLocked {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "lock.fill")
                        .font(.system(size: 40))
                        .foregroundColor(.secondary)
                    Text("Vault is locked")
                        .foregroundColor(.secondary)
                    Text("Tap to unlock")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity)
                Spacer()
            } else {
                // OTP section
                if !entry.otpEntries.isEmpty {
                    Text("One-Time Passwords")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .textCase(.uppercase)

                    ForEach(entry.otpEntries.prefix(3)) { otp in
                        Link(destination: URL(string: "keepassex://otp/\(otp.id)")!) {
                            HStack {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(otp.issuer.isEmpty ? otp.title : otp.issuer)
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                    Text(otp.code)
                                        .font(.system(.title3, design: .monospaced))
                                        .fontWeight(.bold)
                                        .foregroundColor(otp.remainingSeconds <= 5 ? .red : .primary)
                                }
                                Spacer()
                                Text("\(otp.remainingSeconds)s")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.vertical, 4)
                        }
                    }

                    Divider()
                }

                // Recent entries
                Text("Recent")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .textCase(.uppercase)

                ForEach(entry.recentEntries.prefix(4)) { recent in
                    Link(destination: URL(string: "keepassex://entry/\(recent.id)")!) {
                        HStack {
                            Image(systemName: "key.fill")
                                .font(.caption)
                                .foregroundColor(.blue)
                            VStack(alignment: .leading, spacing: 1) {
                                Text(recent.title)
                                    .font(.caption)
                                    .fontWeight(.medium)
                                Text(recent.username)
                                    .font(.caption2)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                            Image(systemName: "chevron.right")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                }
            }
        }
        .padding()
        .containerBackground(.background, for: .widget)
        .widgetURL(URL(string: "keepassex://"))
    }
}

// Lock screen / Dynamic Island widgets
struct AccessoryCircularView: View {
    let entry: VaultEntry

    var body: some View {
        ZStack {
            AccessoryWidgetBackground()
            Image(systemName: entry.isLocked ? "lock.fill" : "lock.open.fill")
                .font(.title3)
                .foregroundColor(entry.isLocked ? .orange : .green)
        }
        .widgetURL(URL(string: "keepassex://unlock"))
    }
}

struct AccessoryRectangularView: View {
    let entry: VaultEntry

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: entry.isLocked ? "lock.fill" : "lock.open.fill")
                .foregroundColor(entry.isLocked ? .orange : .green)
            VStack(alignment: .leading, spacing: 1) {
                Text("KeePassEx")
                    .font(.caption2)
                    .fontWeight(.semibold)
                Text(entry.isLocked ? "Locked" : "Open")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
        .widgetURL(URL(string: "keepassex://unlock"))
    }
}

// ─── Widget Configuration ─────────────────────────────────────────────────────

@main
struct KeePassExWidget: Widget {
    let kind: String = "KeePassExWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: VaultProvider()) { entry in
            KeePassExWidgetEntryView(entry: entry)
        }
        .configurationDisplayName("KeePassEx")
        .description("Quick access to your vault status and OTP codes.")
        .supportedFamilies([
            .systemSmall,
            .systemMedium,
            .systemLarge,
            .accessoryCircular,
            .accessoryRectangular,
        ])
    }
}

// ─── Data Models ──────────────────────────────────────────────────────────────

struct OtpWidgetData: Codable {
    let uuid: String
    let title: String
    let issuer: String
    let secret: String
}

struct RecentWidgetData: Codable {
    let uuid: String
    let title: String
    let username: String
}
