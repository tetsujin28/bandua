# Bandua : état du projet et feuille de route

Document de passage de relais. À lire en premier pour reprendre le projet dans une nouvelle conversation.
Dernière mise à jour : 8 novembre 2026.

## 1. Ce qu'est Bandua

Application gratuite de déchargement de rushes (offload) pour Windows, macOS et Linux, par **Domingo Olmo Martin** (DOP, réalisateur, scénariste).
Elle copie une carte caméra vers une ou plusieurs destinations en une seule lecture, vérifie chaque copie par hash et écrit un rapport PDF, un log, un MHL et un JSON.

- Dépôt : https://github.com/tetsujin28/bandua (privé pour l'instant)
- Release actuelle : v0.1.0 (pré-version bêta), 6 fichiers attachés automatiquement
- Application sœur : **Ataecina** (récupération et réparation de rushes), même auteur
- Nom : Bandua, divinité protectrice de l'ouest de la péninsule Ibérique (le nom évoquerait le « lien », peut-être aussi le passage)
- Ancien nom de travail : Offload

## 2. Architecture

```
core/        moteur Rust (bandua-core) : copie, hash, clips, rapports, i18n, mini générateur PDF
  src/lib.rs      run(), scan(), verify_mhl(), folder_name(), hb() (tailles par langue)
  src/clips.rs    regroupement en clips par marque (RED .RDC, ARRI séquences/MXF, BRAW, Sony, GoPro…)
  src/hash.rs     xxHash64 (défaut), XXH3, MD5, SHA-1
  src/report.rs   PDF, log texte, MHL 1.1, JSON (dans la langue choisie)
  src/i18n.rs     textes FR / EN / ES du moteur et des rapports
  src/pdf.rs      générateur PDF sans dépendance (Helvetica, Courier, WinAnsi)
  src/bin/cli.rs  bandua-cli pour tester le moteur sans interface
src-tauri/   application Tauri 2 (commandes : pick_folder, scan_source, start_offload, cancel_offload, reverify…)
ui/          interface HTML/CSS/JS sans framework ni npm (index.html, i18n.js, app.js)
             s'ouvre aussi dans un navigateur en mode démo (carte RED simulée)
docs/screenshots/  9 captures (3 par langue : offload, verified, report)
.github/workflows/build.yml  build Windows / macOS universel / Linux
```

Fonctionnement du moteur : écriture en `.bandua-part`, relecture de chaque copie depuis le disque, renommage seulement si le hash correspond, sinon `.HASH_ERREUR`. Dossier daté `AAAA-MM-JJ_HHMM_PROJET_JOUR_CAMERA_CARTE` dans chaque destination. Fichiers système ignorés. Date de modification d'origine conservée.

## 3. Ce qui est fait (v0.1.0)

- Copie multi-destinations en une lecture, vérification par relecture, contrôle d'espace libre
- Rapports PDF, log, MHL 1.1, JSON dans la langue de l'interface (`_rapport.pdf`, `_report.pdf`, `_informe.pdf`)
- Re-vérification d'un dossier à partir de son MHL
- Interface sombre, sélecteur de langue FR / EN / ES (bascule à chaud), langue du système par défaut
- Bouton « Nouveau déchargement » après succès, « Relancer cette carte » après erreur ou annulation
- Annulation par double clic (pas de boîte de dialogue bloquante)
- Avertissement de non-responsabilité à accepter au premier lancement, rappelé dans À propos, en pied de PDF et à la fin du log
- Crédits « Bandua by Domingo Olmo Martin » : en-tête, À propos, PDF, log, MHL, installeur
- Licence MIT (fichier `LICENSE` avec avertissement en anglais, français et espagnol)
- README en anglais + `README.fr.md` + `README.es.md`, avec captures dans chaque langue
- Release GitHub remplie automatiquement à chaque publication (workflow `release: published`)
- Description du dépôt en anglais et 11 mots-clés (dit, offload, checksum, mhl, xxhash, filmmaking, red-camera, arri, braw, tauri, rust)

## 4. Décisions prises

- **Pile technique** : Tauri 2 + Rust, interface sans npm (Dom n'a pas npm sur son PC Windows 11)
- **Distribution** : les utilisateurs téléchargent un exécutable, ils n'installent jamais Rust. Applis non signées pour l'instant, avertissement Windows / macOS jugé acceptable
- **Langues** : anglais en premier sur GitHub, traductions en fichiers séparés (README) ou sections repliées (notes de Release)
- **Identité visuelle à venir** : **noir et blanc**, icône **ronde** (plus de dégradé orange). Le vert et le rouge restent réservés à « vérifié » et « erreur »
- **Pistes d'icône** : couronne crénelée (couronne murale), profil de la gardienne façon pièce antique, autel votif vu de face. Dom génère l'icône lui-même dans ComfyUI (Flux 3)
- **Iconographie de Bandua** : seule image connue, la patère de Bandis Araugelensis (Museo Nacional de Arte Romano, Mérida) : figure féminine, couronne murale, corne d'abondance, patère

## 5. Feuille de route proposée (à valider par Dom)

Dom a de l'expérience de plateau et juge certaines fonctions inutiles. **Étape suivante : il fait le tri** (inutile / indispensable / manquant). Liste proposée avant ce tri :

**V0.2 : fiabilité (priorité)**
- Relecture sans cache disque (FILE_FLAG_NO_BUFFERING sous Windows, F_NOCACHE sous macOS, équivalent Linux)
- Double lecture de la source (détection d'un lecteur ou d'un câble défaillant)
- Détection d'une source modifiée pendant la copie

**V0.3 : confort de plateau**
- File d'attente de cartes, éjection automatique
- Presets de nommage par production
- Pause et reprise

**V0.4 : rapports pro**
- Vignettes et métadonnées (ffmpeg pour MOV / MP4 / MXF ; outils RED, ARRI, Blackmagic s'ils sont installés, licences à vérifier)
- Logo et coordonnées sur le PDF
- ASC MHL v2

**Plus tard / à confirmer**
- Copies en cascade, historique, détection de carte déjà copiée

**Probablement inutiles selon une première analyse** (à confirmer par Dom) : intégration Frame.io / cloud, démarrage automatique à l'insertion (risqué), santé des disques SMART, notifications sur téléphone.

## 6. Comparaison avec ShotPut Pro (résumé)

ShotPut Pro est bien plus mûr (vignettes et métadonnées, automatisation, presets, pause, contrôles de source, ASC MHL v2, 20 ans de confiance). Atouts de Bandua : gratuit, open source, Linux, pas de compte ni de licence, français, simplicité.
Positionnement visé : la solution gratuite et sérieuse pour petites équipes, courts métrages, indépendants et étudiants.
Avant toute diffusion large : tests sur de vraies cartes (RED, ARRI, Blackmagic) et tests de panne volontaires (disque débranché, disque plein, fichier corrompu), puis bêta avec quelques collègues.

## 7. Chantiers en attente

1. **Tri des fonctionnalités** par Dom, puis développement de la V0.2
2. **Icône** noir et blanc ronde (Dom), puis passer l'interface en noir et blanc et refaire les 9 captures
3. **Site web** (plus tard), une seule page très frontale :
   barre fixe avec bouton Télécharger · hero « Copied. Verified. Proven. » avec bouton qui détecte le système · bandeau de preuve (1 lecture, xxHash64, PDF + MHL, 3 langues) · le problème en une phrase · 3 étapes · grille de fonctionnalités · caméras compatibles (en texte, pas de logos) · le rapport · galerie · à propos de Dom (photo, bio, pourquoi Bandua, lien vers Ataecina) · FAQ · téléchargement final · pied de page.
   Besoins : bio et photo de Dom, adresse du site (GitHub Pages ou domaine perso), nouvelle icône.
4. **Passage du dépôt en public** : décision de Dom (Settings > Danger Zone > Change visibility)

## 8. Notes pratiques pour travailler sur le dépôt

- Publier une version : changer la version dans `src-tauri/tauri.conf.json` et `src-tauri/Cargo.toml`, puis créer une Release avec le tag `vX.Y.Z`. GitHub Actions fabrique et attache les fichiers (environ 25 minutes)
- Test sans Release : onglet Actions > build > Run workflow (fichiers disponibles en « Artifacts » pendant 90 jours)
- Le glisser-déposer web de GitHub ignore le dossier `.github` : créer ou modifier le workflow via l'éditeur web
- Les fichiers binaires (images, icônes) doivent être glissés par Dom depuis son ordinateur sur la page d'upload GitHub
- Tester le moteur : `cargo test -p bandua-core` et `bandua-cli <source> <dest…> --lang fr|en|es`
- Préférences de rédaction de Dom : pas de tiret cadratin, interface sombre qu'il apprécie, travail progressif
