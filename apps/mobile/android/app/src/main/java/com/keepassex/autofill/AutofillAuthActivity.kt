package com.keepassex.autofill

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.service.autofill.Dataset
import android.view.autofill.AutofillManager
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity

/**
 * Transparent activity shown when vault is locked during AutoFill
 * Prompts biometric or password authentication before filling
 */
class AutofillAuthActivity : FragmentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val biometricManager = BiometricManager.from(this)
        val canUseBiometric = biometricManager.canAuthenticate(
            BiometricManager.Authenticators.BIOMETRIC_STRONG
        ) == BiometricManager.BIOMETRIC_SUCCESS

        if (canUseBiometric) {
            showBiometricPrompt()
        } else {
            // Fall back to password — launch main app
            launchMainApp()
        }
    }

    private fun showBiometricPrompt() {
        val executor = ContextCompat.getMainExecutor(this)

        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                onAuthSuccess()
            }

            override fun onAuthenticationFailed() {
                setResult(Activity.RESULT_CANCELED)
                finish()
            }

            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                if (errorCode == BiometricPrompt.ERROR_NEGATIVE_BUTTON) {
                    launchMainApp()
                } else {
                    setResult(Activity.RESULT_CANCELED)
                    finish()
                }
            }
        }

        val promptInfo = BiometricPrompt.PromptInfo.Builder()
            .setTitle("KeePassEx")
            .setSubtitle("Unlock vault to fill credentials")
            .setNegativeButtonText("Use Password")
            .build()

        BiometricPrompt(this, executor, callback).authenticate(promptInfo)
    }

    private fun onAuthSuccess() {
        // Mark vault as unlocked for autofill
        getSharedPreferences("keepassex_autofill", MODE_PRIVATE)
            .edit()
            .putBoolean("vault_unlocked", true)
            .apply()

        // Return success to AutoFill framework
        val replyIntent = Intent()
        setResult(Activity.RESULT_OK, replyIntent)
        finish()
    }

    private fun launchMainApp() {
        val intent = packageManager.getLaunchIntentForPackage(packageName)
        intent?.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        startActivity(intent)
        setResult(Activity.RESULT_CANCELED)
        finish()
    }
}
