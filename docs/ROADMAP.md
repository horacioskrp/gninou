# Roadmap de gninou

Développement **incrémental** : chaque phase apporte un ensemble cohérent de
fonctionnalités, et **chaque fonctionnalité est testée** avant d'être fusionnée
dans `develop`. On ne passe à une phase suivante qu'une fois la précédente verte.

Référence des fonctionnalités : [FEATURES.md](FEATURES.md) · Workflow : [GITFLOW.md](GITFLOW.md).

---

## Phase 0 — Fondations ✅ (en cours de finalisation)

- [x] Monorepo client (Rust) + serveur (Kotlin)
- [x] Structure modulaire (device, media, control, protocol / net, video, audio, control)
- [x] Mécanisme d'embarquement du serveur (`include_bytes!` + push adb)
- [x] Scripts de build (gradle → cargo)
- [ ] Wrapper Gradle (`gradlew`) généré
- [ ] CI minimale (build client + serveur)

## Phase 1 — Connexion de bout en bout 🎯 (prochaine)

> Objectif : le client détecte l'appareil, pousse et lance le serveur, ouvre le tunnel.
> Pas encore d'image — juste un « hello » qui transite.

- [x] (#1) Détection d'appareil + wrapper adb — tests unitaires verts, e2e à valider sur matériel
- [x] (#2) Push + lancement du serveur via `app_process` — logique testée (unit), e2e sur matériel à valider
- [x] (#3) Tunnel adb + ouverture des sockets (vidéo / audio / contrôle) — logique testée (unit)
- **Test de phase** : handshake client ↔ serveur sur un vrai appareil USB — ⬜ à valider sur matériel (voir tests/e2e/03).

## Phase 2 — Miroir vidéo

> Objectif : voir l'écran du téléphone dans une fenêtre.

- [~] (#4) Capture écran (SurfaceControl) — structure + logique de dimensionnement (test écrit) ; `DisplayInfo.primary()` à brancher par version d'Android, validation SDK/matériel requise
- [~] (#5) Encodage H.264 (MediaCodec) — pipeline capture→encodeur→sink écrit, sélection codec/débit testée ; validation SDK/matériel requise
- [~] (#6) Décodage FFmpeg — framing H.264 (NAL) + Decoder testés ; décodage réel derrière la feature `ffmpeg` (à implémenter/valider)
- [~] (#7) Affichage SDL — cadrage (letterbox) testé ; rendu SDL3 derrière la feature `display` (à implémenter/valider)
- **Test de phase** : affichage fluide de l'écran, mesure latence/FPS — ⬜ à valider sur matériel (voir tests/e2e/07).

## Phase 3 — Contrôle

> Objectif : piloter le téléphone au clavier et à la souris.

- [~] (#8) Souris (clic, déplacement, molette) — encodage des messages + projection coordonnées fenêtre→appareil testés ; capture (SDL #7) et injection (#10) pour le bout en bout
- [~] (#9) Clavier — table keycodes Android + construction des messages testées ; capture (SDL #7) et injection (#10) pour le bout en bout
- [~] (#10) Injection d'événements (InputManager) — désérialisation des messages testée ; injection (API masquée) à valider sur matériel
- **Test de phase** : naviguer et taper du texte sur l'appareil — ⬜ à valider sur matériel (voir tests/e2e/10).

## Phase 4 — Qualité & réglages

- [~] (#11) Codecs configurables (H.265 / AV1) — validation client + négociation serveur (repli) testées ; requête MediaCodecList réelle à brancher
- [~] (#12) Résolution / FPS / bitrate — options câblées client→serveur, parsing débit (K/M) testé ; application MediaFormat à valider sur matériel
- [~] (#13) Rognage (crop) et rotation — parsing/validation (client) + application projection (serveur) ; validation matériel requise
- **Test de phase** : comparatif qualité/latence selon réglages — ⬜ à valider sur matériel.

## Phase 5 — Audio & presse-papiers

- [~] (#14) Audio forwarding (Android 11+) — codec audio + pipeline (client) et résolution (serveur) testés ; capture AudioRecord + lecture réelle à brancher/valider
- [~] (#15) Presse-papiers bidirectionnel — protocole des deux sens testé (DeviceMessage + SetClipboard) ; accès presse-papiers OS/Android à brancher
- **Test de phase** : son synchronisé, copier-coller dans les deux sens — ⬜ à valider sur matériel.

## Phase 6 — Sans fil & multi-appareils

- [x] (#16) Connexion TCP/IP — `adb connect` + validation/normalisation `ip:port` testées ; connexion réseau à valider sur matériel
- [x] (#17) Appairage sans fil (Wi-Fi, Android 11+) — `adb pair` + validation code 6 chiffres testée ; appairage réseau à valider sur matériel
- [x] (#18) Multi-appareils — `--serial` répétable + `--all`, sélection testée, un tunnel/port par appareil ; pilotage parallèle réel dépend du pipeline média
- **Test de phase** : miroir via Wi-Fi, deux appareils en parallèle — ⬜ à valider sur matériel (voir tests/e2e/18).

## Phase 7 — Enregistrement & affichages

- [~] (#19) Enregistrement (mp4/mkv) — option `--record`, choix conteneur par extension + compat codec testés ; multiplexage réel derrière la feature `ffmpeg`
- [~] (#20) Affichage écran éteint — option `--turn-screen-off`, message `SetScreenPowerMode` (encode client + decode serveur testés, mêmes octets) ; extinction réelle via `SurfaceControl` à valider sur matériel
- [~] (#21) Écran virtuel / nouvel affichage — options `--display` / `--new-display` (parsing `LxH[/dpi]` client + `Options` serveur testés) ; création réelle du display virtuel à valider sur matériel
- **Test de phase** : fichier enregistré lisible, display virtuel fonctionnel — ⬜ à valider sur matériel (voir tests/e2e/19 et 21).

## Phase 8 — HID, manette & OTG

- [~] (#22) Clavier/souris physiques simulés (HID/UHID) — option `--hid`, descripteurs + rapports clavier/souris et messages UHID (encode client + decode serveur testés) ; création `/dev/uhid` à valider sur matériel
- [~] (#23) Manette (gamepad) — option `--gamepad`, descripteur + `GamepadState::report` testés ; réutilise le transport UHID (#22) ; manette réelle à valider sur matériel
- [~] (#24) Mode OTG — option `--otg` (contrôle HID seul, validation des conflits + `server_args` control-only testés) ; contrôle réel à valider sur matériel
- **Test de phase** : contrôle HID et manette reconnus par l'appareil — ⬜ à valider sur matériel (voir tests/e2e/22, 23 et 24).

## Phase 9 — Caméra & webcam

- [ ] (#25) Miroir caméra (Android 12+)
- [ ] (#26) Webcam virtuelle (V4L2, Linux)
- **Test de phase** : flux caméra affiché, périphérique V4L2 utilisable.

---

## Suivi

À chaque fonctionnalité livrée : cocher ici **et** mettre à jour l'état + la colonne
« Testé » dans [FEATURES.md](FEATURES.md).
