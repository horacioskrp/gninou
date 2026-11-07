package io.karfi.gninou.server.control

import io.karfi.gninou.server.util.Log

/**
 * Gère les périphériques HID virtuels (clavier/souris simulés) via `/dev/uhid`.
 *
 * Chaque périphérique est créé à partir d'un descripteur de rapport fourni par
 * le client, puis alimenté par des rapports d'entrée. L'appareil les voit comme
 * de vrais périphériques USB HID.
 *
 * ⚠️ Dépend de l'accès noyau à `/dev/uhid` (ioctl/protocole UHID) : **non
 * testable hors appareil**. La (dé)sérialisation des messages est, elle, testée
 * ([ControlMessage]).
 */
class UhidManager {

    private val open = HashSet<Int>()

    /** Crée le périphérique HID `id` à partir de son descripteur de rapport. */
    fun create(id: Int, reportDesc: ByteArray) {
        if (!open.add(id)) {
            Log.i("UHID #$id déjà ouvert")
            return
        }
        // TODO (appareil) : ouvrir /dev/uhid, envoyer UHID_CREATE2 avec reportDesc.
        Log.i("UHID #$id créé (${reportDesc.size} octets) — accès noyau à brancher")
    }

    /** Transmet un rapport d'entrée au périphérique `id`. */
    fun input(id: Int, data: ByteArray) {
        if (id !in open) {
            Log.e("UHID #$id : rapport reçu pour un périphérique inexistant")
            return
        }
        // TODO (appareil) : écrire un paquet UHID_INPUT2 sur /dev/uhid.
    }

    /** Détruit le périphérique HID `id`. */
    fun destroy(id: Int) {
        if (!open.remove(id)) return
        // TODO (appareil) : envoyer UHID_DESTROY puis fermer le descripteur.
        Log.i("UHID #$id détruit")
    }

    /** Ferme tous les périphériques encore ouverts (fin de session). */
    fun closeAll() {
        for (id in open.toList()) destroy(id)
    }
}
