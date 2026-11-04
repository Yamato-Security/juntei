# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## Qu'est-ce que ShimCache { #what-shimcache-is }

Le sous-système de compatibilité des applications de Windows met en cache des chemins d'exécutables et des métadonnées lors de l'évaluation
du comportement de compatibilité/de shim. Le cache survit au renommage, à la suppression ou à la déconnexion des fichiers avec
un volume amovible. Sa structure et les champs disponibles diffèrent de Windows XP à Windows 11.

ShimCache est précieux pour découvrir des chemins historiques et, lorsque le format comporte un indicateur d'exécution
fiable, pour étayer davantage une exécution. L'heure mise en cache est normalement **l'heure de dernière modification du fichier**,
et non l'heure à laquelle Windows l'a vu ou exécuté.

<span class="evidence-strength">Meilleure conclusion :</span> Windows a observé le chemin lors du traitement de
compatibilité. Une entrée seule est une preuve de présence/d'observation ; n'utilisez `Executed` que lorsqu'il est pris en charge.

## Emplacement et collecte { #location-and-collection }

Les octets se trouvent dans la ruche du Registre `SYSTEM`, généralement sous :

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Collectez `SYSTEM`, `SYSTEM.LOG1` et `SYSTEM.LOG2`. Juntei lit tous les jeux de contrôles et identifie
`Select\Current` lorsque c'est possible.

!!! warning "Les entrées actives les plus récentes peuvent ne pas figurer dans la ruche"
    Windows peut conserver les mises à jour de ShimCache en mémoire et les écrire lors de l'arrêt/du redémarrage. Une ruche copiée
    depuis un système en cours d'exécution peut ne pas contenir les observations les plus récentes. L'heure de dernière écriture du cache dans le Registre
    aide à décrire le moment où il a été enregistré, mais n'est pas une heure d'exécution par entrée.

## Précautions d'analyse { #analyst-cautions }

- Une entrée du cache n'est pas, à elle seule, une **preuve d'exécution**.
- `Executed` peut valoir `Yes`, `No` ou `NA` ; la sémantique disponible dépend du format/de la version.
- `LastModifiedTimeUTC` appartient au fichier référencé, et non à l'observation ou à l'exécution.
- L'ordre est utile mais ne constitue pas une chronologie précise universelle ; des doublons apparaissent entre jeux de contrôles/slack.
- Les entrées `FromSlack` ont été récupérées en dehors du cache actif et exigent une prudence supplémentaire.
- Un `SYSTEM` incohérent (dirty) sans journaux exploitables peut omettre des enregistrements actuels. Lisez la synthèse de récupération.
- Les constats signalent des schémas de nommage/de chemin suspects, et non des verdicts de logiciel malveillant.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `ControlSet` | Numéro du jeu de contrôles contenant le cache. Comparez `IsCurrent` avant d'en choisir un. |
| `CacheEntryPosition` | Position/ordre au sein du cache décodé. Utile pour l'ordre relatif, ce n'est pas une heure. |
| `Path` | Chemin de l'exécutable/du package mis en cache. Pivot de recherche principal. |
| `LastModifiedTimeUTC` | Heure de modification du fichier stockée dans l'entrée, et non heure d'exécution/d'observation. |
| `Executed` | `Yes`, `No` ou `NA` selon les indicateurs du format pris en charge. À n'utiliser qu'avec le contexte de version. |
| `Duplicate` | La même entrée a déjà été vue dans un jeu de contrôles antérieur ou dans le slack ; permet d'éviter un double comptage. |
| `SourceFile` | Ruche/valeur brute d'entrée. Essentiel lorsque plusieurs hôtes sont traités. |
| `FromSlack` | Entrée récupérée dans le slack de la valeur du cache plutôt que dans la liste active. Confiance moindre, et possiblement plus ancienne. |
| `CacheFormat` | Structure Windows/de cache décodée. Détermine la disponibilité et la sémantique des champs. |
| `CacheLastWrittenUTC` | Heure d'enregistrement de la clé du Registre/du cache. Il s'agit d'un contexte partagé, et non de l'heure d'exécution de chaque entrée. |
| `InsertFlags`, `ShimFlags` | Indicateurs de compatibilité bruts ; `Executed` est le résultat interprété par Juntei lorsqu'il est pris en charge. |
| `DataSize` | Taille des données de cette entrée, utile pour la validation structurelle. |
| `Machine` | Architecture enregistrée par les formats récents (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Métadonnées mises en cache propres à Windows XP, lorsqu'elles sont présentes. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Identité de l'application empaquetée dans les entrées prises en charge. L'ID d'éditeur est un contexte, et non une preuve de signature. |
| `Findings` | Codes d'anomalie de Juntei, tels qu'un binaire système déplacé ou un chemin de préparation (staging) suspect. Simples pistes. |
| `RegistryPath`, `RegistryValueName` | Clé/valeur exacte de la ruche source. À conserver pour la validation. |
| `CacheValueSize` | Taille de la valeur de Registre entière, et non le `DataSize` de l'entrée individuelle. |
| `IsCurrent` | Indique si le jeu de contrôles de cette entrée est égal à `SYSTEM\Select\Current` ; vide si cela n'est pas connu de manière sûre. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Numéros bruts des jeux de contrôles de `SYSTEM\Select`, pour le contexte et l'analyse de récupération. |

JSON/JSONL conserve en outre les données des entrées, les balises, les sous-enregistrements d'applications empaquetées, les constats structurés,
les problèmes du cache et les détails de récupération.

## Stratégie d'investigation { #investigation-strategy }

1. Privilégiez le jeu de contrôles courant, mais conservez les autres jeux et le slack comme preuves historiques.
2. Recherchez dans les chemins et les constats ; n'utilisez pas `LastModifiedTimeUTC` comme heure d'exécution.
3. Lorsque `Executed == Yes`, corroborez avec Prefetch, SRUM, les journaux d'événements ou d'autres traces d'exécution.
4. Rapprochez les chemins de `$MFT`/USN pour déterminer la création, la suppression et l'historique des métadonnées.
5. Indiquez si les journaux de transactions ont été rejoués et si le cache actif a pu ne pas être écrit sur disque.

## Plus de détails { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [AppCompatCacheParser d'Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)
- [shimcache-forensic de Security Ronin](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant : Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Spécification du format du Registre Windows de Maxim Suhanov](https://github.com/msuhanov/regf)
