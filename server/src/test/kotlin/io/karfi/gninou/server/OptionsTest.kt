package io.karfi.gninou.server

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** Vérifie le parsing des options clé=valeur du serveur. */
class OptionsTest {

    @Test
    fun defauts() {
        val o = Options.parse(arrayOf())
        assertEquals(0, o.displayId)
        assertFalse(o.newDisplay)
        assertEquals("", o.newDisplaySize)
    }

    @Test
    fun display_id() {
        val o = Options.parse(arrayOf("display_id=2"))
        assertEquals(2, o.displayId)
        assertFalse(o.newDisplay)
    }

    @Test
    fun new_display_avec_taille() {
        val o = Options.parse(arrayOf("new_display=1920x1080/420"))
        assertTrue(o.newDisplay)
        assertEquals("1920x1080/420", o.newDisplaySize)
    }

    @Test
    fun new_display_defauts() {
        // `new_display=` (valeur vide) = création avec les défauts de l'appareil.
        val o = Options.parse(arrayOf("new_display="))
        assertTrue(o.newDisplay)
        assertEquals("", o.newDisplaySize)
    }

    @Test
    fun source_camera() {
        val o = Options.parse(
            arrayOf("video_source=camera", "camera_facing=front", "camera_size=1280x720", "camera_fps=30"),
        )
        assertEquals("camera", o.videoSource)
        assertEquals("front", o.cameraFacing)
        assertEquals("1280x720", o.cameraSize)
        assertEquals(30, o.cameraFps)
    }

    @Test
    fun camera_par_id() {
        val o = Options.parse(arrayOf("video_source=camera", "camera_id=2"))
        assertEquals("camera", o.videoSource)
        assertEquals("2", o.cameraId)
    }

    @Test
    fun combinaison_avec_autres_options() {
        val o = Options.parse(arrayOf("max_fps=30", "new_display=800x600", "audio=false"))
        assertEquals(30, o.maxFps)
        assertFalse(o.audio)
        assertTrue(o.newDisplay)
        assertEquals("800x600", o.newDisplaySize)
    }
}
