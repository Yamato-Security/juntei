# Validation des parseurs { #parser-validation }

Les crates de parseurs maintenues par Juntei sont testées sur des preuves publiques et comparées à des implémentations
indépendantes. Ces comparaisons aident à détecter les régressions et les différences d'interprétation ; elles
ne rendent aucun parseur infaillible. Conservez les preuves sources, examinez les champs de problèmes ou de
dommages de chaque enregistrement, et validez les constats importants avec un autre artefact ou une autre implémentation.

Les informations ci-dessous reprennent les informations de validation qui figuraient auparavant dans le README principal
de Juntei. Consultez les dépôts des parseurs liés pour les jeux de test actuels et les détails d'implémentation.

## Amcache { #amcache }

Juntei utilise [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser), un
fork sous licence Apache-2.0 de
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) de Security Ronin. Sa
[documentation de sortie](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
définit chaque champ d'exécutable, de périphérique, de constat et de résumé.

## Table de fichiers principale { #master-file-table }

La crate mft de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) est un fork maintenu de la crate
[mft](https://github.com/omerbenamram/mft) d'Omer Ben-Amram. Sur huit fichiers `$MFT` (l'échantillon
de la crate, le NIST hacking case et six fichiers du DFIR Artifact Museum), 1 048 733 lignes CSV et leurs colonnes
correspondaient dans l'ordre à celles d'un parseur de référence. Les comparaisons ont porté sur la sortie par défaut, `--all-times`,
`--short-names` et `--cutoff` ; la présentation des horodatages constituait la différence intentionnelle.

## Journal des transactions NTFS ($LogFile) { #ntfs-transaction-log-logfile }

La crate logfile de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) est maintenue à partir de
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog). Sa validation couvre six journaux complets du DFIR
Artifact Museum, des jeux de test publics en versions 1.1 et 2.0, les métadonnées redo/undo décodées, et des
écritures de journal comparées à des données NTFS analysées de manière indépendante. Consultez le dépôt du parseur pour le détail
de la couverture et des différences connues.

Juntei ajoute des tests CLI qui comparent chaque champ CSV avec le JSON et le JSONL sur le volume de test public go-ntfs
(2 129 enregistrements), vérifient la séparation des données redo/undo et la longueur des charges utiles brutes, testent des journaux endommagés,
refusent l'écrasement des preuves et empêchent les chemins MFT actuels de suivre des numéros d'enregistrement réutilisés.
Des cas de non-régression distinguent les enregistrements MFT affectés de l'attribut ouvert par lequel ils sont écrits,
vérifient les métadonnées des entrées de répertoire/attributs de données à plat, et empêchent l'appariement de MFT entre volumes.
Les tests `auto-parse` multi-volumes couvrent des fichiers CSV distincts et un fichier JSONL unique avec provenance.

La commande a également été vérifiée sur six journaux Windows collectés de manière privée : les 708 465 enregistrements CSV exportés
correspondaient champ par champ au JSONL, et les exports `auto-parse` par hôte conservaient les mêmes nombres
d'enregistrements. Les preuves sources ne sont pas distribuées avec le dépôt. Ces vérifications contrôlent la cohérence de la sortie,
et non le fait que chaque transaction journalisée représente une action utilisateur achevée.

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) est un fork maintenu de
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) de ForensicRS. Sa sortie décodée a été
comparée à [libscca](https://github.com/libyal/libscca), l'implémentation Prefetch indépendante
de libyal.

## Journal USN { #usn-journal }

La crate usn de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) est un fork
maintenu de [Sootmark usn](https://github.com/Sootmark/usn). Sur six journaux réels, les chemins parents
de Juntei, tenant compte du temps, correspondaient à ceux d'un parseur indépendant partout où ce parseur résolvait un chemin. Juntei
a également résolu 4 321 enregistrements que la comparaison laissait non résolus. Son outil de carving n'a produit aucun faux
enregistrement lors des tests sur 880 Mo de données `$MFT`.

## Index de répertoires ($I30) { #directory-indexes-i30 }

Le module `i30` de la crate mft de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) lit les enregistrements INDX
et leur slack. Sur deux flux `$I30` d'un volume créé avec ntfs-3g (`test_files/i30/` de
Juntei) et sur trois échantillons `$I30` publics écrits par Windows, ses entrées utilisées et celles présentes dans le
slack correspondaient à celles de [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) de Maxim Suhanov, avec les
mêmes noms et horodatages. Lorsque dfir_ntfs signale une référence parente dépassant 32 bits (des octets ayant écrasé
le parent), Juntei laisse le parent vide.

## Secteur de démarrage ($Boot) { #boot-sector-boot }

La crate boot de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) est développée en propre par Yamato
Security. Sur les cinq fichiers `$Boot` publics du DFIR Artifact Museum (Windows 10, 11 et
Server 2012 R2), chaque champ correspondait à celui d'un parseur de référence. Sur des volumes créés avec `mkntfs` avec des secteurs de 512,
1 024 et 4 096 octets et des clusters de 4 Kio à 64 Kio, les tailles, les clusters `$MFT` et `$MFTMirr`
et les numéros de série correspondaient à ceux de `fsstat` de The Sleuth Kit. Le `$Boot` Windows 11 du Museum
porte le numéro de série de volume qu'enregistre le fichier LNK du même hôte, et les cinq fichiers ont le
même code de démarrage.

## Descripteurs de sécurité ($Secure:$SDS) { #security-descriptors-securesds }

La crate secure de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) est développée en propre par Yamato
Security. Sur les cinq flux `$SDS` du Museum (7 808 descripteurs, 60 781 ACE), les
identifiants de sécurité, hachages, décalages, propriétaires, groupes, indicateurs de contrôle et nombres d'ACE correspondaient à ceux d'un parseur
de référence, et le type, les indicateurs, le masque, le SID et les types d'objets de chaque ACE correspondaient à `dissect.ntfs`. Chaque
hachage enregistré est bien celui de son descripteur, chaque copie miroir correspond à son entrée, et chaque descripteur dispose d'un
SDDL, y compris les conditions de ses 3 384 ACE de rappel (callback).

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) est basé sur les crates
[ese](https://github.com/Sootmark/ese) et [srum](https://github.com/Sootmark/srum) de Sootmark. Sur
11 bases de données réelles, chaque colonne des 45 255 entrées émises correspondait à un décodage indépendant des
valeurs exposées par `dissect.esedb`.

## Journalisation des accès utilisateur { #user-access-logging }

`parse-ese ual` lit les bases de données UAL avec le même lecteur ESE. Sur 16 bases de données réelles (308 lignes,
dont 145 lignes de clients), chaque valeur correspondait à un décodage indépendant des valeurs exposées par
`dissect.esedb`. Dans chaque ligne de client, le premier jour comportant un décompte était le jour UTC de
`InsertDate`, le dernier le jour UTC de `LastAccess`, et la somme des décomptes était égale à `TotalAccesses`.

## Raccourcis LNK { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) est un fork maintenu de la crate
[shell](https://github.com/Sootmark/shell) de Sootmark. Il a été comparé sur 1 118 raccourcis
publics. Sur les 30 colonnes CSV partagées avec le parseur de référence, 16 valeurs individuelles différaient :
13 chemins cibles impliquant des éléments endommagés, tronqués, fictifs, MTP ou du Panneau de configuration ; deux
listes de blocs de données supplémentaires impliquant des blocs inconnus ou tronqués ; et une référence MFT que le
parseur de référence lisait à partir d'octets différents. Juntei signale les structures endommagées au lieu de les
normaliser silencieusement.

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) utilise les travaux de Sootmark sur les
fichiers composés et les Jump Lists, les liens intégrés étant décodés par `ys-lnk-parser`. Sur 77 Jump Lists
publiques, les 13 057 cellules de comparaison partagées, y compris les horodatages à 100 ns près, correspondaient au
parseur de référence. Six fichiers refusés par la référence (listes personnalisées vides, pied de page manquant, ou fichier
qui n'était pas une Jump List) ont été soit décodés, soit explicitement signalés par Juntei.

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) est un fork maintenu
de [shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) de
Security Ronin, avec la prise en charge des ruches du Registre et des
journaux de transactions. Sur 20 ruches `SYSTEM` publiques (24 jeux de contrôle et 7 290 entrées) ainsi que 17
valeurs AppCompatCache autonomes, chaque entrée correspondait au chemin, à l'horodatage, à l'indicateur d'exécution, aux indicateurs et à la
position émis par le parseur de référence.

## Quarantaine Microsoft Defender { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) est un
fork maintenu de la crate
[defender](https://github.com/Sootmark/defender) de Sootmark. Un dossier de quarantaine écrit au format de Defender
par l'outillage de test du parseur est décodé de manière cohérente par le lecteur de quarantaine indépendant de
`dissect.target`.

## Artefacts issus du Registre { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) est comparé à
[yarp](https://github.com/msuhanov/yarp) pour les clés et valeurs allouées, la relecture des transactions des ruches « dirty »,
la récupération des cellules libres, le cell slack, les données au-delà des hive bins, ainsi que les enregistrements supprimés ou remplacés par la relecture.
Les comparaisons propres aux artefacts comprennent :

- les services, les valeurs et les actions en cas d'échec provenant de ruches `SYSTEM` de Windows 7 et de Windows 11 « dirty » ;
- RecentDocs décodé avec yarp et libfwsi sur neuf ruches `NTUSER.DAT` publiques ;
- les tâches planifiées décodées de manière indépendante avec yarp sur sept ruches `SOFTWARE` publiques couvrant
  Windows 7–11, Server 2012 R2 et Server 2022 ; et
- les structures de comptes SAM comparées avec `dissect.target` sur sept ruches `SAM` publiques.

Consultez [Remerciements](../acknowledgements.md) pour la liste complète des outils de référence, des recherches sur les formats,
des spécifications et des sources de preuves publiques.
