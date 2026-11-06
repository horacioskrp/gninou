package io.karfi.gninou.server.net

import io.karfi.gninou.server.Options

/**
 * Gère les sockets du tunnel côté device.
 *
 * Le transport (USB / TCP-IP / Wi-Fi) est assuré par adb côté client ;
 * ici on expose simplement les flux locaux (localabstract) une fois le
 * tunnel ouvert : un canal vidéo, un canal audio, un canal contrôle.
 */
class Connection(private val options: Options) {

    /** Ouvre le(s) socket(s) local(aux) attendu(s) par le client. */
    fun open() {
        // TODO: LocalServerSocket("gninou") + accept() des canaux
    }

    // TODO: exposer videoSink / audioSink / controlSource une fois ouverts.
}
