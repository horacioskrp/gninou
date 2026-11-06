# Fonctionnalités de gninou

Liste de référence des fonctionnalités gérées par le projet, avec leur état.
Chaque fonctionnalité est développée sur sa propre branche `feature/*`, **testée**,
puis fusionnée dans `develop` (voir [GITFLOW.md](GITFLOW.md)).

## Légende des états

- ⬜ **À faire** — non commencé
- 🟨 **En cours** — en développement sur une branche `feature/*`
- ✅ **Fait** — implémenté, testé et fusionné dans `develop`

## Tableau de suivi

| # | Fonctionnalité | Composant | Requiert | Phase | État | Testé |
|---|----------------|-----------|----------|-------|------|-------|
| 1 | Détection d'appareil + wrapper adb | client | adb | 1 | ✅ | ✅ (unit) |
| 2 | Push + lancement du serveur (app_process) | client+serveur | — | 1 | ✅ | ✅ (unit) |
| 3 | Tunnel adb (reverse/forward) + sockets | client+serveur | — | 1 | ✅ | ✅ (unit) |
| 4 | Capture écran (SurfaceControl) | serveur | Android 5.0+ | 2 | 🟨 | ⬜ (SDK) |
| 5 | Encodage vidéo (MediaCodec H.264) | serveur | — | 2 | 🟨 | ⬜ (SDK) |
| 6 | Décodage vidéo (FFmpeg) | client | FFmpeg | 2 | 🟨 | ✅ (framing) |
| 7 | Affichage fenêtre (SDL) | client | SDL | 2 | 🟨 | ✅ (cadrage) |
| 8 | Contrôle souris (clic, déplacement, molette) | client+serveur | — | 3 | 🟨 | ✅ (logique) |
| 9 | Contrôle clavier | client+serveur | — | 3 | 🟨 | ✅ (logique) |
| 10 | Injection d'événements (InputManager) | serveur | — | 3 | 🟨 | ✅ (désérialisation) |
| 11 | Codec vidéo configurable (H.265/AV1) | client+serveur | — | 4 | 🟨 | ✅ (logique) |
| 12 | Résolution / FPS / bitrate configurables | client+serveur | — | 4 | ⬜ | ⬜ |
| 13 | Rognage (crop) et rotation | client+serveur | — | 4 | ⬜ | ⬜ |
| 14 | Audio forwarding | client+serveur | Android 11+ | 5 | ⬜ | ⬜ |
| 15 | Presse-papiers bidirectionnel | client+serveur | — | 5 | ⬜ | ⬜ |
| 16 | Connexion TCP/IP | client | — | 6 | ⬜ | ⬜ |
| 17 | Appairage sans fil (Wi-Fi) | client | Android 11+ | 6 | ⬜ | ⬜ |
| 18 | Multi-appareils | client | — | 6 | ⬜ | ⬜ |
| 19 | Enregistrement (mp4/mkv) | client | FFmpeg | 7 | ⬜ | ⬜ |
| 20 | Affichage écran éteint | client+serveur | — | 7 | ⬜ | ⬜ |
| 21 | Écran virtuel / nouvel affichage | client+serveur | Android 10+ | 7 | ⬜ | ⬜ |
| 22 | Clavier/souris physiques simulés (HID/UHID) | client+serveur | — | 8 | ⬜ | ⬜ |
| 23 | Manette (gamepad) | client+serveur | — | 8 | ⬜ | ⬜ |
| 24 | Mode OTG (contrôle sans miroir) | client | — | 8 | ⬜ | ⬜ |
| 25 | Miroir caméra | client+serveur | Android 12+ | 9 | ⬜ | ⬜ |
| 26 | Webcam virtuelle (V4L2) | client | Linux | 9 | ⬜ | ⬜ |

> Ce tableau est mis à jour à chaque fusion d'une `feature/*` : on coche l'état et
> la colonne « Testé » uniquement quand le test de la fonctionnalité est vert.
