# Articles et références { #articles-and-references }

Cette page rassemble des points de départ de grande qualité pour les lecteurs qui souhaitent aller au-delà des guides
des artefacts. Les sections **Plus de détails** de chaque artefact contiennent les liens de bas niveau les plus ciblés.

## Orientation DFIR pour débutants { #beginner-dfir-orientation }

- [Poster SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/) — une carte visuelle des artefacts Windows courants et de leurs emplacements.
- [Poster SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/) — pivots de chasse pratiques et aide-mémoire pour les investigations Windows.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — échantillons d'artefacts publics pour l'apprentissage et la validation des parseurs.
- [Référence des artefacts Velociraptor](https://docs.velociraptor.app/artifact_references/) — descriptions orientées collecte et implémentations VQL pour de nombreux artefacts Windows.
- [Documentation de Plaso](https://plaso.readthedocs.io/) — concepts de chronologie et parseurs pour l'analyse croisée entre artefacts.

## Artefacts du système de fichiers { #file-system-artifacts }

- [Microsoft : Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications : NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft : Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft : USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft : descripteurs de sécurité](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) et [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications : MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — descripteurs de sécurité, ACE et expressions conditionnelles.
- [Documentation du format libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Parseur mft d'Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — parseur NTFS avec récupération (carving) des enregistrements INDX et de l'espace résiduel (slack) des index.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — récupère les noms de fichiers dans l'espace résiduel de `$I30`.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — recherche et analyse détaillées du journal des transactions NTFS.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — corrèle l'historique de `$LogFile`, de `$MFT` et de l'USN.

## Artefacts d'exécution et d'inventaire { #execution-and-inventory-artifacts }

- [AmcacheParser d'Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [amcache-forensic de Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [libscca de libyal](https://github.com/libyal/libscca) — travaux sur le format et la bibliothèque Prefetch.
- [PECmd d'Eric Zimmerman](https://github.com/EricZimmerman/PECmd)
- [Mandiant : Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [AppCompatCacheParser d'Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)

## Artefacts d'activité des utilisateurs { #user-activity-artifacts }

- [Microsoft Open Specifications : Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications : Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LECmd d'Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
- [JLECmd d'Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [SrumECmd d'Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [Microsoft : Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registre et persistance { #registry-and-persistence }

- [Microsoft : types de valeurs du Registre](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Spécification du format du Registre Windows de Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) et [libregf](https://github.com/libyal/libregf) — références pour l'analyse de bas niveau et la récupération des ruches.
- [Documentation de Registry Explorer / RECmd d'Eric Zimmerman](https://ericzimmerman.github.io/)
- [Microsoft : arborescence du Registre des services](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft : schéma du Planificateur de tâches](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK : Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications : SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender et traitement des logiciels malveillants { #defender-and-malware-handling }

- [Microsoft : restaurer les fichiers mis en quarantaine dans Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [Plugin Defender de dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — une implémentation distincte de recherche et d'extraction.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — ressources sur le traitement et l'analyse.

## Projets de parseurs de Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "Les documents externes évoluent"
    Vérifiez les affirmations au regard de la version de Windows concernée et de vos données collectées. Un article utile peut
    décrire une génération particulière du système d'exploitation, une version de parseur ou une expérience plutôt qu'une
    règle universelle.
