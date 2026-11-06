package io.karfi.gninou.server.video

import android.graphics.Rect
import android.os.IBinder
import android.view.Surface
import android.view.SurfaceControl
import io.karfi.gninou.server.Options
import io.karfi.gninou.server.util.Log

/**
 * Capture de l'écran via un **display virtuel** (`SurfaceControl`).
 *
 * Principe (approche sans projection utilisateur, exécutée par `app_process`) :
 *   1. lire les dimensions de l'affichage principal ([DisplayInfo.primary]) ;
 *   2. créer un display virtuel et le projeter sur la [Surface] fournie
 *      (celle de l'encodeur vidéo, cf. #5) ;
 *   3. router le même `layerStack` que l'affichage réel → miroir.
 *
 * ⚠️ `SurfaceControl.createDisplay`, `setDisplaySurface`, `setDisplayProjection`
 * et `setDisplayLayerStack` sont des APIs **masquées**. Ce fichier est écrit pour
 * être compilé contre un `android.jar` de plateforme ; il **n'est pas testable
 * hors appareil** (voir tests/e2e/04).
 */
class ScreenCapture(private val options: Options) {

    private var display: IBinder? = null

    /**
     * Démarre le miroir vers [surface].
     * @return la taille réellement utilisée (après [Options.maxSize]).
     */
    fun start(surface: Surface): Pair<Int, Int> {
        val info = DisplayInfo.primary()
        val (w, h) = info.scaled(options.maxSize)
        Log.i("capture écran ${info.width}x${info.height} -> ${w}x$h (rot ${info.rotation})")

        val secure = false
        val d = SurfaceControl.createDisplay("gninou", secure)
        display = d

        SurfaceControl.openTransaction()
        try {
            SurfaceControl.setDisplaySurface(d, surface)
            // Source = écran réel ; destination = surface de l'encodeur.
            val source = Rect(0, 0, info.width, info.height)
            val dest = Rect(0, 0, w, h)
            SurfaceControl.setDisplayProjection(d, info.rotation, source, dest)
            SurfaceControl.setDisplayLayerStack(d, 0) // layerStack de l'affichage principal
        } finally {
            SurfaceControl.closeTransaction()
        }
        return w to h
    }

    /** Détruit le display virtuel. */
    fun release() {
        display?.let { SurfaceControl.destroyDisplay(it) }
        display = null
        Log.i("capture écran arrêtée")
    }
}
