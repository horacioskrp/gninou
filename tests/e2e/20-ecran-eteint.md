# Test e2e #20 — Affichage écran éteint

Couvre la fonctionnalité #20 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Message de contrôle `SetScreenPowerMode` — **même format d'octets** côté client
(encodage Rust) et serveur (décodage Kotlin) :

```bash
cd client && cargo test
```

Attendu : `protocol::encode_screen_power_off_et_normal` vert
(`[tag=4][mode]`, OFF=0 / NORMAL=2). ✅ 86 tests client.

> Tests serveur (`ControlMessageTest.parse_screen_power_mode`) : mêmes séquences
> `[4,0]` / `[4,2]`. Exécution Gradle à faire une fois le wrapper généré (Phase 0).

## Procédure manuelle (appareil réel)

> L'émission réelle sur le canal contrôle dépend du pipeline de contrôle (#8–#10)
> et l'application via `SurfaceControl` est une API masquée (non testable hors
> appareil).

### Étapes
1. `gninou --turn-screen-off`
   → le miroir démarre ; l'écran physique de l'appareil s'éteint.
2. Pendant la session, le miroir reste fluide écran éteint ; le contrôle
   (souris/clavier) fonctionne toujours.
3. Fermer gninou → l'écran de l'appareil se rallume (mode NORMAL restauré).

### Résultat attendu
1. Écran de l'appareil éteint, miroir toujours actif côté PC.
2. Aucune interruption du flux vidéo/contrôle.
3. Écran restauré en fin de session.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ message de contrôle (format) testé des deux côtés ; extinction réelle
  à valider sur matériel (dépend du pipeline contrôle + API SurfaceControl)
