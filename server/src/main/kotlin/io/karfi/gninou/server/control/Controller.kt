package io.karfi.gninou.server.control

import android.view.MotionEvent
import io.karfi.gninou.server.util.Log
import java.io.DataInputStream
import java.io.InputStream

/**
 * Réception des événements de contrôle et injection sur l'appareil.
 *
 * Lit les [ControlMessage] du canal contrôle (désérialisation testée), les
 * reconstruit en `KeyEvent`/`MotionEvent` et les injecte via [EventInjector].
 */
class Controller(private val injector: EventInjector = EventInjector()) {

    @Volatile
    private var running = false

    /** Boucle de lecture/injection, jusqu'à la fermeture du flux ou [stop]. */
    fun loop(controlSource: InputStream) {
        val input = DataInputStream(controlSource)
        running = true
        while (running) {
            val msg = ControlMessage.readFrom(input) ?: break
            dispatch(msg)
        }
        Log.i("boucle de contrôle terminée")
    }

    fun stop() {
        running = false
    }

    private fun dispatch(msg: ControlMessage) {
        when (msg) {
            is ControlMessage.Key -> injector.key(msg.keycode, msg.pressed)
            is ControlMessage.Pointer -> {
                // Bouton pressé -> DOWN, sinon UP ; MOVE géré par positions successives.
                val action = if (msg.buttons != 0) MotionEvent.ACTION_DOWN else MotionEvent.ACTION_UP
                injector.touch(msg.x, msg.y, action)
            }
            is ControlMessage.Scroll -> {
                // TODO: MotionEvent ACTION_SCROLL avec AXIS_VSCROLL/HSCROLL.
                Log.i("scroll (${msg.h},${msg.v}) @ ${msg.x},${msg.y} — à implémenter")
            }
            is ControlMessage.SetClipboard -> {
                // TODO: ClipboardManager.setPrimaryClip via réflexion.
                Log.i("set-clipboard (${msg.text.length} caractères) — à implémenter")
            }
            is ControlMessage.SetScreenPowerMode -> {
                Log.i("écran -> mode d'alimentation ${msg.mode}")
                ScreenPower.setMode(msg.mode)
            }
        }
    }
}
