# Architecture de gninou

## Principe général

gninou reprend le modèle **client / serveur** de scrcpy :

1. Le **client** (Rust, sur le PC) pousse le binaire du **serveur** sur l'appareil via `adb push`.
2. Il lance le serveur avec `adb shell app_process ...` (aucune app installée de façon permanente).
3. Il ouvre un ou plusieurs tunnels (`adb forward` / `adb reverse`).
4. Le serveur capture l'écran, encode en H.264/H.265 et pousse le flux dans le tunnel.
5. Le client décode (FFmpeg) et affiche (SDL), tout en renvoyant les événements d'entrée.

## Composant serveur (`server/`, Kotlin)

Responsabilités :
- Capture écran via `SurfaceControl` / `MediaProjection`.
- Encodage matériel via `MediaCodec`.
- Capture audio (Android 11+).
- Injection d'événements via les APIs internes (`InputManager`).
- Sérialisation du protocole sur les sockets du tunnel ADB.

Particularité : le code est exécuté par `app_process`, pas packagé en APK classique. Le livrable est un `.dex` / `.jar`.

## Composant client (`client/`, Rust)

Responsabilités :
- Pilotage d'`adb` (push, forward/reverse, lancement du serveur).
- Lecture/écriture des sockets du tunnel.
- Démultiplexage du protocole (vidéo, audio, contrôle).
- Décodage via `ffmpeg` (crate `ffmpeg-next`).
- Rendu et fenêtrage via SDL3 (ou `wgpu` + `winit` à terme).
- Capture clavier/souris/manette et envoi au serveur.

## Protocole (à définir)

Un format de trame simple sera spécifié ici : en-têtes, codecs négociés, messages de contrôle (clé, souris, presse-papiers). Point de départ : s'inspirer du protocole scrcpy puis simplifier.

## Feuille de route (esquisse)

- [ ] Client : wrapper `adb` + ouverture de tunnel.
- [ ] Serveur : capture écran minimale + encodage `MediaCodec`.
- [ ] Transport : première trame vidéo de bout en bout.
- [ ] Client : décodage FFmpeg + affichage SDL.
- [ ] Contrôle : injection clavier/souris.
- [ ] Audio, presse-papiers, enregistrement, OTG.
