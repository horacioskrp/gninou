package io.karfi.gninou.server.video

import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import io.karfi.gninou.server.Options
import io.karfi.gninou.server.util.Log
import java.io.OutputStream

/**
 * Encodage vidéo matériel via `MediaCodec`, alimenté par la capture écran.
 *
 * Chaîne : [ScreenCapture] projette l'écran sur la `Surface` d'entrée de
 * l'encodeur ; on draine les trames encodées (H.264/H.265/AV1) vers le canal
 * vidéo du tunnel.
 *
 * ⚠️ Dépend du framework Android (`MediaCodec`) : **non testable hors appareil**.
 * La sélection du codec et le calcul du débit sont isolés dans [VideoCodec]
 * (logique pure, testée).
 */
class ScreenEncoder(private val options: Options) {

    @Volatile
    private var running = false

    /**
     * Démarre la capture + l'encodage et écrit le flux dans [videoSink]
     * jusqu'à [stop]. Appel bloquant (à lancer dans un thread dédié).
     */
    fun start(videoSink: OutputStream) {
        val codec = VideoCodec.fromId(options.videoCodec)
        val capture = ScreenCapture(options)

        val encoder = MediaCodec.createEncoderByType(codec.mime)
        try {
            // On configure d'abord avec la définition de l'écran (après maxSize).
            val info = DisplayInfo.primary()
            val (w, h) = info.scaled(options.maxSize)
            val format = buildFormat(codec.mime, w, h, options.maxFps)

            encoder.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
            val inputSurface = encoder.createInputSurface()
            encoder.start()

            // La capture écran écrit dans la surface d'entrée de l'encodeur.
            capture.start(inputSurface)
            Log.i("encodage ${codec.id} ${w}x$h @ ${options.maxFps}fps")

            running = true
            drain(encoder, videoSink)
        } finally {
            running = false
            runCatching { encoder.stop() }
            runCatching { encoder.release() }
            capture.release()
            Log.i("encodage arrêté")
        }
    }

    fun stop() {
        running = false
    }

    /** Boucle de drainage des buffers encodés vers le sink. */
    private fun drain(encoder: MediaCodec, sink: OutputStream) {
        val bufferInfo = MediaCodec.BufferInfo()
        while (running) {
            val index = encoder.dequeueOutputBuffer(bufferInfo, TIMEOUT_US)
            if (index < 0) continue
            val buffer = encoder.getOutputBuffer(index)
            if (buffer != null && bufferInfo.size > 0) {
                buffer.position(bufferInfo.offset)
                buffer.limit(bufferInfo.offset + bufferInfo.size)
                val bytes = ByteArray(bufferInfo.size)
                buffer.get(bytes)
                // TODO (#protocole) : préfixer un MediaHeader (pts, taille, keyframe).
                sink.write(bytes)
                sink.flush()
            }
            encoder.releaseOutputBuffer(index, false)
        }
    }

    private fun buildFormat(mime: String, w: Int, h: Int, fps: Int): MediaFormat =
        MediaFormat.createVideoFormat(mime, w, h).apply {
            setInteger(
                MediaFormat.KEY_COLOR_FORMAT,
                MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface,
            )
            setInteger(MediaFormat.KEY_BIT_RATE, VideoCodec.defaultBitrate(w, h, fps))
            setInteger(MediaFormat.KEY_FRAME_RATE, fps)
            setFloat(MediaFormat.KEY_I_FRAME_INTERVAL, I_FRAME_INTERVAL_S)
            // Latence faible : pas de B-frames côté encodeur si supporté.
        }

    companion object {
        private const val TIMEOUT_US = 100_000L
        private const val I_FRAME_INTERVAL_S = 2.0f
    }
}
