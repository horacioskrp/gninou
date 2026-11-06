# Test e2e #8 — Contrôle souris

Couvre la fonctionnalité #8 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Encodage des messages de contrôle et projection des coordonnées fenêtre→appareil :

```bash
cd client && cargo test
```

Attendu : tests de `protocol` (encodage Pointer/Scroll/Key/Clipboard) et de
`control` (mapping, mise à l'échelle, letterbox, hors-zone). ✅ 38 tests au total.

## Dépendances pour le bout en bout

Le contrôle souris complet nécessite :
- la **capture** des événements (backend SDL, feature `display`, #7) ;
- l'**injection** côté serveur (`InputManager`, #10).

## Procédure manuelle (appareil réel)

### Préconditions
- Client `--features display`, serveur avec injection (#10), miroir visible.

### Étapes
1. Cliquer dans la fenêtre à un endroit précis de l'écran miroir.
2. Déplacer en maintenant le clic (glisser).
3. Utiliser la molette.

### Résultat attendu
1. Le clic agit au bon endroit sur l'appareil (coordonnées projetées correctement).
2. Le glisser suit le curseur.
3. La molette fait défiler.
4. Un clic dans une bande du letterbox n'envoie rien (hors zone vidéo).

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ logique testée (unit) ; bout en bout dépend de #7 (capture) et #10 (injection)
