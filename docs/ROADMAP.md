# Bandua : où j'en suis et ce qui vient

Mes notes de travail sur Bandua : l'état du projet, mes choix et la suite.
Dernière mise à jour : 8 novembre 2026 (v0.2.0 publiée).

## Le projet

Bandua est mon application gratuite de déchargement de rushes, pour Windows et macOS. Elle copie une carte caméra vers un ou plusieurs disques en une seule lecture, vérifie chaque copie par hash et produit un rapport PDF, un log, un MHL et un JSON.

Je l'ai d'abord appelée Offload, puis Bandua, du nom d'une divinité protectrice de l'ouest de la péninsule Ibérique. Son nom évoquerait le « lien », peut-être aussi le passage, ce qui colle bien à une appli qui fait passer les rushes d'un support à l'autre en les protégeant.

C'est l'application sœur d'**Ataecina**, mon outil de récupération et de réparation de rushes.

- Dépôt : https://github.com/tetsujin28/bandua (public)
- Site : https://tetsujin28.github.io
- Version publiée : v0.2.0 (pré-version bêta, 8 novembre 2026)

## Feuille de route, par ordre de priorité

### 1. Fiabilité (V0.2) : la base de tout ✅ fait
Une appli de déchargement qui n'est pas irréprochable ne sert à rien. C'est la priorité absolue.
- ✅ Relire les copies sans passer par le cache du système, pour que la vérification se fasse vraiment sur le disque : FILE_FLAG_NO_BUFFERING sous Windows, F_NOCACHE sous macOS (en lecture et en écriture, après F_FULLFSYNC), posix_fadvise(DONTNEED) après synchronisation sous Linux. Tampon aligné, lectures bornées à la taille attendue (un fichier trop court ou trop long est une erreur). Si un volume refuse le contournement, le log et le PDF le disent.
- ✅ Relire la source une seconde fois, pour repérer un lecteur de cartes ou un câble qui flanche. Option « Relire la carte », activée par défaut. Si les deux lectures diffèrent, les copies sont marquées `.HASH_ERREUR`.
- ✅ Détecter une source qui change pendant la copie : taille et date de chaque fichier contrôlées avant et après sa copie, puis nouvel inventaire de la carte à la fin (fichiers apparus, disparus ou modifiés). Le moindre changement fait échouer le déchargement.
- ✅ Plus aucun fichier `.bandua-part` laissé derrière en cas d'erreur d'écriture
- ✅ Tests automatiques des pannes simulées : lecteur instable, fichier modifié avant ou après sa copie, fichier apparu en cours de route
- ✅ Re-vérification d'un MHL avec le même tableau de bord qu'un déchargement : progression, statut de chaque clip (vérifié, abîmé, manquant), annulation, fichiers absents du MHL signalés, rapport PDF et log écrits dans le dossier
- ✅ Interface plus claire : cercle « copié » qui suit la barre orange puis « vérification / patientez » et « terminé / vérifié » en vert, un seul mot sous le pourcentage, compteur de fichiers dès l'analyse, version dans « À propos », clic sur une destination pour ouvrir le dossier copié
- Pistes écartées pour l'instant, à rouvrir si les tests le justifient :
  - macOS : mesurer avec `mincore` ce qui reste en cache et l'invalider avec `msync`, plus une option « vérification stricte » qui démonte et remonte le disque
  - tout copier puis tout vérifier, en relisant les disques et la carte en parallèle (environ deux fois plus rapide avec deux disques et la relecture de la carte)

### 2. Tests en conditions réelles
Avant de faire connaître Bandua, il doit avoir fait ses preuves.
- Décharger de vraies cartes de mes caméras : RED, ARRI, Blackmagic, et quelques caméras moins pro
- Provoquer les pannes : débrancher un disque en pleine copie, remplir un disque, corrompre un fichier, et vérifier que Bandua réagit comme il faut
- Faire tester une bêta à quelques collègues
- Remettre Linux le jour où je pourrai le tester

### 3. Trier les fonctionnalités
Mon expérience du plateau me dit qu'une partie de ce que proposent les logiciels du marché ne sert pas. Je dois faire le tri entre ce qui est inutile, ce qui est indispensable et ce qui manque, avant d'aller plus loin. Mes premiers doutes : l'intégration Frame.io ou cloud, le démarrage automatique à l'insertion d'une carte (trop risqué), la santé des disques (SMART) et les notifications sur téléphone.

### 4. Confort de plateau (V0.3)
- File d'attente de cartes et éjection automatique de la carte vérifiée
- Presets de nommage par production
- Pause et reprise

### 5. Identité visuelle
- Nouvelle icône **ronde, en noir et blanc**, que je génère dans ComfyUI (Flux 3). Pistes : la couronne crénelée, le profil de la gardienne façon pièce antique, l'autel votif vu de face. La seule représentation connue de Bandua, sur la patère de Bandis Araugelensis (Museo Nacional de Arte Romano, Mérida), montre une figure féminine avec une couronne murale, une corne d'abondance et une patère.
- Passer l'interface en noir et blanc. Le vert et le rouge restent réservés à « vérifié » et « erreur ».
- Refaire les 9 captures d'écran avec la nouvelle identité.

### 6. Rapports pro (V0.4)
- Mon logo et mes coordonnées sur le PDF
- ASC MHL v2

### 7. Sortie publique
- ✅ Passer le dépôt en public
- ✅ Créer le site (tetsujin28.github.io, commun avec Ataecina)
- Signer l'application, pour que Windows (SmartScreen), Firefox et macOS (Gatekeeper) cessent de la signaler comme dangereuse. Firefox a déjà bloqué une fabrication de test. Pistes : SignPath Foundation (gratuit pour l'open source, sur dossier) ou le service de signature de Microsoft pour Windows, le compte Apple Developer (99 $ par an) pour macOS.

### 8. Plus tard, si le besoin se confirme
- Copies en cascade (vers un SSD rapide d'abord, puis vers les autres disques)
- Historique des déchargements et détection d'une carte déjà copiée

## Le site, tel que je l'imagine

Une seule page, très frontale :
- une barre fixe avec un bouton Télécharger toujours visible ;
- une ouverture en plein écran avec « Copied. Verified. Proven. » et un bouton qui détecte le système du visiteur ;
- un bandeau de preuve : 1 lecture, xxHash64, PDF + MHL, 3 langues ;
- le problème en une phrase : ne plus jamais formater une carte sur la foi d'une barre de progression ;
- le fonctionnement en 3 étapes, puis la grille des fonctionnalités ;
- les caméras compatibles, en texte, sans les logos des marques ;
- le rapport PDF, puis une galerie de captures ;
- une présentation de moi, avec photo, bio et pourquoi j'ai fait Bandua, et un lien vers Ataecina ;
- une FAQ, puis un dernier bloc de téléchargement et le pied de page.

Il me faudra une bio, une photo, l'adresse du site (GitHub Pages ou mon propre domaine) et la nouvelle icône.

## Face à ShotPut Pro

ShotPut Pro est bien plus mûr : automatisation, presets, pause, contrôles de la source, ASC MHL v2, et surtout vingt ans de confiance des productions. Bandua a d'autres atouts : il est gratuit, open source, sans compte ni licence, traduit en français, et simple.

Mon objectif n'est pas de remplacer ShotPut sur un long métrage. Je veux que Bandua devienne l'outil gratuit et sérieux des petites équipes, des courts métrages, des indépendants et des étudiants, qui copient encore trop souvent leurs cartes à la main sans aucune vérification.

## Ce qui existe déjà (v0.2.0)

- Copie vers plusieurs destinations en une lecture, vérification par relecture, contrôle de l'espace libre
- Relecture des copies sans le cache système, carte lue deux fois (option), carte surveillée pendant le déchargement
- Écriture en `.bandua-part`, renommage seulement si le hash correspond, sinon `.HASH_ERREUR`
- Dossier daté `AAAA-MM-JJ_HHMM_PROJET_JOUR_CAMERA_CARTE` dans chaque destination, fichiers système ignorés, dates de modification d'origine conservées
- Reconnaissance des clips par marque (un .RDC RED, un dossier ARRIRAW, un .braw…)
- Rapports PDF, log, MHL 1.1 et JSON dans la langue de l'interface (`_rapport.pdf`, `_report.pdf`, `_informe.pdf`)
- Re-vérification d'un dossier à partir de son MHL, avec progression, statut par clip et rapport PDF
- Interface sombre en français, anglais et espagnol, avec changement de langue à chaud
- Bouton « Nouveau déchargement » après succès, « Relancer cette carte » après erreur ou annulation, annulation par double clic
- Avertissement de non-responsabilité à accepter au premier lancement, rappelé dans À propos, sur chaque PDF et à la fin du log
- Crédits « Bandua by Domingo Olmo Martin » dans l'appli, les rapports, le MHL et l'installeur
- Licence MIT, avec l'avertissement en anglais, français et espagnol
- README en anglais, français et espagnol, avec captures dans chaque langue
- Release GitHub remplie automatiquement à chaque publication

## Mes choix

- **Tauri 2 et Rust**, avec une interface en HTML, CSS et JavaScript sans npm
- **Des exécutables prêts à l'emploi** : personne n'a besoin d'installer Rust. L'appli n'est pas encore signée, l'avertissement au premier lancement me convenait, mais les alertes des navigateurs posent la question de la signature (voir le point 7)
- **L'anglais en premier sur GitHub**, avec des traductions séparées pour le README et repliées dans les notes de version
- **Pas de version Linux pour l'instant** (décidé le 8 novembre 2026) : je ne peux pas la tester moi-même. Le code reste compatible Linux, il suffira de remettre Linux dans `build.yml` et dans les README le jour où je pourrai la valider.
- **Le code libre (MIT)**, avec une clause claire sur ma non-responsabilité en cas de perte de données

## Organisation du code

```
core/        moteur Rust (bandua-core) : copie, hash, clips, rapports, traductions, générateur PDF
  src/lib.rs      run(), scan(), verify_mhl(), folder_name(), hb()
  src/clips.rs    regroupement des fichiers en clips par marque
  src/hash.rs     xxHash64 (par défaut), XXH3, MD5, SHA-1
  src/rawio.rs    lectures sans cache système, tampon aligné, lectures bornées
  src/report.rs   PDF, log, MHL 1.1, JSON
  src/i18n.rs     textes FR / EN / ES du moteur et des rapports
  src/pdf.rs      générateur PDF sans dépendance
  src/bin/cli.rs  bandua-cli, pour tester le moteur sans interface
src-tauri/   application de bureau Tauri 2
ui/          interface (index.html, i18n.js, app.js), utilisable aussi en mode démo dans un navigateur
docs/        ce document et les captures d'écran
.github/workflows/build.yml  fabrication Windows et macOS universel
```

## Aide-mémoire

- **Publier une version** : changer le numéro dans `src-tauri/tauri.conf.json` et `src-tauri/Cargo.toml`, puis créer une Release avec le tag `vX.Y.Z`. GitHub fabrique et attache les fichiers en 25 minutes environ.
- **Tester sans publier** : Actions > build > Run workflow. Les fichiers restent 90 jours dans « Artifacts ».
- **Envoyer des fichiers sur GitHub** : le glisser-déposer ignore le dossier `.github`, il faut modifier le workflow dans l'éditeur web. Les images et icônes se glissent depuis mon ordinateur.
- **Tester le moteur** : `cargo test -p bandua-core`, puis `bandua-cli <source> <destinations…> --lang fr|en|es`.
