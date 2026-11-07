package io.karfi.gninou.server.control

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

/**
 * Vérifie que la désérialisation serveur correspond exactement à l'encodage
 * du client (mêmes séquences d'octets que les tests Rust de `protocol`).
 */
class ControlMessageTest {

    @Test
    fun parse_pointer() {
        val bytes = byteArrayOf(1, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1)
        assertEquals(ControlMessage.Pointer(1, 2, 1), ControlMessage.parse(bytes))
    }

    @Test
    fun parse_key() {
        val bytes = byteArrayOf(0, 0, 0, 0, 0x41, 1)
        assertEquals(ControlMessage.Key(0x41, true), ControlMessage.parse(bytes))
    }

    @Test
    fun parse_scroll() {
        // x=0,y=0,h=0,v=-1  -> dernier i32 = FF FF FF FF
        val bytes = byteArrayOf(
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1,
        )
        assertEquals(ControlMessage.Scroll(0, 0, 0, -1), ControlMessage.parse(bytes))
    }

    @Test
    fun parse_clipboard() {
        val bytes = byteArrayOf(3, 0, 0, 0, 2, 'h'.code.toByte(), 'i'.code.toByte())
        assertEquals(ControlMessage.SetClipboard("hi"), ControlMessage.parse(bytes))
    }

    @Test
    fun parse_screen_power_mode() {
        // tag=4, mode=0 (OFF)
        assertEquals(
            ControlMessage.SetScreenPowerMode(ControlMessage.POWER_MODE_OFF),
            ControlMessage.parse(byteArrayOf(4, 0)),
        )
        // tag=4, mode=2 (NORMAL)
        assertEquals(
            ControlMessage.SetScreenPowerMode(ControlMessage.POWER_MODE_NORMAL),
            ControlMessage.parse(byteArrayOf(4, 2)),
        )
    }

    @Test
    fun type_inconnu_leve_une_erreur() {
        assertFailsWith<IllegalArgumentException> { ControlMessage.parse(byteArrayOf(99, 0)) }
    }
}
