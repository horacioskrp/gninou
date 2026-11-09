# Test e2e #32 — Interface graphique : fenêtre + écran « Appareils »

Couvre la fonctionnalité #32 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

La GUI (egui/eframe) est **optionnelle**, derrière la feature cargo `gui` : le
build par défaut (CLI) reste léger et sans dépendance graphique. Le
**view-model** (`gui::state`) est pur et toujours compilé — donc testé dans le
build par défaut ; seul le **rendu** egui est derrière la feature.

## Tests unitaires (automatisés)

Transitions pures du view-model : état initial, passage en chargement,
remplissage + comptage, invalidation d'une sélection hors limites, sélection
valide/invalide, invite de connexion sans sélection, annonce de la cible,
résultats de connexion, échec de détection.

```bash
cd client && cargo test
```

Attendu : `gui::state::*` verts. ✅ 151 tests client (view-model GUI compilé et
testé **sans** la feature `gui`).

## Build de la GUI

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur (egui/eframe), binaire `gninou` incluant la
GUI.

## Procédure manuelle (écran réel)

### Préconditions
- Binaire compilé avec `--features gui`. `adb` dans le PATH. Optionnel : un
  appareil USB et/ou en « Débogage sans fil ».

### Étapes
1. `gninou --gui`
   → ouvre une fenêtre « gninou » avec un en-tête, un bouton **Rafraîchir**, un
     bouton **Connecter**, un tableau `Type | Appareil | État` et une barre
     d'état en bas. La liste se charge au démarrage.
2. Cliquer **Rafraîchir** → la liste se met à jour (USB + Wi-Fi annoncés mDNS),
   un indicateur d'activité s'affiche pendant la recherche.
3. Sélectionner une ligne, cliquer **Connecter** (cible Wi-Fi)
   → `adb connect` est lancé ; la barre d'état affiche « Connecté : ip:port »
     ou l'erreur.
4. Sélectionner une ligne USB puis **Connecter** → message indiquant que la
   connexion directe est réservée au Wi-Fi (USB déjà relié).
5. Binaire **sans** la feature : `gninou --gui`
   → message clair « recompilez avec `cargo build --features gui` ».

### Résultat attendu
1. Fenêtre lisible, liste peuplée au lancement.
2. Rafraîchissement fonctionnel avec retour visuel.
3. Connexion réseau déclenchée, résultat affiché en barre d'état.
4. Garde-fou USB explicite.
5. Message d'aide quand la GUI n'est pas incluse.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-model testé (unit) et build GUI vérifié ; rendu egui et
  interactions réelles à valider à l'écran. Détection/connexion synchrones pour
  ce premier incrément (passage en tâche de fond à prévoir).
