package io.karfi.gninou.server.control

/**
 * Réception des événements de contrôle et injection sur l'appareil.
 *
 * Lit les messages du canal contrôle (clavier, souris, molette,
 * presse-papiers) et les injecte via les APIs internes
 * (`InputManager.injectInputEvent`, `ClipboardManager`).
 */
class Controller {

    /** Boucle de lecture/injection des événements. */
    fun loop(/* controlSource: InputStream */) {
        // TODO:
        //  1. désérialiser les ControlMessage depuis controlSource
        //  2. construire les KeyEvent / MotionEvent correspondants
        //  3. InputManager.injectInputEvent(...)
        //  4. gérer le presse-papiers (set/get)
    }
}
