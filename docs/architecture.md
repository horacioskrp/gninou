# Architecture de gninou

## Principe général

gninou reprend le modèle **client / serveur** de scrcpy :

1. Le **client** (Rust, sur le PC) pousse le binaire du **serveur** sur l'appareil via `adb push`.
2. Il lance le serveur avec `adb shell app_process ...` (rien n'est installé de façon permanente).
3. Il ouvre un tunnel (`adb forward` / `adb reverse`) — que la liaison soit **USB, TCP/IP ou Wi-Fi**.
4. Le serveur capture écran + audio, encode, et pousse les flux dans le tunnel.
5. Le client décode (FFmpeg) et affiche (SDL), tout en renvoyant les événements d'entrée.

## Canaux logiques

Le tunnel transporte trois flux :

| Canal | Sens | Contenu |
|-------|------|---------|
| **vidéo** | device → client | trames H.264 / H.265 |
| **audio** | device → client | trames Opus / AAC (Android 11+) |
| **contrôle** | client → device | clavier, souris, molette, presse-papiers |

## Modes de connexion (`client/device/connection.rs`)

- **USB** : câble, appareil ciblé par son numéro de série.
- **TCP/IP** : `adb connect ip:port`.
- **Wi-Fi** : appairage sans fil Android 11+ (`adb pair`), puis même transport que TCP/IP.

## Composant serveur (`server/`, Kotlin)

| Package | Fichier | Responsabilité |
|---------|---------|----------------|
| `server` | `Server.kt` / `Options.kt` | point d'entrée `app_process`, orchestration, options |
| `net` | `Connection.kt` | sockets du tunnel (`localabstract`) |
| `video` | `ScreenEncoder.kt` | capture `SurfaceControl` → `MediaCodec` |
| `audio` | `AudioEncoder.kt` | capture `AudioRecord` (REMOTE_SUBMIX) → encodage |
| `control` | `Controller.kt` | désérialisation + injection via `InputManager` |
| `util` | `Log.kt` | journalisation |

Particularité : exécuté par `app_process`, pas packagé en APK. Livrable = `.dex` / `.jar`.

## Composant client (`client/`, Rust)

| Module | Responsabilité |
|--------|----------------|
| `cli` | parsing des arguments |
| `device::adb` | push du serveur, ouverture du tunnel |
| `device::connection` | choix USB / TCP-IP / Wi-Fi |
| `protocol` | définition des trames (`MediaHeader`, `ControlMessage`) |
| `media::video` | décodage FFmpeg + rendu SDL |
| `media::audio` | décodage + lecture son |
| `control` | capture clavier/souris → `ControlMessage` |

## Empaquetage : du monorepo à l'app unique

Bien que le dépôt ait deux composants, le livrable est **un seul exécutable desktop**.
Le serveur est embarqué dans le client :

| Étape | Outil | Résultat |
|-------|-------|----------|
| 1. Compiler le serveur | Gradle (`gradle jar`) | `server/build/libs/gninou-server-*.jar` |
| 2. Copier dans les assets | `build.rs` / scripts | `client/assets/gninou-server.jar` |
| 3. Compiler le client | Cargo (`include_bytes!`) | `gninou` (le jar est *dans* le binaire) |

Les scripts `scripts/build.sh` / `build.ps1` enchaînent ces étapes. La variable
`GNINOU_BUILD_SERVER=1` dit à `build.rs` de (re)compiler le serveur ; sinon un
placeholder vide est utilisé pour que le client compile seul.

Au **runtime**, `client/device/server_bin.rs` extrait le jar embarqué dans un fichier
temporaire, le pousse (`adb push /data/local/tmp/`), puis l'exécute via `app_process`.
Rien n'est installé durablement sur l'appareil.

## Protocole (à figer)

Format de trame : en-tête (`MediaHeader` : pts, taille, keyframe) + charge utile pour vidéo/audio ; messages typés (`ControlMessage`) pour le contrôle. Point de départ : s'inspirer de scrcpy puis simplifier.

## Feuille de route

- [ ] Client : wrapper `adb` + ouverture de tunnel (USB d'abord).
- [ ] Serveur : capture écran minimale + encodage `MediaCodec`.
- [ ] Transport : première trame vidéo de bout en bout.
- [ ] Client : décodage FFmpeg + affichage SDL.
- [ ] Contrôle : injection clavier/souris.
- [ ] Audio (Android 11+).
- [ ] Connexion TCP/IP puis appairage Wi-Fi.
- [ ] Presse-papiers bidirectionnel, enregistrement, OTG.
