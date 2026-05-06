/**
 * KeePassEx watchOS Complications
 * Shows vault lock status and quick access on watch face
 */
import ClockKit
import SwiftUI

class ComplicationController: NSObject, CLKComplicationDataSource {

    // MARK: - Complication Configuration

    func getComplicationDescriptors(handler: @escaping ([CLKComplicationDescriptor]) -> Void) {
        let descriptors = [
            CLKComplicationDescriptor(
                identifier: "keepassex.lock-status",
                displayName: "KeePassEx",
                supportedFamilies: [
                    .modularSmall,
                    .circularSmall,
                    .utilitarianSmall,
                    .graphicCorner,
                    .graphicCircular,
                ]
            )
        ]
        handler(descriptors)
    }

    // MARK: - Timeline

    func getCurrentTimelineEntry(
        for complication: CLKComplication,
        withHandler handler: @escaping (CLKComplicationTimelineEntry?) -> Void
    ) {
        let entry = makeTimelineEntry(for: complication, date: Date())
        handler(entry)
    }

    func getTimelineEntries(
        for complication: CLKComplication,
        after date: Date,
        limit: Int,
        withHandler handler: @escaping ([CLKComplicationTimelineEntry]?) -> Void
    ) {
        handler(nil)
    }

    // MARK: - Template Building

    private func makeTimelineEntry(
        for complication: CLKComplication,
        date: Date
    ) -> CLKComplicationTimelineEntry? {
        guard let template = makeTemplate(for: complication) else { return nil }
        return CLKComplicationTimelineEntry(date: date, complicationTemplate: template)
    }

    private func makeTemplate(for complication: CLKComplication) -> CLKComplicationTemplate? {
        let isLocked = UserDefaults(suiteName: "group.com.keepassex.app")?
            .bool(forKey: "vault_locked") ?? true

        let icon = isLocked ? "lock.fill" : "key.fill"
        let tint = isLocked ? UIColor.systemOrange : UIColor.systemGreen

        switch complication.family {
        case .modularSmall:
            let template = CLKComplicationTemplateModularSmallSimpleImage()
            template.imageProvider = CLKImageProvider(
                onePieceImage: UIImage(systemName: icon) ?? UIImage()
            )
            template.imageProvider.tintColor = tint
            return template

        case .circularSmall:
            let template = CLKComplicationTemplateCircularSmallSimpleImage()
            template.imageProvider = CLKImageProvider(
                onePieceImage: UIImage(systemName: icon) ?? UIImage()
            )
            return template

        case .graphicCircular:
            let template = CLKComplicationTemplateGraphicCircularImage()
            template.imageProvider = CLKFullColorImageProvider(
                fullColorImage: UIImage(systemName: icon) ?? UIImage()
            )
            return template

        default:
            return nil
        }
    }

    // MARK: - Placeholder

    func getLocalizableSampleTemplate(
        for complication: CLKComplication,
        withHandler handler: @escaping (CLKComplicationTemplate?) -> Void
    ) {
        handler(makeTemplate(for: complication))
    }
}
