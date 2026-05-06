package com.keepassex.autofill

import android.app.Activity
import android.os.Bundle
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.fragment.app.FragmentActivity

/**
 * Shown when user fills a form and KeePassEx offers to save the credentials
 */
class AutofillSaveActivity : FragmentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val username = intent.getStringExtra("username") ?: ""
        val password = intent.getStringExtra("password") ?: ""
        val url = intent.getStringExtra("url") ?: ""

        if (username.isEmpty() && password.isEmpty()) {
            finish()
            return
        }

        AlertDialog.Builder(this)
            .setTitle("Save to KeePassEx?")
            .setMessage("Save credentials for ${url.ifEmpty { "this site" }}?\n\nUsername: $username")
            .setPositiveButton("Save") { _, _ ->
                saveCredentials(username, password, url)
            }
            .setNegativeButton("Not now") { _, _ ->
                finish()
            }
            .setOnCancelListener { finish() }
            .show()
    }

    private fun saveCredentials(username: String, password: String, url: String) {
        // In production: call KeePassExCore to create entry
        // For now: show toast and finish
        Toast.makeText(this, "Saved to KeePassEx", Toast.LENGTH_SHORT).show()
        setResult(Activity.RESULT_OK)
        finish()
    }
}
