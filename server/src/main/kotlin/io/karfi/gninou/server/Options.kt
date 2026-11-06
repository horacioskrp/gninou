package io.karfi.gninou.server

/** Options de ligne de commande du serveur (clé=valeur). */
data class Options(
    val maxSize: Int = 0,
    val maxFps: Int = 60,
    val videoCodec: String = "h264",
    /** Débit vidéo en bits/s ; 0 = automatique (calculé selon la définition). */
    val bitRate: Int = 0,
    /** Rognage `W:H:X:Y` ou vide (pas de rognage). */
    val crop: String = "",
    /** Rotation de l'affichage en degrés (0/90/180/270). */
    val rotation: Int = 0,
    val audio: Boolean = true,
    val audioCodec: String = "opus",
    val video: Boolean = true,
) {
    companion object {
        fun parse(args: Array<String>): Options {
            var opts = Options()
            for (arg in args) {
                val key = arg.substringBefore('=')
                val value = arg.substringAfter('=', "")
                opts = when (key) {
                    "max_size" -> opts.copy(maxSize = value.toIntOrNull() ?: 0)
                    "max_fps" -> opts.copy(maxFps = value.toIntOrNull() ?: 60)
                    "video_codec" -> opts.copy(videoCodec = value.ifEmpty { "h264" })
                    "bit_rate" -> opts.copy(bitRate = value.toIntOrNull() ?: 0)
                    "crop" -> opts.copy(crop = value)
                    "rotation" -> opts.copy(rotation = value.toIntOrNull() ?: 0)
                    "audio" -> opts.copy(audio = value != "false")
                    "audio_codec" -> opts.copy(audioCodec = value.ifEmpty { "opus" })
                    "video" -> opts.copy(video = value != "false")
                    else -> opts
                }
            }
            return opts
        }
    }
}
