# Test e2e #35 — GUI : écran « Options du miroir »

Couvre la fonctionnalité #35 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

Un écran dédié (feature `gui`) pour composer et **valider en direct** les
réglages vidéo du miroir — résolution maximale, FPS, codec, débit, rognage et
rotation (#11/#12/#13) — en réutilisant les analyseurs purs déjà en place
(`parse_bitrate`, `Crop::parse`, `VideoCodec`). L'écran ne lance rien : il
produit les arguments serveur et un aperçu de la commande équivalente. Le
**view-model** (`gui::options`) est pur et toujours compilé — donc testé dans le
build par défaut ; seul le rendu egui est derrière la feature.

## Tests unitaires (automatisés)

Transitions et validations pures : état initial valide, args minimalistes par
défaut, résolution 0 = illimité (omise mais valide), champ invalide détecté,
FPS ≥ 1, codec dans les args et l'aperçu, débit vide = automatique, suffixes
K/M, débit invalide rejeté, crop valide/invalide, rotation non nulle présente,
aperçu complet, reset ; plus la navigation vers l'écran dans `gui::state`.

```bash
cd client && cargo test
```

Attendu : `gui::options::*` et `gui::state::navigation_*` verts. ✅ 189 tests
client (view-models GUI testés **sans** la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur.

## Procédure manuelle (écran réel)

### Préconditions
- Binaire compilé avec `--features gui`.

### Étapes
1. `gninou --gui`, puis onglet **Options** (en-tête).
2. Modifier les champs :
   - **Résolution max** (0 = illimité), **FPS max**, **Codec** (h264/h265/av1),
     **Débit** (ex. `8M`, vide = auto), **Rognage** (`W:H:X:Y`, vide = aucun),
     **Rotation** (0/90/180/270).
3. Vérifier la **marque de validité** (✓/✗) à droite de chaque champ : un débit
   `1.5M` ou un crop `1080:2400` incomplet affiche ✗ et « champs invalides ».
4. Observer les deux aperçus mis à jour en direct : **Commande équivalente**
   (`gninou --max-size … --video-codec …`) et **Arguments transmis au serveur**
   (`max_fps=… video_codec=…`).
5. **Réinitialiser** remet les valeurs par défaut.

### Résultat attendu
1–5. Composition et validation claires des réglages, aperçus cohérents avec les
saisies ; aucun réglage invalide ne passe en « valide ».

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ view-model (validation, args, aperçu, navigation) testé (unit) et
  build GUI vérifié ; rendu egui à valider à l'écran. Le lancement effectif du
  miroir depuis la fenêtre (application des arguments à une session) viendra dans
  un prochain incrément — le pipeline média reste 🟨.
