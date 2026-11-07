package io.karfi.gninou.server.control

import kotlin.test.Test
import kotlin.test.assertContentEquals
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
    fun parse_uhid_create() {
        // tag=5, id=1 (00 01), len=2 (00 02), desc=AB CD
        val bytes = byteArrayOf(5, 0, 1, 0, 2, 0xAB.toByte(), 0xCD.toByte())
        val msg = ControlMessage.parse(bytes) as ControlMessage.UhidCreate
        assertEquals(1, msg.id)
        assertContentEquals(byteArrayOf(0xAB.toByte(), 0xCD.toByte()), msg.reportDesc)
    }

    @Test
    fun parse_uhid_input() {
        // tag=6, id=2 (00 02), len=8 (00 08), 8 octets de rapport clavier
        val bytes = byteArrayOf(6, 0, 2, 0, 8, 0, 0, 0x04, 0, 0, 0, 0, 0)
        val msg = ControlMessage.parse(bytes) as ControlMessage.UhidInput
        assertEquals(2, msg.id)
        assertEquals(8, msg.data.size)
        assertEquals(0x04, msg.data[2].toInt())
    }

    @Test
    fun parse_uhid_destroy() {
        val msg = ControlMessage.parse(byteArrayOf(7, 0, 2)) as ControlMessage.UhidDestroy
        assertEquals(2, msg.id)
    }

    @Test
    fun type_inconnu_leve_une_erreur() {
        assertFailsWith<IllegalArgumentException> { ControlMessage.parse(byteArrayOf(99, 0)) }
    }
}
