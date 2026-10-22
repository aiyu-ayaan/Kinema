package com.aiyu.kinema.update

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.net.Uri
import android.os.Build
import android.provider.Settings
import com.aiyu.kinema.BuildConfig
import com.aiyu.kinema.R
import com.aiyu.kinema.core.Releases
import com.aiyu.kinema.core.UpdateChannel
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request

/**
 * The app keeping itself up to date from its GitHub releases (there is no store in the loop). It checks on launch,
 * downloads the APK into the cache and hands it to Android's package installer, whose confirmation screen always
 * has the last word. A `-dev` build's version never parses, so dev builds are never offered anything.
 */
object AppUpdates {
    sealed interface State {
        data object Idle : State
        data object Checking : State
        data object UpToDate : State
        data class Available(val apk: Releases.Apk) : State
        data class Downloading(val apk: Releases.Apk, val progress: Float) : State
        data class Failed(val message: String) : State
    }

    private val _state = MutableStateFlow<State>(State.Idle)
    val state: StateFlow<State> = _state.asStateFlow()

    // Plain TLS: the app's own client pins the Kinema server's certificate, which GitHub does not have.
    private val http by lazy { OkHttpClient() }

    private fun prefs(context: Context) = context.getSharedPreferences("kinema.updates", Context.MODE_PRIVATE)

    /** The version "Later" was pressed for; the launch check stays quiet about it. */
    fun skipped(context: Context): String? = prefs(context).getString("skipped", null)

    fun skip(context: Context, version: String) {
        prefs(context).edit().putString("skipped", version).apply()
        _state.value = State.Idle
    }

    /** Release notes of the installed version, from the last check; null when it was never released. */
    private val _installedNotes = MutableStateFlow<String?>(null)
    val installedNotes: StateFlow<String?> = _installedNotes.asStateFlow()

    /** False for a `-dev` build: it is its own app, and a release APK would install beside it, not over it. */
    val released: Boolean get() = Releases.versionKey(BuildConfig.VERSION_NAME) != null

    /** The chosen channel, by default the installed build's. */
    fun channel(context: Context): UpdateChannel =
        prefs(context).getString("channel", null)?.let { runCatching { UpdateChannel.valueOf(it) }.getOrNull() }
            ?: Releases.channelOf(BuildConfig.VERSION_NAME)

    fun setChannel(context: Context, channel: UpdateChannel) {
        prefs(context).edit().putString("channel", channel.name).apply()
    }

    suspend fun check(context: Context): State = withContext(Dispatchers.IO) {
        _state.value = State.Checking
        val channel = channel(context)
        val next = runCatching {
            val request = Request.Builder().url(Releases.API).header("Accept", "application/vnd.github+json").build()
            http.newCall(request).execute().use { response ->
                if (!response.isSuccessful) error(context.getString(R.string.update_github_status, response.code))
                val body = response.body?.string().orEmpty()
                _installedNotes.value = Releases.notesOf(body, BuildConfig.VERSION_NAME)
                Releases.pickApk(body, BuildConfig.VERSION_NAME, channel)?.let { State.Available(it) } ?: State.UpToDate
            }
        }.getOrElse { State.Failed(it.message ?: "") }
        _state.value = next
        next
    }

    /** Streams the APK into the cache (an installed update has no further use for it) and starts the install. */
    suspend fun install(context: Context, apk: Releases.Apk) {
        if (!context.packageManager.canRequestPackageInstalls()) {
            // "Install unknown apps" is a per-app setting, not a runtime permission: it can only be opened.
            context.startActivity(
                Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${context.packageName}"))
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
            )
            return
        }
        runCatching {
            val file = download(context, apk)
            commit(context, file)
            _state.value = State.Available(apk)
        }.onFailure { _state.value = State.Failed(it.message ?: "") }
    }

    private suspend fun download(context: Context, apk: Releases.Apk): File = withContext(Dispatchers.IO) {
        val dir = File(context.cacheDir, "updates").apply { mkdirs() }
        dir.listFiles()?.forEach { it.delete() }
        val target = File(dir, "kinema-${apk.version}.apk")
        _state.value = State.Downloading(apk, 0f)
        http.newCall(Request.Builder().url(apk.url).build()).execute().use { response ->
            val body = response.body
            if (!response.isSuccessful || body == null) error(context.getString(R.string.update_download_status, response.code))
            val total = body.contentLength().takeIf { it > 0 } ?: apk.size
            var written = 0L
            body.byteStream().use { input ->
                target.outputStream().use { output ->
                    val buffer = ByteArray(64 * 1024)
                    while (true) {
                        val read = input.read(buffer)
                        if (read == -1) break
                        output.write(buffer, 0, read)
                        written += read
                        if (total > 0) _state.value = State.Downloading(apk, (written.toFloat() / total).coerceIn(0f, 1f))
                    }
                }
            }
        }
        target
    }

    /**
     * A PackageInstaller session rather than an ACTION_VIEW intent, so a refusal (an APK signed with another key)
     * comes back to [InstallReceiver] with a reason instead of a dialog that silently closes.
     */
    private suspend fun commit(context: Context, apk: File) = withContext(Dispatchers.IO) {
        val app = context.applicationContext
        val installer = app.packageManager.packageInstaller
        val params = PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL).apply {
            setAppPackageName(app.packageName)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) setRequireUserAction(PackageInstaller.SessionParams.USER_ACTION_REQUIRED)
        }
        val id = installer.createSession(params)
        installer.openSession(id).use { session ->
            session.openWrite("kinema.apk", 0, apk.length()).use { sink ->
                apk.inputStream().use { it.copyTo(sink) }
                session.fsync(sink)
            }
            val status = PendingIntent.getBroadcast(
                app, id, Intent(app, InstallReceiver::class.java).setAction(InstallReceiver.ACTION),
                PendingIntent.FLAG_MUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
            )
            session.commit(status.intentSender)
        }
    }

    internal fun fail(message: String) {
        _state.value = State.Failed(message)
    }
}

/** Where the install session answers. Not exported: only the platform, answering this app's session, sends it. */
class InstallReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != ACTION) return
        when (intent.getIntExtra(PackageInstaller.EXTRA_STATUS, Int.MIN_VALUE)) {
            // The system's confirmation screen, handed back rather than shown.
            PackageInstaller.STATUS_PENDING_USER_ACTION -> {
                @Suppress("DEPRECATION")
                val confirm = if (Build.VERSION.SDK_INT >= 33) intent.getParcelableExtra(Intent.EXTRA_INTENT, Intent::class.java)
                else intent.getParcelableExtra(Intent.EXTRA_INTENT)
                confirm?.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)?.let { runCatching { context.startActivity(it) } }
            }
            PackageInstaller.STATUS_SUCCESS, PackageInstaller.STATUS_FAILURE_ABORTED -> Unit
            PackageInstaller.STATUS_FAILURE_CONFLICT -> AppUpdates.fail(context.getString(com.aiyu.kinema.R.string.update_conflict))
            else -> AppUpdates.fail(intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE).orEmpty())
        }
    }

    companion object {
        const val ACTION = "com.aiyu.kinema.UPDATE_INSTALL_STATUS"
    }
}
