package io.karfi.gninou.server

/**
 * Point d'entrée du serveur gninou.
 *
 * Exécuté sur l'appareil Android via `app_process`, p.ex. :
 *
 *   adb push gninou-server.jar /data/local/tmp/
 *   adb shell CLASSPATH=/data/local/tmp/gninou-server.jar \
 *       app_process / io.karfi.gninou.server.Server <args>
 *
 * Squelette : parse les options et trace. La capture écran, l'encodage
 * MediaCodec et l'injection d'entrées restent à implémenter.
 */
object Server {

    @JvmStatic
    fun main(args: Array<String>) {
        val options = Options.parse(args)
        log("gninou-server démarré, options=$options")

        // TODO:
        //  1. ouvrir le socket du tunnel (localabstract:gninou)
        //  2. négocier le codec / la résolution
        //  3. capturer l'écran (SurfaceControl) -> MediaCodec
        //  4. pousser le flux encodé dans le socket
        //  5. lire les événements de contrôle et les injecter (InputManager)
        log("pipeline non implémenté — squelette uniquement")
    }

    private fun log(message: String) {
        // Sur l'appareil, la sortie standard remonte dans la console adb.
        System.out.println("[gninou-server] $message")
    }
}

/** Options de ligne de commande du serveur. */
data class Options(
    val maxSize: Int = 0,
    val maxFps: Int = 60,
    val videoCodec: String = "h264",
    val audio: Boolean = true,
) {
    companion object {
        fun parse(args: Array<String>): Options {
            var opts = Options()
            for (arg in args) {
                val (key, value) = arg.substringBefore('=') to arg.substringAfter('=', "")
                opts = when (key) {
                    "max_size" -> opts.copy(maxSize = value.toIntOrNull() ?: 0)
                    "max_fps" -> opts.copy(maxFps = value.toIntOrNull() ?: 60)
                    "video_codec" -> opts.copy(videoCodec = value.ifEmpty { "h264" })
                    "audio" -> opts.copy(audio = value != "false")
                    else -> opts
                }
            }
            return opts
        }
    }
}
