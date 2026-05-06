/**
 * KeePassEx iOS — App Delegate
 */
import UIKit
import React
import React_RCTAppDelegate

@main
class AppDelegate: RCTAppDelegate {

    override func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        self.moduleName = "KeePassEx"
        self.dependencyProvider = RCTAppDependencyProvider()

        // Disable screen capture for security
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleScreenCapture),
            name: UIScreen.capturedDidChangeNotification,
            object: nil
        )

        return super.application(application, didFinishLaunchingWithOptions: launchOptions)
    }

    override func sourceURL(for bridge: RCTBridge) -> URL? {
        self.bundleURL()
    }

    override func bundleURL() -> URL? {
        #if DEBUG
        return RCTBundleURLProvider.sharedSettings().jsBundleURL(forBundleRoot: "index")
        #else
        return Bundle.main.url(forResource: "main", withExtension: "jsbundle")
        #endif
    }

    // MARK: - Screen Capture Protection

    @objc private func handleScreenCapture() {
        if UIScreen.main.isCaptured {
            // Blur sensitive content when screen is being captured/recorded
            NotificationCenter.default.post(name: .keepassexScreenCaptured, object: nil)
        } else {
            NotificationCenter.default.post(name: .keepassexScreenCaptureEnded, object: nil)
        }
    }

    // MARK: - Background / Foreground

    override func applicationDidEnterBackground(_ application: UIApplication) {
        // Lock vault when app goes to background
        NotificationCenter.default.post(name: .keepassexAppBackground, object: nil)
    }

    override func applicationWillEnterForeground(_ application: UIApplication) {
        NotificationCenter.default.post(name: .keepassexAppForeground, object: nil)
    }
}

// MARK: - Notification Names

extension Notification.Name {
    static let keepassexScreenCaptured = Notification.Name("keepassex.screenCaptured")
    static let keepassexScreenCaptureEnded = Notification.Name("keepassex.screenCaptureEnded")
    static let keepassexAppBackground = Notification.Name("keepassex.appBackground")
    static let keepassexAppForeground = Notification.Name("keepassex.appForeground")
}
