# Test e2e #22 — Clavier/souris physiques simulés (HID/UHID)

Couvre la fonctionnalité #22 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Descripteurs HID, rapports d'entrée clavier/souris, usages HID, et messages de
contrôle UHID (**mêmes octets** client/serveur) :

```bash
cd client && cargo test
```

Attendu : `control::hid::*` + `protocol::encode_uhid_*` verts. ✅ 99 tests client.

> Tests serveur (`ControlMessageTest.parse_uhid_*`) : mêmes séquences d'octets.
> Exécution Gradle à faire une fois le wrapper généré (Phase 0).

## Procédure manuelle (appareil réel)

> La création réelle des périphériques `/dev/uhid` (accès noyau) n'est pas
> testable hors appareil et s'appuie sur le pipeline de contrôle (#8–#10).

### Étapes
1. `gninou --hid`
   → au démarrage, un clavier et une souris USB « apparaissent » côté appareil.
2. Taper au clavier / bouger la souris dans la fenêtre gninou.
3. Ouvrir un champ protégé (ex. mot de passe d'un jeu) et saisir du texte.

### Résultat attendu
1. L'appareil détecte un clavier + une souris HID (visibles dans les réglages
   « périphériques d'entrée »).
2. La saisie et les mouvements sont pris en compte comme venant d'un vrai
   périphérique USB.
3. Les champs refusant l'injection logicielle acceptent la saisie HID.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ descripteurs/rapports + protocole UHID testés (client + serveur) ;
  création `/dev/uhid` réelle à valider sur matériel
