package io.karfi.gninou.server.control

import android.os.SystemClock
import android.view.InputEvent
import android.view.KeyEvent
import android.view.MotionEvent
import io.karfi.gninou.server.util.Log

/**
 * Injecte des événements d'entrée sur l'appareil.
 *
 * Utilise `InputManager.injectInputEvent` (API **masquée**, atteinte par
 * réflexion), comme le font les outils de contrôle exécutés via `app_process`.
 *
 * ⚠️ Dépend du framework Android : **non testable hors appareil**. La
 * désérialisation des messages est, elle, testée ([ControlMessage]).
 */
class EventInjector {

    private val injectInputEvent: (InputEvent) -> Boolean = buildInjector()

    /** Injecte une touche (appui ou relâche). */
    fun key(keycode: Int, down: Boolean) {
        val now = SystemClock.uptimeMillis()
        val action = if (down) KeyEvent.ACTION_DOWN else KeyEvent.ACTION_UP
        val event = KeyEvent(now, now, action, keycode, 0)
        injectInputEvent(event)
    }

    /** Injecte un événement tactile (down/move/up) aux coordonnées données. */
    fun touch(x: Int, y: Int, action: Int) {
        val now = SystemClock.uptimeMillis()
        val event = MotionEvent.obtain(
            now, now, action, x.toFloat(), y.toFloat(), 0,
        )
        try {
            injectInputEvent(event)
        } finally {
            event.recycle()
        }
    }

    private fun buildInjector(): (InputEvent) -> Boolean {
        return try {
            // getService("input") -> IInputManager ; injectInputEvent(event, mode)
            val serviceManager = Class.forName("android.os.ServiceManager")
            val binder = serviceManager
                .getMethod("getService", String::class.java)
                .invoke(null, "input")
            val stub = Class.forName("android.hardware.input.IInputManager\$Stub")
            val manager = stub
                .getMethod("asInterface", Class.forName("android.os.IBinder"))
                .invoke(null, binder)
            val inject = manager.javaClass.getMethod(
                "injectInputEvent",
                InputEvent::class.java,
                Int::class.javaPrimitiveType,
            )
            val mode = 0 // INJECT_INPUT_EVENT_MODE_ASYNC
            ({ event: InputEvent -> inject.invoke(manager, event, mode) as Boolean })
        } catch (t: Throwable) {
            Log.e("injection indisponible (API masquée introuvable)", t)
            ({ _: InputEvent -> false })
        }
    }
}
