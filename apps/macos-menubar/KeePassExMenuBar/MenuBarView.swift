// MenuBarView — SwiftUI popover UI for the KeePassEx menu bar app
// Compact, keyboard-navigable interface for quick vault access

import SwiftUI

struct MenuBarView: View {
    @EnvironmentObject var vm: MenuBarViewModel
    @FocusState private var searchFocused: Bool

    var body: some View {
        VStack(spacing: 0) {
            // ─── Header ───────────────────────────────────────────────────────
            HStack {
                Image(systemName: "key.fill")
                    .foregroundColor(.accentColor)
                Text(vm.vaultName.isEmpty ? "KeePassEx" : vm.vaultName)
                    .font(.headline)
                    .lineLimit(1)
                Spacer()
                statusIndicator
                menuButton
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color(NSColor.windowBackgroundColor))

            Divider()

            if !vm.isConnected {
                notConnectedView
            } else if vm.isVaultLocked {
                lockedView
            } else {
                unlockedView
            }
        }
        .frame(width: 320)
        .onAppear {
            searchFocused = true
        }
    }

    // ─── Status indicator ─────────────────────────────────────────────────────

    private var statusIndicator: some View {
        Circle()
            .fill(vm.isConnected ? (vm.isVaultLocked ? Color.orange : Color.green) : Color.red)
            .frame(width: 8, height: 8)
            .help(
                vm.isConnected
                    ? (vm.isVaultLocked ? "Vault locked" : "Vault unlocked") : "Not connected")
    }

    // ─── Menu button ──────────────────────────────────────────────────────────

    private var menuButton: some View {
        Menu {
            Button("Open KeePassEx") { vm.openMainApp() }
            Divider()
            if !vm.isVaultLocked {
                Button("Lock Vault") { vm.lockVault() }
            }
            Divider()
            Button("Quit") { NSApp.terminate(nil) }
        } label: {
            Image(systemName: "ellipsis.circle")
                .foregroundColor(.secondary)
        }
        .menuStyle(.borderlessButton)
        .fixedSize()
    }

    // ─── Not connected ────────────────────────────────────────────────────────

    private var notConnectedView: some View {
        VStack(spacing: 16) {
            Image(systemName: "wifi.slash")
                .font(.system(size: 32))
                .foregroundColor(.secondary)
            Text("KeePassEx not running")
                .font(.headline)
            Text("Open the KeePassEx app to use the menu bar.")
                .font(.caption)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            Button("Open KeePassEx") { vm.openMainApp() }
                .buttonStyle(.borderedProminent)
        }
        .padding(24)
        .frame(maxWidth: .infinity)
    }

    // ─── Locked vault ─────────────────────────────────────────────────────────

    private var lockedView: some View {
        VStack(spacing: 16) {
            Image(systemName: "lock.fill")
                .font(.system(size: 32))
                .foregroundColor(.orange)
            Text("Vault is locked")
                .font(.headline)
            Text("Unlock your vault in KeePassEx to access entries.")
                .font(.caption)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            Button("Unlock in KeePassEx") { vm.openMainApp() }
                .buttonStyle(.borderedProminent)
        }
        .padding(24)
        .frame(maxWidth: .infinity)
    }

    // ─── Unlocked vault ───────────────────────────────────────────────────────

    private var unlockedView: some View {
        VStack(spacing: 0) {
            // Search bar
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundColor(.secondary)
                TextField("Search entries...", text: $vm.searchQuery)
                    .textFieldStyle(.plain)
                    .focused($searchFocused)
                if !vm.searchQuery.isEmpty {
                    Button {
                        vm.searchQuery = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color(NSColor.controlBackgroundColor))

            Divider()

            // Entry list
            if vm.filteredEntries.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        if vm.searchQuery.isEmpty {
                            sectionHeader("Recent")
                        }
                        ForEach(vm.filteredEntries) { entry in
                            EntryRow(entry: entry, otpCode: vm.otpCodes[entry.id], vm: vm)
                            if entry.id != vm.filteredEntries.last?.id {
                                Divider().padding(.leading, 44)
                            }
                        }
                    }
                }
                .frame(maxHeight: 360)
            }

            Divider()

            // Footer
            HStack {
                Text("\(vm.filteredEntries.count) entries")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                Button("Open KeePassEx") { vm.openMainApp() }
                    .font(.caption)
                    .buttonStyle(.plain)
                    .foregroundColor(.accentColor)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
        }
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 24))
                .foregroundColor(.secondary)
            Text(
                vm.searchQuery.isEmpty
                    ? "No recent entries" : "No results for \"\(vm.searchQuery)\""
            )
            .font(.caption)
            .foregroundColor(.secondary)
        }
        .padding(24)
        .frame(maxWidth: .infinity)
    }

    private func sectionHeader(_ title: String) -> some View {
        HStack {
            Text(title.uppercased())
                .font(.system(size: 10, weight: .semibold))
                .foregroundColor(.secondary)
                .tracking(0.5)
            Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.top, 8)
        .padding(.bottom, 4)
    }
}

// ─── Entry Row ────────────────────────────────────────────────────────────────

struct EntryRow: View {
    let entry: MenuBarEntry
    let otpCode: OtpCode?
    let vm: MenuBarViewModel

    @State private var isHovered = false
    @State private var copied = false

    var body: some View {
        HStack(spacing: 10) {
            // Icon
            ZStack {
                RoundedRectangle(cornerRadius: 6)
                    .fill(Color.accentColor.opacity(0.12))
                    .frame(width: 32, height: 32)
                Image(systemName: iconName(for: entry.iconId))
                    .font(.system(size: 14))
                    .foregroundColor(.accentColor)
            }

            // Info
            VStack(alignment: .leading, spacing: 2) {
                Text(entry.title)
                    .font(.system(size: 13, weight: .medium))
                    .lineLimit(1)
                if !entry.username.isEmpty {
                    Text(entry.username)
                        .font(.system(size: 11))
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
            }

            Spacer()

            // OTP countdown
            if let otp = otpCode {
                OtpBadge(otp: otp)
            }

            // Action buttons (shown on hover)
            if isHovered {
                HStack(spacing: 4) {
                    actionButton(icon: "doc.on.doc", tooltip: "Copy Password") {
                        Task { await vm.copyPassword(entryId: entry.id) }
                        flashCopied()
                    }
                    actionButton(icon: "person", tooltip: "Copy Username") {
                        Task { await vm.copyUsername(entryId: entry.id) }
                    }
                    if entry.hasOtp {
                        actionButton(icon: "clock", tooltip: "Copy OTP") {
                            Task { await vm.copyOtp(entryId: entry.id) }
                        }
                    }
                }
                .transition(.opacity)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(
            isHovered ? Color(NSColor.selectedContentBackgroundColor).opacity(0.1) : Color.clear
        )
        .onHover { isHovered = $0 }
        .animation(.easeInOut(duration: 0.15), value: isHovered)
        .overlay(
            copied ? copiedOverlay : nil
        )
    }

    private func actionButton(icon: String, tooltip: String, action: @escaping () -> Void)
        -> some View
    {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .frame(width: 24, height: 24)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(4)
        }
        .buttonStyle(.plain)
        .help(tooltip)
    }

    private var copiedOverlay: some View {
        HStack {
            Spacer()
            Text("Copied!")
                .font(.caption)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Color.green)
                .foregroundColor(.white)
                .cornerRadius(4)
            Spacer()
        }
        .background(Color(NSColor.windowBackgroundColor).opacity(0.9))
    }

    private func flashCopied() {
        copied = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            copied = false
        }
    }

    private func iconName(for iconId: Int) -> String {
        switch iconId {
        case 0: return "key.fill"
        case 1: return "globe"
        case 8: return "envelope.fill"
        case 9: return "creditcard.fill"
        case 10: return "building.columns.fill"
        case 11: return "iphone"
        default: return "lock.fill"
        }
    }
}

// ─── OTP Badge ────────────────────────────────────────────────────────────────

struct OtpBadge: View {
    let otp: OtpCode

    var body: some View {
        HStack(spacing: 4) {
            // Countdown ring
            ZStack {
                Circle()
                    .stroke(Color.secondary.opacity(0.2), lineWidth: 2)
                Circle()
                    .trim(from: 0, to: otp.progress)
                    .stroke(otp.remainingSeconds > 5 ? Color.green : Color.red, lineWidth: 2)
                    .rotationEffect(.degrees(-90))
                    .animation(.linear(duration: 1), value: otp.progress)
            }
            .frame(width: 16, height: 16)

            Text(otp.code)
                .font(.system(size: 12, weight: .medium, design: .monospaced))
                .foregroundColor(otp.remainingSeconds > 5 ? .primary : .red)
        }
    }
}
