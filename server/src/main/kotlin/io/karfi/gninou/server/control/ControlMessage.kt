package io.karfi.gninou.server.control

import java.io.DataInputStream
import java.io.EOFException

/**
 * Messages de contrôle reçus du client, **miroir** de l'encodage Rust
 * (`client/src/protocol` : tag `u8` puis champs big-endian).
 *
 * La désérialisation ([parse]/[readFrom]) est **pure et testable**, sans
 * dépendance au framework Android.
 */
sealed interface ControlMessage {
    data class Key(val keycode: Int, val pressed: Boolean) : ControlMessage
    data class Pointer(val x: Int, val y: Int, val buttons: Int) : ControlMessage
    data class Scroll(val x: Int, val y: Int, val h: Int, val v: Int) : ControlMessage
    data class SetClipboard(val text: String) : ControlMessage

    companion object {
        const val TYPE_KEY = 0
        const val TYPE_POINTER = 1
        const val TYPE_SCROLL = 2
        const val TYPE_SET_CLIPBOARD = 3

        /** Lit un message depuis un flux (big-endian). `null` en fin de flux. */
        fun readFrom(input: DataInputStream): ControlMessage? {
            val tag = try {
                input.readUnsignedByte()
            } catch (_: EOFException) {
                return null
            }
            return when (tag) {
                TYPE_KEY -> Key(input.readInt(), input.readUnsignedByte() != 0)
                TYPE_POINTER -> Pointer(input.readInt(), input.readInt(), input.readInt())
                TYPE_SCROLL ->
                    Scroll(input.readInt(), input.readInt(), input.readInt(), input.readInt())
                TYPE_SET_CLIPBOARD -> {
                    val len = input.readInt()
                    val bytes = ByteArray(len)
                    input.readFully(bytes)
                    SetClipboard(String(bytes, Charsets.UTF_8))
                }
                else -> throw IllegalArgumentException("type de message inconnu : $tag")
            }
        }

        /** Parse un message unique depuis un tableau d'octets (pratique pour les tests). */
        fun parse(bytes: ByteArray): ControlMessage =
            readFrom(DataInputStream(bytes.inputStream()))
                ?: throw IllegalArgumentException("données vides")
    }
}
