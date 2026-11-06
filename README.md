# gninou

> Afficher et contrôler un appareil Android depuis le bureau — réécriture moderne inspirée de [scrcpy](https://github.com/Genymobile/scrcpy).

**gninou** est composé de deux moitiés :

| Composant | Dossier | Langage | Rôle |
|-----------|---------|---------|------|
| **Client desktop** | [`client/`](client/) | **Rust** | Reçoit le flux vidéo/audio, décode (FFmpeg) et affiche (SDL), renvoie les événements clavier/souris. |
| **Serveur device** | [`server/`](server/) | **Kotlin** | Tourne sur l'appareil Android via `app_process`, capture l'écran (`MediaCodec` + `SurfaceControl`) et injecte les entrées. |

Les deux dialoguent via **ADB** (tunnel USB ou TCP/IP).

## Architecture (vue d'ensemble)

```
┌────────────────────┐        ADB tunnel         ┌─────────────────────┐
│  Client (Rust)     │  <─── vidéo/audio H.264 ── │  Serveur (Kotlin)   │
│  FFmpeg + SDL      │  ─── events clavier/souris ─>│  app_process        │
│  (PC Win/mac/Linux)│                            │  (Android 5.0+)     │
└────────────────────┘                            └─────────────────────┘
```

Voir [`docs/architecture.md`](docs/architecture.md) pour le détail.

## Prérequis

- **Client** : Rust (toolchain stable), `pkg-config`, FFmpeg et SDL3 (dev headers), `adb` dans le PATH.
- **Serveur** : JDK 17+, Android SDK, Gradle (wrapper fourni à générer).

## Démarrage rapide (client)

```bash
cd client
cargo run -- --help
```

## État du projet

🚧 Squelette initial — la logique de capture, d'encodage et de transport reste à implémenter.

## Licence

Apache-2.0 (à confirmer).
