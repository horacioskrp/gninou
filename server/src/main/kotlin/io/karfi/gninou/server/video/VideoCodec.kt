package io.karfi.gninou.server.video

/**
 * Codecs vidéo gérés et paramètres associés.
 *
 * Logique **pure et testable** : correspondance nom → type MIME et calcul d'un
 * débit par défaut. Aucune dépendance au framework Android ici.
 */
enum class VideoCodec(val id: String, val mime: String) {
    H264("h264", "video/avc"),
    H265("h265", "video/hevc"),
    AV1("av1", "video/av01");

    companion object {
        /** Résout un codec par son identifiant (insensible à la casse). */
        fun fromId(id: String): VideoCodec =
            entries.firstOrNull { it.id.equals(id, ignoreCase = true) }
                ?: throw IllegalArgumentException("codec vidéo inconnu : $id")

        /**
         * Débit par défaut (bits/s) en fonction de la définition.
         * Heuristique simple : ~0,2 bit par pixel et par image, bornée.
         */
        fun defaultBitrate(width: Int, height: Int, fps: Int): Int {
            val pixels = width.toLong() * height.toLong()
            val bps = (pixels * fps * 0.2).toLong()
            return bps.coerceIn(2_000_000, 40_000_000).toInt()
        }
    }
}
