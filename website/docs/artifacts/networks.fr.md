# Réseaux { #networks }

## De quoi s'agit-il { #what-they-are }

Windows mémorise les réseaux auxquels un système s'est connecté et la façon dont son réseau était configuré.

- **`SOFTWARE` :**
  - **Profils réseau** (`NetworkList\Profiles`) : un par réseau auquel le système s'est connecté,
    avec son nom (le SSID d'un réseau Wi-Fi), sa catégorie (public, privé, domaine), son type de connexion
    (filaire, sans fil, VPN, haut débit mobile), ainsi que la date de la première et de la dernière connexion du système à celui-ci.
  - **Signatures** (`NetworkList\Signatures`) : ce qui identifie chaque réseau, comme l'adresse
    MAC de la passerelle et le suffixe DNS.
  - **Cartes réseau** (`NetworkCards`) : les adaptateurs.
- **`SYSTEM`** (par jeu de contrôle) :
  - **Interfaces TCP/IP :** adresses, serveur et bail DHCP, serveurs DNS, domaine.
  - **Partages** proposés par le système (`LanmanServer\Shares`).
  - **Proxys de ports** (`PortProxy`) : les tunnels configurés par `netsh interface portproxy`, que
    les attaquants utilisent pour pivoter et pour atteindre RDP via un autre port.
  - **Routes persistantes.**
  - **Le fuseau horaire**, qui permet de convertir en UTC les heures locales des profils.

<span class="evidence-strength">Meilleure conclusion :</span> le système s'est connecté à un réseau portant
ce nom et cette passerelle, pour la première et la dernière fois à ces heures. Il disposait de cette adresse fournie par ce serveur DHCP,
et proposait ces partages et ces tunnels au moment où la ruche a été enregistrée. Le nom d'un réseau ne suffit pas à
identifier où il se trouvait : de nombreux réseaux s'appellent « Network » ou partagent un SSID.

## Emplacement { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

Collectez les deux ruches avec leurs journaux de transactions. La ruche SYSTEM est nécessaire pour convertir les heures
des profils en UTC.

## Précautions d'analyse { #analyst-cautions }

- **Les heures des profils sont locales.** `DateCreated` et `DateLastConnected` sont écrites dans l'heure
  locale du système.
  - `FirstConnectedLocal` et `LastConnectedLocal` les conservent telles quelles.
  - `FirstConnectedUTC` et `LastConnectedUTC` les convertissent avec le fuseau horaire de la ruche
    SYSTEM du système (`TimeZone`) et sa règle d'heure d'été.
  - Une heure antérieure à un changement de fuseau horaire, ou provenant d'une année ayant une autre règle
    d'heure d'été (dates américaines antérieures à 2007), peut être décalée de la différence.
  - Sans ruche SYSTEM lisible et sans jeu de contrôle actuel valide, seules les heures locales sont fournies. Les heures sautées lors du passage à l'heure d'été et les heures répétées ambiguës n'ont pas d'horodatage UTC unique prouvé : l'UTC est laissé vide et `Problems` explique la limitation. Les décalages ou dates invalides ne sont pas non plus convertis.
- **Première et dernière uniquement.** Chaque profil possède une première et une dernière connexion, et non chaque
  connexion. La dernière écriture de la clé du profil correspond généralement à la dernière connexion.
- **Les noms se répètent.** Windows nomme les réseaux filaires non gérés « Network », « Network 2 », etc.
  Les SSID Wi-Fi ne sont pas uniques. Utilisez `GatewayMac` et `DnsSuffix` pour distinguer les réseaux, et
  corrélez-les avec les journaux DHCP et l'inventaire des actifs. L'adresse MAC d'une passerelle n'est pas nécessairement le BSSID d'un point d'accès Wi-Fi et ne permet pas à elle seule d'établir un emplacement physique.
- **Les partages, proxys et routes** relèvent de la configuration : un partage ou un proxy de port peut être configuré et
  ne jamais être utilisé. Chaque valeur n'a pas d'horodatage propre ; la dernière écriture de la clé correspond au moment où l'une d'entre elles
  a été modifiée.
- **Interfaces :** une clé d'interface sous un GUID sans adresse correspond à un adaptateur qui n'est pas
  configuré. Avec DHCP, les adresses sont celles du dernier bail. Une adresse statique restant d'avant DHCP n'est
  pas affichée comme étant utilisée.
- **Jeux de contrôle :** par défaut, le jeu de contrôle utilisé en dernier par Windows est lu. `--all-control-sets`
  les lit tous, ce qui peut révéler un partage ou un proxy supprimé du jeu actuel.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

Le résumé répertorie les réseaux auxquels le système s'est connecté en dernier (les plus récents en premier), ainsi que chaque partage, proxy de port et
route persistante.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | L'hôte et la ruche. |
| `ControlSet`, `IsCurrentControlSet` | Pour les enregistrements SYSTEM : le jeu de contrôle, et s'il s'agit de celui utilisé en dernier par Windows. |
| `Source` | `network_profile`, `network_signature` (une signature sans profil), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | Le nom d'un profil (le SSID d'un réseau Wi-Fi) ; la description d'une signature. |
| `Category`, `ConnectionType` | `public`, `private`, `domain` ; `wired`, `wireless`, `vpn`, `mobile_broadband` (sinon le nombre). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` et `DateLastConnected` converties en UTC avec `TimeZone`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Les mêmes, telles quelles (heure locale, à la milliseconde). |
| `TimeZone` | Le fuseau horaire utilisé pour la conversion. |
| `GatewayMAC`, `DNS-Suffix` | L'adresse MAC de la passerelle et le suffixe DNS des signatures du profil (`|` entre plusieurs valeurs). |
| `InterfaceGUID` | Le GUID d'une interface ; le `ServiceName` d'une carte réseau. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Les adresses utilisées par une interface, son serveur DHCP et son bail. |
| `ShareName`, `SharePath` | Un partage et le dossier qu'il propose. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Un proxy de port : ce sur quoi il écoute et vers où il redirige. |
| `Route` | Une route persistante telle que stockée : `destination,netmask,gateway,metric`. |
| `Details` | Autres valeurs, sous la forme `Name=Value; …` (CSV) ou d'un objet (JSON) : la description d'un profil, `Managed`, le premier nom et les clés de signature ; l'adaptateur d'une interface (provenant de `NetworkCards`), les masques, passerelles, serveurs DNS et le domaine ; le type, la remarque et les autres champs d'un partage ; le type d'un proxy ; les composantes d'une route. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | La clé, la valeur pour un partage, un proxy ou une route, et la dernière écriture de la clé. |
| `DataRaw` | Octets qui ne se décodent pas : une heure qui n'est pas un `SYSTEMTIME` valide, un partage qui n'est pas une liste de chaînes. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs de type ou de taille incorrects, dates impossibles et noms qui ne respectent pas leur forme. |

## Stratégie d'investigation { #investigation-strategy }

1. Triez les profils par `LastConnectedUTC` et examinez les réseaux autour de l'incident : réseaux
   Wi-Fi inconnus, points d'accès mobiles, VPN.
2. Utilisez `GatewayMac` et `DnsSuffix` pour distinguer les réseaux portant le même nom, et pour faire correspondre un
   réseau entre plusieurs hôtes.
3. Examinez chaque proxy de port et chaque partage. Un proxy de port vers le port 3389 ou 445 d'un hôte interne, ou le partage
   d'un dossier de préparation (`C:\ProgramData`, `C:\Users\Public`), constitue une piste. Corrélez avec les journaux
   d'événements (création de processus `netsh`, événements de partage 5142/5144) et les horodatages des clés.
4. Utilisez les adresses et les heures de bail des interfaces pour situer l'hôte sur le réseau, et faites-les correspondre
   avec les journaux du pare-feu, du DHCP et du proxy.
5. Utilisez `--all-control-sets` et `parse-registry read --recover-deleted` pour les partages,
   proxys et profils supprimés.

## Plus de détails { #further-details }

- [Microsoft : netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft : TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001 : Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
