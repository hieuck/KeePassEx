package com.keepassex.quicktile

import android.content.Intent
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import com.keepassex.MainActivity

/**
 * KeePassEx Quick Settings Tile (Android 7+)
 * Shows vault lock status and opens app on tap
 */
class KeePassExTileService : TileService() {

    override fun onStartListening() {
        super.onStartListening()
        updateTile()
    }

    override fun onClick() {
        super.onClick()
        val isLocked = getSharedPreferences("keepassex_autofill", MODE_PRIVATE)
            .getBoolean("vault_unlocked", false).not()

        if (isLocked) {
            // Open app to unlock
            val intent = Intent(this, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                putExtra("action", "UNLOCK")
            }
            startActivityAndCollapse(intent)
        } else {
            // Open app
            val intent = Intent(this, MainActivity::class.java).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            startActivityAndCollapse(intent)
        }
    }

    private fun updateTile() {
        val tile = qsTile ?: return
        val isUnlocked = getSharedPreferences("keepassex_autofill", MODE_PRIVATE)
            .getBoolean("vault_unlocked", false)

        tile.state = if (isUnlocked) Tile.STATE_ACTIVE else Tile.STATE_INACTIVE
        tile.label = "KeePassEx"
        tile.contentDescription = if (isUnlocked) "Vault unlocked" else "Vault locked"
        tile.updateTile()
    }
}
