package com.aiyu.kinema.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.Code
import androidx.compose.material.icons.rounded.Description
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.SystemUpdate
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearWavyProgressIndicator
import androidx.compose.material3.MaterialShapes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.aiyu.kinema.BuildConfig
import com.aiyu.kinema.R
import com.aiyu.kinema.core.UpdateChannel
import com.aiyu.kinema.update.AppUpdates
import com.mikepenz.markdown.coil3.Coil3ImageTransformerImpl
import com.mikepenz.markdown.m3.Markdown
import com.mikepenz.markdown.m3.markdownTypography
import kotlinx.coroutines.launch

private const val REPO_URL = "https://github.com/aiyu-ayaan/Kinema"
private const val GITHUB_URL = "https://github.com/aiyu-ayaan"
private const val PORTFOLIO_URL = "https://aiyu.co.in"

/** What the Android app is built on; the web app keeps its own list. Keep in sync with gradle/libs.versions.toml. */
private val libraries = listOf(
    Triple("Kotlin", "https://github.com/JetBrains/kotlin", "Apache-2.0"),
    Triple("kotlinx.coroutines", "https://github.com/Kotlin/kotlinx.coroutines", "Apache-2.0"),
    Triple("kotlinx.serialization", "https://github.com/Kotlin/kotlinx.serialization", "Apache-2.0"),
    Triple("Jetpack Compose", "https://developer.android.com/jetpack/compose", "Apache-2.0"),
    Triple("Material 3 for Compose", "https://developer.android.com/jetpack/androidx/releases/compose-material3", "Apache-2.0"),
    Triple("Navigation 3", "https://developer.android.com/guide/navigation/navigation-3", "Apache-2.0"),
    Triple("AndroidX Activity, Lifecycle, Core", "https://developer.android.com/jetpack/androidx", "Apache-2.0"),
    Triple("AndroidX Graphics Shapes", "https://developer.android.com/jetpack/androidx/releases/graphics", "Apache-2.0"),
    Triple("Media3 ExoPlayer", "https://github.com/androidx/media", "Apache-2.0"),
    Triple("OkHttp", "https://github.com/square/okhttp", "Apache-2.0"),
    Triple("Coil", "https://github.com/coil-kt/coil", "Apache-2.0"),
    Triple("Markdown Renderer", "https://github.com/mikepenz/multiplatform-markdown-renderer", "Apache-2.0"),
)

@Composable
private fun rememberOpenUrl(): (String) -> Unit {
    val context = LocalContext.current
    return remember(context) { { url -> context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) } }
}

/** A pushed page with a back arrow and a scrolling column. */
@Composable
private fun SubPage(title: String, content: @Composable ColumnScope.() -> Unit) {
    val navigator = LocalNavigator.current
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
        Column(Modifier.widthIn(max = 840.dp).fillMaxSize().statusBarsPadding()) {
            Row(Modifier.padding(horizontal = 4.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                IconButton(navigator.back) { Icon(Icons.AutoMirrored.Rounded.ArrowBack, stringResource(R.string.action_back)) }
                Text(title, style = MaterialTheme.typography.headlineSmall, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f).semantics { heading() })
            }
            Column(
                Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = 16.dp).padding(bottom = 24.dp).navigationBarsPadding(),
                verticalArrangement = Arrangement.spacedBy(12.dp),
                content = content,
            )
        }
    }
}

@Composable
private fun PageTitle(text: String) =
    Text(text, style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.primary, modifier = Modifier.padding(start = 8.dp, top = 8.dp).semantics { heading() })

/** One tappable row: shaped icon, title, subtitle and an optional trailing slot. */
@Composable
fun RowCard(title: String, subtitle: String?, icon: ImageVector, onClick: (() -> Unit)? = null, trailing: @Composable RowScope.() -> Unit = {}) {
    val c = MaterialTheme.colorScheme
    Surface(color = c.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.fillMaxWidth()) {
        Row(
            Modifier.then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier).padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ShapedIcon(icon, MaterialShapes.Cookie9Sided, c.secondaryContainer, c.onSecondaryContainer)
            Spacer(Modifier.width(16.dp))
            Column(Modifier.weight(1f)) {
                Text(title, style = MaterialTheme.typography.titleMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (!subtitle.isNullOrBlank()) Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant, maxLines = 3, overflow = TextOverflow.Ellipsis)
            }
            trailing()
        }
    }
}

@Composable
private fun OpenIcon() = Icon(Icons.AutoMirrored.Rounded.OpenInNew, null, tint = MaterialTheme.colorScheme.onSurfaceVariant)

/** Release notes as written on the release: Markdown with headings, lists, links, tables and images. */
@Composable
fun ReleaseNotes(markdown: String, modifier: Modifier = Modifier) {
    val type = MaterialTheme.typography
    Markdown(
        markdown,
        modifier = modifier.fillMaxWidth(),
        // Notes sit inside a page that already has a title: their headings stay below it.
        typography = markdownTypography(h1 = type.titleLarge, h2 = type.titleMedium, h3 = type.titleSmall, h4 = type.titleSmall, h5 = type.labelLarge, h6 = type.labelLarge),
        imageTransformer = Coil3ImageTransformerImpl,
    )
}

@Composable
fun UpdateProgress(progress: Float) = LinearWavyProgressIndicator(progress = { progress.coerceIn(0f, 1f) }, modifier = Modifier.fillMaxWidth())

/**
 * Settings › Updates: what is installed, whether the channel has something newer, and the release notes.
 * The notes are those of the update when there is one, otherwise those of the installed version.
 */
@Composable
fun UpdatesScreen() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val c = MaterialTheme.colorScheme
    val installed = BuildConfig.VERSION_NAME
    val released = AppUpdates.released
    val state by AppUpdates.state.collectAsStateWithLifecycle()
    val installedNotes by AppUpdates.installedNotes.collectAsStateWithLifecycle()
    var channel by remember { mutableStateOf(AppUpdates.channel(context)) }
    var asked by remember { mutableIntStateOf(0) }
    LaunchedEffect(channel, asked) { AppUpdates.check(context) }

    val found = when (val s = state) {
        is AppUpdates.State.Available -> s.apk
        is AppUpdates.State.Downloading -> s.apk
        else -> null
    }
    val progress = (state as? AppUpdates.State.Downloading)?.progress
    val failed = state as? AppUpdates.State.Failed

    SubPage(stringResource(R.string.update_title)) {
        Surface(color = if (found != null) c.primaryContainer else c.surfaceContainerHigh, shape = MaterialTheme.shapes.extraLarge, modifier = Modifier.fillMaxWidth()) {
            Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Icon(Icons.Rounded.SystemUpdate, contentDescription = null)
                Text(
                    when {
                        found != null -> stringResource(R.string.update_available, found.version)
                        state == AppUpdates.State.Checking || state == AppUpdates.State.Idle -> stringResource(R.string.update_checking)
                        failed != null -> stringResource(R.string.update_check_failed)
                        !released -> stringResource(R.string.update_dev_build)
                        else -> stringResource(R.string.update_up_to_date)
                    },
                    style = MaterialTheme.typography.titleLarge,
                )
                Text(
                    when {
                        failed != null && failed.message.isNotBlank() -> stringResource(R.string.update_failed, failed.message)
                        failed != null -> stringResource(R.string.update_check_failed_body, installed)
                        !released -> stringResource(R.string.update_dev_build_body, installed)
                        else -> stringResource(R.string.update_you_have, installed)
                    },
                    style = MaterialTheme.typography.bodyMedium,
                )
                progress?.let { UpdateProgress(it) }
                if (found != null) {
                    Button({ scope.launch { AppUpdates.install(context, found) } }, Modifier.fillMaxWidth().heightIn(min = 56.dp), enabled = progress == null) {
                        Text(progress?.let { stringResource(R.string.update_downloading_percent, (it * 100).toInt()) } ?: stringResource(R.string.update_download_install))
                    }
                } else if (state != AppUpdates.State.Checking) {
                    OutlinedButton({ asked++ }, Modifier.fillMaxWidth().heightIn(min = 56.dp)) { Text(stringResource(R.string.update_check_again)) }
                }
            }
        }

        if (released) {
            PageTitle(stringResource(R.string.update_channel))
            val channels = listOf(UpdateChannel.STABLE to R.string.update_channel_stable, UpdateChannel.BETA to R.string.update_channel_beta, UpdateChannel.ALPHA to R.string.update_channel_alpha)
            SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                channels.forEachIndexed { index, (item, label) ->
                    SegmentedButton(
                        channel == item,
                        { AppUpdates.setChannel(context, item); channel = item },
                        SegmentedButtonDefaults.itemShape(index, channels.size),
                        enabled = progress == null,
                    ) { Text(stringResource(label)) }
                }
            }
            Text(stringResource(R.string.update_channel_hint), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant)
        }

        val notes = found?.notes?.takeIf { it.isNotBlank() } ?: installedNotes
        PageTitle(if (found != null) stringResource(R.string.update_whats_new_in, found.version) else stringResource(R.string.update_in_this_version))
        when {
            !notes.isNullOrBlank() -> ReleaseNotes(notes)
            state == AppUpdates.State.Checking -> Text(stringResource(R.string.update_loading_notes), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant)
            else -> Text(stringResource(R.string.update_no_notes), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant)
        }
    }
}

/** Settings › About Kinema: the app, its developer and what it is built on. */
@Composable
fun AboutScreen() {
    val open = rememberOpenUrl()
    val navigator = LocalNavigator.current
    val c = MaterialTheme.colorScheme
    SubPage(stringResource(R.string.about_title)) {
        PageTitle(stringResource(R.string.about_section_app))
        RowCard("Kinema", stringResource(R.string.about_version_build, BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE), Icons.Rounded.Info)
        RowCard(stringResource(R.string.about_privacy_title), stringResource(R.string.about_privacy_subtitle), Icons.Rounded.Lock)
        RowCard(stringResource(R.string.about_source_code), "github.com/aiyu-ayaan/Kinema", Icons.Rounded.Code, onClick = { open(REPO_URL) }) { OpenIcon() }

        PageTitle(stringResource(R.string.about_section_developer))
        Column(Modifier.fillMaxWidth().padding(vertical = 8.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Image(painterResource(R.drawable.developer), "aiyu-ayaan", Modifier.size(96.dp).clip(CircleShape), contentScale = ContentScale.Crop)
            Text("aiyu-ayaan", style = MaterialTheme.typography.titleLarge, modifier = Modifier.padding(top = 8.dp))
            Text(stringResource(R.string.about_developer_tagline), style = MaterialTheme.typography.bodyMedium, color = c.onSurfaceVariant, textAlign = TextAlign.Center)
        }
        RowCard(stringResource(R.string.about_github), "github.com/aiyu-ayaan", Icons.Rounded.Code, onClick = { open(GITHUB_URL) }) { OpenIcon() }
        RowCard(stringResource(R.string.about_portfolio), "aiyu.co.in", Icons.Rounded.Language, onClick = { open(PORTFOLIO_URL) }) { OpenIcon() }

        PageTitle(stringResource(R.string.about_section_open_source))
        RowCard(stringResource(R.string.about_acknowledgements), stringResource(R.string.about_acknowledgements_subtitle), Icons.Rounded.Description, onClick = { navigator.go(AcknowledgementsKey) })
    }
}

@Composable
fun AcknowledgementsScreen() {
    val open = rememberOpenUrl()
    SubPage(stringResource(R.string.about_acknowledgements)) {
        libraries.forEach { (name, url, license) ->
            RowCard(name, "$license · ${url.removePrefix("https://")}", Icons.Rounded.Description, onClick = { open(url) }) { OpenIcon() }
        }
        RowCard("TMDB", stringResource(R.string.about_tmdb), Icons.Rounded.Description, onClick = { open("https://www.themoviedb.org") }) { OpenIcon() }
        RowCard("Plex Media Server", stringResource(R.string.about_plex), Icons.Rounded.Description, onClick = { open("https://www.plex.tv") }) { OpenIcon() }
    }
}
