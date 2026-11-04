# Listes MRU des éléments récemment utilisés (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## Que sont ces listes { #what-the-lists-are }

Windows et ses programmes conservent de courtes listes de ce qu'un utilisateur a saisi ou ouvert en dernier, dans le fichier
`NTUSER.DAT` de l'utilisateur :

- **RunMRU :** commandes saisies dans la boîte de dialogue Exécuter (Win+R) : `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths :** chemins saisis dans la barre d'adresse de l'Explorateur.
- **WordWheelQuery :** termes recherchés dans la zone de recherche de l'Explorateur.
- **TypedURLs :** adresses saisies dans la barre d'adresse d'Internet Explorer et, depuis Windows 8,
  `TypedURLsTime`, le moment où chacune a été saisie.
- **Paint et WordPad :** les fichiers que chacun a ouverts en dernier (`Recent File List`).
- **ACMru (Windows XP) :** les termes de l'Assistant Recherche, par type de recherche.

<span class="evidence-strength">Meilleure conclusion :</span> l'utilisateur a saisi cette commande, ce chemin,
ce terme de recherche ou cette adresse, ou le programme a ouvert ce fichier. Seule l'entrée la plus récente de chaque liste
est datée, par l'heure de dernière écriture de sa clé, à l'exception des adresses saisies, que `TypedURLsTime` date.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

Les sous-clés de `ACMru` correspondent aux types de recherche : `5603` noms de fichiers et de dossiers, `5604` mots ou expressions dans
les fichiers, `5647` ordinateurs et personnes, `5001` Internet.

Collectez `NTUSER.DAT` avec ses journaux de transactions pour chaque profil.

## Précautions d'analyse { #analyst-cautions }

- **Un seul horodatage par liste.** L'heure de dernière écriture d'une clé change lorsque sa liste change ; elle ne date donc que
  l'entrée la plus récente (`IsMostRecent`, `LastUsedUTC`). Les autres sont ordonnées, mais non datées.
  `TypedURLsTime` date chaque adresse saisie (`TypedTimeUTC`).
- **Saisi ne signifie pas exécuté.** RunMRU enregistre ce qui a été saisi dans la boîte de dialogue Exécuter, que la commande se soit
  exécutée avec succès ou non. Windows stocke chaque commande avec un `\1` final ; Juntei le supprime. Corrélez
  avec Prefetch, BAM, UserAssist et les événements de création de processus.
- **Les listes sont courtes.** Chacune ne conserve que ses entrées les plus récentes, et les paramètres de confidentialité ainsi que les outils
  de nettoyage les effacent. Utilisez `parse-registry read --recover-deleted` pour les valeurs supprimées.
- **Les termes sont ce qui a été saisi.** WordWheelQuery et ACMru conservent le texte saisi, et non ce qui a été trouvé.
- **L'ordre provient de la liste.** `MRUList` de RunMRU et `MRUListEx` de WordWheelQuery donnent
  l'ordre ; pour les autres, c'est le numéro de la valeur qui le donne (`url1`, `File1` et `000` sont les plus récentes).

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

Le résumé indique le nombre d'entrées par liste et les dernières entrées datées (adresses saisies et entrée la plus récente de chaque
liste), de la plus récente à la plus ancienne.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Hôte, propriétaire du profil (d'après le chemin ou les dossiers du shell), SID issu de `ProfileList` du système, et ruche. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` ou `ACMru`. |
| `Subkey` | Sous-clé de `ACMru` : le type de recherche (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | Position dans la liste (0 correspond à la plus récente) ; vide lorsque la liste ne nomme pas la valeur. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | La commande (sans son `\1`), le chemin, le terme de recherche, l'adresse ou le fichier. |
| `TypedTimeUTC` | Moment où l'adresse a été saisie, d'après `TypedURLsTime`. |
| `LastUsedUTC` | Heure de dernière écriture de la clé, uniquement pour l'entrée la plus récente. |
| `KeyPath`, `KeyLastWrittenUTC` | La clé et sa dernière écriture : l'heure de son entrée la plus récente, et non de chaque ligne. |
| `DataRaw` | Les octets de la valeur (en hexadécimal) lorsqu'ils ne peuvent pas être décodés. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs que la liste nomme mais que la clé ne contient pas, valeurs qu'elle ne nomme pas, valeurs mal nommées, types incorrects, termes tronqués, horodatages qui ne sont pas des FILETIME. |

## Stratégie d'investigation { #investigation-strategy }

1. Lisez RunMRU pour repérer les commandes, scripts, partages réseau et outils d'administration saisis par l'utilisateur.
2. Lisez TypedPaths et WordWheelQuery pour repérer les dossiers et fichiers que l'utilisateur a recherchés :
   identifiants, exports, archives, emplacements amovibles et réseau.
3. Placez les adresses saisies sur la chronologie à l'aide de `TypedTimeUTC`.
4. Corrélez avec Prefetch, BAM, UserAssist, ShellBags, les fichiers LNK, les Jump Lists et l'historique du navigateur.

## Plus de détails { #further-details }

- [libyal winreg-kb : clés Most recently used (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
