package io.karfi.gninou.server.audio

import io.karfi.gninou.server.Options

/**
 * Capture et encodage audio (Android 11+).
 *
 * Capture via `AudioRecord` en mode `REMOTE_SUBMIX` (playback capture),
 * encodage Opus/AAC, puis envoi des trames vers le canal audio du tunnel.
 */
class AudioEncoder(private val options: Options) {

    /** Démarre la boucle de capture/encodage audio. */
    fun start(/* audioSink: OutputStream */) {
        // TODO:
        //  1. AudioRecord (REMOTE_SUBMIX, 48 kHz stéréo)
        //  2. MediaCodec audio (opus/aac)
        //  3. drainer les buffers encodés -> audioSink
    }
}
