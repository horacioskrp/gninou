# Test e2e #15 — Presse-papiers bidirectionnel

Couvre la fonctionnalité #15 de [FEATURES.md](../../docs/FEATURES.md) et sert de
**test de clôture de la Phase 5**.

## Tests unitaires (automatisés)

Protocole des deux sens (mêmes octets client/serveur) :

```bash
cd client && cargo test        # DeviceMessage::decode + ControlMessage::encode(SetClipboard)
cd server && ./gradlew test    # DeviceMessage.encode + ControlMessage.parse(SetClipboard)
```

✅ 62 tests côté client ; tests serveur via Gradle.

## Sens et dépendances

- **PC → appareil** : `ControlMessage::SetClipboard` (client) → `ClipboardManager.setPrimaryClip` (serveur, API masquée).
- **Appareil → PC** : `DeviceMessage::Clipboard` (serveur) → écriture dans le presse-papiers du PC (lib OS côté client).

## Procédure manuelle (appareil réel)

### Étapes
1. Copier du texte sur le PC → coller sur l'appareil.
2. Copier du texte sur l'appareil → coller sur le PC.
3. Texte Unicode (accents, emoji) dans les deux sens.

### Résultat attendu
1. Le presse-papiers de l'appareil reçoit le texte du PC.
2. Le presse-papiers du PC reçoit le texte de l'appareil.
3. L'UTF-8 est préservé dans les deux sens.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ protocole testé (unit, 2 sens) ; accès presse-papiers OS/Android à brancher — **clôture Phase 5**
