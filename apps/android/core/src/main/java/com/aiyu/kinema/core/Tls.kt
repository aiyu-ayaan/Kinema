package com.aiyu.kinema.core

import java.net.InetSocketAddress
import java.security.KeyStore
import java.security.MessageDigest
import java.security.cert.Certificate
import java.security.cert.X509Certificate
import javax.net.ssl.HostnameVerifier
import javax.net.ssl.HttpsURLConnection
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLSocket
import javax.net.ssl.TrustManagerFactory
import javax.net.ssl.X509TrustManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * A home server makes its own certificate, which no authority vouches for. The app shows its fingerprint once,
 * the user approves it, and from then on only that exact certificate (or one a real authority signed, such as a
 * reverse proxy on the public address) is accepted.
 */
object Tls {
    fun fingerprint(cert: Certificate): String =
        MessageDigest.getInstance("SHA-256").digest(cert.encoded).joinToString(":") { "%02X".format(it) }

    private val system: X509TrustManager by lazy {
        TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm())
            .apply { init(null as KeyStore?) }
            .trustManagers.filterIsInstance<X509TrustManager>().first()
    }

    /** System authorities, plus the one certificate whose fingerprint is [pin]. */
    class PinnedTrustManager(private val pin: String?) : X509TrustManager {
        override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) {
            if (pin != null && chain.isNotEmpty() && fingerprint(chain[0]) == pin) return
            system.checkServerTrusted(chain, authType)
        }

        override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) = system.checkClientTrusted(chain, authType)
        override fun getAcceptedIssuers(): Array<X509Certificate> = system.acceptedIssuers
    }

    /** A self-signed certificate lists the address it was made on; the server's address can change, its pin cannot. */
    fun hostnameVerifier(pin: String?) = HostnameVerifier { host, session ->
        (pin != null && runCatching { fingerprint(session.peerCertificates[0]) == pin }.getOrDefault(false)) ||
            HttpsURLConnection.getDefaultHostnameVerifier().verify(host, session)
    }

    fun sslContext(trust: X509TrustManager): SSLContext = SSLContext.getInstance("TLS").apply { init(null, arrayOf(trust), null) }

    /**
     * Reads the certificate [host] presents so the user can approve it. Trusts nothing and sends nothing:
     * the socket closes right after the handshake.
     */
    suspend fun fetchFingerprint(host: String, port: Int): String = withContext(Dispatchers.IO) {
        val readOnly = object : X509TrustManager {
            override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) = Unit
            override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) = Unit
            override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
        }
        (sslContext(readOnly).socketFactory.createSocket() as SSLSocket).use { socket ->
            socket.connect(InetSocketAddress(host, port), 5_000)
            socket.soTimeout = 5_000
            socket.startHandshake()
            fingerprint(socket.session.peerCertificates[0])
        }
    }
}
