package com.aiyu.kinema.update

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.LinearWavyProgressIndicator
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.aiyu.kinema.BuildConfig
import com.aiyu.kinema.R
import kotlinx.coroutines.launch

/** Checks once per launch and offers a newer release; "Later" is remembered for that version. */
@Composable
fun UpdatePrompt() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val uri = LocalUriHandler.current
    val state by AppUpdates.state.collectAsStateWithLifecycle()
    var closed by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { if (AppUpdates.state.value == AppUpdates.State.Idle) AppUpdates.check() }

    val apk = when (val s = state) {
        is AppUpdates.State.Available -> s.apk
        is AppUpdates.State.Downloading -> s.apk
        else -> null
    }
    if (apk == null || closed || apk.version == AppUpdates.skipped(context)) return
    val progress = (state as? AppUpdates.State.Downloading)?.progress

    AlertDialog(
        onDismissRequest = { if (progress == null) closed = true },
        title = { Text(stringResource(R.string.update_available, apk.version)) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(stringResource(R.string.update_body, BuildConfig.VERSION_NAME))
                if (progress != null) LinearWavyProgressIndicator(progress = { progress }, modifier = Modifier.fillMaxWidth())
                TextButton({ uri.openUri(apk.page) }) { Text(stringResource(R.string.update_whats_new), color = MaterialTheme.colorScheme.primary) }
            }
        },
        confirmButton = {
            Button({ scope.launch { AppUpdates.install(context, apk) } }, enabled = progress == null) { Text(stringResource(R.string.update_install)) }
        },
        dismissButton = {
            TextButton({ AppUpdates.skip(context, apk.version); closed = true }, enabled = progress == null) { Text(stringResource(R.string.update_later)) }
        },
    )
}
