package com.keepassex.wear.complication

import android.app.PendingIntent
import android.content.Intent
import androidx.wear.watchface.complications.data.*
import androidx.wear.watchface.complications.datasource.ComplicationRequest
import androidx.wear.watchface.complications.datasource.SuspendingComplicationDataSourceService
import com.keepassex.wear.data.WearDataStore
import com.keepassex.wear.presentation.MainActivity

/**
 * KeePassEx watch face complication
 * Shows vault lock status (🔒 / 🔓) on the watch face
 */
class KeePassExComplicationService : SuspendingComplicationDataSourceService() {

    override fun getPreviewData(type: ComplicationType): ComplicationData? {
        return when (type) {
            ComplicationType.SHORT_TEXT -> ShortTextComplicationData.Builder(
                text = PlainComplicationText.Builder("🔑").build(),
                contentDescription = PlainComplicationText.Builder("KeePassEx").build()
            ).build()
            ComplicationType.SMALL_IMAGE -> SmallImageComplicationData.Builder(
                smallImage = SmallImage.Builder(
                    image = android.graphics.drawable.Icon.createWithResource(this, android.R.drawable.ic_lock_lock),
                    type = SmallImageType.ICON
                ).build(),
                contentDescription = PlainComplicationText.Builder("KeePassEx").build()
            ).build()
            else -> null
        }
    }

    override suspend fun onComplicationRequest(request: ComplicationRequest): ComplicationData? {
        val isLocked = WearDataStore.isLocked
        val icon = if (isLocked) "🔒" else "🔓"
        val desc = if (isLocked) "Vault locked" else "Vault open"

        val tapIntent = PendingIntent.getActivity(
            this, 0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        return when (request.complicationType) {
            ComplicationType.SHORT_TEXT -> ShortTextComplicationData.Builder(
                text = PlainComplicationText.Builder(icon).build(),
                contentDescription = PlainComplicationText.Builder(desc).build()
            )
                .setTapAction(tapIntent)
                .build()

            else -> null
        }
    }
}
