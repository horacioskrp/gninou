package io.karfi.gninou.server.video

import io.karfi.gninou.server.Options

/**
 * Capture de l'écran et encodage vidéo.
 *
 * Capture via `SurfaceControl` (display virtuel) alimentant la surface
 * d'entrée d'un `MediaCodec` configuré pour le codec demandé. Les trames
 * encodées sont poussées vers le canal vidéo du tunnel.
 */
class ScreenEncoder(private val options: Options) {

    /** Démarre la boucle de capture/encodage. */
    fun start(/* videoSink: OutputStream */) {
        // TODO:
        //  1. créer un display virtuel (SurfaceControl.createDisplay)
        //  2. configurer MediaCodec (options.videoCodec, maxSize, maxFps)
        //  3. router le display vers la surface d'entrée du codec
        //  4. drainer les buffers encodés -> videoSink
    }
}
