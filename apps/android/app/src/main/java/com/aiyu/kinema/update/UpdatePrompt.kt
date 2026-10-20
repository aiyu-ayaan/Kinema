package com.aiyu.kinema.update

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.SystemUpdate
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.aiyu.kinema.BuildConfig
import com.aiyu.kinema.R
import com.aiyu.kinema.ui.ReleaseNotes
import com.aiyu.kinema.ui.UpdateProgress
import kotlinx.coroutines.launch

/** At start: offers a newer release on the chosen channel with its notes. "Later" is remembered for that version. */
@Composable
fun UpdatePrompt() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val state by AppUpdates.state.collectAsStateWithLifecycle()
    var closed by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { if (AppUpdates.released && AppUpdates.state.value == AppUpdates.State.Idle) AppUpdates.check(context) }

    val apk = when (val s = state) {
        is AppUpdates.State.Available -> s.apk
        is AppUpdates.State.Downloading -> s.apk
        else -> null
    }
    if (apk == null || closed || apk.version == AppUpdates.skipped(context)) return
    val progress = (state as? AppUpdates.State.Downloading)?.progress

    AlertDialog(
        onDismissRequest = { if (progress == null) closed = true },
        icon = { Icon(Icons.Rounded.SystemUpdate, null) },
        title = { Text(stringResource(R.string.update_available, apk.version)) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Text(stringResource(R.string.update_prompt_body, BuildConfig.VERSION_NAME))
                // Long notes scroll inside the dialog so its buttons stay in reach.
                if (apk.notes.isNotBlank()) ReleaseNotes(apk.notes, Modifier.heightIn(max = 260.dp).verticalScroll(rememberScrollState()))
                progress?.let { UpdateProgress(it) }
            }
        },
        confirmButton = {
            TextButton({ scope.launch { AppUpdates.install(context, apk) } }, enabled = progress == null) {
                Text(progress?.let { stringResource(R.string.update_downloading_percent, (it * 100).toInt()) } ?: stringResource(R.string.update_install))
            }
        },
        dismissButton = {
            TextButton({ AppUpdates.skip(context, apk.version); closed = true }, enabled = progress == null) { Text(stringResource(R.string.update_later)) }
        },
    )
}
