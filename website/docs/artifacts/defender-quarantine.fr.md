# Quarantaine Microsoft Defender { #microsoft-defender-quarantine }

## Contenu de la quarantaine { #what-the-quarantine-contains }

La quarantaine de Microsoft Defender conserve des métadonnées sur une détection ainsi que le contenu chiffré des
ressources mises en quarantaine. Juntei associe les enregistrements Entries, ResourceData et Resources pour indiquer
le libellé de la menace, l'heure de mise en quarantaine, les chemins logique et physique d'origine, les métadonnées du fichier, le propriétaire,
le `Zone.Identifier` d'origine du téléchargement, les empreintes et les problèmes d'intégrité.

Cet artefact peut survivre aux journaux d'événements qui avaient initialement enregistré la détection/la correction. Il peut
également conserver les octets suspects exacts en vue d'une rétro-ingénierie dans un environnement isolé.

<span class="evidence-strength">Meilleure conclusion :</span> Defender a créé l'enregistrement de quarantaine pour
la menace/ressource nommée à l'heure enregistrée. Le libellé du produit n'est pas une preuve indépendante de la présence d'un logiciel malveillant,
et la mise en quarantaine seule ne prouve pas une exécution antérieure.

## Emplacement et collecte { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

Collectez l'intégralité du répertoire `Quarantine`. Une entrée sans fichiers de contenu/d'enregistrement correspondants reste
une métadonnée utile, mais ne permet pas d'obtenir les octets du fichier ni les empreintes calculées.

## Sécurité et précautions d'analyse { #safety-and-analyst-cautions }

- Le contenu extrait est probablement malveillant. Utilisez un système d'analyse isolé et ne l'exécutez jamais.
- `ThreatName` est la signature/classification de Defender à ce moment-là. Des faux positifs et une
  reclassification ultérieure sont possibles.
- L'heure de mise en quarantaine est l'heure de correction/de mise en quarantaine, pas nécessairement celle du téléchargement, de la création ou de l'exécution.
- Un fichier mis en quarantaine peut avoir été détecté avant son exécution ou après celle-ci.
- Le chemin d'origine et le Zone.Identifier sont précieux, mais peuvent être absents ou contrôlés par l'attaquant.
- Des enregistrements orphelins peuvent résulter de la rétention/du nettoyage, d'une collecte incomplète ou d'une corruption.
- Juntei vérifie les tailles, les CRC, les identifiants et les empreintes ; examinez `Problems` avant de vous fier aux octets récupérés.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

Par exemple, `tool.exe` devient `tool.exe.quarantined`. Les flux auxiliaires incluent leur type dans
le nom, par exemple `tool.exe.Zone.Identifier.quarantined`. Juntei supprime également l'autorisation d'exécution.
L'extension et les autorisations réduisent le risque d'exécution accidentelle, mais les octets récupérés doivent tout de même être
traités comme un logiciel malveillant.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Record` | `resource`, entrée sans ressources, `orphan-content` ou `orphan-record`. Les orphelins révèlent une liaison/collecte incomplète. |
| `QuarantineTime` | Heure de mise en quarantaine/de correction par Defender en UTC, et non heure d'exécution. |
| `ThreatName`, `ThreatID` | Classification de Defender et identifiant numérique. À traiter comme un verdict/contexte du fournisseur. |
| `ResourceType` | Fichier, clé de Registre, service, élément de démarrage, tâche planifiée ou autre catégorie de ressource. |
| `Path`, `PhysicalPath` | Emplacement logique/physique d'origine. Pivot de corrélation principal ; conservez l'orthographe exacte. |
| `FileSize` | Taille enregistrée par les métadonnées de la ressource. À comparer avec `ContentSize` et la MFT. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Métadonnées capturées pour la ressource mise en quarantaine ; distinctes de l'heure de mise en quarantaine et vulnérables au timestomping. |
| `Attr` | Attributs de fichier enregistrés. Le statut masqué/système peut constituer un contexte utile. |
| `Owner` | SID issu du descripteur de sécurité récupéré ; le propriétaire n'est pas nécessairement l'utilisateur ayant exécuté le fichier. |
| `ZoneID`, `ReferrerURL`, `HostURL` | Zone de téléchargement et valeurs d'URL issues de `Zone.Identifier`. Pistes solides sur l'origine lorsqu'elles sont présentes ; l'ADS peut être absent/supprimé. |
| `ContentSize` | Taille réelle du contenu principal récupéré. À comparer avec la valeur enregistrée `FileSize`. |
| `MD5`, `SHA1`, `SHA256` | Calculés à partir des octets récupérés. Utilisez SHA-256 pour l'identification moderne du contenu ; conservez-les tous pour l'interopérabilité. |
| `Streams` | Types/noms/tailles des flux récupérés ; peuvent inclure des ADS, des données de sécurité ou une ruche du Registre. |
| `ResourceID` | Identifiant reliant les enregistrements de métadonnées et de contenu de la ressource. |
| `EntryID` | GUID de l'entrée de quarantaine ; permet aussi de faire le lien avec l'historique Defender et les événements. |
| `ScanID` | Identifiant d'analyse Defender permettant de regrouper les ressources/actions. |
| `QuarantineFolder` | Dossier source racine/distinction entre hôtes. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Fichiers source exacts utilisés pour construire la ligne. Essentiels pour la validation. |
| `ExtractedFile` | Fichier principal mis en quarantaine écrit par `--extract` ; vide lorsqu'aucune extraction principale n'a eu lieu. Les flux auxiliaires sont écrits à côté. `auto-parse -x` l'indique à partir de son répertoire de sortie (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | Incohérence d'intégrité, contenu manquant, désaccord d'identifiants, dommages ou contexte orphelin. Ne les masquez pas. |

JSON/JSONL conserve la ressource structurée, la liste des flux, les entrées d'enregistrement et tous les détails
des relations, au lieu du texte concaténé du CSV.

## Comportement de l'extraction { #extraction-behavior }

Juntei utilise le nom de fichier d'origine suivi d'une extension finale `.quarantined` lorsque l'entrée de quarantaine
en enregistre un. Il utilise l'identifiant du contenu pour les ressources orphelines ou autres que des fichiers, et ajoute
l'identifiant pour résoudre les collisions de noms de fichiers. Les caractères non valides dans les noms de fichiers Windows sont remplacés. Les flux
de sécurité et les flux alternatifs sont écrits à côté du fichier principal. Chaque flux de sécurité donne également lieu à un rapport
texte lisible. Juntei supprime l'autorisation d'exécution ;
un contenu existant différent n'est pas écrasé sans `-C`. Ces mesures réduisent les accidents, mais ne
rendent pas les octets sûrs.

### Lecture du descripteur de sécurité récupéré { #reading-the-recovered-security-descriptor }

Un fichier `.security.quarantined` contient des métadonnées de sécurité Windows binaires ; il n'est donc pas lisible dans un
éditeur de texte. Il s'agit d'un flux de sauvegarde distinct, et non d'une autre copie de l'exécutable. Juntei le
décode automatiquement lors de l'extraction, avec `parse-file defender-quarantine --extract` comme avec `auto-parse -x` :

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

Le nom du fichier texte suit le nom réel du fichier binaire, y compris un éventuel identifiant de collision. Le texte
généré est de l'UTF-8 ordinaire et bénéficie des mêmes protections contre l'écrasement que les autres fichiers extraits. Il indique
le nom du fichier binaire, son nombre d'octets et son SHA-256, afin que le rapport puisse être rapproché de sa source.

| Champ du rapport | Interprétation |
|---|---|
| Propriétaire et groupe principal | SID enregistrés dans le descripteur. Les SID connus reçoivent des noms tels que `BUILTIN\Administrators` ou `NT AUTHORITY\SYSTEM`. Les autres SID restent non résolus ; un RID familier ne suffit pas à nommer un compte local/de domaine. Le propriétaire ne prouve pas qui a créé ou exécuté un fichier. |
| Indicateurs de contrôle | Indicateurs numériques d'origine et leurs noms, y compris la présence d'ACL, l'héritage et la protection. |
| DACL | Entrées de contrôle d'accès (ACE) ordonnées : type autorisation/refus, SID du compte, masque d'autorisations, droits et indicateurs d'héritage. Il s'agit d'entrées enregistrées, et non d'un calcul de l'accès effectif pour le jeton d'un utilisateur. |
| SACL | Entrées d'audit ou d'intégrité obligatoire enregistrées. Les indicateurs d'audit des réussites/échecs décrivent une configuration, et non un historique des accès. L'absence de SACL n'établit pas que tout audit était désactivé. |
| GUID d'objet | GUID facultatifs pour les ACE propres à un objet ; les droits propres à l'objet sont conservés sous forme numérique plutôt que d'être présentés à tort comme des droits sur les fichiers. |
| SDDL | Représentation en Security Descriptor Definition Language du propriétaire, du groupe et des entrées d'ACL prises en charge. Consultez séparément les indicateurs de contrôle. Juntei n'émet pas de chaîne SDDL partielle si un composant est endommagé ou non pris en charge. |
| État du décodage et problèmes | `complete` ou `incomplete`, avec les raisons. Les ACE non prises en charge conservent leur forme hexadécimale brute dans le rapport. Le binaire d'origine est toujours conservé lorsque l'extraction réussit. Les problèmes de décodage apparaissent également dans le champ `Problems` de l'enregistrement de quarantaine. |

Une DACL absente, une DACL NULL et une DACL vide sont signalées séparément. Une DACL absente ou NULL n'impose
aucune restriction d'accès discrétionnaire ; une DACL vide ne contient aucune ACE accordant l'accès. Les autres vérifications d'accès
et privilèges de Windows restent pertinents. Les ACE héritées et « inherit-only » restent explicitement signalées.

Par exemple, une entrée accordant un contrôle total hérité aux Administrateurs peut apparaître ainsi :

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

Le décodeur fonctionne hors ligne sous Windows, macOS et Linux, sans API Windows, Python ni résolution de comptes.
Il décode les ACE de base d'autorisation/refus/audit/alarme, les ACE d'objet et les étiquettes d'intégrité obligatoire. Les ACE conditionnelles,
de rappel (callback), d'attribut de ressource et les autres ACE non prises en charge sont explicitement identifiées et conservées sous forme
brute ; leurs conditions d'accès ne sont pas évaluées. Un descripteur endommagé n'interrompt pas l'extraction
des autres flux.

## Stratégie d'investigation { #investigation-strategy }

1. Partez de `ThreatName`, `Path`, `QuarantineTime`, `EntryId` et des empreintes.
2. Validez `Problems` et comparez la taille enregistrée avec le contenu récupéré.
3. Pivotez sur le SHA-256 vers d'autres hôtes, l'EDR, les sandbox et les données de renseignement sur les menaces.
4. Corrélez le chemin/l'heure d'origine avec `$MFT`, le journal USN, Prefetch, Amcache et les journaux d'événements.
5. Utilisez les champs URL/zone pour enquêter sur l'accès initial, mais validez avec les preuves du navigateur, du proxy et du DNS.

## Plus de détails { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Plugin Defender de dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft : correction et quarantaine de l'Antivirus Microsoft Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft : descripteurs de sécurité](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft : format de chaîne des descripteurs de sécurité](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft : chaînes ACE](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
