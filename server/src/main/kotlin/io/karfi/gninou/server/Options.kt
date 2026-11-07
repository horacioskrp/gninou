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
    /** Identifiant de l'affichage à refléter (0 = principal). */
    val displayId: Int = 0,
    /** Créer un nouvel affichage virtuel (Android 10+) plutôt que refléter. */
    val newDisplay: Boolean = false,
    /** Spécification `LxH`, `LxH/dpi` ou vide (défauts) pour l'affichage virtuel. */
    val newDisplaySize: String = "",
    /** Source vidéo : "display" (défaut) ou "camera". */
    val videoSource: String = "display",
    /** Identifiant de caméra explicite, ou vide. */
    val cameraId: String = "",
    /** Orientation caméra : "back"/"front"/"external", ou vide. */
    val cameraFacing: String = "",
    /** Taille de capture caméra `LxH`, ou vide (défauts). */
    val cameraSize: String = "",
    /** Images par seconde caméra ; 0 = défaut. */
    val cameraFps: Int = 0,
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
                    "display_id" -> opts.copy(displayId = value.toIntOrNull() ?: 0)
                    "new_display" -> opts.copy(newDisplay = true, newDisplaySize = value)
                    "video_source" -> opts.copy(videoSource = value.ifEmpty { "display" })
                    "camera_id" -> opts.copy(cameraId = value)
                    "camera_facing" -> opts.copy(cameraFacing = value)
                    "camera_size" -> opts.copy(cameraSize = value)
                    "camera_fps" -> opts.copy(cameraFps = value.toIntOrNull() ?: 0)
                    else -> opts
                }
            }
            return opts
        }
    }
}
