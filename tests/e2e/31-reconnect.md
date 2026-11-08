# Test e2e #31 — Reconnexion automatique Wi-Fi

Couvre la fonctionnalité #31 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Politique de backoff (délais exponentiels plafonnés, saturants ; borne du
nombre d'essais) et enchaînement du driver de reprise (succès immédiat, succès
après échecs, abandon après le maximum), testé avec un espion de délai (sans
attente réelle).

```bash
cd client && cargo test
```

Attendu : `device::reconnect::*` verts (délai exponentiel, délai plafonné, au
moins un essai, succès du premier coup sans attente, succès après deux échecs
avec délais 10/20 ms, abandon après max_attempts). ✅ 142 tests client.

## Procédure manuelle (appareil réel, Wi-Fi)

### Préconditions
- Appareil joignable en réseau (Wi-Fi/TCP-IP), p. ex. après `--wifi` (#27),
  `--wifi-discover` (#28) ou `--pair-assistant` (#29).

### Étapes
1. `gninou --tcpip 192.168.1.42:5555 --reconnect`
   → si `adb connect` échoue (appareil momentanément injoignable), gninou
     retente automatiquement : délais 0,5 s, 1 s, 2 s… plafonnés à 10 s,
     jusqu'à 10 tentatives par défaut.
2. Couper puis rétablir le Wi-Fi de l'appareil pendant les tentatives
   → la connexion est reprise dès que l'appareil redevient joignable
     (« reconnexion réussie (tentative N) »).
3. `gninou --tcpip <ip injoignable> --reconnect --reconnect-attempts 3`
   → abandon clair après 3 tentatives
     (« reconnexion abandonnée après 3 tentative(s) »).
4. Sans `--reconnect` : un seul essai, échec immédiat si injoignable.

### Résultat attendu
1. Reprises espacées par un backoff exponentiel plafonné.
2. Reconnexion effective au retour du réseau.
3. Abandon borné et message clair.
4. Comportement historique inchangé sans l'option.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ politique de backoff + enchaînement du driver testés (unit) ;
  reprise réseau réelle (`adb connect`) à valider sur matériel
