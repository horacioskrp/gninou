# Test e2e #23 — Manette (gamepad)

Couvre la fonctionnalité #23 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Descripteur manette et rapport d'entrée (boutons little-endian, hat, sticks,
gâchettes) :

```bash
cd client && cargo test
```

Attendu : `control::gamepad::*` verts (descripteur, état repos, boutons LE, hat
masqué, axes). ✅ 104 tests client.

> La manette réutilise le transport UHID de #22 (`UhidCreate`/`UhidInput`/
> `UhidDestroy`, décodés et testés côté serveur) : aucun nouveau message.

## Procédure manuelle (appareil réel)

> La création du périphérique `/dev/uhid` (accès noyau) et la capture du
> contrôleur PC (backend d'affichage, #7) ne sont pas testables hors appareil.

### Étapes
1. Brancher une manette au PC, puis `gninou --gamepad`.
2. Ouvrir un jeu compatible manette sur l'appareil.
3. Boutons, croix directionnelle, sticks et gâchettes.

### Résultat attendu
1. L'appareil détecte une manette HID (visible dans les réglages d'entrée).
2. Le jeu reconnaît la manette.
3. Chaque contrôle est transmis : boutons (16), hat 8 directions, 2 sticks
   analogiques, 2 gâchettes.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ descripteur + rapport manette testés (client) ; transport UHID testé
  des deux côtés ; manette réelle à valider sur matériel
