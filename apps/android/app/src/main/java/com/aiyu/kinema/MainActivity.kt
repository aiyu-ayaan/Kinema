package com.aiyu.kinema

import android.app.Application
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import android.view.animation.PathInterpolator
import android.window.SplashScreenView
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.annotation.RequiresApi
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.ImageLoader
import coil3.SingletonImageLoader
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import coil3.request.crossfade
import com.aiyu.kinema.core.KinemaRepository
import com.aiyu.kinema.core.Session
import com.aiyu.kinema.ui.AppNavigation
import com.aiyu.kinema.ui.ConnectScreen
import com.aiyu.kinema.ui.LoadingBox
import com.aiyu.kinema.ui.LocalApi
import com.aiyu.kinema.ui.LocalRepository
import com.aiyu.kinema.ui.ProfilesScreen
import com.aiyu.kinema.ui.UnreachableScreen
import com.aiyu.kinema.ui.theme.KinemaTheme
import okhttp3.Call
import java.time.Duration
import java.time.Instant
import okhttp3.OkHttpClient

class KinemaApp : Application(), SingletonImageLoader.Factory {
    val repository by lazy { KinemaRepository(this) }

    // Artwork goes through the current server client: same certificate pin, same session, home or public address.
    override fun newImageLoader(context: android.content.Context): ImageLoader {
        val fallback by lazy { OkHttpClient() }
        val calls = Call.Factory { request -> (repository.api.value?.client ?: fallback).newCall(request) }
        return ImageLoader.Builder(context)
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { calls })) }
            .crossfade(true)
            .build()
    }
}

class MainActivity : ComponentActivity() {
    private val repository get() = (application as KinemaApp).repository

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) splashScreen.setOnExitAnimationListener(::exitSplash)
        enableEdgeToEdge()
        setContent {
            val appearance by repository.appearance.collectAsStateWithLifecycle()
            val api by repository.api.collectAsStateWithLifecycle()
            val dark = when (appearance.theme) {
                "light" -> false
                "dark" -> true
                else -> isSystemInDarkTheme()
            }
            // System bar icons follow the in-app theme, not only the system setting.
            DisposableEffect(dark) {
                enableEdgeToEdge(
                    statusBarStyle = SystemBarStyle.auto(Color.TRANSPARENT, Color.TRANSPARENT) { dark },
                    navigationBarStyle = SystemBarStyle.auto(Color.TRANSPARENT, Color.TRANSPARENT) { dark },
                )
                onDispose { }
            }
            KinemaTheme(dark = dark, dynamicColor = appearance.dynamicColor) {
                // Surface (not a bare Box) so LocalContentColor is onSurface for every screen.
                Surface(Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.surface) {
                    CompositionLocalProvider(LocalRepository provides repository, LocalApi provides api) { KinemaRoot() }
                }
            }
        }
    }

    /** Lets the projector animation finish, then the beam floods forward as the splash fades into the app. */
    @RequiresApi(Build.VERSION_CODES.S)
    private fun exitSplash(view: SplashScreenView) {
        val end = view.iconAnimationStart?.plus(view.iconAnimationDuration ?: Duration.ZERO)
        val wait = end?.let { Duration.between(Instant.now(), it).toMillis().coerceIn(0, 1000) } ?: 0
        val emphasized = PathInterpolator(0.3f, 0f, 0.8f, 0.15f)
        view.iconView?.animate()?.setStartDelay(wait)?.setDuration(320)?.setInterpolator(emphasized)
            ?.scaleX(1.6f)?.scaleY(1.6f)?.alpha(0f)
        view.animate().setStartDelay(wait + 80).setDuration(280).alpha(0f).withEndAction(view::remove)
    }
}

/** Connect, pick a profile, browse: driven by the session. Steps swap with a fade-through. */
@Composable
private fun KinemaRoot() {
    val session by LocalRepository.current.session.collectAsStateWithLifecycle()
    val motion = MaterialTheme.motionScheme
    AnimatedContent(
        targetState = session,
        contentKey = { it::class },
        transitionSpec = {
            (fadeIn(motion.defaultEffectsSpec()) + scaleIn(motion.defaultSpatialSpec(), initialScale = 0.92f))
                .togetherWith(fadeOut(motion.fastEffectsSpec()))
        },
        label = "session",
    ) { s ->
        when (s) {
            Session.Loading -> LoadingBox()
            Session.NoServer -> ConnectScreen()
            is Session.Unreachable -> UnreachableScreen(s)
            is Session.SignedOut -> ProfilesScreen(s)
            is Session.SignedIn -> AppNavigation()
        }
    }
}
