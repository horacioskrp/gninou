package io.karfi.gninou.server.audio

/**
 * Codecs audio gérés et paramètres associés.
 *
 * Logique **pure et testable** : correspondance nom → type MIME. Le codec
 * `RAW` correspond au PCM non compressé (pas de MIME d'encodeur).
 */
enum class AudioCodec(val id: String, val mime: String?) {
    OPUS("opus", "audio/opus"),
    AAC("aac", "audio/mp4a-latm"),
    RAW("raw", null);

    val isCompressed: Boolean get() = mime != null

    companion object {
        fun fromId(id: String): AudioCodec =
            entries.firstOrNull { it.id.equals(id, ignoreCase = true) }
                ?: throw IllegalArgumentException("codec audio inconnu : $id")
    }
}
