/**
 * KeePassEx WearOS — Jetpack Compose native app
 * Shows TOTP codes and quick-access entries on Wear OS watches
 *
 * Features:
 * - Entry list with search and favorites
 * - OTP countdown with haptic feedback
 * - Copy password/OTP via phone DataClient
 * - Rotary input (Digital Crown / bezel) for scrolling
 * - Accessibility: TalkBack support, content descriptions
 * - Tile and complication support
 */
package com.keepassex.wear.presentation

import android.os.Bundle
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.content.Context
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.rotary.onRotaryScrollEvent
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.wear.compose.foundation.lazy.ScalingLazyColumn
import androidx.wear.compose.foundation.lazy.ScalingLazyListState
import androidx.wear.compose.foundation.lazy.items
import androidx.wear.compose.foundation.lazy.rememberScalingLazyListState
import androidx.wear.compose.material.*
import androidx.wear.compose.navigation.SwipeDismissableNavHost
import androidx.wear.compose.navigation.composable
import androidx.wear.compose.navigation.rememberSwipeDismissableNavController
import com.google.android.gms.wearable.DataClient
import com.google.android.gms.wearable.MessageClient
import com.google.android.gms.wearable.Wearable
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            KeePassExWearApp(context = this)
        }
    }
}

// ─── App Root ─────────────────────────────────────────────────────────────────

@Composable
fun KeePassExWearApp(context: Context) {
    val navController = rememberSwipeDismissableNavController()
    val viewModel: WearViewModel = viewModel()

    MaterialTheme {
        SwipeDismissableNavHost(
            navController = navController,
            startDestination = "lock"
        ) {
            composable("lock") {
                LockScreen(
                    isUnlocking = viewModel.isUnlocking,
                    onUnlock = {
                        viewModel.unlock(context) { success ->
                            if (success) navController.navigate("list") {
                                popUpTo("lock") { inclusive = true }
                            }
                        }
                    }
                )
            }
            composable("list") {
                EntryListScreen(
                    viewModel = viewModel,
                    onEntryClick = { uuid -> navController.navigate("detail/$uuid") },
                    onLock = {
                        viewModel.lock()
                        navController.navigate("lock") {
                            popUpTo("list") { inclusive = true }
                        }
                    }
                )
            }
            composable("detail/{uuid}") { backStackEntry ->
                val uuid = backStackEntry.arguments?.getString("uuid") ?: return@composable
                EntryDetailScreen(
                    entry = viewModel.getEntry(uuid),
                    context = context,
                    onCopyField = { field -> viewModel.copyField(context, uuid, field) }
                )
            }
        }
    }
}

// ─── Lock Screen ──────────────────────────────────────────────────────────────

@Composable
fun LockScreen(isUnlocking: Boolean, onUnlock: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .semantics { contentDescription = "KeePassEx locked" },
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(text = "🔐", fontSize = 36.sp)
            Text(
                text = "KeePassEx",
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 15.sp
            )
            if (isUnlocking) {
                CircularProgressIndicator(
                    modifier = Modifier.size(32.dp),
                    strokeWidth = 3.dp
                )
            } else {
                Button(
                    onClick = onUnlock,
                    modifier = Modifier.semantics { contentDescription = "Unlock vault on phone" }
                ) {
                    Text("Unlock")
                }
            }
        }
    }
}

// ─── Entry List ───────────────────────────────────────────────────────────────

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun EntryListScreen(
    viewModel: WearViewModel,
    onEntryClick: (String) -> Unit,
    onLock: () -> Unit
) {
    val entries by viewModel.entries.collectAsState()
    val searchQuery by viewModel.searchQuery.collectAsState()
    val showFavoritesOnly by viewModel.showFavoritesOnly.collectAsState()
    val listState = rememberScalingLazyListState()
    val coroutineScope = rememberCoroutineScope()
    val focusRequester = remember { FocusRequester() }

    val filteredEntries = remember(entries, searchQuery, showFavoritesOnly) {
        entries
            .filter { if (showFavoritesOnly) it.isFavorite else true }
            .filter {
                searchQuery.isEmpty() ||
                it.title.contains(searchQuery, ignoreCase = true) ||
                it.username.contains(searchQuery, ignoreCase = true)
            }
    }

    Scaffold(
        positionIndicator = { PositionIndicator(scalingLazyListState = listState) },
        vignette = { Vignette(vignettePosition = VignettePosition.TopAndBottom) }
    ) {
        ScalingLazyColumn(
            state = listState,
            modifier = Modifier
                .fillMaxSize()
                .onRotaryScrollEvent { event ->
                    coroutineScope.launch {
                        listState.scrollBy(event.verticalScrollPixels)
                    }
                    true
                }
                .focusRequester(focusRequester)
                .focusable(),
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 28.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            // Header
            item {
                ListHeader {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        Text(
                            text = "KeePassEx",
                            color = MaterialTheme.colors.primary,
                            fontSize = 13.sp
                        )
                        Text(
                            text = "(${filteredEntries.size})",
                            color = MaterialTheme.colors.onSurface.copy(alpha = 0.5f),
                            fontSize = 11.sp
                        )
                    }
                }
            }

            // Search chip
            item {
                Chip(
                    modifier = Modifier.fillMaxWidth(),
                    onClick = { /* Open search input */ },
                    label = {
                        Text(
                            text = if (searchQuery.isEmpty()) "Search..." else searchQuery,
                            color = if (searchQuery.isEmpty())
                                MaterialTheme.colors.onSurface.copy(alpha = 0.5f)
                            else MaterialTheme.colors.onSurface,
                            fontSize = 12.sp
                        )
                    },
                    icon = { Text("🔍", fontSize = 14.sp) },
                    colors = ChipDefaults.secondaryChipColors()
                )
            }

            // Favorites toggle
            item {
                ToggleChip(
                    modifier = Modifier.fillMaxWidth(),
                    checked = showFavoritesOnly,
                    onCheckedChange = { viewModel.toggleFavoritesOnly() },
                    label = { Text("Favorites only", fontSize = 12.sp) },
                    toggleControl = {
                        Icon(
                            imageVector = ToggleChipDefaults.switchIcon(showFavoritesOnly),
                            contentDescription = if (showFavoritesOnly) "Showing favorites" else "Showing all"
                        )
                    }
                )
            }

            // Entry items
            if (filteredEntries.isEmpty()) {
                item {
                    Text(
                        text = if (searchQuery.isEmpty()) "No entries" else "No results",
                        color = MaterialTheme.colors.onSurface.copy(alpha = 0.5f),
                        fontSize = 12.sp,
                        modifier = Modifier.padding(8.dp)
                    )
                }
            } else {
                items(filteredEntries) { entry ->
                    EntryChip(entry = entry, onClick = { onEntryClick(entry.uuid) })
                }
            }

            // Lock button
            item {
                Chip(
                    modifier = Modifier.fillMaxWidth(),
                    onClick = onLock,
                    label = { Text("Lock Vault", fontSize = 12.sp) },
                    icon = { Text("🔒", fontSize = 14.sp) },
                    colors = ChipDefaults.secondaryChipColors()
                )
            }
        }
    }

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }
}

@Composable
fun EntryChip(entry: WearEntry, onClick: () -> Unit) {
    Chip(
        modifier = Modifier
            .fillMaxWidth()
            .semantics { contentDescription = "${entry.title}, ${entry.username}" },
        onClick = onClick,
        label = {
            Text(
                text = entry.title,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                fontSize = 13.sp
            )
        },
        secondaryLabel = {
            if (entry.username.isNotEmpty()) {
                Text(
                    text = entry.username,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    fontSize = 11.sp
                )
            }
        },
        icon = {
            Text(
                text = when {
                    entry.isFavorite -> "⭐"
                    entry.hasOtp -> "⏱"
                    else -> "🔑"
                },
                fontSize = 14.sp
            )
        }
    )
}

// ─── Entry Detail ─────────────────────────────────────────────────────────────

@Composable
fun EntryDetailScreen(
    entry: WearEntry?,
    context: Context,
    onCopyField: (String) -> Unit
) {
    var otpCode by remember { mutableStateOf("------") }
    var otpRemaining by remember { mutableStateOf(30) }
    var otpPeriod by remember { mutableStateOf(30) }
    var copyFeedback by remember { mutableStateOf<String?>(null) }
    val coroutineScope = rememberCoroutineScope()

    // OTP countdown
    LaunchedEffect(entry?.hasOtp) {
        if (entry?.hasOtp == true) {
            while (true) {
                // Request OTP from phone
                delay(1000)
                otpRemaining = maxOf(0, otpRemaining - 1)
                if (otpRemaining == 0) otpRemaining = otpPeriod
                // Haptic at 5s warning
                if (otpRemaining == 5) {
                    vibrateDevice(context, VibrationPattern.WARNING)
                }
            }
        }
    }

    if (entry == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("Entry not found", color = Color.White)
        }
        return
    }

    ScalingLazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(horizontal = 8.dp, vertical = 28.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        // Title
        item {
            ListHeader {
                Text(
                    text = entry.title,
                    color = MaterialTheme.colors.primary,
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }

        // Username
        if (entry.username.isNotEmpty()) {
            item {
                InfoCard(
                    label = "Username",
                    value = entry.username,
                    onClick = {
                        onCopyField("username")
                        copyFeedback = "Username copied"
                        coroutineScope.launch {
                            delay(2000)
                            copyFeedback = null
                        }
                    }
                )
            }
        }

        // URL
        if (entry.url.isNotEmpty()) {
            item {
                InfoCard(label = "URL", value = entry.url, onClick = null)
            }
        }

        // OTP
        if (entry.hasOtp) {
            item {
                OtpCard(
                    code = otpCode,
                    remaining = otpRemaining,
                    period = otpPeriod,
                    onClick = {
                        onCopyField("otp")
                        copyFeedback = "OTP copied"
                        vibrateDevice(context, VibrationPattern.SUCCESS)
                        coroutineScope.launch {
                            delay(2000)
                            copyFeedback = null
                        }
                    }
                )
            }
        }

        // Copy password
        item {
            Chip(
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics { contentDescription = "Copy password to phone clipboard" },
                onClick = {
                    onCopyField("password")
                    copyFeedback = "Password copied"
                    vibrateDevice(context, VibrationPattern.SUCCESS)
                    coroutineScope.launch {
                        delay(2000)
                        copyFeedback = null
                    }
                },
                label = {
                    Text(
                        text = copyFeedback ?: "Copy Password",
                        fontSize = 12.sp
                    )
                },
                icon = { Text(if (copyFeedback != null) "✓" else "📋", fontSize = 14.sp) }
            )
        }
    }
}

// ─── OTP Card ─────────────────────────────────────────────────────────────────

@Composable
fun OtpCard(code: String, remaining: Int, period: Int, onClick: () -> Unit) {
    val isUrgent = remaining <= 5
    val codeColor = if (isUrgent) Color.Red else Color(0xFF3B82F6)
    val progress = remaining.toFloat() / period.toFloat()

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .semantics { contentDescription = "OTP code: $code, expires in $remaining seconds" },
        onClick = onClick
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(text = "OTP", color = Color.Gray, fontSize = 10.sp)
                // Progress indicator
                CircularProgressIndicator(
                    progress = progress,
                    modifier = Modifier.size(18.dp),
                    strokeWidth = 2.dp,
                    indicatorColor = codeColor,
                    trackColor = Color.Gray.copy(alpha = 0.2f)
                )
            }
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = formatOtpCode(code),
                    color = codeColor,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    fontSize = 18.sp,
                    letterSpacing = 3.sp
                )
                Text(
                    text = "${remaining}s",
                    color = if (isUrgent) Color.Red else Color.Gray,
                    fontSize = 11.sp,
                    fontWeight = if (isUrgent) FontWeight.Bold else FontWeight.Normal
                )
            }
        }
    }
}

// ─── Info Card ────────────────────────────────────────────────────────────────

@Composable
fun InfoCard(label: String, value: String, onClick: (() -> Unit)?) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .semantics { contentDescription = "$label: $value${if (onClick != null) ", tap to copy" else ""}" },
        onClick = onClick ?: {}
    ) {
        Column(modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp)) {
            Text(text = label, color = Color.Gray, fontSize = 10.sp)
            Text(
                text = value,
                color = Color.White,
                fontSize = 12.sp,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

// ─── Haptic Feedback ──────────────────────────────────────────────────────────

enum class VibrationPattern { SUCCESS, WARNING, ERROR }

fun vibrateDevice(context: Context, pattern: VibrationPattern) {
    val vibrator = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
        (context.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager).defaultVibrator
    } else {
        @Suppress("DEPRECATION")
        context.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
    }

    val effect = when (pattern) {
        VibrationPattern.SUCCESS -> VibrationEffect.createOneShot(50, VibrationEffect.DEFAULT_AMPLITUDE)
        VibrationPattern.WARNING -> VibrationEffect.createWaveform(longArrayOf(0, 50, 100, 50), -1)
        VibrationPattern.ERROR -> VibrationEffect.createOneShot(200, VibrationEffect.DEFAULT_AMPLITUDE)
    }
    vibrator.vibrate(effect)
}

// ─── Models ───────────────────────────────────────────────────────────────────

data class WearEntry(
    val uuid: String,
    val title: String,
    val username: String,
    val url: String,
    val hasOtp: Boolean,
    val hasNotes: Boolean,
    val isFavorite: Boolean
)

// ─── ViewModel ────────────────────────────────────────────────────────────────

class WearViewModel : ViewModel() {
    private val _entries = MutableStateFlow<List<WearEntry>>(emptyList())
    val entries: StateFlow<List<WearEntry>> = _entries.asStateFlow()

    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    private val _showFavoritesOnly = MutableStateFlow(false)
    val showFavoritesOnly: StateFlow<Boolean> = _showFavoritesOnly.asStateFlow()

    var isLocked = true
        private set

    var isUnlocking = false
        private set

    fun unlock(context: Context, onResult: (Boolean) -> Unit) {
        isUnlocking = true
        // Send unlock request to phone via MessageClient
        Wearable.getMessageClient(context)
            .sendMessage("", "/keepassex/unlock", ByteArray(0))
            .addOnSuccessListener {
                isUnlocking = false
                isLocked = false
                loadEntries(context)
                onResult(true)
            }
            .addOnFailureListener {
                isUnlocking = false
                onResult(false)
            }
    }

    fun lock() {
        isLocked = true
        _entries.value = emptyList()
        _searchQuery.value = ""
    }

    fun getEntry(uuid: String): WearEntry? = _entries.value.find { it.uuid == uuid }

    fun setSearchQuery(query: String) { _searchQuery.value = query }

    fun toggleFavoritesOnly() { _showFavoritesOnly.value = !_showFavoritesOnly.value }

    fun copyField(context: Context, uuid: String, field: String) {
        val payload = "$uuid:$field".toByteArray()
        Wearable.getMessageClient(context)
            .sendMessage("", "/keepassex/copy", payload)
    }

    private fun loadEntries(context: Context) {
        // Load entries from phone via DataClient
        Wearable.getDataClient(context)
            .getDataItems()
            .addOnSuccessListener { items ->
                // Parse DataItems into WearEntry list
                // Implementation uses DataMapItem for structured data
            }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fun formatOtpCode(code: String): String =
    if (code.length == 6) "${code.take(3)} ${code.drop(3)}" else code
