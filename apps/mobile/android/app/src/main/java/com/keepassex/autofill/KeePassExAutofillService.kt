/**
 * KeePassEx Android AutoFill Service
 * Implements Android Autofill Framework (API 26+)
 */
package com.keepassex.autofill

import android.app.assist.AssistStructure
import android.os.CancellationSignal
import android.service.autofill.*
import android.view.autofill.AutofillId
import android.view.autofill.AutofillValue
import android.widget.RemoteViews
import android.content.Intent
import android.content.IntentSender
import android.app.PendingIntent
import android.content.Context
import com.keepassex.R

class KeePassExAutofillService : AutofillService() {

    override fun onFillRequest(
        request: FillRequest,
        cancellationSignal: CancellationSignal,
        callback: FillCallback
    ) {
        val structure = request.fillContexts.last().structure
        val parser = AssistStructureParser(structure)

        if (!parser.hasLoginForm) {
            callback.onSuccess(null)
            return
        }

        // Check if vault is unlocked
        val prefs = getSharedPreferences("keepassex_autofill", Context.MODE_PRIVATE)
        val isUnlocked = prefs.getBoolean("vault_unlocked", false)

        if (!isUnlocked) {
            // Show authentication UI
            val authIntent = Intent(this, AutofillAuthActivity::class.java)
            val intentSender: IntentSender = PendingIntent.getActivity(
                this, 0, authIntent,
                PendingIntent.FLAG_CANCEL_CURRENT or PendingIntent.FLAG_IMMUTABLE
            ).intentSender

            val authPresentation = RemoteViews(packageName, R.layout.autofill_item).apply {
                setTextViewText(R.id.autofill_title, "🔐 KeePassEx")
                setTextViewText(R.id.autofill_subtitle, "Tap to unlock vault")
            }

            val response = FillResponse.Builder()
                .setAuthentication(
                    parser.autofillIds.toTypedArray(),
                    intentSender,
                    authPresentation
                )
                .build()

            callback.onSuccess(response)
            return
        }

        // Load matching entries
        val packageName = structure.activityComponent.packageName
        val webDomain = parser.webDomain
        val entries = loadMatchingEntries(packageName, webDomain)

        if (entries.isEmpty()) {
            callback.onSuccess(null)
            return
        }

        val responseBuilder = FillResponse.Builder()

        for (entry in entries) {
            val dataset = buildDataset(entry, parser)
            responseBuilder.addDataset(dataset)
        }

        // Save callback for new credentials
        if (parser.usernameId != null && parser.passwordId != null) {
            val saveInfo = SaveInfo.Builder(
                SaveInfo.SAVE_DATA_TYPE_USERNAME or SaveInfo.SAVE_DATA_TYPE_PASSWORD,
                arrayOf(parser.usernameId!!, parser.passwordId!!)
            ).build()
            responseBuilder.setSaveInfo(saveInfo)
        }

        callback.onSuccess(responseBuilder.build())
    }

    override fun onSaveRequest(request: SaveRequest, callback: SaveCallback) {
        val structure = request.fillContexts.last().structure
        val parser = AssistStructureParser(structure)

        // Extract filled values
        val username = parser.usernameValue ?: ""
        val password = parser.passwordValue ?: ""

        if (username.isNotEmpty() || password.isNotEmpty()) {
            // Launch save dialog
            val saveIntent = Intent(this, AutofillSaveActivity::class.java).apply {
                putExtra("username", username)
                putExtra("password", password)
                putExtra("url", parser.webDomain ?: "")
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            startActivity(saveIntent)
        }

        callback.onSuccess()
    }

    private fun buildDataset(entry: AutofillEntry, parser: AssistStructureParser): Dataset {
        val presentation = RemoteViews(packageName, R.layout.autofill_item).apply {
            setTextViewText(R.id.autofill_title, entry.title)
            setTextViewText(R.id.autofill_subtitle, entry.username)
        }

        val builder = Dataset.Builder(presentation)

        parser.usernameId?.let { id ->
            builder.setValue(id, AutofillValue.forText(entry.username))
        }

        parser.passwordId?.let { id ->
            builder.setValue(id, AutofillValue.forText(entry.password))
        }

        return builder.build()
    }

    private fun loadMatchingEntries(packageName: String, webDomain: String?): List<AutofillEntry> {
        // Load from shared preferences (populated by main app)
        val prefs = getSharedPreferences("keepassex_autofill", Context.MODE_PRIVATE)
        val entriesJson = prefs.getString("entries", "[]") ?: "[]"

        return try {
            val jsonArray = org.json.JSONArray(entriesJson)
            val entries = mutableListOf<AutofillEntry>()

            for (i in 0 until jsonArray.length()) {
                val obj = jsonArray.getJSONObject(i)
                val entry = AutofillEntry(
                    uuid = obj.getString("uuid"),
                    title = obj.getString("title"),
                    username = obj.getString("username"),
                    password = obj.getString("password"),
                    url = obj.optString("url"),
                    packageNames = obj.optJSONArray("packageNames")?.let { arr ->
                        (0 until arr.length()).map { arr.getString(it) }
                    } ?: emptyList()
                )

                // Match by package name or web domain
                val matches = entry.packageNames.contains(packageName) ||
                    (webDomain != null && entry.url.contains(webDomain))

                if (matches) entries.add(entry)
            }

            entries
        } catch (e: Exception) {
            emptyList()
        }
    }
}

// ─── Assist Structure Parser ──────────────────────────────────────────────────

class AssistStructureParser(structure: AssistStructure) {

    var usernameId: AutofillId? = null
    var passwordId: AutofillId? = null
    var usernameValue: String? = null
    var passwordValue: String? = null
    var webDomain: String? = null
    val autofillIds = mutableListOf<AutofillId>()
    var hasLoginForm = false

    init {
        parseStructure(structure)
    }

    private fun parseStructure(structure: AssistStructure) {
        for (i in 0 until structure.windowNodeCount) {
            parseNode(structure.getWindowNodeAt(i).rootViewNode)
        }
        hasLoginForm = passwordId != null
    }

    private fun parseNode(node: AssistStructure.ViewNode) {
        val hints = node.autofillHints ?: emptyArray()
        val inputType = node.inputType
        val autofillId = node.autofillId

        if (autofillId != null) {
            autofillIds.add(autofillId)
        }

        // Detect username field
        if (hints.any { it.contains("username", ignoreCase = true) ||
                it.contains("email", ignoreCase = true) } ||
            (inputType and android.text.InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS != 0)) {
            usernameId = autofillId
            usernameValue = node.autofillValue?.textValue?.toString()
        }

        // Detect password field
        if (hints.any { it.contains("password", ignoreCase = true) } ||
            (inputType and android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD != 0) ||
            (inputType and android.text.InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD != 0)) {
            passwordId = autofillId
            passwordValue = node.autofillValue?.textValue?.toString()
        }

        // Web domain
        if (node.webDomain != null) {
            webDomain = node.webDomain
        }

        // Recurse
        for (i in 0 until node.childCount) {
            parseNode(node.getChildAt(i))
        }
    }
}

// ─── Models ───────────────────────────────────────────────────────────────────

data class AutofillEntry(
    val uuid: String,
    val title: String,
    val username: String,
    val password: String,
    val url: String,
    val packageNames: List<String> = emptyList()
)
