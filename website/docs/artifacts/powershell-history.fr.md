# Historique PowerShell { #powershell-history }

## Ce que l’historique enregistre { #what-the-history-records }

PSReadLine sauvegarde les lignes de commande saisies dans un hôte PowerShell interactif. Le fichier contient le texte dans l’ordre de sauvegarde, sans horodatage par commande.

## Emplacements et collecte { #locations-and-collection }

Sous Windows, l’emplacement par défaut pour Windows PowerShell 5.1 et PowerShell 7+ est :

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Les autres hôtes utilisent dans ce répertoire des noms comme `Visual Studio Code Host_history.txt`. Sous Linux et macOS, PowerShell utilise normalement `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. Le paramètre `HistorySavePath` de PSReadLine peut rediriger le fichier ; ces chemins par défaut sont des hypothèses de recherche, pas une liste exhaustive des emplacements.

L’entrée par répertoire et `auto-parse` sélectionnent récursivement les fichiers `<Host>_history.txt` situés directement dans un dossier `PSReadLine`, sans tenir compte de la casse ASCII. Ils ne sélectionnent pas les fichiers `*_history.txt` situés ailleurs. Utilisez explicitement `-f` pour un fichier renommé ou une copie hors d’un répertoire `PSReadLine`. Préservez si possible la structure du profil d’origine pour l’attribution à un utilisateur.

## Précautions d’analyse { #analyst-cautions }

- Aucune ligne n’a de date. `FileLastWriteTime` est la date de modification du fichier. Par défaut, PSReadLine sauvegarde l’historique après chaque commande (`HistorySaveStyle` `SaveIncrementally`) ; cette date est donc généralement proche du moment où la dernière ligne a été ajoutée, mais elle n’indique rien sur les lignes précédentes.
- Une ligne sauvegardée ne prouve pas qu’une commande a été exécutée ni qu’elle s’est terminée avec succès. Recoupez l’exécution et sa chronologie avec les journaux d’événements PowerShell, la télémétrie des processus et d’autres artefacts d’exécution.
- L’historique peut être modifié, supprimé ou désactivé (`HistorySaveStyle` `SaveNothing`). PSReadLine omet aussi les lignes qui semblent contenir des secrets, comme celles qui comportent `password`, `token` ou `apikey`, et Windows PowerShell ISE n’utilise pas PSReadLine. Les scripts et les commandes exécutées avec `-Command` ou `-EncodedCommand` ne sont pas saisis ; ils ne sont donc pas sauvegardés.
- Une commande de plusieurs lignes est sauvegardée avec un accent grave à la fin de chaque ligne sauf la dernière. Juntei conserve ces lignes comme des enregistrements distincts, reliés par `EntryStartLine`. Une ligne qui se termine par un accent grave saisi a le même aspect ; lisez donc ensemble les lignes d’une entrée.

## Analyser avec Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV est le format de fichier par défaut. Avec `-t csv`, `-t json` ou `-t jsonl` sans `-o`, les enregistrements sont écrits sur stdout. Sans ces deux options, seul un résumé s’affiche. JSON produit des objets distincts mis en forme ; JSONL produit un objet par ligne. Ajoutez `-C` pour écraser une sortie existante ; les alias du fichier source sont protégés contre le remplacement.

## Encodage et traitement des lignes { #encoding-and-line-handling }

- Une marque d’ordre des octets UTF-8, UTF-16LE ou UTF-16BE détermine l’encodage et est retirée au début du fichier. Sans BOM, l’entrée est toujours traitée en UTF-8. UTF-16 sans BOM n’est pas deviné ; les caractères NUL sont conservés et signalés comme une possible altération de l’encodage.
- Les séquences UTF-8 invalides et les substituts UTF-16 non appariés deviennent U+FFFD, et l’altération est signalée dans `Problems`. Un octet UTF-16 isolé en fin de fichier et les erreurs de lecture sont aussi signalés ; les lignes lisibles sont conservées. Consultez `Problems` et le journal du parseur avant de conclure à l’exhaustivité.
- Les fichiers sont traités en flux. Au maximum 1,048,576 octets d’entrée sont conservés par ligne physique, hors BOM et séparateur LF (un CR avant LF compte dans la limite). Les lignes plus longues sont tronquées à la frontière d’un caractère entier ; les octets restants sont consommés et `Problems` indique le nombre d’octets omis. La limite porte sur les octets d’entrée encodés, pas sur le nombre de caractères.
- Les fins de ligne LF et CRLF sont retirées, ainsi qu’un CR final sur une ligne non terminée. Les autres espaces et tabulations sont conservés. Les lignes vides ne produisent pas d’enregistrement, mais comptent dans les numéros de ligne et terminent une chaîne de continuation ; les lignes composées uniquement de caractères d’espacement restent des enregistrements. Une erreur de lecture peut produire une ligne de diagnostic vide pour indiquer où la lecture s’est arrêtée.
- Les lignes restent séparées, avec les accents graves et l’indentation. `EntryStartLine` désigne la première ligne d’une chaîne consécutive dont les lignes précédentes se terminent par un accent grave. Aucune syntaxe shell n’est interprétée et aucune commande n’est reconstruite. La troncature peut affecter l’attribution des continuations ; consultez `Problems` avant de regrouper ces lignes.

## Champs de sortie CSV { #csv-output-fields }

| Champ | Interprétation |
|---|---|
| `SourceFile` | Chemin relatif au répertoire d’entrée, avec `/` ; avec `-f`, uniquement le nom du fichier. Aucun chemin source absolu n’est exporté. |
| `User` | Nom du profil tiré du chemin. Le dossier situé au-dessus de `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (ou de `.local/share/powershell/PSReadLine`) nomme l’utilisateur, y compris lorsque l’entrée est ce profil ou le dossier `PSReadLine` lui-même. L’entrée par répertoire reconnaît aussi les structures `Users`/`Documents and Settings` sous la racine d’entrée. Les valeurs inconnues sont vides en CSV et `null` en JSON. |
| `ShellHost` | Préfixe du nom de fichier avant `_history.txt`, par exemple `ConsoleHost` ; inconnu pour les fichiers renommés sans ce suffixe. |
| `LineNumber` | Numéro de ligne physique à partir de 1, y compris les lignes vides ignorées. |
| `EntryStartLine` | Première ligne physique de la chaîne de continuation, ou numéro de cette ligne elle-même. |
| `Command` | Ligne physique décodée sans sa fin de ligne, soumise à la limite documentée. |
| `FileLastWriteTime` | Date de modification du fichier source, répétée sur chaque ligne. **Métadonnée du fichier, jamais une date d’exécution de commande.** L’analyse d’image utilise les métadonnées sources d’origine, pas la date de staging. |
| `FileSize` | Taille du fichier source en octets. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` ou `UTF-16BE`. |
| `Problems` | Diagnostics d’altération et de troncature des lignes ; séparés par des points-virgules en CSV, sous forme de tableau en JSON/JSONL. Les diagnostics du fichier figurent aussi dans le journal. |

Les sorties JSON et JSONL comportent les mêmes champs, avec les nombres typés, `null` pour les valeurs inconnues et `Problems` sous forme de liste. Tous les formats conservent le texte des commandes tel qu’il a été sauvegardé.

`FileLastWriteTime` utilise par défaut UTC au format ISO 8601, par exemple `2026-10-20T08:46:38.5Z`. Seul `auto-parse` accepte `-Z, --timezone` pour modifier le décalage affiché. La commande autonome `parse-file powershell-history` n’a pas d’option de fuseau horaire.

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit `ShellHistory/PowerShellHistory.csv` (`PowerShellHistory.json` ou `PowerShellHistory.jsonl` avec `-t json` ou `-t jsonl`), accompagné de `ShellHistory/PowerShellHistory-logs.txt`. Depuis des images VHDX et E01, les fichiers d’historique sont copiés, et `FileLastWriteTime` est la date enregistrée dans l’image. La chronologie combinée `-j` inclut chaque ligne, avec l’identifiant d’`Artifact` `powershell-history`. Les lignes n’ont pas de dates ; [`super-timeline`](../commands/super-timeline.md) n’en produit donc aucun événement et les compte comme des enregistrements sans date.

## Stratégie d’investigation { #investigation-strategy }

1. Lisez l’historique de chaque profil, y compris `Windows\System32\config\systemprofile` et les profils situés sous `Windows\ServiceProfiles`, qui contiennent les commandes saisies dans des shells exécutés sous ces comptes.
2. Lisez ensemble les entrées multilignes et recherchez les téléchargements, les commandes encodées, les modifications des paramètres de sécurité, les commandes liées aux comptes et aux identifiants, ainsi que les connexions à distance.
3. Recoupez avec les journaux d’événements PowerShell (journalisation des blocs de script, événement 4104), les événements de création de processus et les fichiers Prefetch de `powershell.exe` et de `pwsh.exe`.
4. Comparez `FileLastWriteTime` avec les dates `$MFT` et du journal USN du fichier d’historique, ainsi qu’avec les ouvertures de session, pour cerner le moment où les dernières lignes ont été ajoutées.

## Plus de détails { #further-details }

- [Microsoft — about_PSReadLine : historique des commandes](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption : HistorySavePath et HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows : journalisation des blocs de script](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — comment le fichier d’historique est sauvegardé et lu](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
