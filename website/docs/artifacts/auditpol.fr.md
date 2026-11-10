# Stratégie d’audit { #audit-policy }

## De quoi s'agit-il { #what-it-is }

La stratégie d'audit indique quels événements de sécurité Windows consigne dans le journal d'événements Security. Elle est définie
par sous-catégorie sur Windows Vista et versions ultérieures (59 sur Windows 10 et 11), et par catégorie sur
Windows XP et Server 2003. Chacune consigne les succès, les échecs, les deux, ou rien.

LSA conserve la stratégie en vigueur dans la ruche `SECURITY`, qu'elle provienne de `auditpol`, de la stratégie de sécurité
locale ou de la stratégie de groupe.

<span class="evidence-strength">Meilleure conclusion :</span> le système était configuré pour consigner ces événements (ou
non) lors de la dernière écriture de la ruche. Lorsqu'une sous-catégorie n'est pas auditée, l'absence d'un événement n'indique
rien sur le fait que l'activité ait eu lieu ou non.

## Emplacement { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista et versions ultérieures conservent un en-tête de 12 octets, puis un paramètre par sous-catégorie, catégorie par
catégorie, puis le nombre de sous-catégories de chaque catégorie. Les versions ne diffèrent que par ces nombres :
chacune ajoute des sous-catégories à la fin d'une catégorie (56 sur les ruches Windows 8.x et Server 2012 R2
lues, 59 sur Windows 10, 11 et Server 2022). Windows XP et Server 2003 conservent l'état d'activation de l'audit
(un octet, suivi d'un remplissage que Windows n'efface pas) et neuf paramètres de catégorie. Cette disposition a été
vérifiée sur l'image publique Windows XP du Hacking Case de CFReDS du NIST, qui n'audite rien ; les
valeurs de ses paramètres de catégorie suivent la documentation de Microsoft.

Collectez `SECURITY` avec ses journaux de transactions, ainsi que la ruche `SYSTEM` du système, qui donne le nom de
l'hôte.

## Précautions d'analyse { #analyst-cautions }

- **Les données inconnues restent inconnues.** Les révisions non prises en charge sont conservées avec des diagnostics. Les indicateurs
  de paramètre inconnus laissent `Success` et `Failure` vides et sont exclus des décomptes de paramètres audités.
  Une table de décompte endommagée ne peut pas transformer le mot réservé en un autre paramètre d'audit.
  Les captures peuvent contenir des emplacements supplémentaires au-delà du catalogue de noms/GUID. Leurs paramètres et leur
  `SubcategoryId` numérique sont conservés avec des noms/GUID vides ; un emplacement non mappé n'est pas en soi
  une preuve de corruption.


- **L'ordre est celui de Windows.** Au sein d'une catégorie, les paramètres suivent l'ordre des
  identifiants de message des sous-catégories, et non celui de leurs GUID. Special Logon (`%%12548`, index
  4 de Logon/Logoff) précède IPsec Quick Mode. Les outils qui les lisent dans l'ordre des GUID signalent la valeur par
  défaut de Windows 10 comme Special Logon « No Auditing » et IPsec Quick Mode « Success ». `SubcategoryId` est le
  même identifiant que celui que donne l'événement 4719 (la stratégie d'audit du système a été modifiée), ce qui permet de faire correspondre les deux.
- **La stratégie en vigueur, pas sa source.** La stratégie de groupe et la stratégie locale écrivent la même valeur.
  Le fichier `audit.csv` de la stratégie de groupe (dans SYSVOL et `C:\Windows\System32\GroupPolicy`) montre ce qui
  devait être appliqué (`Machine\Microsoft\Windows NT\Audit\audit.csv` d'une stratégie).
- **Évolutions dans le temps :** `KeyLastWrittenUTC` indique approximativement la dernière modification de la stratégie. L'événement 4719
  consigne chaque modification (tant que le journal Security la conserve). Un attaquant peut désactiver l'audit,
  puis le réactiver.
- **Sous-catégories ou catégories.** Sur Windows Vista et versions ultérieures, ce sont les sous-catégories que
  Windows utilise (sauf si l'option de sécurité `SCENoApplyLegacyAuditPolicy` est désactivée et qu'une stratégie de catégorie
  héritée les remplace).
- **Windows XP et 2003 :** lorsque l'audit est désactivé (`AuditingEnabled` False), aucune catégorie n'est auditée,
  quel que soit son paramètre.
- **Événements :** `Events` liste les principaux événements de chaque sous-catégorie, d'après la documentation de Microsoft
  sur les paramètres de stratégie d'audit avancée. Ce sont ceux qu'une investigation recherche habituellement, et non
  tous les événements que la sous-catégorie consigne.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Pour chaque ruche, le résumé indique sa disposition, le nombre de paramètres audités et la date de dernière écriture
de la stratégie. Il liste ce qui est audité en succès et en échec, en succès uniquement et en échec
uniquement, puis les sous-catégories non auditées, avec leurs principaux événements : ces événements sont
désactivés dans la stratégie capturée ; cela n'établit pas la stratégie en vigueur lors d'une activité antérieure.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Hôte (provenant de la ruche `SYSTEM` du système) et ruche `SECURITY`. |
| `Layout` | `subcategories` (Windows Vista et versions ultérieures) ou `categories` (XP et 2003) ; vide lorsque la valeur ne se décode pas. |
| `Category`, `Subcategory` | Tels que `auditpol` les nomme. |
| `SubcategoryGUID`, `SubcategoryID` | Le GUID de la sous-catégorie, et son identifiant de message tel que le donne l'événement 4719 (`%%12548`). |
| `Setting` | `none`, `success`, `failure` ou `success_and_failure`. |
| `Success`, `Failure` | Indiquent si les succès et les échecs sont consignés. |
| `AuditingEnabled` | Windows XP et 2003 : indique si l'audit est activé. |
| `Events` | Les principaux identifiants d'événement de la sous-catégorie. |
| `KeyLastWrittenUTC` | Dernière écriture de `PolAdtEv` : date approximative de la dernière modification de la stratégie. |
| `DataRaw` | Les octets de la valeur (hexadécimal), lorsqu'elle ne se décode pas entièrement. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Une disposition ou une taille inconnue, des décomptes incohérents et des valeurs supérieures à 3. |

## Stratégie d'investigation { #investigation-strategy }

1. Avant de conclure que quelque chose ne s'est pas produit, vérifiez si sa sous-catégorie était auditée :
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (tâches planifiées, 4698), Security System Extension (installations de services, 4697), User Account et
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. Comparez la stratégie avec le journal Security : la présence d'événements de sous-catégories qui ne sont pas auditées actuellement
   signifie que la stratégie a changé. Recherchez l'événement 4719 et un `KeyLastWrittenUTC` récent.
3. Comparez avec le fichier `audit.csv` de la stratégie de groupe pour distinguer une stratégie à l'échelle du domaine d'une modification locale.
4. Utilisez `parse-registry read --recover-deleted` et les journaux de transactions pour retrouver les versions antérieures de
   `PolAdtEv`.

## Plus de détails { #further-details }

- [Microsoft : paramètres de stratégie d'audit de sécurité avancée](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft : constantes d'audit (GUID des sous-catégories)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft : 4719(S) La stratégie d'audit du système a été modifiée](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
