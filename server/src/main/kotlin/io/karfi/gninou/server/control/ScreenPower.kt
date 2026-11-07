package io.karfi.gninou.server.control

import io.karfi.gninou.server.util.Log

/**
 * Pilote l'alimentation de l'écran **physique** de l'appareil pendant le miroir
 * (fonctionnalité « écran éteint »).
 *
 * Applique le mode via `SurfaceControl.setDisplayPowerMode` (API **masquée**,
 * atteinte par réflexion), sur le display physique interne.
 *
 * ⚠️ Dépend du framework Android : **non testable hors appareil**. Le décodage
 * du message correspondant est, lui, testé ([ControlMessage.SetScreenPowerMode]).
 */
object ScreenPower {

    /** Applique un mode ([ControlMessage.POWER_MODE_OFF]/`POWER_MODE_NORMAL`). */
    fun setMode(mode: Int) {
        try {
            val surfaceControl = Class.forName("android.view.SurfaceControl")
            // getInternalDisplayToken() (API 29+) ; repli getBuiltInDisplay(0) plus ancien.
            val token = runCatching {
                surfaceControl.getMethod("getInternalDisplayToken").invoke(null)
            }.recoverCatching {
                surfaceControl
                    .getMethod("getBuiltInDisplay", Int::class.javaPrimitiveType)
                    .invoke(null, 0)
            }.getOrNull()

            if (token == null) {
                Log.e("écran : jeton d'affichage interne introuvable")
                return
            }

            surfaceControl
                .getMethod(
                    "setDisplayPowerMode",
                    Class.forName("android.os.IBinder"),
                    Int::class.javaPrimitiveType,
                )
                .invoke(null, token, mode)
        } catch (t: Throwable) {
            Log.e("écran : réglage de l'alimentation indisponible (API masquée)", t)
        }
    }
}
