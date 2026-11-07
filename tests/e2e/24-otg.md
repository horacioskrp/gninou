# Test e2e #24 — Mode OTG (contrôle sans miroir)

Couvre la fonctionnalité #24 de [FEATURES.md](../../docs/FEATURES.md).

## Tests unitaires (automatisés)

Validation des conflits d'options OTG et arguments serveur « contrôle seul » :

```bash
cd client && cargo test
```

Attendu : `cli::otg_*` verts (OTG seul ok ; conflits record/display/new_display
rejetés). ✅ 106 tests client.

> En OTG, `server_args()` renvoie `video=false, audio=false` : le pipeline média
> n'est pas démarré, seuls les périphériques HID (#22/#23) sont créés.

## Procédure manuelle (appareil réel)

### Étapes
1. `gninou --otg` → clavier + souris HID, sans fenêtre de miroir.
2. `gninou --otg --gamepad` → ajoute une manette.
3. `gninou --otg --record x.mp4` → erreur (incompatible, pas de vidéo).
4. `gninou --otg --new-display 800x600` → erreur (pas de miroir en OTG).

### Résultat attendu
1. L'appareil est piloté au clavier/souris comme par des périphériques USB
   physiques ; aucun flux vidéo ni audio n'est établi.
2. La manette s'ajoute aux périphériques HID.
3–4. Erreurs de validation claires **avant** tout déploiement.

### Résultat observé
- Date : _à remplir_
- Statut : ⬜ validation des options + arguments serveur testés ; contrôle HID réel
  à valider sur matériel (dépend du pipeline contrôle + accès /dev/uhid)
