# Test e2e #26 — Webcam virtuelle (V4L2, Linux)

Couvre la fonctionnalité #26 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Validation du périphérique `/dev/videoN`, garde de plateforme et compteurs :

```bash
cd client && cargo test
```

Attendu : `media::v4l2::*` verts (chemins valides/invalides, refus hors Linux).
✅ 116 tests client.

## Procédure manuelle (Linux, appareil réel)

> Nécessite le module `v4l2loopback` et un build avec la feature `v4l2`.

### Préconditions
```bash
sudo modprobe v4l2loopback video_nr=2 card_label="gninou" exclusive_caps=1
```

### Étapes
1. `gninou --v4l2-sink /dev/video2`
   → le flux de l'appareil est écrit dans `/dev/video2`.
2. Ouvrir une appli de visioconférence / OBS et choisir la caméra « gninou ».
3. `gninou --v4l2-sink /dev/videoX` → erreur (périphérique invalide).
4. Sous Windows/macOS : `gninou --v4l2-sink /dev/video2` → avertissement
   « disponible uniquement sous Linux ».

### Résultat attendu
1. `/dev/video2` diffuse l'écran/caméra de l'appareil.
2. Les autres applis voient une webcam « gninou » utilisable.
3. Erreur de validation **avant** tout déploiement.
4. Avertissement clair hors Linux (pas de sortie V4L2).

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ validation périphérique + garde plateforme testées ; écriture V4L2
  réelle à valider sur Linux (feature `v4l2` + `v4l2loopback`)
