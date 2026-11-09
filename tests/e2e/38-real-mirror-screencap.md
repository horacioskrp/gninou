# Test e2e #38 — GUI : écran réel de l'appareil dans la fenêtre de miroir

Couvre la fonctionnalité #38 de [FEATURES.md](../../docs/FEATURES.md).

## Principe

La fenêtre de miroir dédiée (#37) affiche désormais l'**écran réel** de
l'appareil. Chaque fenêtre lance un **fil de capture** qui interroge en boucle
`adb exec-out screencap -p`, décode le PNG renvoyé en trame RGBA
(`device::screencap`) et met à jour la texture egui en continu. Le débit est
modeste (quelques images/seconde — c'est un échantillonnage, pas un flux vidéo),
mais on voit vraiment l'appareil. La **capture d'écran** et l'**enregistrement**
portent sur l'image réelle affichée.

C'est la voie **pragmatique** en attendant le miroir H.264 temps réel (#6) : pas
besoin du pipeline de décodage vidéo complet pour afficher l'appareil.

- Le **décodage** PNG → `Frame` (`device::screencap::decode_png`) est pur et
  testé (RGBA tel quel, RGB complété en alpha opaque, entrée invalide rejetée).
- La **capture** (`adb exec-out screencap`), le fil d'arrière-plan, le rendu de
  la texture et les écritures PNG vivent dans `device::screencap` (I/O) et
  `gui::app` (feature `gui`).

## Tests unitaires (automatisés)

- `device::screencap` : `decode_rgba_identique`, `decode_rgb_ajoute_alpha_opaque`,
  `decode_png_invalide_echoue`.

```bash
cd client && cargo test
```

Attendu : `device::screencap::*` verts. ✅ 216 tests client (décodage testé
**sans** la feature `gui`).

## Build

```bash
cd client && cargo build --features gui
```

Attendu : compilation sans erreur.

## Procédure manuelle (écran réel)

### Préconditions
- Binaire compilé avec `--features gui`.
- **Un appareil Android réel branché et prêt** (`adb devices` le liste comme
  `device`). La capture passe par `adb exec-out screencap`, donc adb doit être
  dans le `PATH` et l'appareil autorisé.

### Étapes
1. `adb devices` → vérifier que l'appareil est `device` (et non `unauthorized`).
2. `gninou --gui`, onglet **Appareils** → sur la ligne de l'appareil prêt,
   cliquer **« 🖥 Miroir »**.
3. La fenêtre de miroir s'ouvre : d'abord un **indicateur « Capture de l'écran
   en cours… »**, puis l'**écran réel de l'appareil** apparaît et se rafraîchit.
4. Manipuler le téléphone (ouvrir une appli, défiler) → la fenêtre suit, à
   quelques images/seconde.
5. **Capturer** (PNG dans `gninou-captures/`) → le fichier contient bien l'écran
   réel. **Enregistrer** quelques secondes → la séquence d'images reflète l'écran.
6. Zoom/ajustement et plein écran fonctionnent sur l'image réelle.
7. Fermer la fenêtre → le fil de capture s'arrête proprement (join au `Drop`).

### Résultat attendu
1–7. L'écran réel de l'appareil s'affiche dans la fenêtre dédiée et se met à
jour en continu ; capture et enregistrement portent sur l'image réelle ; la
fermeture arrête le fil de capture sans fuite.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ décodage PNG testé (unit) et build GUI vérifié ; capture `adb
  screencap` → affichage de l'écran réel à confirmer à l'écran (nécessite un
  appareil prêt). Le débit reste celui d'un échantillonnage screencap ; le
  miroir H.264 temps réel (#6) et le contrôle (#8–#10) restent à venir.
