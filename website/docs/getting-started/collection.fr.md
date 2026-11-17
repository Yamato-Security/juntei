# Collecte des preuves { #collecting-evidence }

Juntei analyse des fichiers déjà collectés sur Windows. Utilisez un outil d'acquisition fiable et préservez les métadonnées d'origine autant que possible.

[auto-parse --vhdx](../commands/auto-parse.md#vhdx-disk-images): Linux / macOS / Windows.

[auto-parse --e01](../commands/auto-parse.md#e01-and-ex01-disk-images): Linux / macOS / Windows.

## Emplacements courants { #common-locations }

| Artefact | Emplacement habituel |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | Fichier de métadonnées du volume NTFS `C:\$MFT` |
| `$LogFile` | Fichier de métadonnées du volume NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| Journal USN | Flux de métadonnées NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Journalisation des accès utilisateur (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer et l'ancien Edge (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`, `%USERPROFILE%\Documents\Outlook Files\*.pst` (`%LOCALAPPDATA%\Microsoft\Outlook\*.pst` avant Outlook 2010), `%APPDATA%\Microsoft\Templates\*.oft`, et les fichiers `.msg` où qu'ils aient été enregistrés |
| Navigateurs web | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` et les dossiers de profil des autres navigateurs |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` et d'autres emplacements utilisateur/système |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` et `CustomDestinations` |
| Cache bitmap RDP | `%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\` (`%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\` sous Windows XP et versions antérieures) |
| Corbeille | `$Recycle.Bin\<SID>\$I*` et `$R*` sur chaque volume (`RECYCLER\<SID>\INFO2` et `D*` sous Windows XP et versions antérieures) |
| ShimCache | Ruche `SYSTEM` : `...\Session Manager\AppCompatCache\AppCompatCache` |
| Quarantaine Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Registre de la machine | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Registre de l'utilisateur | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Fichiers à collecter ensemble { #files-that-must-travel-together }

- Collectez les ruches du Registre avec les fichiers `.LOG`, `.LOG1` et `.LOG2` adjacents. Juntei peut rejouer les journaux
  pour les commandes de Registre et pour ShimCache lorsque la ruche est dans un état « dirty ».
- Collectez le `SRUDB.dat` de SRUM de manière cohérente. Juntei signale actuellement une base de données ESE « dirty », mais
  ne rejoue pas `SRU*.log` ; les enregistrements présents uniquement dans ces journaux ne sont pas disponibles.
- Collectez `$MFT` à côté de `$Extend/$J` lorsque c'est possible. L'enregistrement USN stocke des références de fichiers parents,
  et non un chemin complet ; la MFT améliore la résolution des chemins.
- Collectez `$LogFile` avec le `$MFT` du même volume, à des moments aussi proches que possible. `$LogFile` boucle
  rapidement ; une collecte plus tardive peut faire perdre entièrement les enregistrements redo/undo antérieurs.
- Collectez SRUM avec la ruche `SOFTWARE` correspondante pour résoudre les noms de comptes et de profils Wi-Fi.
- Collectez l'intégralité du répertoire `Sum` de la journalisation des accès utilisateur : `SystemIdentity.mdb` fournit
  les noms des rôles et liste les bases de données annuelles `{GUID}.mdb`. Comme pour SRUM, Juntei ne rejoue pas les
  journaux de transactions `Svc*.log` de l'UAL.
- Collectez les bases de données des navigateurs avec les fichiers `-wal` situés à côté (`places.sqlite-wal`,
  `cookies.sqlite-wal`) : les modifications les plus récentes, ainsi que les versions antérieures des pages à partir
  desquelles les entrées supprimées sont récupérées, ne s'y trouvent que jusqu'au prochain point de contrôle du
  navigateur.
- Collectez l'intégralité du dossier `WebCache` : Windows garde `WebCacheV01.dat` ouvert tant que l'utilisateur a
  une session ouverte, et ses modifications les plus récentes ne se trouvent que dans les journaux de transactions
  `V01*.log`, que Juntei ne rejoue pas.
- Collectez les fichiers OST et PST d'Outlook avec Outlook fermé, ou à partir d'une image disque ou d'un cliché
  instantané de volume : Outlook les garde ouverts tant qu'il s'exécute. Cherchez aussi des PST en dehors des dossiers
  par défaut : les utilisateurs déplacent les archives vers d'autres dossiers, lecteurs et partages de fichiers.
- Collectez l'intégralité de l'arborescence `Quarantine` de Defender afin que Entries, ResourceData et Resources puissent être mis en correspondance.

## Précautions sur un système actif { #live-system-cautions }

Les fichiers ouverts peuvent changer pendant une copie. Les ruches du Registre et les bases de données ESE peuvent être en état « dirty » interne.
ShimCache peut ne pas écrire ses entrées en mémoire les plus récentes avant l'arrêt. Un outil de collecte forensique
qui utilise des instantanés de volume ou un accès brut est préférable à une simple copie de fichiers.

## Sécurité { #safety }

Les preuves peuvent contenir des logiciels malveillants, des échantillons d'exploits, des fichiers LNK malveillants et du texte de formules de tableur.
Effectuez l'analyse sur un système isolé, n'exécutez pas les flux récupérés et n'utilisez `--extract` que si vous
avez l'intention de manipuler du contenu dangereux. Conservez séparément la source intacte.
