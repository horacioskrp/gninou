# Test e2e #9 — Contrôle clavier

Couvre la fonctionnalité #9 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Correspondance touches → keycodes Android et construction des messages :

```bash
cd client && cargo test
```

Attendu : tests de `control::keyboard` verts (lettres a–z, chiffres 0–9, touches
spéciales, rejet des touches inconnues). ✅ 43 tests au total.

## Dépendances pour le bout en bout

- **Capture** des événements clavier (backend SDL, feature `display`, #7).
- **Injection** côté serveur (`InputManager`, #10).

## Procédure manuelle (appareil réel)

### Préconditions
- Client `--features display`, serveur avec injection (#10), miroir visible.

### Étapes
1. Donner le focus à la fenêtre, taper du texte (lettres, chiffres, espace, entrée).
2. Utiliser Retour arrière et les flèches.
3. Tester Back/Home.

### Résultat attendu
1. Le texte saisi apparaît sur l'appareil.
2. Retour arrière et flèches fonctionnent.
3. Back/Home déclenchent la navigation système.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ mapping testé (unit) ; bout en bout dépend de #7 (capture) et #10 (injection)
