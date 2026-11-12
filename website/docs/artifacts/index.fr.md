# Guide des artefacts { #artifact-guide }

Windows conserve des traces qui se recoupent dans le système de fichiers, le Registre, les bases de données de compatibilité,
l'historique de l'interface utilisateur, la comptabilisation des ressources et les produits de sécurité. Juntei analyse ces sources,
mais l'enquêteur doit toujours interpréter ce que signifie chacune d'elles.

## Choisir un artefact selon la question { #choose-an-artifact-by-question }

| Question d'investigation | Commencer par | Corroborer avec |
|---|---|---|
| Quelles lignes de commande PowerShell ont été sauvegardées ? | [Historique PowerShell](powershell-history.md) | Journaux d’événements PowerShell, télémétrie des processus, Prefetch |
| Cet exécutable a-t-il été exécuté ? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), journaux d'événements |
| Ce fichier a-t-il jamais été présent ? | [Amcache](amcache.md), [$MFT](mft.md) | [Journal USN](usn-journal.md), [$I30](i30.md), ShimCache |
| Quand un fichier a-t-il été créé, renommé ou supprimé ? | [Journal USN](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), quarantaine Defender |
| Qu'est-ce qu'un utilisateur a mis à la Corbeille, et quand ? | [Corbeille](recycle-bin.md) | [Journal USN](usn-journal.md), [$MFT](mft.md), [LNK](lnk.md), [Jump Lists](jump-lists.md) |
| Quel nom ou quel horodatage existait avant une modification des métadonnées ? | [$LogFile](logfile.md) | [$MFT](mft.md), [Journal USN](usn-journal.md), [$I30](i30.md) |
| Qui est propriétaire d'un fichier, et qui peut le modifier ? | [$Secure:$SDS](secure-sds.md) avec [$MFT](mft.md) | [SAM](sam.md), journaux d'événements Security |
| Sur quel volume se trouvait la cible d'un raccourci ou d'un fichier Prefetch ? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [Périphériques USB](usb.md) |
| Qu'a ouvert un utilisateur ? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Un programme a-t-il transféré une grande quantité de données ? | [SRUM](srum.md) | Télémétrie du pare-feu, du proxy, DNS, des paquets ou de l'EDR |
| Quels sites web un utilisateur a-t-il visités, et qu'a-t-il téléchargé ? | [Navigateurs web](browser.md), [Internet Explorer et l'ancien Edge](webcache.md) | Journaux du proxy et DNS, `$MFT` et journal USN, Defender, SRUM |
| Avec qui un utilisateur a-t-il échangé des e-mails, et qu'est-ce qui y était joint ? | [Outlook](outlook.md) | Journaux des serveurs et passerelles de messagerie, `$MFT` et journal USN, Defender, LNK et Jump Lists |
| Quels clients et quels comptes ont utilisé ce serveur, et quels jours ? | [Journalisation des accès utilisateur (UAL)](ual.md) | Journaux d'événements Security, journaux SMB et RDP, journaux du pare-feu |
| Une persistance a-t-elle été configurée ? | [Services](services.md), [TaskCache](taskcache.md), [Registre](registry.md) | Journaux d'événements, Prefetch, `$MFT` |
| Qu'a supprimé Defender ? | [Quarantaine Defender](defender-quarantine.md) | Journaux d'événements, `$MFT`, journal USN |
| Des comptes locaux ont-ils été ajoutés ou modifiés ? | [SAM](sam.md) | Journaux d'événements Security et historique des transactions du Registre |

## Lire les avertissements avant les champs { #read-warnings-before-fields }

Chaque page d'artefact suit la même séquence :

1. **Ce qu'il est** et à quelles questions il peut répondre.
2. **Où le trouver** et comment la collecte l'affecte.
3. **Ce qu'il prouve — et ce qu'il ne prouve pas.**
4. **Comment l'analyser** avec Juntei.
5. **Comment lire chaque champ de sortie.**
6. **Comment le corréler** avec des preuves indépendantes.
7. **Plus de détails** pour la recherche de bas niveau.

!!! danger "N'assimilez pas une constatation du parseur à de la malveillance"
    Les constatations de Juntei identifient des motifs qui méritent un examen, comme un nom de binaire système en dehors de
    `System32`. Les programmes d'installation, les administrateurs, les couches de compatibilité et les mises à jour logicielles peuvent produire
    des enregistrements inhabituels. Validez à l'aide du contexte et de preuves indépendantes.
