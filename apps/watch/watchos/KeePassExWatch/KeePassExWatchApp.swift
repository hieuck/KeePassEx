/**
 * KeePassEx watchOS App Entry Point
 */
import SwiftUI

@main
struct KeePassExWatchApp: App {
    @StateObject private var vaultManager = WatchVaultManager()
    @StateObject private var connectivityManager = WatchConnectivityManager.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(vaultManager)
                .environmentObject(connectivityManager as! ObservableObject as! WatchConnectivityManager)
        }
    }
}
