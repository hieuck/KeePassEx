/**
 * KeePassEx WearOS Tile
 * Shows vault status and quick actions on the watch tile
 */
package com.keepassex.wear.tile

import androidx.wear.tiles.*
import androidx.wear.tiles.material.*
import androidx.wear.tiles.material.layouts.*
import com.google.android.horologist.tiles.SuspendingTileService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class KeePassExTileService : SuspendingTileService() {

    override suspend fun resourcesRequest(
        requestParams: RequestBuilders.ResourcesRequest
    ): ResourceBuilders.Resources {
        return ResourceBuilders.Resources.Builder()
            .setVersion("1")
            .build()
    }

    override suspend fun tileRequest(
        requestParams: RequestBuilders.TileRequest
    ): TileBuilders.Tile {
        val isLocked = withContext(Dispatchers.IO) {
            getSharedPreferences("keepassex", MODE_PRIVATE)
                .getBoolean("vault_locked", true)
        }

        val layout = buildLayout(isLocked)

        return TileBuilders.Tile.Builder()
            .setResourcesVersion("1")
            .setTileTimeline(
                TimelineBuilders.Timeline.Builder()
                    .addTimelineEntry(
                        TimelineBuilders.TimelineEntry.Builder()
                            .setLayout(layout)
                            .build()
                    )
                    .build()
            )
            .setFreshnessIntervalMillis(60_000L) // Refresh every minute
            .build()
    }

    private fun buildLayout(isLocked: Boolean): LayoutElementBuilders.LayoutElement {
        val statusText = if (isLocked) "Vault Locked" else "Vault Open"
        val statusIcon = if (isLocked) "🔒" else "🔓"

        return PrimaryLayout.Builder(deviceParameters())
            .setContent(
                CompactChip.Builder(
                    this,
                    "$statusIcon $statusText",
                    clickable(isLocked),
                    deviceParameters()
                )
                .setChipColors(
                    if (isLocked) ChipColors.primaryChipColors(Colors.DEFAULT)
                    else ChipColors.secondaryChipColors(Colors.DEFAULT)
                )
                .build()
            )
            .build()
    }

    private fun deviceParameters(): DeviceParametersBuilders.DeviceParameters {
        return DeviceParametersBuilders.DeviceParameters.Builder()
            .setScreenWidthDp(resources.configuration.screenWidthDp)
            .setScreenHeightDp(resources.configuration.screenHeightDp)
            .build()
    }

    private fun clickable(isLocked: Boolean): ModifiersBuilders.Clickable {
        val action = if (isLocked) "UNLOCK" else "OPEN"
        return ModifiersBuilders.Clickable.Builder()
            .setOnClick(
                ActionBuilders.LaunchAction.Builder()
                    .setAndroidActivity(
                        ActionBuilders.AndroidActivity.Builder()
                            .setPackageName("com.keepassex.wear")
                            .setClassName("com.keepassex.wear.presentation.MainActivity")
                            .addKeyToExtraMapping(
                                "action",
                                ActionBuilders.AndroidStringExtra.Builder()
                                    .setValue(action)
                                    .build()
                            )
                            .build()
                    )
                    .build()
            )
            .build()
    }
}
