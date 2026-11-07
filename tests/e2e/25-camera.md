# Test e2e #25 — Miroir caméra (Android 12+)

Couvre la fonctionnalité #25 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Choix de la source, orientation, taille/fps caméra (client) et options serveur :

```bash
cd client && cargo test
```

Attendu : `media::camera::*` verts (source display/camera, facing, taille, args).
✅ 112 tests client.

> Tests serveur (`OptionsTest.source_camera` / `camera_par_id`) : clés
> `video_source`, `camera_*`. Exécution Gradle une fois le wrapper généré.

## Procédure manuelle (appareil réel)

> La capture réelle (Camera2 / surface d'entrée de l'encodeur) est une API
> dépendante d'Android (non testable hors appareil).

### Étapes
1. `gninou --video-source camera` → caméra par défaut.
2. `gninou --video-source camera --camera-facing front` → caméra frontale.
3. `gninou --video-source camera --camera-id 2 --camera-size 1280x720 --camera-fps 30`.
4. `gninou --video-source camera --camera-facing oops` → erreur d'orientation.

### Résultat attendu
1–3. La fenêtre gninou affiche le flux de la caméra choisie, à la taille/fps
   demandés ; l'écran de l'appareil n'est pas reflété.
4. Erreur de validation **avant** tout déploiement.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ options source/caméra testées (client + serveur) ; capture caméra
  réelle à valider sur matériel Android 12+
