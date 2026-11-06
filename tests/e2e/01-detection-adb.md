# Test e2e #1 — Détection d'appareil (wrapper adb)

Couvre la fonctionnalité #1 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Logique de parsing et de sélection, sans appareil :

```bash
cd client && cargo test
```

Attendu : les tests du module `device::adb` passent (parsing de `adb devices`,
sélection par série, cas 0/1/plusieurs appareils, états non prêts). ✅ 9 tests verts.

## Procédure manuelle (appareil réel)

### Préconditions
- `adb` installé et dans le PATH.
- Appareil branché en USB, **débogage USB activé**, autorisation accordée.

### Étapes
1. Aucun appareil branché → lancer `cargo run`.
2. Brancher un appareil, autoriser le débogage → relancer `cargo run`.
3. Brancher deux appareils → `cargo run`, puis `cargo run -- --serial <SERIAL>`.

### Résultat attendu
1. Erreur claire « aucun appareil détecté… ».
2. Log « appareil sélectionné : <SERIAL> (état : Online) ».
3. Sans `--serial` : erreur « plusieurs appareils détectés, précisez --serial… » ;
   avec `--serial` : l'appareil demandé est sélectionné.

### Résultat observé
- Date : _à remplir_
- Appareil(s) : _à remplir_
- Statut : ⬜ en attente de validation sur matériel réel
