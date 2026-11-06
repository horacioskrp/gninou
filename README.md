# gninou

> Affiche les appareils Android (**vidéo + audio**) connectés en **USB, TCP/IP ou Wi-Fi**, et permet de les **contrôler au clavier et à la souris** de l'ordinateur. **Sans root** et **sans app installée** sur l'appareil.

Réécriture moderne inspirée de [scrcpy](https://github.com/Genymobile/scrcpy).

## Deux composants

| Composant | Dossier | Langage | Rôle |
|-----------|---------|---------|------|
| **Client desktop** | [`client/`](client/) | **Rust** | Pilote adb, reçoit vidéo/audio, décode (FFmpeg) + affiche (SDL), renvoie clavier/souris. |
| **Serveur device** | [`server/`](server/) | **Kotlin** | Tourne via `app_process`, capture écran + audio, injecte les entrées. |

Les deux dialoguent via **ADB** (USB, TCP/IP ou Wi-Fi), sur trois canaux logiques : **vidéo**, **audio**, **contrôle**.

## Arborescence

```
gninou/
├── client/                         # CLIENT DESKTOP (Rust) = l'app finale
│   ├── Cargo.toml
│   ├── build.rs                    # gradle -> jar -> assets/ (embarquement)
│   ├── assets/                     # gninou-server.jar embarqué (build)
│   └── src/
│       ├── main.rs                 # orchestration
│       ├── cli.rs                  # arguments
│       ├── protocol/               # trames vidéo/audio/contrôle
│       ├── device/
│       │   ├── adb.rs              # ensure / push / tunnel
│       │   ├── connection.rs       # USB · TCP-IP · Wi-Fi
│       │   └── server_bin.rs       # extrait + push + lance le serveur embarqué
│       ├── media/
│       │   ├── video.rs            # décodage + affichage
│       │   └── audio.rs            # lecture audio
│       └── control/                # clavier/souris -> protocole
│
├── server/                         # SERVEUR DEVICE (Kotlin)
│   ├── build.gradle.kts
│   └── src/main/kotlin/io/karfi/gninou/server/
│       ├── Server.kt               # point d'entrée (app_process)
│       ├── Options.kt
│       ├── net/Connection.kt       # sockets du tunnel
│       ├── video/ScreenEncoder.kt  # SurfaceControl + MediaCodec
│       ├── audio/AudioEncoder.kt   # capture audio (Android 11+)
│       ├── control/Controller.kt   # injection d'événements
│       └── util/Log.kt
│
├── scripts/                        # build.sh / build.ps1 (gradle -> cargo)
└── docs/architecture.md            # détail + feuille de route
```

## Comment l'app finale fonctionne

L'utilisateur n'installe **que le client** (un exécutable desktop). Le serveur Kotlin
compilé (`gninou-server.jar`) est **embarqué dans le binaire** (`include_bytes!`).
Au lancement, le client l'extrait, le pousse sur l'appareil via `adb push`, l'exécute
via `app_process`, ouvre le tunnel, puis décode/affiche. À la fermeture, rien ne reste
sur l'appareil — d'où le « sans app installée ».

## Build complet (client + serveur embarqué)

Nécessite le SDK Android (`android.jar` dans `server/gradle.properties`) :

```bash
scripts/build.sh --release          # Linux / macOS
```
```powershell
.\scripts\build.ps1 -Release        # Windows
```

Le script compile le serveur (Gradle), copie le jar dans `client/assets/`,
puis build le client avec `GNINOU_BUILD_SERVER=1` pour l'embarquer.

> Sans SDK Android, `cargo build` fonctionne quand même : `build.rs` place un
> placeholder vide à la place du jar (l'app démarre mais ne peut pas pousser de
> serveur réel — utile pour développer le client seul).

## Architecture

```
┌────────────────────┐        tunnel ADB          ┌─────────────────────┐
│  Client (Rust)     │  ◄── vidéo (H.264/H.265) ── │  Serveur (Kotlin)   │
│  FFmpeg + SDL      │  ◄── audio (Opus/AAC) ───── │  app_process        │
│  Win / mac / Linux │  ── contrôle (clavier/souris)─►│  Android 5.0+     │
└────────────────────┘   USB · TCP/IP · Wi-Fi      └─────────────────────┘
```

Voir [`docs/architecture.md`](docs/architecture.md).

## Prérequis

- **Client** : Rust (stable), `pkg-config`, FFmpeg + SDL3 (dev headers), `adb` dans le PATH.
- **Serveur** : JDK 17+, Android SDK (`android.jar`), Gradle (wrapper à générer).

## Démarrage rapide (client)

```bash
cd client
cargo run -- --help
```

Exemples d'usage visés :

```bash
gninou                               # premier appareil USB
gninou --tcpip 192.168.1.42:5555     # Wi-Fi / TCP-IP
gninou -m 1024 --max-fps 60 --no-audio
```

## État du projet

🚧 Squelette structuré. La capture, l'encodage, le transport et le rendu restent à implémenter (voir les `TODO` et la feuille de route).

## Licence

Apache-2.0 (à confirmer).
