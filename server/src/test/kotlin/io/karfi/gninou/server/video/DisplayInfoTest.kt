package io.karfi.gninou.server.video

import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * Tests de la logique pure de dimensionnement (sans appareil ni SDK runtime).
 * Exécutables avec `./gradlew test` une fois l'environnement Gradle + android.jar
 * configuré.
 */
class DisplayInfoTest {

    @Test
    fun pas_de_limite_aligne_sur_8() {
        assertEquals(1080 to 1920, DisplayInfo.computeScaledSize(1080, 1920, 0))
    }

    @Test
    fun plus_petit_que_max_inchange() {
        assertEquals(720 to 1280, DisplayInfo.computeScaledSize(720, 1280, 2000))
    }

    @Test
    fun reduit_en_gardant_le_ratio() {
        // 1080x1920, max 1024 sur le bord le plus long -> hauteur 1024, largeur ~576
        val (w, h) = DisplayInfo.computeScaledSize(1080, 1920, 1024)
        assertEquals(1024, h)
        assertEquals(576, w) // 1080*1024/1920 = 576, déjà multiple de 8
    }

    @Test
    fun resultat_toujours_multiple_de_8() {
        val (w, h) = DisplayInfo.computeScaledSize(1000, 1777, 800)
        assertEquals(0, w % 8)
        assertEquals(0, h % 8)
    }
}
