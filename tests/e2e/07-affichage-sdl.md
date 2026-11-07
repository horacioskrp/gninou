# Test e2e #7 — Affichage SDL

Couvre la fonctionnalité #7 de [FEATURES.md](../../docs/FEATURES.md) et sert de
**test de clôture de la Phase 2** (miroir vidéo visible à l'écran).

## Tests unitaires (automatisés)

Logique de cadrage (letterbox, conservation du ratio) :

```bash
cd client && cargo test
```

Attendu : tests de `media::display::fit_rect` verts. ✅ 28 tests au total.

## Rendu réel (feature `display`)

```bash
cd client && cargo build --features display,ffmpeg   # SDL3 + FFmpeg requis
```

> Non vérifié dans l'environnement de dev (SDL3/FFmpeg absents) ; `SdlDisplay`
> est encore un squelette (TODO : fenêtre, texture IYUV, boucle de rendu).

## Procédure manuelle (bout en bout, appareil réel) — test de Phase 2

### Préconditions
- Client compilé `--features display,ffmpeg`, serveur opérationnel (#4/#5), tunnel OK.

### Étapes
1. Lancer le client face à un appareil branché.
2. Observer la fenêtre.

### Résultat attendu
1. Une fenêtre s'ouvre et affiche l'écran de l'appareil en temps réel.
2. Image centrée, ratio conservé (bandes si la fenêtre ne colle pas au ratio).
3. Fluidité correcte ; latence et FPS mesurés et notés.

### Résultat observé
- Date : _à remplir_
- Appareil(s) / latence / FPS : _à remplir_
- Statut : ⬜ en attente (SDL3 + FFmpeg + matériel) — **clôture Phase 2**
