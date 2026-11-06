package io.karfi.gninou.server.audio

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class AudioCodecTest {

    @Test
    fun resolution_des_codecs() {
        assertEquals(AudioCodec.OPUS, AudioCodec.fromId("opus"))
        assertEquals(AudioCodec.AAC, AudioCodec.fromId("AAC"))
        assertEquals("audio/opus", AudioCodec.OPUS.mime)
    }

    @Test
    fun raw_non_compresse() {
        assertFalse(AudioCodec.RAW.isCompressed)
        assertNull(AudioCodec.RAW.mime)
        assertTrue(AudioCodec.OPUS.isCompressed)
    }

    @Test
    fun codec_inconnu_leve_une_erreur() {
        assertFailsWith<IllegalArgumentException> { AudioCodec.fromId("mp3") }
    }
}
