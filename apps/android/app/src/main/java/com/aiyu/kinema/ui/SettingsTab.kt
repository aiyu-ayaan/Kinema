package com.aiyu.kinema.ui

import android.os.Build
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Cloud
import androidx.compose.material.icons.rounded.DarkMode
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.LightMode
import androidx.compose.material.icons.rounded.Palette
import androidx.compose.material.icons.rounded.PictureInPictureAlt
import androidx.compose.material.icons.rounded.Public
import androidx.compose.material.icons.rounded.Security
import androidx.compose.material.icons.rounded.SettingsBrightness
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.aiyu.kinema.BuildConfig
import com.aiyu.kinema.R
import com.aiyu.kinema.core.Route
import com.aiyu.kinema.core.Session
import com.aiyu.kinema.core.normalizeServerUrl
import kotlinx.coroutines.launch

@Composable
private fun Section(title: String, index: Int, content: @Composable ColumnScope.() -> Unit) {
    Column(Modifier.enterStaggered(index), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(title, style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.primary, modifier = Modifier.padding(start = 8.dp, top = 8.dp).semantics { heading() })
        Surface(color = MaterialTheme.colorScheme.surfaceContainerLow, shape = MaterialTheme.shapes.extraLarge) {
            Column(Modifier.fillMaxWidth().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp), content = content)
        }
    }
}

@Composable
fun SettingsTab() {
    val repository = LocalRepository.current
    val session by repository.session.collectAsStateWithLifecycle()
    val user = (session as? Session.SignedIn)?.user
    val info by repository.serverInfo.collectAsStateWithLifecycle()
    val c = MaterialTheme.colorScheme
    var confirmForget by remember { mutableStateOf(false) }
    // The name may have been changed from the web or another phone since the app connected.
    LaunchedEffect(Unit) { repository.refreshInfo() }

    ScreenColumn {
        Text(stringResource(R.string.tab_settings), style = MaterialTheme.typography.headlineLarge, modifier = Modifier.padding(start = 4.dp, top = 16.dp).semantics { heading() })
        if (user != null) {
            Surface(color = c.primaryContainer, contentColor = c.onPrimaryContainer, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.enterStaggered(0)) {
                Row(Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    Avatar(user, size = 64.dp)
                    Column(Modifier.weight(1f)) {
                        Text(user.name, style = MaterialTheme.typography.titleLarge)
                        Text(info?.name ?: "", style = MaterialTheme.typography.bodyMedium)
                    }
                    FilledTonalButton(repository::logout) { Text(stringResource(R.string.settings_switch_profile)) }
                }
            }
        }
        ConnectionSection()
        AppearanceSection()
        PlaybackSection()
        Section(stringResource(R.string.settings_server), 5) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                ShapedIcon(Icons.Rounded.Security, MaterialShapes.Clover4Leaf, c.tertiaryContainer, c.onTertiaryContainer)
                Column(Modifier.weight(1f)) {
                    Text(info?.let { "${it.name} · ${it.version}" } ?: "", style = MaterialTheme.typography.titleSmall)
                    Text(stringResource(if (repository.pinnedFingerprint != null) R.string.settings_cert_pinned else R.string.settings_cert_public), style = MaterialTheme.typography.bodySmall, color = c.onSurfaceVariant)
                }
            }
            repository.pinnedFingerprint?.let {
                Text(it, fontFamily = FontFamily.Monospace, style = MaterialTheme.typography.labelSmall, color = c.onSurfaceVariant)
            }
            OutlinedButton({ confirmForget = true }, Modifier.fillMaxWidth()) { Text(stringResource(R.string.connect_change_server), color = c.error) }
        }
        Text(
            stringResource(R.string.settings_version, BuildConfig.VERSION_NAME),
            style = MaterialTheme.typography.labelMedium, color = c.onSurfaceVariant,
            modifier = Modifier.fillMaxWidth().padding(8.dp),
        )
    }

    if (confirmForget) {
        AlertDialog(
            onDismissRequest = { confirmForget = false },
            title = { Text(stringResource(R.string.forget_title)) },
            text = { Text(stringResource(R.string.forget_body)) },
            confirmButton = { TextButton({ confirmForget = false; repository.forgetServer() }) { Text(stringResource(R.string.forget_action), color = c.error) } },
            dismissButton = { TextButton({ confirmForget = false }) { Text(stringResource(R.string.action_cancel)) } },
        )
    }
}

/** Where the app reaches the server: the home address, the public one away from home, and which is in use now. */
@Composable
private fun ConnectionSection() {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    val c = MaterialTheme.colorScheme
    val route by repository.route.collectAsStateWithLifecycle()
    val local by repository.localUrl.collectAsStateWithLifecycle()
    val override by repository.publicOverride.collectAsStateWithLifecycle()
    val serverPublic by repository.serverPublicUrl.collectAsStateWithLifecycle()
    var draft by rememberSaveable(override) { mutableStateOf(override) }
    var error by remember { mutableStateOf<String?>(null) }
    var testing by remember { mutableStateOf(false) }
    var result by remember { mutableStateOf<Boolean?>(null) }
    val invalid = stringResource(R.string.connect_invalid)

    Section(stringResource(R.string.settings_connection), 1) {
        val away = route == Route.PUBLIC
        val dot by animateColorAsState(if (away) c.tertiary else c.primary, label = "route")
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            ShapedIcon(if (away) Icons.Rounded.Public else Icons.Rounded.Home, MaterialShapes.Cookie9Sided, c.secondaryContainer, c.onSecondaryContainer)
            Column(Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(Modifier.size(8.dp).background(dot, CircleShape))
                    Spacer(Modifier.width(8.dp))
                    Text(stringResource(if (away) R.string.settings_route_public else R.string.settings_route_local), style = MaterialTheme.typography.titleSmall)
                }
                Text(stringResource(R.string.settings_route_hint), style = MaterialTheme.typography.bodySmall, color = c.onSurfaceVariant)
            }
        }
        ListItem(
            headlineContent = { Text(stringResource(R.string.settings_local_url)) },
            supportingContent = { Text(local.ifBlank { "—" }) },
            leadingContent = { Icon(Icons.Rounded.Home, null) },
            colors = ListItemDefaults.colors(containerColor = Color.Transparent),
        )
        OutlinedTextField(
            draft, { draft = it; error = null; result = null },
            label = { Text(stringResource(R.string.settings_public_url)) },
            placeholder = { Text(serverPublic.ifBlank { "https://media.example.com" }) },
            leadingIcon = { Icon(Icons.Rounded.Cloud, null) },
            singleLine = true,
            isError = error != null,
            supportingText = {
                Text(
                    error ?: when {
                        result == true -> stringResource(R.string.settings_public_ok)
                        result == false -> stringResource(R.string.settings_public_unreachable)
                        override.isBlank() && serverPublic.isNotBlank() -> stringResource(R.string.settings_public_from_server, serverPublic)
                        else -> stringResource(R.string.settings_public_hint)
                    },
                )
            },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Done, autoCorrectEnabled = false),
            keyboardActions = KeyboardActions(onDone = {}),
            shape = MaterialTheme.shapes.large,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            OutlinedButton(
                {
                    val url = normalizeServerUrl(draft.ifBlank { serverPublic })
                    if (url == null) { error = invalid; return@OutlinedButton }
                    testing = true
                    scope.launch { result = repository.probe(url) != null; testing = false }
                },
                enabled = !testing && (draft.isNotBlank() || serverPublic.isNotBlank()),
            ) { Text(stringResource(R.string.settings_public_test)) }
            AnimatedVisibility(testing) { LoadingIndicator(Modifier.size(32.dp)) }
            Spacer(Modifier.weight(1f))
            Button(
                { scope.launch { error = repository.setPublicUrl(draft)?.let { invalid } } },
                enabled = draft.trim() != override,
            ) { Text(stringResource(R.string.action_save)) }
        }
    }
}

@Composable
private fun PlaybackSection() {
    val repository = LocalRepository.current
    val pip by repository.pictureInPicture.collectAsStateWithLifecycle()
    Section(stringResource(R.string.settings_playback), 3) {
        ListItem(
            headlineContent = { Text(stringResource(R.string.settings_pip)) },
            supportingContent = { Text(stringResource(if (pip) R.string.settings_pip_on else R.string.settings_pip_off)) },
            leadingContent = { Icon(Icons.Rounded.PictureInPictureAlt, null) },
            trailingContent = { Switch(pip, repository::setPictureInPicture) },
            colors = ListItemDefaults.colors(containerColor = Color.Transparent),
        )
    }
}

@Composable
private fun AppearanceSection() {
    val repository = LocalRepository.current
    val appearance by repository.appearance.collectAsStateWithLifecycle()
    Section(stringResource(R.string.settings_appearance), 2) {
        val options = listOf(
            Triple("system", R.string.settings_theme_system, Icons.Rounded.SettingsBrightness),
            Triple("light", R.string.settings_theme_light, Icons.Rounded.LightMode),
            Triple("dark", R.string.settings_theme_dark, Icons.Rounded.DarkMode),
        )
        SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
            options.forEachIndexed { i, (key, label, icon) ->
                SegmentedButton(
                    selected = appearance.theme == key,
                    onClick = { repository.setTheme(key) },
                    shape = SegmentedButtonDefaults.itemShape(i, options.size),
                    icon = { SegmentedButtonDefaults.Icon(appearance.theme == key) { Icon(icon, null, Modifier.size(SegmentedButtonDefaults.IconSize)) } },
                ) { Text(stringResource(label)) }
            }
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            ListItem(
                headlineContent = { Text(stringResource(R.string.settings_dynamic)) },
                supportingContent = { Text(stringResource(R.string.settings_dynamic_hint)) },
                leadingContent = { Icon(Icons.Rounded.Palette, null) },
                trailingContent = { Switch(appearance.dynamicColor, repository::setDynamicColor) },
                colors = ListItemDefaults.colors(containerColor = Color.Transparent),
            )
        }
        Spacer(Modifier.height(0.dp))
    }
}
