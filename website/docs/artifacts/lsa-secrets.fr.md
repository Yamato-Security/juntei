# Secrets LSA { #lsa-secrets }

## De quoi s'agit-il { #what-they-are }

L'autorité de sécurité locale (LSA, Local Security Authority) conserve des secrets dans la ruche `SECURITY`. Parmi eux figurent le
mot de passe d'ouverture de session automatique, les mots de passe des comptes sous lesquels s'exécutent les services, le mot de passe
de domaine du compte d'ordinateur, les clés système DPAPI et la clé avec laquelle sont chiffrées les ouvertures de session de domaine mises en cache. Elle
conserve également les ouvertures de session de domaine mises en cache, afin que les utilisateurs du domaine puissent se connecter sans contrôleur de domaine.

`parse-registry lsa-secrets` liste les secrets que détient un système et la date à laquelle ils ont été définis, ainsi que le nombre
d'ouvertures de session mises en cache qu'il conserve et utilise. **Il ne lit que les noms et les heures : rien n'est déchiffré, et
aucun octet de secret n'est lu ni écrit.** Juntei lit des preuves ; il n'extrait pas d'identifiants.

<span class="evidence-strength">Meilleure conclusion :</span> le système détenait un secret portant ce nom, défini
à cette heure. Le secret d'un service (`_SC_<service>`) signifie que LSA stocke le mot de passe du compte
sous lequel s'exécute ce service.

## Emplacement { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Nom | Type | Description |
|---|---|---|
| `DefaultPassword` | `autologon` | Mot de passe avec lequel Winlogon ouvre automatiquement la session. |
| `_SC_<service>` | `service_account` | Mot de passe du compte sous lequel s'exécute `<service>`. |
| `$MACHINE.ACC` | `machine_account` | Mot de passe du compte d'ordinateur dans son domaine. |
| `DPAPI_SYSTEM` | `dpapi_system` | Clés DPAPI du système. |
| `NL$KM` | `cached_logon_key` | Clé avec laquelle sont chiffrées les ouvertures de session mises en cache. |
| `L$_SQSA_<SID>` | `security_questions` | Questions de sécurité d'un compte local (Windows 10 et versions ultérieures). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Identifiants d'accès à distance et VPN. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Autres données privées, selon le préfixe que leur attribue `LsaStorePrivateData`. |

Collectez `SECURITY` avec ses journaux de transactions, ainsi que la ruche système `SYSTEM`, qui nomme
l'hôte.

## Précautions d'analyse { #analyst-cautions }

- **Des noms et des heures, pas des valeurs.** La présence de données dans une valeur est déduite de sa taille. Son contenu
  n'est pas lu.
- **`DefaultPassword` ne signifie pas ouverture de session automatique.** Ce secret existe sur la plupart des systèmes, y compris
  ceux qui n'ouvrent jamais de session automatiquement. La clé
  `Microsoft\Windows NT\CurrentVersion\Winlogon` de `SOFTWARE` (`AutoAdminLogon`, `DefaultUserName`) indique
  si l'ouverture de session automatique est configurée. Un `CurrentSetUTC` récent est une modification qui mérite une explication.
- **Secrets de service :** `_SC_<service>` apparaît lorsqu'un service est configuré pour s'exécuter sous un compte doté
  d'un mot de passe. Comparez-le avec `parse-registry services` (`ObjectName`), et recherchez les services
  installés récemment ou inconnus. Le secret peut subsister après la disparition du service.
- **Heures :** `CurrentSetUTC` et `OldSetUTC` correspondent au moment où Windows a défini les valeurs actuelle et
  précédente. Le mot de passe `$MACHINE.ACC` change tous les 30 jours par défaut ; ses heures évoluent donc.
  `KeyLastWrittenUTC` et les heures des clés `CurrVal`/`OldVal` indiquent approximativement la dernière modification
  du secret.
- **Ouvertures de session mises en cache :** un emplacement est utilisé lorsque son enregistrement nomme un utilisateur. L'en-tête de l'enregistrement n'est pas
  chiffré ; seule la longueur de son nom d'utilisateur est lue. `CachedLogonsKept` est le compteur de `NL$Control`,
  qui suit `CachedLogonsCount` (10 par défaut). Des emplacements utilisés signifient que des utilisateurs du domaine se sont connectés
  de manière interactive, et leurs identifiants peuvent être cassés hors ligne à partir de la ruche. Les noms des utilisateurs
  sont chiffrés et ne sont pas décodés. Seuls les enregistrements binaires validés, dotés d'un en-tête complet et d'une longueur de nom d'utilisateur cohérente, sont comptés comme utilisés. Si `Problems` signale des emplacements endommagés, le nombre d'emplacements utilisés ne couvre que les emplacements validés ; n'interprétez pas les emplacements endommagés comme vides.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

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
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Pour chaque ruche, la synthèse liste chaque secret avec son type et sa date de définition. Elle nomme ensuite
les services dont LSA stocke les mots de passe de compte, et compte les emplacements d'ouverture de session mis en cache qui sont utilisés.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Hôte (issu de la ruche système `SYSTEM`), et ruche `SECURITY`. |
| `Secret`, `Kind` | Nom du secret et sa nature (voir le tableau ci-dessus) ; `Cache` et `cached_logons` pour l'enregistrement des ouvertures de session mises en cache. |
| `ServiceName`, `UserSID` | Service auquel se rapporte un secret `_SC_`, et SID de l'utilisateur fourni par un nom (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` et `OupdTime` : moment où les valeurs actuelle et précédente ont été définies. |
| `HasCurrentValue`, `HasOldValue` | Indique si `CurrVal` et `OldVal` contiennent des données. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | Dernières écritures des clés `CurrVal`, `OldVal` et du secret. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Enregistrement des ouvertures de session mises en cache : emplacements, emplacements utilisés et compteur de `NL$Control`. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Heures malformées ou hors plage (seules les métadonnées temporelles sont affichées), en-têtes de cache endommagés, types de valeur de cache incorrects et clés illisibles. |

## Stratégie d'investigation { #investigation-strategy }

1. Recherchez les secrets `_SC_` de services inconnus ou récemment installés, et rapprochez-les de
   `parse-registry services` et de l'événement 7045 (installation d'un service).
2. Comparez les heures de `DefaultPassword` avec les paramètres d'ouverture de session automatique de `Winlogon`.
3. Sur les membres d'un domaine, notez les emplacements d'ouverture de session mis en cache qui sont utilisés, ainsi que la dernière écriture de `Cache`.
4. Comparez les heures des secrets avec la chronologie : un secret défini pendant une période d'intrusion peut
   signaler un nouveau compte de service, une ouverture de session automatique ou une modification du compte d'ordinateur.

## Plus de détails { #further-details }

- [Microsoft : objet de données privées](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft : informations d'ouverture de session de domaine mises en cache](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb : identifiants mis en cache](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
