# Accès à distance (côté utilisateur) { #remote-access-user-side }

## De quoi s'agit-il { #what-it-is }

Le `NTUSER.DAT` d'un utilisateur conserve des traces des connexions à distance que l'utilisateur a établies depuis l'ordinateur.
Elles constituent le côté source du déplacement latéral :

- **Bureau à distance (`mstsc`) :**
  - les ordinateurs saisis dans le client, du plus récent au plus ancien ;
  - une clé par ordinateur auquel l'utilisateur s'est connecté, avec le nom de compte utilisé et l'empreinte
    du certificat de serveur que l'utilisateur a accepté ;
  - les ordinateurs que l'utilisateur a autorisés à utiliser des périphériques locaux.
- **PuTTY :**
  - les sessions enregistrées : hôte, port, compte, protocole, fichier de clé, proxy et redirections de ports ;
  - les dernières sessions lancées ;
  - chaque clé de serveur SSH que l'utilisateur a acceptée.
- **WinSCP :** les sessions enregistrées et les clés de serveur SSH acceptées, sous la même forme que celles de PuTTY.
- **Lecteurs mappés :** chaque lecteur mappé destiné à être restauré à l'ouverture de session, et les chemins saisis dans
  *Connecter un lecteur réseau*.
- **Partages :** les partages montés par l'Explorateur (`MountPoints2`).

Ils montrent où un compte s'est rendu ensuite et avec quels identifiants, même après la disparition des journaux propres à
l'ordinateur distant.

<span class="evidence-strength">Meilleure conclusion :</span> le profil de cet utilisateur s'est connecté à cette cible, ou a enregistré
une connexion vers celle-ci. Une session enregistrée ou un nom saisi ne prouve pas à lui seul qu'une
connexion a réussi. Une clé d'hôte acceptée ou une clé `Servers` comportant un `UsernameHint` est plus probante :
le client a atteint le serveur.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

Collectez `NTUSER.DAT` avec ses journaux de transactions pour chaque profil. Collectez également la ruche `SOFTWARE` du système :
elle fournit le SID de chaque utilisateur.

## Précautions d'analyse { #analyst-cautions }

- **Les heures des clés sont des heures de liste.** `Default`, `SshHostKeys` et `Map Network Drive MRU` contiennent des listes,
  et la dernière écriture de leur clé correspond à la dernière modification de la liste. Cette heure appartient au premier élément
  (`MruPosition` 0). Pour les autres, ce n'est qu'une borne supérieure. Une clé `Servers\<host>`, de session,
  `Network\<letter>` ou de partage possède sa propre heure.
- **Enregistré ne signifie pas utilisé.** Une session PuTTY ou WinSCP peut être enregistrée sans jamais se connecter. PuTTY
  réécrit `Recent sessions` lorsqu'une session enregistrée est lancée, et ajoute une clé d'hôte lorsqu'un serveur est
  atteint pour la première fois.
- **Mots de passe :**
  - WinSCP peut stocker les mots de passe de session, de tunnel et de proxy.
  - PuTTY ne stocke que le mot de passe d'un proxy, en clair.
  - `PasswordStored` et le `StoredPasswords` de `Details` indiquent quelles valeurs sont définies. Elles ne sont jamais
    décodées ni écrites en sortie.
- **WinSCP omet les valeurs par défaut.** Une session sans `FSProtocol` est en SFTP, et une session sans
  `PortNumber` utilise le port par défaut du protocole. `Protocol` et `Port` restent alors vides plutôt
  que d'être devinés.
- **Les empreintes des clés d'hôte** sont au format `SHA256:…` d'OpenSSH, tel que l'affichent `ssh-keygen -l`, `ssh` et PuTTY 0.75
  et versions ultérieures. Juntei les calcule à partir de la clé que conservent PuTTY et WinSCP. Comparez-les avec
  la clé propre au serveur, les fichiers `known_hosts` et les journaux pour identifier le serveur derrière un nom ou une adresse
  IP. Une clé SSH-1 (`rsa`) n'en a pas.
- **Lecteurs mappés :** `RemoteUsername` n'est renseigné que lorsque le lecteur a été mappé avec un autre
  compte ; Windows écrit un `REG_DWORD` 0 lorsqu'il a utilisé le compte propre de l'utilisateur.
- **Partages :** `MountPoints2` contient aussi des clés pour les volumes et les lettres de lecteur. Seuls les partages réseau
  (`##server#share`) sont signalés ici ; `\\server@SSL@443\DavWWWRoot` est un partage WebDAV.
- **Suppression :** effacer les listes, ou utiliser un client portable, ne laisse aucune trace ici. Comparez avec
  les Jump Lists, Prefetch, les fichiers `.rdp`, le cache bitmap RDP et les journaux de la cible.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

Le résumé dénombre les connexions par client. Il liste les connexions dont les clés ont été écrites en dernier,
de la plus récente à la plus ancienne, ainsi que les sessions qui conservent un mot de passe.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Hôte, nom du dossier de profil, SID de l'utilisateur provenant du `ProfileList` du système (avec la ruche `SOFTWARE` du système), et ruche. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` ou `network_share`. |
| `Kind` | La provenance de l'enregistrement : `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | L'ordinateur (nom d'hôte ou adresse IP) et le port, ou le partage (`\\server\share`). Un port n'est indiqué que lorsque Windows ou le client en a écrit un. |
| `RemoteUsername` | Le compte utilisé sur la cible : `UsernameHint`, le `UserName` d'une session (ou le `user@` du `HostName` de PuTTY), le `UserName` d'un lecteur mappé. |
| `Protocol` | Le `Protocol` de PuTTY (`ssh`, `telnet`, `raw`, …), le `FSProtocol` de WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), le fournisseur d'un lecteur mappé (`Microsoft Windows Network`). |
| `MRU-Position` | La place dans sa liste (0 est la plus récente) : `MRU<n>`, `MRUList`, `Recent sessions` de PuTTY. |
| `SessionName` | Le nom d'une session enregistrée, avec `%20` et les autres séquences d'échappement décodées. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | Le type d'une clé d'hôte SSH acceptée (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), son empreinte OpenSSH et ses nombres tels que stockés. |
| `DriveLetter` | La lettre d'un lecteur mappé. |
| `PasswordStored` | Indique si la session conserve un mot de passe ; jamais décodé. |
| `Details` | Autres paramètres, sous la forme `Name=Value; …` (CSV) ou d'un objet (JSON) : le `CertHash` du certificat RDP, `LocalDevices`, le fichier de clé d'une session, le proxy, le tunnel SSH, les redirections de ports, la commande et les répertoires distants, `StoredPasswords`, le `Label` d'un partage, le `ConnectionType` d'un lecteur. |
| `KeyPath`, `ValueName` | La clé, et la valeur pour un enregistrement correspondant à une seule valeur. |
| `KeyLastWrittenUTC` | La dernière écriture de la clé (voir les précautions concernant les listes). |
| `DataRaw` | Les octets de la valeur (hexadécimal), lorsqu'elle n'est pas une chaîne ou ne se décode pas. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs de type incorrect (avec leurs octets), clés d'hôte illisibles, lettres de `MRUList` absentes et valeurs qu'il ne liste pas. |

## Stratégie d'investigation { #investigation-strategy }

1. Listez les cibles de chaque utilisateur par client, et vérifiez-les par rapport aux systèmes que cet utilisateur est censé
   atteindre. Recherchez les partages administratifs (`C$`, `ADMIN$`) et les adresses IP.
2. Notez les comptes utilisés (`RemoteUsername`) : administrateurs de domaine et comptes locaux d'autres
   machines.
3. Faites correspondre `HostKeyFingerprint` avec les clés des serveurs, et `CertHash` avec les certificats des
   serveurs RDP.
4. Corrélez avec les événements d'ouverture de session des cibles (4624 de type 10 et 3, 1149), les Jump Lists et les fichiers LNK
   (`mstsc.exe`, `putty.exe`, fichiers ouverts depuis des partages), Prefetch et le cache bitmap RDP.
5. Utilisez `parse-registry read --recover-deleted` pour les entrées supprimées, et des copies plus anciennes du
   `NTUSER.DAT` du profil (clichés instantanés de volume, sauvegardes) pour les listes effacées.

## Plus de détails { #further-details }

- [Microsoft : supprimer des entrées de la zone Ordinateur de Connexion Bureau à distance](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [FAQ PuTTY : où PuTTY stocke-t-il ses données ?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP : stockage de la configuration](https://winscp.net/eng/docs/config)
- [libyal winreg-kb : Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb : Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
