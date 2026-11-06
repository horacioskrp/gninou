# Test e2e #10 — Injection d'événements (serveur)

Couvre la fonctionnalité #10 de [FEATURES.md](../../docs/FEATURES.md) et sert de
**test de clôture de la Phase 3** (contrôle souris + clavier opérationnel).

## Tests unitaires (automatisés)

Désérialisation des messages de contrôle (miroir exact de l'encodage client) :

```bash
cd server && ./gradlew test
```

Attendu : `ControlMessageTest` vert (Pointer/Key/Scroll/Clipboard, type inconnu rejeté).
⚠️ Nécessite Gradle + `android.jar` ; non exécutable dans l'env de dev actuel.

> L'injection elle-même (`EventInjector` via `InputManager`, API masquée) n'est pas
> testable hors appareil.

## Procédure manuelle (appareil réel) — test de Phase 3

### Préconditions
- Chaîne complète opérationnelle : miroir visible (#7), capture d'entrées (SDL),
  serveur avec injection, tunnel OK.
- Débogage USB autorisé (certains appareils exigent « USB debugging (Security Settings) »).

### Étapes
1. Cliquer à un endroit précis de l'écran miroir → l'appareil réagit au bon endroit.
2. Glisser pour faire défiler une liste.
3. Taper du texte au clavier ; utiliser Retour arrière, flèches, Back/Home.

### Résultat attendu
1. Clics, glissers et saisie clavier agissent correctement sur l'appareil.
2. En cas de refus : message « INJECT_EVENTS permission » → activer l'option de sécurité.

### Résultat observé
- Date : _à remplir_
- Appareil(s) / version Android : _à remplir_
- Statut : ⬜ désérialisation testée (unit) ; injection à valider sur matériel — **clôture Phase 3**
