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
| 12 | Résolution / FPS / bitrate configurables | client+serveur | — | 4 | 🟨 | ✅ (parsing) |
| 13 | Rognage (crop) et rotation | client+serveur | — | 4 | 🟨 | ✅ (parsing) |
| 14 | Audio forwarding | client+serveur | Android 11+ | 5 | 🟨 | ✅ (codec/pipeline) |
| 15 | Presse-papiers bidirectionnel | client+serveur | — | 5 | 🟨 | ✅ (protocole) |
| 16 | Connexion TCP/IP | client | — | 6 | ✅ | ✅ (validation) |
| 17 | Appairage sans fil (Wi-Fi) | client | Android 11+ | 6 | ✅ | ✅ (validation) |
| 18 | Multi-appareils | client | — | 6 | ✅ | ✅ (sélection) |
| 19 | Enregistrement (mp4/mkv) | client | FFmpeg | 7 | 🟨 | ✅ (conteneur/compat) |
| 20 | Affichage écran éteint | client+serveur | — | 7 | 🟨 | ✅ (protocole) |
| 21 | Écran virtuel / nouvel affichage | client+serveur | Android 10+ | 7 | 🟨 | ✅ (options) |
| 22 | Clavier/souris physiques simulés (HID/UHID) | client+serveur | — | 8 | 🟨 | ✅ (descripteurs/protocole) |
| 23 | Manette (gamepad) | client+serveur | — | 8 | 🟨 | ✅ (descripteur/rapport) |
| 24 | Mode OTG (contrôle sans miroir) | client | — | 8 | 🟨 | ✅ (options) |
| 25 | Miroir caméra | client+serveur | Android 12+ | 9 | 🟨 | ✅ (options) |
| 26 | Webcam virtuelle (V4L2) | client | Linux | 9 | 🟨 | ✅ (périphérique) |
| 27 | Wi-Fi automatique (USB → sans-fil) | client | — | 10 | 🟨 | ✅ (extraction IP) |
| 28 | Découverte mDNS (débogage sans fil) | client | Android 11+ | 10 | 🟨 | ✅ (parsing mDNS) |
| 29 | Assistant d'appairage interactif | client | Android 11+ | 10 | 🟨 | ✅ (code/résolution d'adresse) |
| 30 | Découverte enrichie (`--list`) | client | — | 10 | 🟨 | ✅ (agrégation/formatage) |
| 31 | Reconnexion automatique Wi-Fi | client | — | 10 | 🟨 | ✅ (backoff/reprises) |
| 32 | Interface graphique — fenêtre + écran Appareils | client | feature `gui` | 11 | 🟨 | ✅ (view-model) |
| 33 | GUI — écran d'appairage Wi-Fi (assistant) | client | feature `gui` | 11 | 🟨 | ✅ (view-model) |
| 34 | GUI — écran Connexion Wi-Fi (USB→sans-fil, mDNS, reconnexion) | client | feature `gui` | 11 | 🟨 | ✅ (view-model) |
| 35 | GUI — écran Options du miroir (résolution/FPS/codec/débit/crop/rotation) | client | feature `gui` | 11 | 🟨 | ✅ (view-model) |

> Ce tableau est mis à jour à chaque fusion d'une `feature/*` : on coche l'état et
> la colonne « Testé » uniquement quand le test de la fonctionnalité est vert.
