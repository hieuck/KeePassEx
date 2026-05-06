package com.keepassex.widget

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.Context
import android.content.Intent
import android.widget.RemoteViews
import com.keepassex.MainActivity
import com.keepassex.R
import org.json.JSONArray

/**
 * KeePassEx Android Home Screen Widget
 * Shows vault lock status and quick OTP access
 */
class KeePassExWidget : AppWidgetProvider() {

    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray
    ) {
        appWidgetIds.forEach { widgetId ->
            updateWidget(context, appWidgetManager, widgetId)
        }
    }

    companion object {
        fun updateWidget(
            context: Context,
            appWidgetManager: AppWidgetManager,
            widgetId: Int
        ) {
            val prefs = context.getSharedPreferences("keepassex_autofill", Context.MODE_PRIVATE)
            val isLocked = !prefs.getBoolean("vault_unlocked", false)
            val vaultName = prefs.getString("vault_name", "KeePassEx") ?: "KeePassEx"

            val views = RemoteViews(context.packageName, R.layout.widget_keepassex)

            // Update lock status
            views.setTextViewText(R.id.widget_vault_name, vaultName)
            views.setTextViewText(
                R.id.widget_status,
                if (isLocked) "🔒 Locked" else "🔓 Open"
            )

            // Load OTP entries
            val otpJson = prefs.getString("widget_otp_entries", "[]") ?: "[]"
            try {
                val otpArray = JSONArray(otpJson)
                if (otpArray.length() > 0 && !isLocked) {
                    val firstOtp = otpArray.getJSONObject(0)
                    views.setTextViewText(R.id.widget_otp_title, firstOtp.optString("title"))
                    views.setTextViewText(R.id.widget_otp_code, "--- ---") // Placeholder
                    views.setViewVisibility(R.id.widget_otp_section, android.view.View.VISIBLE)
                } else {
                    views.setViewVisibility(R.id.widget_otp_section, android.view.View.GONE)
                }
            } catch (e: Exception) {
                views.setViewVisibility(R.id.widget_otp_section, android.view.View.GONE)
            }

            // Tap action — open app
            val intent = Intent(context, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                putExtra("action", if (isLocked) "UNLOCK" else "OPEN")
            }
            val pendingIntent = PendingIntent.getActivity(
                context, widgetId, intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
            views.setOnClickPendingIntent(R.id.widget_root, pendingIntent)

            appWidgetManager.updateAppWidget(widgetId, views)
        }
    }
}
