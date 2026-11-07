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
    data class SetScreenPowerMode(val mode: Int) : ControlMessage
    data class UhidCreate(val id: Int, val reportDesc: ByteArray) : ControlMessage
    data class UhidInput(val id: Int, val data: ByteArray) : ControlMessage
    data class UhidDestroy(val id: Int) : ControlMessage

    companion object {
        const val TYPE_KEY = 0
        const val TYPE_POINTER = 1
        const val TYPE_SCROLL = 2
        const val TYPE_SET_CLIPBOARD = 3
        const val TYPE_SET_SCREEN_POWER_MODE = 4
        const val TYPE_UHID_CREATE = 5
        const val TYPE_UHID_INPUT = 6
        const val TYPE_UHID_DESTROY = 7

        /** Modes d'alimentation de l'écran (valeurs SurfaceControl). */
        const val POWER_MODE_OFF = 0
        const val POWER_MODE_NORMAL = 2

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
                TYPE_SET_SCREEN_POWER_MODE -> SetScreenPowerMode(input.readUnsignedByte())
                TYPE_UHID_CREATE -> {
                    val id = input.readUnsignedShort()
                    val len = input.readUnsignedShort()
                    val desc = ByteArray(len)
                    input.readFully(desc)
                    UhidCreate(id, desc)
                }
                TYPE_UHID_INPUT -> {
                    val id = input.readUnsignedShort()
                    val len = input.readUnsignedShort()
                    val data = ByteArray(len)
                    input.readFully(data)
                    UhidInput(id, data)
                }
                TYPE_UHID_DESTROY -> UhidDestroy(input.readUnsignedShort())
                else -> throw IllegalArgumentException("type de message inconnu : $tag")
            }
        }

        /** Parse un message unique depuis un tableau d'octets (pratique pour les tests). */
        fun parse(bytes: ByteArray): ControlMessage =
            readFrom(DataInputStream(bytes.inputStream()))
                ?: throw IllegalArgumentException("données vides")
    }
}
