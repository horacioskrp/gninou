package io.karfi.gninou.server.control

import kotlin.test.Test
import kotlin.test.assertContentEquals

/**
 * Vérifie que l'encodage appareil→client correspond exactement au décodage
 * attendu côté client (mêmes octets que le test Rust `decode_clipboard`).
 */
class DeviceMessageTest {

    @Test
    fun encode_clipboard() {
        val bytes = DeviceMessage.Clipboard("hi").encode()
        // tag(0) + len(2) big-endian + "hi"
        assertContentEquals(
            byteArrayOf(0, 0, 0, 0, 2, 'h'.code.toByte(), 'i'.code.toByte()),
            bytes,
        )
    }
}
