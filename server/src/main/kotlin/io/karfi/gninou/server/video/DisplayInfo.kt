package io.karfi.gninou.server.video

import io.karfi.gninou.server.util.Log

/**
 * Informations sur l'affichage à capturer.
 *
 * ⚠️ La récupération de ces infos passe par des APIs **masquées** du framework
 * (`android.view.SurfaceControl`, `android.view.DisplayInfo`), dont la signature
 * varie selon la version d'Android. Ce code est compilé contre un `android.jar`
 * via réflexion et **ne peut pas être testé hors appareil**.
 *
 * @property width  largeur physique (px)
 * @property height hauteur physique (px)
 * @property rotation rotation (0, 1, 2, 3 = 0°, 90°, 180°, 270°)
 */
data class DisplayInfo(
    val width: Int,
    val height: Int,
    val rotation: Int,
) {
    /** Dimensions après application de la taille max (bord le plus long). */
    fun scaled(maxSize: Int): Pair<Int, Int> = computeScaledSize(width, height, maxSize)

    companion object {
        /**
         * Lit les infos de l'affichage principal.
         *
         * TODO (dépend de la version d'Android) :
         *  - API ≤ 28 : `SurfaceControl.getBuiltInDisplay(0)` + `getDisplayInfo`
         *  - API 29    : `getPhysicalDisplayIds()` + `getPhysicalDisplayToken` + `getDisplayInfo`
         *  - API ≥ 30 : `getPhysicalDisplayIds()` + `getDisplayInfo(token)`
         * On centralise ici la bascule par version via réflexion.
         */
        fun primary(): DisplayInfo {
            Log.w("DisplayInfo.primary() non implémenté (APIs masquées, à brancher par version)")
            // Valeur de repli pour que le reste du pipeline soit testable en dev.
            return DisplayInfo(width = 1080, height = 1920, rotation = 0)
        }

        /**
         * Calcule les dimensions en respectant `maxSize` (0 = pas de limite) et
         * en gardant le ratio ; le résultat est aligné sur 8 px (contrainte des
         * encodeurs vidéo). Logique **pure et testable**.
         */
        fun computeScaledSize(width: Int, height: Int, maxSize: Int): Pair<Int, Int> {
            if (maxSize <= 0) return align8(width) to align8(height)
            val major = maxOf(width, height)
            if (major <= maxSize) return align8(width) to align8(height)
            val scale = maxSize.toDouble() / major
            val w = align8((width * scale).toInt())
            val h = align8((height * scale).toInt())
            return w to h
        }

        private fun align8(v: Int): Int = (v / 8) * 8
    }
}
