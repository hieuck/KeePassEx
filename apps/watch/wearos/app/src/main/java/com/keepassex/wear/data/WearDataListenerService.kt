package com.keepassex.wear.data

import com.google.android.gms.wearable.DataEventBuffer
import com.google.android.gms.wearable.MessageEvent
import com.google.android.gms.wearable.WearableListenerService
import org.json.JSONArray
import org.json.JSONObject

/**
 * Listens for data and messages from the paired Android phone
 * Receives vault entries and lock status updates
 */
class WearDataListenerService : WearableListenerService() {

    override fun onDataChanged(dataEvents: DataEventBuffer) {
        dataEvents.forEach { event ->
            val path = event.dataItem.uri.path ?: return@forEach

            when {
                path.startsWith("/keepassex/entries") -> {
                    val data = event.dataItem.data ?: return@forEach
                    val json = String(data)
                    updateEntries(json)
                }
                path.startsWith("/keepassex/status") -> {
                    val data = event.dataItem.data ?: return@forEach
                    val json = JSONObject(String(data))
                    updateVaultStatus(
                        locked = json.optBoolean("locked", true),
                        vaultName = json.optString("vaultName", "")
                    )
                }
            }
        }
    }

    override fun onMessageReceived(messageEvent: MessageEvent) {
        when (messageEvent.path) {
            "/keepassex/otp-response" -> {
                val json = JSONObject(String(messageEvent.data))
                val requestId = json.optString("requestId")
                val code = json.optString("code")
                val remaining = json.optInt("remaining", 30)
                OtpResponseBus.deliver(requestId, code, remaining)
            }
            "/keepassex/unlock-response" -> {
                val success = JSONObject(String(messageEvent.data)).optBoolean("success", false)
                UnlockResponseBus.deliver(success)
            }
        }
    }

    private fun updateEntries(json: String) {
        try {
            val array = JSONArray(json)
            val entries = (0 until array.length()).map { i ->
                val obj = array.getJSONObject(i)
                WearEntry(
                    uuid = obj.getString("uuid"),
                    title = obj.getString("title"),
                    username = obj.optString("username"),
                    hasOtp = obj.optBoolean("hasOtp", false)
                )
            }
            WearDataStore.entries = entries
        } catch (e: Exception) {
            // Ignore parse errors
        }
    }

    private fun updateVaultStatus(locked: Boolean, vaultName: String) {
        WearDataStore.isLocked = locked
        WearDataStore.vaultName = vaultName

        // Update shared preferences for tile
        getSharedPreferences("keepassex", MODE_PRIVATE)
            .edit()
            .putBoolean("vault_locked", locked)
            .putString("vault_name", vaultName)
            .apply()
    }
}

// ─── Simple in-memory data store ─────────────────────────────────────────────

object WearDataStore {
    var entries: List<WearEntry> = emptyList()
    var isLocked: Boolean = true
    var vaultName: String = ""
}

data class WearEntry(
    val uuid: String,
    val title: String,
    val username: String,
    val hasOtp: Boolean
)

// ─── Response buses (simple callback mechanism) ───────────────────────────────

object OtpResponseBus {
    private val callbacks = mutableMapOf<String, (String, Int) -> Unit>()

    fun register(requestId: String, callback: (String, Int) -> Unit) {
        callbacks[requestId] = callback
    }

    fun deliver(requestId: String, code: String, remaining: Int) {
        callbacks.remove(requestId)?.invoke(code, remaining)
    }
}

object UnlockResponseBus {
    private var callback: ((Boolean) -> Unit)? = null

    fun register(callback: (Boolean) -> Unit) {
        this.callback = callback
    }

    fun deliver(success: Boolean) {
        callback?.invoke(success)
        callback = null
    }
}
