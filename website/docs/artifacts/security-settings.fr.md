# Paramètres de sécurité { #security-settings }

## De quoi s'agit-il { #what-they-are }

Les attaquants modifient les paramètres de sécurité de Windows pour rester cachés et accéder aux identifiants. Ils
désactivent Microsoft Defender ou en excluent leurs outils, font conserver à Windows les mots de passe en clair,
ouvrent le Bureau à distance, désactivent le pare-feu et réduisent au silence la journalisation PowerShell. Le Registre conserve
ces paramètres :

- **`SOFTWARE` :** Defender, ses exclusions et ses stratégies ; l'UAC ; les stratégies du Bureau à distance et du pare-feu ;
  la stratégie d'exécution de PowerShell, ses stratégies de journalisation et ses fournisseurs AMSI ; la Restauration du système ;
  les canaux du journal des événements.
- **`SYSTEM`** (par jeu de contrôles) : `WDigest`, la protection LSA, Credential Guard, Restricted Admin,
  NTLM ; le Bureau à distance ; les profils du pare-feu ; SMBv1 ; les heures de dernier accès ; la taille de chaque journal d'événements.
- **`NTUSER.DAT` :** la stratégie d'exécution PowerShell et les stratégies de journalisation de l'utilisateur, ainsi qu'AMSI pour
  Windows Script Host.

`parse-registry security-settings` signale chaque paramètre trouvé sous forme d'enregistrement. Chaque enregistrement contient
la valeur du paramètre, sa signification, et indique s'il **affaiblit la configuration par défaut** : s'il désactive une
protection ou une source de preuves, ou s'il l'abaisse en deçà de ce que fait la version actuelle de Windows (10 et 11)
en l'absence de cette valeur.

<span class="evidence-strength">Meilleure conclusion :</span> ce paramètre avait cette valeur lorsque la ruche
a été enregistrée, et la clé a été écrite pour la dernière fois à cette heure. Un paramètre affaibli est une piste, et non une preuve de
compromission. Les administrateurs, les stratégies de groupe, d'autres produits de sécurité et les versions plus anciennes de Windows définissent
aussi certaines de ces valeurs, et la signification l'indique lorsque c'est courant.

## Emplacement { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Collectez les trois ruches avec leurs journaux de transactions. Un paramètre modifié peu avant la collecte
peut ne figurer que dans les journaux.

## Précautions d'analyse { #analyst-cautions }

- **La configuration stockée n'est pas l'état de protection effectif.** Les valeurs de Registre collectées et les lignes de diagnostic relatives aux listes de valeurs endommagées
  sont signalées. Des données manquantes peuvent refléter des valeurs par défaut, des sources non collectées ou un autre mécanisme
  de gestion ; elles ne prouvent pas que la protection est activée ou désactivée. `WeakensDefault` signale
  les paramètres susceptibles d'abaisser la référence indiquée lorsqu'ils sont appliqués.
- **Les anciens indicateurs de Defender peuvent être ignorés.** `DisableAntiSpyware` et `DisableAntivirus` ne
  prouvent pas que Defender s'est arrêté. Leur applicabilité dépend de la version de la plateforme, de l'intégration (onboarding) et de la protection
  contre les falsifications. [Microsoft explique ces anciens paramètres](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Les exclusions gérées modernes peuvent être indisponibles dans les captures du Registre.** À partir de la plateforme
  4.18.25110.6, les appareils utilisant la gestion de la configuration de Defender for Endpoint nécessitent les cmdlets PowerShell
  Defender prises en charge pour récupérer la configuration des exclusions. L'absence dans un Registre hors ligne
  n'établit donc pas l'absence d'exclusions.
  [Recommandations de configuration de Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Affaibli ne signifie pas malveillant.** Exemples :
  - `DisableAntiSpyware` = 1 est également défini par Windows lorsqu'un autre antivirus est inscrit.
  - Le Bureau à distance est délibérément ouvert sur les serveurs.
  - Windows 7 conservait des journaux d'événements plus petits et ne mettait pas à jour les heures de dernier accès.
- **Les valeurs par défaut dépendent de la version.** Certaines valeurs étaient auparavant la valeur par défaut :
  - `NtfsDisableLastAccessUpdate` = 1 était la valeur par défaut avant Windows 10 1803. Elle est signalée
    sans verdict, mais `0x80000001`, défini manuellement sur une version ultérieure, est considéré comme affaibli.
  - `RunAsPPL`, `LsaCfgFlags` et `UseLogonCredential` sont comparés à la version actuelle de Windows.
- **Exclusions de Defender :** chaque chemin, extension, processus ou adresse constitue un enregistrement. Chaque exclusion
  affaiblit Defender. `TemporaryPaths`, que Defender ajoute lui-même, n'a pas de verdict. Recherchez des dossiers accessibles en écriture par l'utilisateur,
  des extensions `.ps1`/`.exe` et des LOLBins (`rundll32.exe`, `powershell.exe`).
- **Les clés de stratégie** (`Policy` = True) sont écrites par les stratégies de groupe, par des outils et par des attaquants qui
  savent que les stratégies l'emportent sur les paramètres locaux. Corrélez-les avec les GPO du domaine.
- **Canaux du journal des événements :** seuls 25 canaux pertinents pour la sécurité sont signalés, avec l'indication que Windows
  les active ou non par défaut. `TaskScheduler/Operational` et `DNS-Client/Operational`, par
  exemple, sont désactivés par défaut. Utilisez `parse-registry read` pour les autres.
- **Heures des clés :** `KeyLastWrittenUTC` est celle de la clé ; ce n'est donc pas nécessairement le moment où cette valeur
  a changé. Il est courant qu'une clé `Policies` entière soit écrite en une seule fois.
- **Jeux de contrôles :** par défaut, c'est le jeu de contrôles utilisé en dernier par Windows qui est lu. `--all-control-sets`
  les lit tous.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

La synthèse compte les paramètres par domaine. Elle liste les paramètres qui affaiblissent la configuration par défaut ainsi que chaque
exclusion de Defender.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Hôte, ruche et son type (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | Pour `SYSTEM` : le jeu de contrôles, et l'indication qu'il s'agit ou non de celui utilisé en dernier par Windows ; inconnu lorsque `Select\Current` est absent ou non valide. |
| `User`, `UserSID` | Pour `NTUSER.DAT` : le nom du dossier de profil, et le SID de l'utilisateur issu de `ProfileList` du système. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | Nature du paramètre : `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | La valeur : un nombre en décimal, une chaîne ou l'exclusion elle-même. Vide lorsque la valeur n'est pas du type du paramètre. |
| `Meaning` | Signification de la valeur. |
| `WeakensDefault` | `True` lorsque la valeur affaiblit ce que fait la version actuelle de Windows sans elle, `False` lorsque ce n'est pas le cas, vide lorsque cela ne s'applique pas (une taille, un port, un fournisseur AMSI) ou ne peut être déterminé. |
| `Policy` | Indique si la clé se trouve sous une clé `Policies`. |
| `KeyPath`, `ValueName` | La clé et la valeur. Le nom de valeur d'une exclusion est l'élément exclu ; un fournisseur AMSI n'en a pas. |
| `KeyLastWrittenUTC` | Dernière écriture de la clé. |
| `DataRaw` | Octets de la valeur (hexadécimal), lorsqu'ils ne sont pas du type du paramètre. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs de type incorrect et clés illisibles. |

## Stratégie d'investigation { #investigation-strategy }

1. Exécutez avec `--weakened-only` sur chaque hôte et comparez les hôtes entre eux : un paramètre
   présent sur un seul hôte se remarque.
2. Confrontez chaque exclusion de Defender aux outils trouvés sur l'hôte et dans les journaux de Defender
   (l'événement 5007 enregistre les modifications de configuration).
3. Corrélez les heures des clés avec les ouvertures de session, les installations de services et la création de processus. Recherchez
   les lignes de commande `reg.exe`, `Set-MpPreference`, `netsh advfirewall` et `powershell` (événement
   4688, Sysmon 1 et 13, PowerShell 4104).
4. Pour les paramètres liés aux identifiants (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`), recherchez ensuite des vidages d'identifiants et du pass-the-hash.
5. Utilisez `parse-registry read --recover-deleted` pour les valeurs supprimées afin de rétablir une valeur par défaut, et
   `--all-control-sets` pour les jeux de contrôles plus anciens.

## Plus de détails { #further-details }

- [Microsoft : configurer et valider les exclusions de l'Antivirus Microsoft Defender](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft : paramètres et configuration du Contrôle de compte d'utilisateur](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft : configurer la protection LSA supplémentaire](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft : about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001 : Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
