package io.karfi.gninou.server

/** Options de ligne de commande du serveur (clé=valeur). */
data class Options(
    val maxSize: Int = 0,
    val maxFps: Int = 60,
    val videoCodec: String = "h264",
    val audio: Boolean = true,
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
                    "audio" -> opts.copy(audio = value != "false")
                    "video" -> opts.copy(video = value != "false")
                    else -> opts
                }
            }
            return opts
        }
    }
}
