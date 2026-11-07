# À propos de Juntei { #about-juntei }

Juntei est un parseur d'artefacts forensiques Windows écrit en Rust, un langage sûr en mémoire. Il complète
[Hayabusa](https://github.com/Yamato-Security/hayabusa) : Hayabusa analyse les journaux d'événements Windows,
tandis que Juntei analyse les artefacts persistants liés au système de fichiers, au Registre, à l'exécution, à l'activité des utilisateurs, à l'utilisation des ressources
et aux produits de sécurité.

Le même binaire fonctionne sous Windows, macOS et Linux. Les analystes peuvent examiner les preuves Windows collectées et les images disque avec une interface cohérente, plutôt qu'un programme distinct pour chaque artefact.

## Objectifs de conception { #design-goals }

- **Analyse défendable :** conserver les valeurs brutes et signaler les dommages au lieu d'inventer silencieusement des données.
- **Utile pour tous :** l'utilisation par défaut doit être simple pour les débutants, tout en restant personnalisable et complète
  pour les utilisateurs chevronnés.
- **Binaire autonome :** ne nécessite aucune bibliothèque d'exécution. Il fonctionne
  immédiatement.
- **Ultra-rapide :** vise à être le parseur le plus rapide tout en ne nécessitant pas une grande quantité de
  mémoire.
- **Utilisation multiplateforme :** analyser des preuves Windows sans avoir besoin d'un poste de travail Windows.
- **Automatisation :** CSV pour les tableurs et les outils de chronologie ; JSON/JSONL pour `jq`, les scripts et les lacs de données.
- **Échec sûr :** les enregistrements malformés sont signalés et ignorés lorsque c'est possible.
- **Filiation transparente :** Juntei identifie les parseurs open source et les outils de référence sur lesquels repose chaque implémentation.

## Ce que Juntei ne fait pas { #what-juntei-does-not-do }

Juntei n'acquiert pas de disque en direct et ne remplace pas la préservation des preuves ; les seules images disque qu'il
monte sont des images VHDX, en lecture seule. Il
ne combine pas automatiquement tous les artefacts en un récit d'incident unique. Il analyse les
preuves que vous fournissez et produit des enregistrements adaptés à l'investigation et à la corrélation.

Juntei ne fera pas non plus l'analyse à votre place. Un spécialiste DFIR est nécessaire pour analyser les preuves.

## Licence { #license }

Juntei est publié sous la
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) et est protégé par le droit d'auteur de Shirofune Security Co., Ltd. Son code source était publié sous
AGPLv3 avant la version 1.0.0. Les licences des crates ainsi que des autres codes et données tiers avec lesquels il est
construit figurent dans
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
Les projets sur lesquels Juntei s'appuie et avec lesquels il a été vérifié sont répertoriés dans
[Remerciements](../acknowledgements.md).
