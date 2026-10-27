package com.aiyu.kinema.core

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import java.net.URI
import kotlin.math.roundToInt
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

sealed interface Session {
    data object Loading : Session
    /** First run, or the user chose another server. */
    data object NoServer : Session
    data class Unreachable(val server: String, val error: ApiException) : Session
    /** Connected, nobody signed in: pick a profile. [needsSetup] until the admin finishes setup in the web app. */
    data class SignedOut(val server: String, val serverName: String, val profiles: List<User>, val needsSetup: Boolean) : Session
    data class SignedIn(val server: String, val user: User) : Session
}

sealed interface ConnectResult {
    data object Connected : ConnectResult
    /** Self-signed server: show [fingerprint] and call [KinemaRepository.connect] again with it once the user approves. */
    data class Untrusted(val url: String, val fingerprint: String) : ConnectResult
    data class Failed(val error: ApiException) : ConnectResult
}

/** Which address the app is using now: the home one, or the public one away from home. */
enum class Route { LOCAL, PUBLIC }

data class Appearance(val theme: String = "system", val dynamicColor: Boolean = false)

/** The saved server, its two addresses and the session. One instance per process. */
class KinemaRepository(context: Context) {
    private val store = SecureStore(context.applicationContext)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    private val _api = MutableStateFlow<KinemaApi?>(null)
    /** Client of the address in use; replaced when the app moves between the home and the public address. */
    val api: StateFlow<KinemaApi?> = _api

    private val _session = MutableStateFlow<Session>(Session.Loading)
    val session: StateFlow<Session> = _session

    private val _route = MutableStateFlow(Route.LOCAL)
    val route: StateFlow<Route> = _route

    private val _localUrl = MutableStateFlow(store.get(LOCAL).orEmpty())
    /** The address the app first connected to, used at home. */
    val localUrl: StateFlow<String> = _localUrl

    private val _publicOverride = MutableStateFlow(store.get(PUBLIC_OVERRIDE).orEmpty())
    /** The public address set in this app; empty means the one the server's admin set. */
    val publicOverride: StateFlow<String> = _publicOverride

    private val _serverPublicUrl = MutableStateFlow(store.get(PUBLIC_SERVER).orEmpty())
    /** The public address the server announces (`/server/info`). */
    val serverPublicUrl: StateFlow<String> = _serverPublicUrl

    private val _serverInfo = MutableStateFlow<ServerInfo?>(null)
    val serverInfo: StateFlow<ServerInfo?> = _serverInfo

    private val _appearance = MutableStateFlow(Appearance(store.get(THEME) ?: "system", store.get(DYNAMIC) == "true"))
    val appearance: StateFlow<Appearance> = _appearance

    private val _pictureInPicture = MutableStateFlow(store.get(PIP) != "false")
    /** Leaving the app while a video plays shrinks it into a floating window; on by default. Off: it plays on in the background. */
    val pictureInPicture: StateFlow<Boolean> = _pictureInPicture

    fun setPictureInPicture(on: Boolean) {
        store.put(PIP, on.toString())
        _pictureInPicture.value = on
    }

    /** The player's last volume (0..2, above 1 is boost), shared by every title; the phone keeps the media volume itself. */
    var playerVolume: Float = store.get(VOLUME)?.toFloatOrNull() ?: -1f
        set(value) {
            // Whole percents: a swipe writes once per step, not on every touch event.
            val v = (value * 100).roundToInt() / 100f
            if (field == v) return
            field = v
            store.put(VOLUME, v.toString())
        }

    private val _changes = MutableSharedFlow<Unit>(extraBufferCapacity = 4)
    /** Watch state changed (played, marked watched): screens that show it reload. */
    val changes: SharedFlow<Unit> = _changes

    val pinnedFingerprint: String? get() = store.get(PIN)

    init {
        scope.launch { restore() }
        watchNetwork(context)
    }

    /** The public address that counts: this app's own, else the server's. */
    fun publicUrl(): String = _publicOverride.value.ifBlank { _serverPublicUrl.value }

    private fun candidates() = listOf(_localUrl.value, publicUrl(), store.get(SERVER).orEmpty()).filter { it.isNotBlank() }.distinct()

    private suspend fun reachable(url: String) =
        runCatching { KinemaApi(url, store.get(PIN), connectSeconds = 2) { null }.serverInfo() }.getOrNull()

    private fun open(server: String) {
        store.put(SERVER, server)
        _api.value = KinemaApi(server, store.get(PIN)) { store.get(TOKEN) }
        _route.value = if (server == _localUrl.value || _localUrl.value.isBlank()) Route.LOCAL else Route.PUBLIC
    }

    /** [info] always comes over a verified connection (a public authority or the approved pin), so its fingerprint can be trusted. */
    private fun rememberInfo(info: ServerInfo) {
        _serverInfo.value = info
        _serverPublicUrl.value = info.publicUrl
        store.put(PUBLIC_SERVER, info.publicUrl.ifBlank { null })
        findHome(info)
    }

    private var homeSearch: Job? = null

    /**
     * Connected only by the public address: look for the same server on this network, so at home the video goes
     * straight over Wi-Fi instead of out through the proxy and back. "Same" means it shows the certificate the
     * server named over the verified public connection; then that certificate is pinned without asking.
     */
    private fun findHome(info: ServerInfo) {
        val fingerprint = info.tlsFingerprint.ifBlank { return }
        val local = _localUrl.value
        if (local.isNotBlank() && isLanHost(runCatching { URI(local).host }.getOrNull().orEmpty())) return
        // Another certificate is already approved for this server: keep it rather than swap pins silently.
        if (store.get(PIN).let { it != null && it != fingerprint }) return
        if (homeSearch?.isActive == true) return
        homeSearch = scope.launch {
            val home = ServerDiscovery.scan().firstOrNull { found ->
                val uri = runCatching { URI(found.url) }.getOrNull() ?: return@firstOrNull false
                uri.scheme == "https" && uri.port != -1 &&
                    runCatching { Tls.fetchFingerprint(uri.host.trim('[', ']'), uri.port) }.getOrNull() == fingerprint
            } ?: return@launch
            store.put(PIN, fingerprint)
            store.put(LOCAL, home.url)
            _localUrl.value = home.url
            follow()
        }
    }

    /** Re-reads the server's name and addresses, e.g. after the admin renamed it from another device. */
    suspend fun refreshInfo() {
        request { it.serverInfo() }.onSuccess(::rememberInfo)
    }

    /** The first address that answers now, home first, with its server info. */
    private suspend fun best(): Pair<String, ServerInfo>? {
        for (url in candidates()) reachable(url)?.let { return url to it }
        return null
    }

    /**
     * Moves to the home or the public address, whichever answers now. The session token is valid on both,
     * so nobody signs in again. Runs when the network changes.
     */
    suspend fun follow() {
        val current = _api.value?.baseUrl ?: return
        if (_session.value !is Session.SignedIn && _session.value !is Session.Unreachable) return
        val (url, info) = best() ?: return
        rememberInfo(info)
        if (url == current && _session.value is Session.SignedIn) return
        if (_session.value is Session.Unreachable) return restore()
        open(url)
        (_session.value as? Session.SignedIn)?.let { _session.value = it.copy(server = url) }
    }

    private var followJob: Job? = null

    private fun watchNetwork(context: Context) {
        val cm = context.getSystemService(ConnectivityManager::class.java) ?: return
        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) = schedule()
            override fun onLost(network: Network) = schedule()
            private fun schedule() {
                scope.launch {
                    followJob?.cancel()
                    // Wi-Fi to mobile data hands over in steps; wait until it settles.
                    followJob = launch { delay(1_500); follow() }
                }
            }
        }
        runCatching { cm.registerDefaultNetworkCallback(callback) }
    }

    /** Picks up where the last launch left off. Also the "Try again" action of [Session.Unreachable]. */
    suspend fun restore() {
        val saved = store.get(SERVER) ?: run { _session.value = Session.NoServer; return }
        _session.value = Session.Loading
        val found = best()
        val server = found?.first ?: saved
        found?.second?.let(::rememberInfo)
        open(server)
        val api = _api.value ?: return
        try {
            if (store.get(TOKEN) != null) {
                try {
                    _session.value = Session.SignedIn(server, api.me())
                    return
                } catch (e: ApiException) {
                    if (e.status != 401) throw e
                    store.put(TOKEN, null)
                }
            }
            signedOut()
        } catch (e: ApiException) {
            _session.value = Session.Unreachable(server, e)
        }
    }

    private suspend fun signedOut() {
        val api = _api.value ?: return
        val info = api.serverInfo().also(::rememberInfo)
        val status = api.authStatus()
        _session.value = Session.SignedOut(api.baseUrl, info.name, if (status.needsSetup) emptyList() else api.profiles(), status.needsSetup)
    }

    /**
     * Connects to the server at [url] (typed or discovered). [trustFingerprint] is the self-signed certificate
     * the user just approved.
     */
    suspend fun connect(url: String, trustFingerprint: String? = null): ConnectResult {
        val candidate = KinemaApi(url, trustFingerprint) { null }
        return try {
            val info = candidate.serverInfo()
            store.put(PIN, trustFingerprint)
            store.put(TOKEN, null)
            store.put(LOCAL, url)
            _localUrl.value = url
            homeSearch?.cancel()
            rememberInfo(info)
            open(url)
            signedOut()
            ConnectResult.Connected
        } catch (e: ApiException) {
            if (e.status != ApiException.UNTRUSTED || trustFingerprint != null) return ConnectResult.Failed(e)
            val uri = URI(url)
            runCatching { Tls.fetchFingerprint(uri.host.trim('[', ']'), if (uri.port == -1) 443 else uri.port) }
                .fold({ ConnectResult.Untrusted(url, it) }, { ConnectResult.Failed(e) })
        }
    }

    /** Null on success, otherwise the error to show. */
    suspend fun login(user: User, password: String?, pin: String?): ApiException? {
        val api = _api.value ?: return ApiException("No server", ApiException.OFFLINE)
        return try {
            val session = api.login(user.id, password, pin)
            store.put(TOKEN, session.token)
            _session.value = Session.SignedIn(api.baseUrl, session.user)
            null
        } catch (e: ApiException) {
            e
        }
    }

    /** Back to the profile picker. */
    fun logout() {
        val api = _api.value ?: return
        scope.launch {
            runCatching { api.logout() }
            store.put(TOKEN, null)
            runCatching { signedOut() }.onFailure { _session.value = Session.Unreachable(api.baseUrl, it as? ApiException ?: ApiException("", 0)) }
        }
    }

    /** Back to the connect screen. The saved server, its addresses and its approved certificate are dropped. */
    fun forgetServer() {
        homeSearch?.cancel()
        listOf(SERVER, LOCAL, PUBLIC_OVERRIDE, PUBLIC_SERVER, PIN, TOKEN).forEach { store.put(it, null) }
        _localUrl.value = ""
        _publicOverride.value = ""
        _serverPublicUrl.value = ""
        _serverInfo.value = null
        _api.value = null
        _session.value = Session.NoServer
    }

    /** Sets this app's public address ("" uses the server's). Null on success, otherwise why it was refused. */
    suspend fun setPublicUrl(input: String): String? {
        val url = if (input.isBlank()) "" else normalizeServerUrl(input) ?: return input
        store.put(PUBLIC_OVERRIDE, url.ifBlank { null })
        _publicOverride.value = url
        follow()
        return null
    }

    /** Tries an address now; the server info when it answers. */
    suspend fun probe(url: String): ServerInfo? = reachable(url)

    fun setTheme(theme: String) {
        store.put(THEME, theme)
        _appearance.value = _appearance.value.copy(theme = theme)
    }

    fun setDynamicColor(on: Boolean) {
        store.put(DYNAMIC, on.toString())
        _appearance.value = _appearance.value.copy(dynamicColor = on)
    }

    /** Tell the screens that watch state changed. */
    fun notifyChanged() {
        _changes.tryEmit(Unit)
    }

    /**
     * Saves where playback is. Runs on the repository's own scope so the last report survives the player closing;
     * screens reload when [notify] (the viewer left the player).
     */
    fun saveProgress(itemId: Long, positionS: Double, watched: Boolean, notify: Boolean = false) {
        scope.launch {
            request { it.progress(itemId, positionS, watched) }
            if (notify) notifyChanged()
        }
    }

    /** Runs [block] on the current client; null when there is none or it failed. */
    suspend fun <T> request(block: suspend (KinemaApi) -> T): Result<T> {
        val api = _api.value ?: return Result.failure(ApiException("No server", ApiException.OFFLINE))
        return try {
            Result.success(block(api))
        } catch (e: ApiException) {
            if (e.status == 401) {
                store.put(TOKEN, null)
                runCatching { signedOut() }
            }
            Result.failure(e)
        }
    }

    private companion object {
        const val SERVER = "server"
        const val LOCAL = "local"
        const val PUBLIC_OVERRIDE = "public_override"
        const val PUBLIC_SERVER = "public_server"
        const val PIN = "pin"
        const val TOKEN = "token"
        const val THEME = "theme"
        const val DYNAMIC = "dynamic_color"
        const val PIP = "picture_in_picture"
        const val VOLUME = "player_volume"
    }
}
