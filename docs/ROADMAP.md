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
- [ ] (#7) Affichage SDL
- **Test de phase** : affichage fluide de l'écran, mesure latence/FPS.

## Phase 3 — Contrôle

> Objectif : piloter le téléphone au clavier et à la souris.

- [ ] (#8) Souris (clic, déplacement, molette)
- [ ] (#9) Clavier
- [ ] (#10) Injection d'événements (InputManager)
- **Test de phase** : naviguer et taper du texte sur l'appareil.

## Phase 4 — Qualité & réglages

- [ ] (#11) Codecs configurables (H.265 / AV1)
- [ ] (#12) Résolution / FPS / bitrate
- [ ] (#13) Rognage (crop) et rotation
- **Test de phase** : comparatif qualité/latence selon réglages.

## Phase 5 — Audio & presse-papiers

- [ ] (#14) Audio forwarding (Android 11+)
- [ ] (#15) Presse-papiers bidirectionnel
- **Test de phase** : son synchronisé, copier-coller dans les deux sens.

## Phase 6 — Sans fil & multi-appareils

- [ ] (#16) Connexion TCP/IP
- [ ] (#17) Appairage sans fil (Wi-Fi, Android 11+)
- [ ] (#18) Multi-appareils
- **Test de phase** : miroir via Wi-Fi, deux appareils en parallèle.

## Phase 7 — Enregistrement & affichages

- [ ] (#19) Enregistrement (mp4/mkv)
- [ ] (#20) Affichage écran éteint
- [ ] (#21) Écran virtuel / nouvel affichage
- **Test de phase** : fichier enregistré lisible, display virtuel fonctionnel.

## Phase 8 — HID, manette & OTG

- [ ] (#22) Clavier/souris physiques simulés (HID/UHID)
- [ ] (#23) Manette (gamepad)
- [ ] (#24) Mode OTG
- **Test de phase** : contrôle HID et manette reconnus par l'appareil.

## Phase 9 — Caméra & webcam

- [ ] (#25) Miroir caméra (Android 12+)
- [ ] (#26) Webcam virtuelle (V4L2, Linux)
- **Test de phase** : flux caméra affiché, périphérique V4L2 utilisable.

---

## Suivi

À chaque fonctionnalité livrée : cocher ici **et** mettre à jour l'état + la colonne
« Testé » dans [FEATURES.md](FEATURES.md).
