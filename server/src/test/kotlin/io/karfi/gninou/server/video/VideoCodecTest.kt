package io.karfi.gninou.server.video

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** Tests de la logique pure de sélection de codec et de débit. */
class VideoCodecTest {

    @Test
    fun resolution_des_codecs_connus() {
        assertEquals(VideoCodec.H264, VideoCodec.fromId("h264"))
        assertEquals(VideoCodec.H265, VideoCodec.fromId("H265"))
        assertEquals("video/av01", VideoCodec.fromId("av1").mime)
    }

    @Test
    fun codec_inconnu_leve_une_erreur() {
        assertFailsWith<IllegalArgumentException> { VideoCodec.fromId("vp9") }
    }

    @Test
    fun bitrate_borne_bas() {
        // Petite définition -> plancher 2 Mbps.
        assertEquals(2_000_000, VideoCodec.defaultBitrate(320, 240, 30))
    }

    @Test
    fun bitrate_borne_haut() {
        // Très grande définition -> plafond 40 Mbps.
        assertEquals(40_000_000, VideoCodec.defaultBitrate(3840, 2160, 60))
    }

    @Test
    fun bitrate_intermediaire_dans_les_bornes() {
        val b = VideoCodec.defaultBitrate(1080, 1920, 30)
        assertTrue(b in 2_000_000..40_000_000)
    }
}
