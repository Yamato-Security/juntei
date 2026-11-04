# Preuves et interprétation { #evidence-and-interpretation }

L'investigation numérique ne consiste pas seulement à lire des champs. Il s'agit de déterminer quelle observation le champ étaye,
avec quelle force, et ce qui pourrait produire la même observation de manière innocente.

## Quatre catégories de preuves utiles { #four-useful-evidence-categories }

| Catégorie | Signification | Exemples |
|---|---|---|
| Présence | Un chemin ou un fichier était connu de Windows | Enregistrement de fichier Amcache, entrée ShimCache, enregistrement `$MFT` |
| Exécution | Windows a enregistré le lancement d'un programme ou un état d'exécution | Heure d'exécution Prefetch ; indicateur d'exécution ShimCache pris en charge |
| Interaction de l'utilisateur | Un utilisateur ou une application a référencé/ouvert un élément | RecentDocs, Jump List, LNK du dossier Recent |
| Activité du système | Windows a enregistré une utilisation de ressources ou une configuration | Compteurs SRUM, configuration de service, action TaskCache |

Un artefact peut étayer plus d'une catégorie, mais ne le promouvez jamais automatiquement. Par exemple,
une entrée Amcache comportant un chemin suspect étaye la **présence**. Une heure d'exécution Prefetch correspondante ajoute
une preuve d'**exécution** plus solide. Un enregistrement de création USN et un chemin `$MFT` peuvent situer le fichier sur le disque, et
la quarantaine Defender peut expliquer pourquoi il a disparu.

## Les horodatages sont des affirmations, pas une vérité universelle { #timestamps-are-statements-not-universal-truth }

Demandez-vous ce qui a produit chaque horodatage :

- Les horodatages de `$STANDARD_INFORMATION` et de `$FILE_NAME` décrivent les métadonnées NTFS et peuvent diverger.
- Une heure d'exécution Prefetch décrit un lancement enregistré, mais seules les heures d'exécution récentes subsistent.
- Une heure ShimCache correspond généralement à l'heure de dernière modification du fichier mis en cache, et non à l'heure d'exécution.
- L'heure de dernière écriture d'une clé de Registre s'applique à la clé dans son ensemble, et non à une valeur individuelle.
- RecentDocs attribue l'heure de la clé à l'élément le plus récent ; les éléments plus anciens ont un ordre, mais généralement pas d'heure d'ouverture.
- Les horodatages SRUM identifient souvent un intervalle d'agrégation plutôt que l'opération réseau exacte.

Utilisez l'UTC pendant l'analyse, conservez la valeur d'origine lorsqu'elle est disponible et documentez toute conversion.

## Preuves positives et négatives { #positive-and-negative-evidence }

La présence d'un enregistrement bien compris peut constituer une preuve positive solide. L'absence est généralement
plus faible, car les artefacts sont écrasés par rotation, les caches sont plafonnés, la collecte peut être incomplète, les paramètres varient
et des outils de nettoyage existent. Dites « non observé dans l'artefact collecté » plutôt que « ne s'est pas produit ».

## Échelle de corroboration { #corroboration-ladder }

1. Validez le fichier source et le chemin de collecte.
2. Confirmez que le parseur n'a signalé aucun dommage pertinent ni aucun journal de transactions non appliqué.
3. Interprétez le champ conformément à la page de l'artefact.
4. Trouvez un artefact indépendant étayant la même action.
5. Vérifiez les journaux d'événements, la télémétrie réseau, la mémoire ou le contenu des fichiers lorsqu'ils sont disponibles.
6. Consignez la version exacte de Juntei, la commande, les filtres et l'empreinte de la source.

!!! tip "Rédigez vos conclusions à la mesure de la force des preuves"
    Préférez « Prefetch enregistre l'exécution de `tool.exe` à 10:02:11Z » à « l'attaquant a exécuté le
    logiciel malveillant exactement à 10:02:11Z ». La seconde affirmation ajoute une identité et une intention que Prefetch
    ne peut pas établir à lui seul.
