package io.karfi.gninou.server.audio

import io.karfi.gninou.server.Options
import io.karfi.gninou.server.util.Log
import java.io.OutputStream

/**
 * Capture et encodage audio (Android 11+).
 *
 * Capture via `AudioRecord` en mode `REMOTE_SUBMIX` (playback capture),
 * encodage selon [AudioCodec] (sauf `RAW` = PCM), puis envoi des trames vers
 * le canal audio du tunnel.
 *
 * ⚠️ Dépend du framework Android : **non testable hors appareil**. La
 * résolution du codec est isolée dans [AudioCodec] (logique pure, testée).
 */
class AudioEncoder(private val options: Options) {

    @Volatile
    private var running = false

    /** Démarre la capture + l'encodage et écrit vers [audioSink]. Bloquant. */
    fun start(audioSink: OutputStream) {
        val codec = AudioCodec.fromId(options.audioCodec)
        Log.i("capture audio ${codec.id} @ ${SAMPLE_RATE}Hz stéréo")
        running = true
        try {
            // TODO:
            //  1. AudioRecord (AudioPlaybackCaptureConfiguration / REMOTE_SUBMIX, 48 kHz stéréo)
            //  2. si codec.isCompressed : MediaCodec(codec.mime) ; sinon PCM direct
            //  3. drainer les buffers -> audioSink (prefixe MediaHeader quand le protocole sera fige)
            @Suppress("UNUSED_EXPRESSION")
            audioSink
        } finally {
            running = false
            Log.i("capture audio arrêtée")
        }
    }

    fun stop() {
        running = false
    }

    companion object {
        const val SAMPLE_RATE = 48_000
    }
}
