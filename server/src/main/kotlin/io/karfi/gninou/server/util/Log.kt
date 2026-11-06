package io.karfi.gninou.server.util

/**
 * Journalisation minimale. Sur l'appareil, la sortie standard remonte
 * dans la console adb du client.
 */
object Log {
    private const val TAG = "[gninou-server]"

    fun i(message: String) = println("$TAG I $message")
    fun w(message: String) = println("$TAG W $message")
    fun e(message: String, t: Throwable? = null) {
        println("$TAG E $message")
        t?.printStackTrace()
    }
}
