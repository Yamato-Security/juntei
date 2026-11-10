# Remerciements { #acknowledgements }

Juntei existe parce que les communautés DFIR et open source ont publié des parseurs, des travaux de recherche sur les formats,
des sorties de référence, des spécifications et des preuves de test. Les projets ci-dessous ont guidé l'implémentation,
la validation, le nommage des champs ou la sortie attendue par les analystes. Figurer dans cette liste ne signifie pas que leur code est
intégré à Juntei ; consultez chaque projet pour connaître sa licence.

## Fondations des parseurs maintenues par Yamato Security { #parser-foundations-maintained-by-yamato-security }

| Domaine de Juntei | Parseur maintenu | Projet amont/fondation |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| Journal USN | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| `$LogFile` NTFS | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Index de répertoires (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`, module `i30`) | Recherches sur l'index slack citées ci-dessous |
| `$Boot`, `$Secure:$SDS` NTFS | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crates `boot` et `secure`) | MS-DTYP de Microsoft et les recherches NTFS citées ci-dessous |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) et [ese](https://github.com/Sootmark/ese) |
| Navigateurs web / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite) et [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Travaux de Sootmark shell sur les fichiers composés et les Jump Lists |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| Images disque VHDX (`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) et [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), exposés via [fuser](https://github.com/cberner/fuser) | MS-VHDX de Microsoft |
| Registre | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Recherches sur le format du Registre et la récupération citées ci-dessous |

## Outils de référence et validation indépendante { #reference-tools-and-independent-validation }

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), par Hexastrike,
  a fourni le catalogue de vérifications de persistance, la liste d'autorisation par défaut, l'instantané LOLBAS et le comportement
  de référence de la commande Rust native `persistence-hunter`. Sa licence MIT et sa mention de copyright
  sont conservées dans `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`. Le projet original
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) a inspiré le projet Python ;
  consultez les [remerciements relatifs à la persistance](persistence-hunter/acknowledgements.md) pour la filiation complète.

- La **spécification regf** et **yarp** de Maxim Suhanov ont guidé les structures du Registre, la relecture
  des journaux de transactions et la récupération des enregistrements supprimés ; son **dfir_ntfs** a servi de référence pour les enregistrements INDX et
  le carving de l'index slack.
- **INDXRipper** de Harel Segev constitue une référence supplémentaire pour le carving des noms de fichiers dans l'index slack.
- **dfir_ntfs**, **LogFileParser** de Joakim Schicht et **ntfs-linker** de Stroz Friedberg ont guidé
  la validation et l'interprétation de `$LogFile` ; **go-ntfs** de Velocidex a fourni des preuves de test NTFS publiques.
- Les projets libyal de Joachim Metz **libregf**, **libfsntfs**, **libscca**, **libfwsi** et
  **libesedb** offrent des recherches détaillées sur les formats ainsi que des références d'analyse indépendantes.
- **dissect.esedb**, **dissect.ntfs** et **dissect.target** de Fox-IT ont fourni des interprétations indépendantes de SRUM,
  de l'UAL, de `$Secure:$SDS`, de SAM et de Defender utilisées lors de la validation.
- `fsstat` de **The Sleuth Kit** a servi de référence pour les secteurs de démarrage NTFS des volumes créés avec
  `mkntfs`.
- Certains outils d'Eric Zimmerman ont établi les noms de champs et les attentes des examinateurs utilisés pour les comparaisons.
- **regipy**, **python-registry**, **Notatin** et **artemis** ont fourni des implémentations du Registre,
  des tests ou des exemples d'interprétation.
- **LnkParse3**, **lnk_parser** et **jumplist_parser** d'AbdulRhmanAlfaifi, ainsi que JLECmd, ont fourni
  des comportements LNK/Jump List indépendants et des cas de test publics.
- **prefetch-forensic** de Security Ronin et le projet **forensicnomicon** ont guidé les observations
  à forte valeur de Juntei sur Prefetch/Amcache/ShimCache.
- **defender-dump** et dissect.target fournissent des implémentations distinctes pour comprendre les structures
  de quarantaine de Defender et un comportement d'extraction sûr.
- **Plaso**, **Chainsaw** et **Velociraptor** ont fourni des comparaisons de parseurs, des preuves de test ou
  des comportements de référence orientés collecte pour plusieurs artefacts.
- **Hindsight** d'Obsidian Forensics et les définitions **SQLiteHunter** de Velocidex ont servi de références pour
  l'historique, les téléchargements, les cookies et l'historique des formulaires des navigateurs.

## Spécifications et recherches { #specifications-and-research }

Nous remercions Microsoft d'avoir publié les Open Specifications relatives aux Shell Links, aux Compound Binary Files,
aux protocoles NTFS/SAM, aux structures du Registre, des services et des tâches, ainsi que la documentation des API. Nous remercions également les
auteurs des recherches forensiques sur AppCompatCache, Amcache, SRUM, Prefetch, le Registre et NTFS, qui ont rendu
ces artefacts compréhensibles pour les praticiens.

## Preuves de test { #test-evidence }

Les preuves publiques du
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) d'Andrew Rathbun, de NIST CFReDS, de Plaso,
de Regipy, d'Artemis, des dépôts de test de parseurs et d'autres collections dûment licenciées permettent d'effectuer des tests
de non-régression sur de véritables données Windows. Juntei conserve les notes d'attribution et de licence à côté des fichiers
de test inclus.

## Contributeurs { #contributors }

Merci à chaque contributeur, relecteur, rapporteur d'incidents et utilisateur de Juntei et des parseurs de Yamato Security
qui a fourni des cas limites. Les contributeurs actuels sont répertoriés par GitHub sur la
[page des contributeurs de Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors).

## Licence { #license }

Juntei est publié sous la
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) et son copyright appartient à Shirofune Security Co., Ltd. Son code source était publié sous
l'AGPLv3 avant la version 1.0.0. Les licences des crates et des autres codes et données tiers avec lesquels
il est construit figurent dans
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
