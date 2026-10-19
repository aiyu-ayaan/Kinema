package com.aiyu.kinema.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.Dns
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Pin
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Security
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material.icons.rounded.Web
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.material3.toShape
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.aiyu.kinema.R
import com.aiyu.kinema.core.ApiException
import com.aiyu.kinema.core.ConnectResult
import com.aiyu.kinema.core.FoundServer
import com.aiyu.kinema.core.ServerDiscovery
import com.aiyu.kinema.core.Session
import com.aiyu.kinema.core.User
import com.aiyu.kinema.core.normalizeServerUrl
import com.aiyu.kinema.ui.theme.rememberReducedMotion
import kotlinx.coroutines.launch

/** What to tell the user about a failed request. */
@Composable
fun errorText(e: Throwable?): String = when ((e as? ApiException)?.status) {
    null -> e?.message ?: stringResource(R.string.error_generic)
    ApiException.OFFLINE -> stringResource(R.string.error_offline)
    ApiException.UNTRUSTED -> stringResource(R.string.error_untrusted)
    else -> e.message?.ifBlank { null } ?: stringResource(R.string.error_generic)
}

/** Two large tonal shapes drifting behind the screens before sign-in. Decoration only; still with animations off. */
@Composable
private fun Backdrop() {
    val c = MaterialTheme.colorScheme
    val turn = if (rememberReducedMotion()) 0f else rememberInfiniteTransition(label = "backdrop")
        .animateFloat(0f, 360f, infiniteRepeatable(tween(60_000, easing = LinearEasing)), label = "turn").value
    Box(Modifier.fillMaxSize().clipToBounds()) {
        Box(Modifier.align(Alignment.TopStart).offset((-110).dp, (-90).dp).size(320.dp).graphicsLayer { rotationZ = turn }.clip(MaterialShapes.Cookie12Sided.toShape()).background(c.primaryContainer.copy(alpha = 0.55f)))
        Box(Modifier.align(Alignment.BottomEnd).offset(120.dp, 100.dp).size(340.dp).graphicsLayer { rotationZ = -turn }.clip(MaterialShapes.Flower.toShape()).background(c.tertiaryContainer.copy(alpha = 0.5f)))
    }
}

/** Frame of the screens before sign-in: the morphing mark, a headline, then the content rising in after it. */
@Composable
private fun EntryColumn(title: String, subtitle: String, content: @Composable ColumnScope.() -> Unit) {
    Box(Modifier.fillMaxSize()) {
        Backdrop()
        Column(
            Modifier.align(Alignment.TopCenter).widthIn(max = 560.dp).fillMaxSize().systemBarsPadding().imePadding()
                .verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Spacer(Modifier.height(24.dp))
            BrandMark(Modifier.enterStaggered(0), size = 132.dp)
            Spacer(Modifier.height(8.dp))
            Text(title, style = MaterialTheme.typography.displaySmall, textAlign = TextAlign.Center, modifier = Modifier.enterStaggered(1).semantics { heading() })
            Text(subtitle, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center, modifier = Modifier.enterStaggered(2))
            Spacer(Modifier.height(8.dp))
            content()
        }
    }
}

@Composable
private fun Panel(index: Int, content: @Composable ColumnScope.() -> Unit) {
    Surface(color = MaterialTheme.colorScheme.surfaceContainerLow, shape = MaterialTheme.shapes.extraLarge, shadowElevation = 2.dp, modifier = Modifier.fillMaxWidth().enterStaggered(index)) {
        Column(Modifier.padding(20.dp).animateContentSize(MaterialTheme.motionScheme.defaultSpatialSpec()), verticalArrangement = Arrangement.spacedBy(12.dp), content = content)
    }
}

/** First run: finds servers on this network, or takes an address. A self-signed server asks to trust its certificate. */
@Composable
fun ConnectScreen() {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    var found by remember { mutableStateOf<List<FoundServer>?>(null) }
    var scanRun by remember { mutableStateOf(0) }
    var address by rememberSaveable { mutableStateOf("") }
    var busy by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var trust by remember { mutableStateOf<ConnectResult.Untrusted?>(null) }
    val invalid = stringResource(R.string.connect_invalid)
    val failedText = @Composable { e: ApiException -> errorText(e) }
    var failure by remember { mutableStateOf<ApiException?>(null) }

    LaunchedEffect(scanRun) {
        found = null
        found = ServerDiscovery.scan()
    }

    fun connect(url: String, pin: String? = null) {
        busy = url
        error = null
        failure = null
        scope.launch {
            when (val r = repository.connect(url, pin)) {
                ConnectResult.Connected -> Unit
                is ConnectResult.Untrusted -> trust = r
                is ConnectResult.Failed -> failure = r.error
            }
            busy = null
        }
    }

    EntryColumn(stringResource(R.string.connect_title), stringResource(R.string.connect_subtitle)) {
        Panel(3) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(stringResource(R.string.connect_found), style = MaterialTheme.typography.titleMedium, modifier = Modifier.weight(1f).semantics { heading() })
                IconButton({ scanRun++ }, enabled = found != null) { Icon(Icons.Rounded.Refresh, stringResource(R.string.connect_rescan)) }
            }
            val motion = MaterialTheme.motionScheme
            AnimatedContent(found, label = "found", transitionSpec = { fadeIn(motion.defaultEffectsSpec()).togetherWith(fadeOut(motion.fastEffectsSpec())) }) { list ->
                when {
                    list == null -> Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp)) {
                        LoadingIndicator(Modifier.size(40.dp))
                        Text(stringResource(R.string.connect_scanning), color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    list.isEmpty() -> Text(stringResource(R.string.connect_none), color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodyMedium)
                    else -> Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        list.forEachIndexed { i, server -> ServerRow(server, busy == server.url, Modifier.enterStaggered(i)) { connect(server.url) } }
                    }
                }
            }
        }
        Panel(4) {
            Text(stringResource(R.string.connect_manual), style = MaterialTheme.typography.titleMedium, modifier = Modifier.semantics { heading() })
            val submit = {
                val url = normalizeServerUrl(address)
                if (url == null) error = invalid else connect(url)
            }
            OutlinedTextField(
                address, { address = it; error = null },
                label = { Text(stringResource(R.string.connect_address)) },
                placeholder = { Text("192.168.1.10:3000") },
                leadingIcon = { Icon(Icons.Rounded.Web, null) },
                singleLine = true,
                isError = error != null,
                supportingText = { Text(error ?: stringResource(R.string.connect_address_hint)) },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Go, autoCorrectEnabled = false),
                keyboardActions = KeyboardActions(onGo = { submit() }),
                shape = MaterialTheme.shapes.large,
                modifier = Modifier.fillMaxWidth(),
            )
            Button(submit, enabled = address.isNotBlank() && busy == null, modifier = Modifier.fillMaxWidth().height(56.dp)) {
                if (busy != null && found?.none { it.url == busy } != false) LoadingIndicator(Modifier.size(24.dp), color = MaterialTheme.colorScheme.onPrimary)
                else {
                    Text(stringResource(R.string.connect_action))
                    Spacer(Modifier.width(8.dp))
                    Icon(Icons.AutoMirrored.Rounded.ArrowForward, null)
                }
            }
        }
        failure?.let { Text(failedText(it), color = MaterialTheme.colorScheme.error, textAlign = TextAlign.Center, modifier = Modifier.fillMaxWidth()) }
    }

    trust?.let { t ->
        AlertDialog(
            onDismissRequest = { trust = null },
            icon = { Icon(Icons.Rounded.Security, null) },
            title = { Text(stringResource(R.string.trust_title)) },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(stringResource(R.string.trust_body, t.url))
                    Surface(color = MaterialTheme.colorScheme.surfaceContainerHighest, shape = MaterialTheme.shapes.small) {
                        Text(t.fingerprint, fontFamily = FontFamily.Monospace, style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(12.dp))
                    }
                }
            },
            confirmButton = { Button({ trust = null; connect(t.url, t.fingerprint) }) { Text(stringResource(R.string.trust_accept)) } },
            dismissButton = { TextButton({ trust = null }) { Text(stringResource(R.string.action_cancel)) } },
        )
    }
}

@Composable
private fun ServerRow(server: FoundServer, busy: Boolean, modifier: Modifier = Modifier, onClick: () -> Unit) {
    val c = MaterialTheme.colorScheme
    val source = remember { MutableInteractionSource() }
    ListItem(
        headlineContent = { Text(server.name, style = MaterialTheme.typography.titleMedium) },
        supportingContent = { Text(if (server.thisComputer) stringResource(R.string.connect_this_computer, server.url) else server.url, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        leadingContent = { ShapedIcon(Icons.Rounded.Dns, MaterialShapes.Clover4Leaf, c.primaryContainer, c.onPrimaryContainer) },
        trailingContent = { if (busy) LoadingIndicator(Modifier.size(32.dp)) else Icon(Icons.AutoMirrored.Rounded.ArrowForward, null) },
        colors = ListItemDefaults.colors(containerColor = c.surfaceContainerHigh),
        modifier = modifier.pressScale(source).clip(MaterialTheme.shapes.large).clickable(source, null, enabled = !busy, onClick = onClick),
    )
}

/** The saved server did not answer at any of its addresses. */
@Composable
fun UnreachableScreen(s: Session.Unreachable) {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    Box(Modifier.fillMaxSize().systemBarsPadding()) {
        MessageState(
            Icons.Rounded.CloudOff,
            stringResource(R.string.unreachable_title),
            stringResource(R.string.unreachable_body, s.server) + "\n\n" + errorText(s.error),
            action = stringResource(R.string.action_retry),
            onAction = { scope.launch { repository.restore() } },
        )
        TextButton(repository::forgetServer, Modifier.align(Alignment.BottomCenter).padding(24.dp)) { Text(stringResource(R.string.connect_change_server)) }
    }
}

/** Fixed hues per avatar color (web `avatarColors.ts`), so a profile looks the same everywhere. */
private fun avatarBrush(name: String?): Brush {
    val (a, b) = when (name) {
        "ocean" -> 0xFF3B82F6 to 0xFF1E3A8A
        "forest" -> 0xFF22A06B to 0xFF135C3D
        "plum" -> 0xFFA855F7 to 0xFF5B2192
        "sun" -> 0xFFF5B12A to 0xFFB26B05
        "rose" -> 0xFFEC4899 to 0xFF9D174D
        "slate" -> 0xFF64748B to 0xFF2B3442
        "teal" -> 0xFF14B8A6 to 0xFF0F5E57
        else -> 0xFFE8553A to 0xFF9C2F1C
    }
    return Brush.linearGradient(listOf(Color(a), Color(b)))
}

private val AvatarShapes = listOf(MaterialShapes.Cookie9Sided, MaterialShapes.Clover4Leaf, MaterialShapes.Sunny, MaterialShapes.Puffy, MaterialShapes.SoftBurst, MaterialShapes.Gem)

/** A profile's initial on its color, on an expressive shape picked from its id. */
@Composable
fun Avatar(user: User, modifier: Modifier = Modifier, size: androidx.compose.ui.unit.Dp = 96.dp) {
    Box(modifier.size(size).clip(AvatarShapes[(user.id % AvatarShapes.size).toInt()].toShape()).background(avatarBrush(user.avatar)), contentAlignment = Alignment.Center) {
        Text(user.name.take(1).uppercase(), color = Color.White, style = if (size > 64.dp) MaterialTheme.typography.displaySmall else MaterialTheme.typography.titleLarge)
    }
}

/** "Who's watching?": the profiles of the server. Locked ones ask for their password or PIN in a sheet. */
@Composable
fun ProfilesScreen(s: Session.SignedOut) {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    var unlocking by remember { mutableStateOf<User?>(null) }
    var busy by remember { mutableStateOf<Long?>(null) }
    var error by remember { mutableStateOf<ApiException?>(null) }
    var confirmForget by remember { mutableStateOf(false) }

    EntryColumn(stringResource(R.string.profiles_title), s.serverName) {
        if (s.needsSetup) {
            Panel(3) {
                Text(stringResource(R.string.profiles_setup_title), style = MaterialTheme.typography.titleMedium)
                Text(stringResource(R.string.profiles_setup_body, s.server), color = MaterialTheme.colorScheme.onSurfaceVariant)
                FilledTonalButton({ scope.launch { repository.restore() } }) { Text(stringResource(R.string.action_retry)) }
            }
        } else {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(20.dp, Alignment.CenterHorizontally), verticalArrangement = Arrangement.spacedBy(20.dp), modifier = Modifier.fillMaxWidth()) {
                s.profiles.forEachIndexed { i, user ->
                    val source = remember { MutableInteractionSource() }
                    Column(
                        Modifier.enterStaggered(3 + i).width(112.dp).pressScale(source).clip(MaterialTheme.shapes.large)
                            .clickable(source, null, enabled = busy == null) {
                                if (user.hasPassword || user.hasPin) unlocking = user
                                else {
                                    busy = user.id
                                    scope.launch { error = repository.login(user, null, null); busy = null }
                                }
                            }.padding(8.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Avatar(user)
                            if (busy == user.id) LoadingIndicator(Modifier.size(48.dp), color = Color.White)
                        }
                        Spacer(Modifier.height(8.dp))
                        Text(user.name, style = MaterialTheme.typography.titleMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        if (user.hasPassword || user.hasPin) Icon(Icons.Rounded.Lock, stringResource(R.string.profiles_locked), Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
        error?.let { Text(errorText(it), color = MaterialTheme.colorScheme.error, textAlign = TextAlign.Center) }
        Spacer(Modifier.height(8.dp))
        TextButton({ confirmForget = true }, Modifier.navigationBarsPadding()) { Text(stringResource(R.string.connect_change_server)) }
    }

    unlocking?.let { user -> UnlockSheet(user, onDismiss = { unlocking = null }) }
    if (confirmForget) {
        AlertDialog(
            onDismissRequest = { confirmForget = false },
            title = { Text(stringResource(R.string.forget_title)) },
            text = { Text(stringResource(R.string.forget_body)) },
            confirmButton = { TextButton({ confirmForget = false; repository.forgetServer() }) { Text(stringResource(R.string.forget_action)) } },
            dismissButton = { TextButton({ confirmForget = false }) { Text(stringResource(R.string.action_cancel)) } },
        )
    }
}

@Composable
private fun UnlockSheet(user: User, onDismiss: () -> Unit) {
    val repository = LocalRepository.current
    val scope = rememberCoroutineScope()
    val sheet = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val usePin = user.hasPin && !user.hasPassword
    var secret by remember { mutableStateOf("") }
    var shown by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<ApiException?>(null) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    val submit = {
        busy = true
        error = null
        scope.launch {
            error = repository.login(user, secret.takeUnless { usePin }, secret.takeIf { usePin })
            busy = false
        }
    }
    ModalBottomSheet(onDismiss, sheetState = sheet) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 24.dp).padding(bottom = 24.dp).imePadding(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Avatar(user, size = 72.dp)
            Text(user.name, style = MaterialTheme.typography.headlineSmall)
            OutlinedTextField(
                secret, { secret = it; error = null },
                label = { Text(stringResource(if (usePin) R.string.unlock_pin else R.string.unlock_password)) },
                leadingIcon = { Icon(if (usePin) Icons.Rounded.Pin else Icons.Rounded.Lock, null) },
                trailingIcon = {
                    IconButton({ shown = !shown }) {
                        Icon(if (shown) Icons.Rounded.VisibilityOff else Icons.Rounded.Visibility, stringResource(if (shown) R.string.unlock_hide else R.string.unlock_show))
                    }
                },
                visualTransformation = if (shown) VisualTransformation.None else PasswordVisualTransformation(),
                singleLine = true,
                isError = error != null,
                supportingText = error?.let { { Text(errorText(it)) } },
                keyboardOptions = KeyboardOptions(keyboardType = if (usePin) KeyboardType.NumberPassword else KeyboardType.Password, imeAction = ImeAction.Done),
                keyboardActions = KeyboardActions(onDone = { if (secret.isNotEmpty()) submit() }),
                shape = MaterialTheme.shapes.large,
                modifier = Modifier.fillMaxWidth().focusRequester(focus),
            )
            AnimatedVisibility(true) {
                Button({ submit() }, enabled = secret.isNotEmpty() && !busy, modifier = Modifier.fillMaxWidth().height(56.dp)) {
                    if (busy) LoadingIndicator(Modifier.size(24.dp), color = MaterialTheme.colorScheme.onPrimary) else Text(stringResource(R.string.unlock_action))
                }
            }
        }
    }
}
