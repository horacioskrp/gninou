# Test e2e #36 — GUI : onglet « Miroir » (affichage, zoom, capture, enregistrement)

Couvre la fonctionnalité #36 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

Un onglet dédié (feature `gui`) qui **affiche le flux vidéo dans la fenêtre** et
offre les commodités attendues d'un miroir : ajustement/zoom (redimensionnement),
**capture d'écran** en PNG et **enregistrement** d'une suite d'images, plus le
plein écran et une pause. En attendant le décodage vidéo réel (#6) alimenté par
un appareil, l'onglet affiche une **mire de test animée** (`media::frame`) : cela
valide tout le chemin de rendu (texture egui) et d'écriture (PNG) dès maintenant.

Le **view-model** (`gui::mirror`) et le type d'image (`media::frame`) sont purs
et toujours compilés — donc testés dans le build par défaut ; seul le rendu egui
et les écritures PNG (`png`, derrière la feature `gui`) sont optionnels.

## Tests unitaires (automatisés)

- `media::frame` : image noire opaque valide, compte de pixels, dimensions de la
  mire, dimensions minimales (0×0 → 1×1), barre qui se déplace avec le tick et se
  place à la bonne colonne.
- `gui::mirror` : état initial, ajustement en gardant les proportions, taille
  réelle, zoom explicite, image vide → taille nulle, zoom +/− borné (25–400 %),
  pause, nommage des captures, nom d'image d'enregistrement, comptage des images
  enregistrées, mise à jour des dimensions ; plus la navigation vers l'onglet
  dans `gui::state`.

```bash
cd client && cargo test
```

Attendu : `media::frame::*`, `gui::mirror::*` et `gui::state::navigation_*`
verts. ✅ 207 tests client (view-models GUI testés **sans** la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur (ajoute la dépendance `png`).

## Procédure manuelle (écran réel)

### Préconditions
- Binaire compilé avec `--features gui`.

### Étapes
1. `gninou --gui`, puis onglet **Miroir** (en-tête).
2. L'image s'affiche et **s'anime** (barre claire qui défile) : le rendu de
   trames fonctionne.
3. **Redimensionnement** : boutons **Ajuster** (proportions), **100 %** (taille
   réelle), **➖/➕** (zoom 25–400 %) ; le libellé indique le mode/zoom courant.
   Redimensionner la fenêtre : en mode Ajuster, l'image suit.
4. **⏸ Pause / ▶ Lecture** : fige ou reprend l'animation.
5. **📸 Capturer** : écrit `gninou-capture-NNN.png` dans `gninou-captures/`
   (sous le dossier courant) ; le chemin s'affiche en bas.
6. **⏺ Enregistrer / ⏹ Arrêter** : crée `gninou-captures/rec-<horodatage>/` et y
   écrit une suite d'images ; le compteur « ● N images » progresse, puis le
   total s'affiche à l'arrêt.
7. **⛶ Plein écran** : bascule la fenêtre en plein écran (re-cliquer pour revenir).

### Résultat attendu
1–7. Le flux (mire) s'affiche et s'anime ; zoom/ajustement, pause, capture PNG,
enregistrement d'images et plein écran fonctionnent depuis la fenêtre.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-model + type image testés (unit) et build GUI vérifié ; rendu
  egui, captures/enregistrement PNG à valider à l'écran. La **mire** tient lieu
  de source tant que le décodage vidéo réel (#6) n'est pas branché sur un
  appareil — le remplacement de la mire par les trames décodées est l'étape
  suivante.
