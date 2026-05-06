// KeePassEx Menu Bar App — macOS
// Native SwiftUI menu bar app for quick vault access.
// No competitor (KeePass, KeePassXC, Keepassium) has a dedicated menu bar app.
//
// Features:
// - Quick search entries from menu bar
// - Copy password/username with one click
// - OTP display with countdown timer
// - Lock/unlock vault
// - Keyboard shortcut: Cmd+Shift+K
// - Syncs with main KeePassEx desktop app via XPC

import SwiftUI
import AppKit

@main
struct KeePassExMenuBarApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    var body: some Scene {
        // No main window — menu bar only
        Settings {
            EmptyView()
        }
    }
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var popover: NSPopover?

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Hide from Dock — menu bar only app
        NSApp.setActivationPolicy(.accessory)

        // Create status bar item
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        if let button = statusItem?.button {
            button.image = NSImage(systemSymbolName: "key.fill", accessibilityDescription: "KeePassEx")
            button.image?.isTemplate = true // Adapts to light/dark menu bar
            button.action = #selector(togglePopover)
            button.target = self
        }

        // Create popover
        let popover = NSPopover()
        popover.contentSize = NSSize(width: 320, height: 480)
        popover.behavior = .transient
        popover.contentViewController = NSHostingController(
            rootView: MenuBarView()
                .environmentObject(MenuBarViewModel())
        )
        self.popover = popover

        // Register global keyboard shortcut: Cmd+Shift+K
        NSEvent.addGlobalMonitorForEvents(matching: .keyDown) { [weak self] event in
            if event.modifierFlags.contains([.command, .shift]) && event.keyCode == 40 { // K
                DispatchQueue.main.async {
                    self?.togglePopover()
                }
            }
        }
    }

    @objc func togglePopover() {
        guard let button = statusItem?.button else { return }
        if let popover = popover {
            if popover.isShown {
                popover.performClose(nil)
            } else {
                popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
                popover.contentViewController?.view.window?.makeKey()
            }
        }
    }
}
