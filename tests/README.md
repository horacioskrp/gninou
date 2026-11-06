# Tests de bout en bout

Ce dossier regroupe les **procédures de test** des fonctionnalités qui nécessitent
un appareil Android réel ou un émulateur (capture d'écran, injection d'entrées,
audio, Wi-Fi…), qu'un simple test unitaire ne peut pas couvrir.

## Organisation

Un fichier par fonctionnalité testée, nommé d'après son numéro dans
[../docs/FEATURES.md](../docs/FEATURES.md) :

```
tests/
├── README.md
└── e2e/
    └── 01-detection-adb.md      # exemple (à créer avec la fonctionnalité)
```

## Format d'une procédure

Chaque fichier décrit :

1. **Préconditions** (appareil branché, débogage USB activé, etc.)
2. **Étapes** à exécuter
3. **Résultat attendu**
4. **Résultat observé** (rempli lors du test, avec la date et l'appareil utilisé)

Un test est « vert » quand le résultat observé correspond au résultat attendu.
On met alors à jour la colonne « Testé » dans FEATURES.md et on coche la roadmap.

## Tests unitaires

Les tests unitaires vivent dans leur composant respectif :
- client : `cargo test` (modules `#[cfg(test)]` et dossier `client/tests/`)
- serveur : `./gradlew test`
