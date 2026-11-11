# Comptes locaux SAM { #sam-local-accounts }

## Ce que contient la SAM { #what-the-sam-contains }

La ruche du gestionnaire des comptes de sécurité (Security Account Manager, `SAM`) stocke les enregistrements des comptes locaux. Juntei décode les
noms de comptes, les RID, les champs de profil/commentaire, les indicateurs de contrôle de compte, les horodatages de dernière ouverture de session et de mot de passe, les compteurs,
les indications de mot de passe, la liaison à un compte Microsoft, les chemins de profil et le fait que Windows masque ou non le compte
dans l'interface de connexion.

Ces informations sont utiles pour repérer des administrateurs locaux nouvellement créés, des comptes dormants réactivés, des modifications de mot de passe,
des comptes masqués et des comptes préparés pour un déplacement latéral.

<span class="evidence-strength">Meilleure conclusion :</span> la SAM locale représentait le compte avec
l'état enregistré. Il ne s'agit pas d'un historique d'authentification complet, et elle ne décrit pas les comptes de domaine.

## Emplacement et collecte { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Collectez la ruche et les journaux avec des privilèges/un accès brut suffisants. L'identité de l'hôte peut être enrichie à partir de
la ruche `SYSTEM` correspondante dans une collecte.

## Précautions d'analyse { #analyst-cautions }

- Cette commande analyse les comptes **locaux**, et non les utilisateurs du domaine Active Directory.
- Juntei n'extrait volontairement pas les hachages de mots de passe.
- Le RID 500 correspond au compte Administrateur intégré, même s'il a été renommé ; le nom seul ne suffit pas.
- `LastLogon` et les compteurs reflètent l'état du Registre, et non une piste d'audit complète par session.
- `HiddenFromLogonUI` masque un compte sur l'écran de connexion ; il ne désactive pas le compte.
- Les comptes intégrés `DefaultAccount` et `WDAGUtilityAccount` peuvent être masqués de manière légitime.
- Des indicateurs de compte nuls provenant d'une valeur `F` endommagée ou absente ne doivent pas être interprétés comme un compte activé.
- `NamesKeyLastWrittenUTC` donne une approximation du contexte d'enregistrement du nom de compte, mais ne garantit pas l'heure de création.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Contexte de l'hôte et ruche SAM exacte. |
| `RID` | Identificateur relatif. Pivot solide pour l'identité du compte ; 500 correspond à l'Administrateur intégré. |
| `User` | Nom actuel du compte local. Peut être renommé ; corrélez par RID. |
| `FullName`, `Comment`, `UserComment` | Texte descriptif ; contrôlable par un attaquant et utile pour détecter une usurpation (masquerading). |
| `AccountFlags` | États décodés tels que désactivé, verrouillé, mot de passe non requis, compte normal, mot de passe sans expiration. |
| `AccountFlagsValue` | Indicateurs numériques bruts pour la validation et les bits non pris en charge. |
| `CountryCode`, `CodePage` | Métadonnées de paramètres régionaux/page de codes du compte ; généralement du contexte, et non une attribution. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | Même état de dernière ouverture de session, en précision complète et en secondes entières pour la compatibilité. Pas un historique complet. |
| `PasswordLastSetUTC` | Heure de modification du mot de passe enregistrée. Examinez les valeurs récentes ou anciennes inattendues. |
| `AccountExpirationUTC` | Expiration configurée lorsqu'elle est définie. Null peut signifier jamais/non défini. |
| `LastFailedLogonUTC` | Dernier échec d'ouverture de session locale enregistré dans l'état de la SAM ; corroborez avec les événements Security. |
| `FailedLogonCount`, `LogonCount` | Compteurs cumulatifs/d'état dont le comportement de mise à jour est propre à Windows ; à utiliser comme contexte, et non comme décompte exact d'événements. |
| `PasswordHint` | Indication sensible fournie par l'utilisateur ; peut révéler des schémas de mots de passe. À manipuler avec les précautions appropriées. |
| `InternetUserName` | Identité du compte Microsoft lié, le cas échéant. |
| `HiddenFromLogonUI` | État de masquage dans l'interface. Examinez les RID élevés et les comptes personnalisés inattendus ; ne signifie pas désactivé. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Configuration de l'environnement/de l'ouverture de session du compte local ; des scripts ou chemins UNC inhabituels peuvent indiquer une persistance. |
| `KeyLastWrittenUTC` | Heure de modification de la clé de compte du RID. Au niveau de la clé, et non propre à un champ. |
| `NamesKeyLastWrittenUTC` | Heure de modification de la clé de correspondance du nom d'utilisateur ; approximation utile du cycle de vie du compte. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Valeurs FILETIME 64 bits brutes conservées pour la validation et la gestion des valeurs sentinelles particulières. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de relecture du Registre. |
| `Problems` | Désaccord entre F/V/Names, entrées manquantes, décalages mal formés ou dommages de décodage. |

## Stratégie d'investigation { #investigation-strategy }

1. Comparez le RID, le nom, les indicateurs, le chemin de profil et `NamesKeyLastWrittenUTC` à la référence de l'hôte.
2. Examinez les comptes activés de RID 1000 et plus, ceux dont le mot de passe n'est pas requis ou n'expire jamais, ainsi que les comptes masqués ou récemment modifiés.
3. Corrélez avec les événements Security 4720/4722/4724/4732, les événements d'ouverture de session, la création de profils et l'activité à distance.
4. Recherchez dans l'historique de récupération du Registre si un compte a été supprimé avant la collecte.
5. Distinguez les comptes légitimes de provisionnement/par défaut des comptes interactifs créés par un attaquant.

## Plus de détails { #further-details }

- [Microsoft Open Specifications : SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Plugin SAM de dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
