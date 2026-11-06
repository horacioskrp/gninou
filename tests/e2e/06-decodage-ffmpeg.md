# Test e2e #6 — Décodage vidéo (FFmpeg)

Couvre la fonctionnalité #6 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Framing H.264 (Annex B → unités NAL, types, images clés) et statistiques du décodeur :

```bash
cd client && cargo test
```

Attendu : tests de `media::h264` et `media::video` verts. ✅ 24 tests au total.

## Décodage réel (feature `ffmpeg`)

Le décodage effectif est derrière une feature optionnelle (FFmpeg requis pour compiler) :

```bash
cd client && cargo build --features ffmpeg     # nécessite FFmpeg installé
```

> Non vérifié dans l'environnement de dev (FFmpeg absent) ; `FfmpegDecoder` est
> encore un squelette (TODO : ouverture du décodeur + échange paquets/frames).

## Procédure manuelle (bout en bout, appareil réel)

### Préconditions
- Client compilé avec `--features ffmpeg`, serveur opérationnel (#4/#5), tunnel OK.

### Étapes
1. Lancer le client, laisser le flux arriver quelques secondes.
2. Observer les stats de décodage (unités NAL, images clés).

### Résultat attendu
1. Les compteurs d'unités NAL et d'images clés augmentent.
2. Les frames sont décodées sans erreur (prélude à l'affichage #7).

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ framing testé (unit) ; décodage FFmpeg réel à implémenter + valider
