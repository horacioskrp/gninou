package io.karfi.gninou.server.net

import android.net.LocalSocket
import android.net.LocalSocketAddress
import io.karfi.gninou.server.Options
import io.karfi.gninou.server.util.Log

/**
 * Gère la connexion du serveur au tunnel adb.
 *
 * Le client (côté PC) écoute sur un port TCP et installe un `adb reverse`
 * mappant `localabstract:gninou` vers ce port. Côté device, le serveur se
 * connecte simplement au socket local abstrait de même nom : adb achemine
 * le trafic jusqu'au client.
 *
 * Sur cette connexion transiteront les canaux vidéo / audio / contrôle.
 */
class Connection(private val options: Options) {

    private var socket: LocalSocket? = null

    /** Se connecte au socket local abstrait exposé via `adb reverse`. */
    fun open() {
        val s = LocalSocket()
        s.connect(LocalSocketAddress(SOCKET_NAME, LocalSocketAddress.Namespace.ABSTRACT))
        socket = s
        Log.i("connecté au tunnel ($SOCKET_NAME)")
        // TODO: handshake (version/codec), puis exposer videoSink/audioSink/controlSource.
    }

    fun close() {
        socket?.close()
        socket = null
    }

    companion object {
        /** Doit correspondre à SOCKET_NAME côté client (device/tunnel.rs). */
        const val SOCKET_NAME = "gninou"
    }
}
