package io.karfi.gninou.server.control

import java.io.ByteArrayOutputStream
import java.io.DataOutputStream

/**
 * Messages envoyés par l'appareil vers le client (canal appareil→PC),
 * **miroir** du décodage Rust (`client/src/protocol` : `DeviceMessage`).
 *
 * Sérialisation **pure et testable**, sans dépendance au framework Android.
 */
sealed interface DeviceMessage {
    data class Clipboard(val text: String) : DeviceMessage

    fun encode(): ByteArray {
        val out = ByteArrayOutputStream()
        DataOutputStream(out).use { d ->
            when (this) {
                is Clipboard -> {
                    d.writeByte(TYPE_CLIPBOARD)
                    val bytes = text.toByteArray(Charsets.UTF_8)
                    d.writeInt(bytes.size)
                    d.write(bytes)
                }
            }
        }
        return out.toByteArray()
    }

    companion object {
        const val TYPE_CLIPBOARD = 0
    }
}
