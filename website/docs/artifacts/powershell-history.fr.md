# Historique PowerShell { #powershell-history }

## Ce que l’historique enregistre { #what-the-history-records }

PSReadLine sauvegarde les lignes de commande saisies dans un hôte PowerShell interactif. Le fichier contient le texte dans l’ordre de sauvegarde, sans horodatage par commande. L’historique peut être modifié, effacé, désactivé ou filtré et ne prouve pas qu’une commande s’est terminée avec succès. Recoupez l’exécution et sa chronologie avec les journaux d’événements PowerShell, la télémétrie des processus et d’autres artefacts d’exécution.

## Emplacement et collecte { #location-and-collection }

Sous Windows, l’emplacement par défaut pour Windows PowerShell 5.1 et PowerShell 7+ est :

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Les autres hôtes utilisent dans ce répertoire des noms comme `Visual Studio Code Host_history.txt`. Sous Linux et macOS, PowerShell utilise normalement `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. Le paramètre `HistorySavePath` de PSReadLine peut rediriger le fichier ; ces chemins par défaut sont des hypothèses de recherche, pas une liste exhaustive des emplacements.

L’entrée par répertoire et `auto-parse` sélectionnent récursivement les fichiers `<Host>_history.txt` situés directement dans un dossier `PSReadLine`, sans tenir compte de la casse ASCII. Ils ne sélectionnent pas les fichiers `*_history.txt` situés ailleurs. Utilisez explicitement `-f` pour un fichier renommé ou une copie hors d’un répertoire `PSReadLine`. Préservez si possible la structure du profil d’origine pour l’attribution à un utilisateur.

## Analyser avec Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV est le format de fichier par défaut. Avec `-t csv`, `-t json` ou `-t jsonl` sans `-o`, les enregistrements sont écrits sur stdout. Sans ces deux options, seul un résumé s’affiche. JSON produit des objets distincts mis en forme ; JSONL produit un objet par ligne. Ajoutez `-C` pour écraser une sortie existante ; les alias du fichier source sont protégés contre le remplacement.

`auto-parse` enregistre `ShellHistory/PowerShellHistory.csv` (ou le format JSON choisi), y compris pour les images VHDX/E01. L’historique ne fournit aucun événement à `auto-parse -j` ni à `super-timeline`.

## Encodage et traitement des lignes { #encoding-and-line-handling }

- Une marque d’ordre des octets UTF-8, UTF-16LE ou UTF-16BE détermine l’encodage et est retirée au début du fichier. Sans BOM, l’entrée est toujours traitée en UTF-8. UTF-16 sans BOM n’est pas deviné ; les caractères NUL sont conservés et signalés comme une possible altération de l’encodage.
- Les séquences UTF-8 invalides et les substituts UTF-16 non appariés deviennent U+FFFD, et l’altération est signalée dans `Problems`. Un octet UTF-16 isolé en fin de fichier et les erreurs de lecture sont aussi signalés ; les lignes lisibles sont conservées. Consultez `Problems` et le journal du parseur avant de conclure à l’exhaustivité.
- Les fichiers sont traités en flux. Au maximum 1,048,576 octets d’entrée sont conservés par ligne physique, hors BOM et séparateur LF (un CR avant LF compte dans la limite). Les lignes plus longues sont tronquées à la frontière d’un caractère entier ; les octets restants sont consommés et `Problems` indique le nombre d’octets omis. La limite porte sur les octets d’entrée encodés, pas sur le nombre de caractères.
- Les fins de ligne LF et CRLF sont retirées, ainsi qu’un CR final sur une ligne non terminée. Les autres espaces et tabulations sont conservés. Les lignes vides ne produisent pas d’enregistrement, mais comptent dans les numéros de ligne et terminent une chaîne de continuation ; les lignes composées uniquement de caractères d’espacement restent des enregistrements. Une erreur de lecture peut produire une ligne de diagnostic vide pour indiquer où la lecture s’est arrêtée.
- PSReadLine stocke les entrées multilignes avec un accent grave final sur chaque ligne physique poursuivie. Les lignes restent séparées, avec les accents graves et l’indentation. `EntryStartLine` désigne la première ligne d’une chaîne consécutive dont les lignes précédentes se terminent par un accent grave. Aucune syntaxe shell n’est interprétée et aucune commande n’est reconstruite. La troncature peut affecter l’attribution des continuations ; consultez `Problems` avant de regrouper ces lignes.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `SourceFile` | Chemin relatif au répertoire d’entrée, avec `/` ; avec `-f`, uniquement le nom du fichier. Aucun chemin source absolu n’est exporté. |
| `User` | Nom du profil déduit du chemin conservé. L’entrée par répertoire reconnaît aussi les structures `Users`/`Documents and Settings` sous la racine d’entrée ; `-f` exige le suffixe habituel du chemin de profil PSReadLine. Les valeurs inconnues sont vides en CSV et `null` en JSON. |
| `ShellHost` | Préfixe du nom de fichier avant `_history.txt`, par exemple `ConsoleHost` ; inconnu pour les fichiers renommés sans ce suffixe. |
| `LineNumber` | Numéro de ligne physique à partir de 1, y compris les lignes vides ignorées. |
| `EntryStartLine` | Première ligne physique de la chaîne de continuation, ou numéro de cette ligne elle-même. |
| `Command` | Ligne physique décodée sans sa fin de ligne, soumise à la limite documentée et à la protection CSV. |
| `FileLastWriteTime` | Date de modification du fichier source, répétée sur chaque ligne. **Métadonnée du fichier, jamais une date d’exécution de commande.** L’analyse d’image utilise les métadonnées sources d’origine, pas la date de staging. |
| `FileSize` | Taille du fichier source en octets. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` ou `UTF-16BE`. |
| `Problems` | Diagnostics d’altération et de troncature des lignes ; séparés par des points-virgules en CSV, sous forme de tableau en JSON/JSONL. Les diagnostics du fichier figurent aussi dans le journal. |

`FileLastWriteTime` utilise par défaut UTC au format ISO 8601, par exemple `2026-10-20T08:46:38.5Z`. Seul `auto-parse` accepte `-Z, --timezone` pour modifier le décalage affiché. La commande autonome `parse-file powershell-history` n’a pas d’option de fuseau horaire.

Chaque cellule de texte CSV est contrôlée pour détecter les préfixes de formules de tableur (`=`, `+`, `-`, `@`, y compris les formes pleine chasse et les préfixes après des espaces ou certains caractères invisibles/de contrôle). Les cellules concernées et celles commençant par un caractère de contrôle reçoivent une apostrophe initiale. Cela peut modifier les commandes et les chemins en CSV. JSON et JSONL conservent le texte décodé sans l’apostrophe CSV ; les remplacements au décodage et la troncature restent applicables. Utilisez JSONL pour comparer le texte des commandes sauvegardées et conservez toujours les fichiers d’historique sources.
