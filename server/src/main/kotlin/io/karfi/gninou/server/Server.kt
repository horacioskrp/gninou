package io.karfi.gninou.server

import io.karfi.gninou.server.audio.AudioEncoder
import io.karfi.gninou.server.control.Controller
import io.karfi.gninou.server.net.Connection
import io.karfi.gninou.server.util.Log
import io.karfi.gninou.server.video.ScreenEncoder

/**
 * Point d'entrée du serveur gninou.
 *
 * Exécuté sur l'appareil Android via `app_process`, p.ex. :
 *
 *   adb push gninou-server.jar /data/local/tmp/
 *   adb shell CLASSPATH=/data/local/tmp/gninou-server.jar \
 *       app_process / io.karfi.gninou.server.Server <args>
 *
 * Il orchestre trois responsabilités, chacune dans son package :
 *   - net     : socket du tunnel (USB / TCP-IP / Wi-Fi côté device).
 *   - video   : capture écran + encodage MediaCodec.
 *   - audio   : capture audio + encodage (Android 11+).
 *   - control : lecture des événements et injection (InputManager).
 */
object Server {

    @JvmStatic
    fun main(args: Array<String>) {
        val options = Options.parse(args)
        Log.i("gninou-server démarré, options=$options")

        val connection = Connection(options)
        val screen = ScreenEncoder(options)
        val audio = if (options.audio) AudioEncoder(options) else null
        val controller = Controller()

        // TODO:
        //  1. connection.open()                  // sockets video/audio/control
        //  2. screen.start(connection.videoSink)
        //  3. audio?.start(connection.audioSink)
        //  4. controller.loop(connection.controlSource)   // injection d'events
        Log.w("pipeline non implémenté — squelette uniquement")

        // Évite l'avertissement « unused » tant que le pipeline n'est pas branché.
        listOf<Any?>(connection, screen, audio, controller)
    }
}
