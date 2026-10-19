package com.aiyu.kinema.core

import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.NetworkInterface
import java.net.Socket
import java.net.SocketTimeoutException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

data class FoundServer(val name: String, val url: String, val version: String? = null, val thisComputer: Boolean = false)

/** Finds Kinema servers on the local network: broadcasts a probe, servers answer (backend `discovery.rs`). */
object ServerDiscovery {
    private const val PORT = 48555
    private const val DEV_PORT = 3000
    private val PROBE = "KINEMA_DISCOVER".toByteArray()

    suspend fun scan(timeoutMillis: Long = 1_800): List<FoundServer> = withContext(Dispatchers.IO) {
        val found = LinkedHashMap<String, FoundServer>()
        runCatching {
            DatagramSocket().use { socket ->
                socket.broadcast = true
                socket.soTimeout = 300
                // The limited broadcast can leave by mobile data; each interface's own broadcast address reaches Wi-Fi.
                (broadcastAddresses() + InetAddress.getByName("255.255.255.255")).forEach {
                    runCatching { socket.send(DatagramPacket(PROBE, PROBE.size, it, PORT)) }
                }
                val buffer = ByteArray(512)
                val deadline = System.currentTimeMillis() + timeoutMillis
                while (System.currentTimeMillis() < deadline) {
                    val packet = DatagramPacket(buffer, buffer.size)
                    try {
                        socket.receive(packet)
                    } catch (_: SocketTimeoutException) {
                        continue
                    }
                    val host = packet.address.hostAddress ?: continue
                    parseReply(String(packet.data, 0, packet.length), host)?.let { found[it.url] = it }
                }
            }
        }
        // Broadcasts don't cross an emulator's NAT or a USB cable. A dev server reachable through `adb reverse`
        // (`pnpm android:dev` sets it) or on the emulator's host address is offered too; on a phone nothing listens there.
        if (found.isEmpty()) {
            listOf("localhost", "10.0.2.2").firstOrNull { host ->
                runCatching { Socket().use { it.connect(InetSocketAddress(host, DEV_PORT), 400) } }.isSuccess
            }?.let { found["dev"] = FoundServer("Kinema", "https://$it:$DEV_PORT", thisComputer = true) }
        }
        found.values.toList()
    }

    private fun broadcastAddresses(): List<InetAddress> = runCatching {
        NetworkInterface.getNetworkInterfaces().asSequence()
            .filter { it.isUp && !it.isLoopback }
            .flatMap { it.interfaceAddresses.asSequence() }
            .mapNotNull { it.broadcast }
            .toList()
    }.getOrDefault(emptyList())

    /** Null for anything that is not a Kinema server. IPv6 hosts get brackets. */
    fun parseReply(text: String, host: String): FoundServer? = runCatching {
        val reply = Json.parseToJsonElement(text).jsonObject
        if (reply["service"]?.jsonPrimitive?.contentOrNull != "kinema") return null
        val port = reply["port"]?.jsonPrimitive?.intOrNull?.takeIf { it in 1..65535 } ?: return null
        val scheme = if (reply["tls"]?.jsonPrimitive?.booleanOrNull == false) "http" else "https"
        val name = reply["name"]?.jsonPrimitive?.contentOrNull?.take(60)?.ifBlank { null } ?: "Kinema"
        val h = if (':' in host) "[${host.substringBefore('%')}]" else host
        FoundServer(name, "$scheme://$h:$port", reply["version"]?.jsonPrimitive?.contentOrNull)
    }.getOrNull()
}

/**
 * What the user typed, as a base URL: `192.168.1.10:3000`, `media.example.com` or a full URL. No scheme means
 * https; a bare host on the LAN gets the default port. Null when it is not an address.
 */
fun normalizeServerUrl(input: String): String? {
    var s = input.trim().trimEnd('/')
    if (s.isEmpty() || s.any { it.isWhitespace() }) return null
    val explicit = s.startsWith("https://") || s.startsWith("http://")
    if (!explicit) s = "https://$s"
    val uri = runCatching { java.net.URI(s) }.getOrNull() ?: return null
    val host = uri.host ?: return null
    if (uri.port == -1 && !explicit && isLanHost(host)) s = s.replaceFirst(host, "$host:3000")
    return s.removeSuffix("/api")
}

/** Private addresses and `.local` names: where a home server listens on its own port, not behind a proxy on 443. */
fun isLanHost(host: String): Boolean {
    if (host == "localhost" || host.endsWith(".local") || host == "10.0.2.2") return true
    val p = host.split('.').mapNotNull { it.toIntOrNull() }
    if (p.size != 4) return false
    return p[0] == 10 || p[0] == 127 || (p[0] == 192 && p[1] == 168) || (p[0] == 172 && p[1] in 16..31)
}
