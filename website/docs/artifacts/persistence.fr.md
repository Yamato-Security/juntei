# Démarrages automatiques du Registre { #registry-autostarts }

## De quoi s'agit-il { #what-they-are }

Windows lance des programmes et charge des DLL à partir de nombreux emplacements du Registre : au démarrage, à l'ouverture de session, lorsqu'un
programme démarre ou se termine, lorsque `cmd.exe` s'ouvre, lorsqu'une classe COM est créée, ou lorsque le spouleur,
LSA, netsh ou le service de temps charge ses plug-ins. Les attaquants utilisent ces emplacements pour la persistance,
l'élévation de privilèges et l'injection de DLL, et les logiciels légitimes les utilisent aussi. Juntei les lit
dans `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` et `UsrClass.dat` et les réunit dans une seule table. Il écrit un enregistrement par
programme, DLL ou paquet, et associe à chacun sa technique MITRE ATT&CK.

<span class="evidence-strength">Meilleure conclusion :</span> la ruche a configuré ce programme ou cette DLL
pour qu'il démarre ou soit chargé. Cela ne prouve pas que le programme s'est exécuté. Son exécution nécessite les conditions que le
mécanisme désigne (une ouverture de session, le démarrage du programme débogué, la création de la classe), ainsi que d'autres
preuves d'exécution.

## Emplacements { #locations }

| Ruche | Ce qui est lu |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` et `Policies\Explorer\Run`, avec leurs copies `Wow6432Node` et du mode d'installation Terminal Server ; `Explorer\StartupApproved\Run`, `Run32` et `StartupFolder` ; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … et `Notify`) ; `Windows` `AppInit_DLLs` avec `LoadAppInit_DLLs` ; `Image File Execution Options\*` `Debugger` (avec `UseFilter`) et `VerifierDlls` ; `SilentProcessExit\*` `MonitorProcess` ; `Active Setup\Installed Components\*` `StubPath` ; `NetSh` ; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, variables du profileur .NET de `Environment`) ; `Control\Lsa` `Authentication`, `Security` et `Notification Packages` (+`OSConfig`) ; `SecurityProviders` ; moniteurs, fournisseurs et processeurs d'impression de `Control\Print` ; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | les clés `Run` et `StartupApproved` de l'utilisateur ; `Load` et `Run` de `Windows NT\CurrentVersion\Windows` ; `Shell` de `Winlogon` ; `Environment` (`UserInitMprLogonScript`, profileur .NET) ; `SCRNSAVE.EXE` de `Control Panel\Desktop` ; `PackagedAppXDebug` ; `AutoRun` de `Command Processor` ; `Software\Classes` (en tant que `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` et `LocalServer32` (ainsi que ceux de `Wow6432Node`) ; `shell\open\command` des types de programmes (`exefile`, `batfile`, `scrfile`, …) et de ce que lancent les programmes à élévation automatique (`ms-settings`, `mscfile`, `Folder`, …) ; `ActivatableClasses\…\DebugInformation` |

Collectez chaque ruche avec ses journaux de transactions. Les ruches utilisateur se trouvent dans `Users\<name>\NTUSER.DAT` et
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Précautions d'analyse { #analyst-cautions }

- **Les valeurs par défaut sont également signalées.** La plupart des enregistrements d'un système sain sont des valeurs par défaut de Windows :
  - `Userinit` vaut `userinit.exe` ;
  - `Shell` vaut `explorer.exe` ;
  - `KnownDLLs`, les paquets LSA, les moniteurs d'impression, les assistants netsh et les composants Active Setup possèdent tous des entrées d'origine.

  Recherchez ce qui diffère d'une installation saine de la même version.
- **`StartupApproved` ne couvre que la liste Démarrage du Gestionnaire des tâches.** Il indique si une entrée `Run`
  ou un fichier du dossier Démarrage y est désactivé, et depuis quand. Une entrée sans valeur `Run` correspondante
  est signalée, avec la mention « only in StartupApproved ». Elle peut révéler un démarrage automatique qui
  a été supprimé.
- **`OverridesMachineRegistration` nécessite la ruche `SOFTWARE` du système.** Une classe COM ou une
  commande d'un utilisateur est comparée à celle-ci, qui est trouvée de la même manière que la ruche `SYSTEM` de l'hôte. Un
  remplacement correspond au schéma du détournement COM (T1546.015). OneDrive et d'autres installations par utilisateur
  enregistrent aussi leurs propres classes, qui ne remplacent rien.
- **`ImagePath` et `Args` ne sont séparés de la ligne de commande que lorsque cela peut être déterminé.**
  - Un programme entre guillemets est séparé au niveau du guillemet fermant.
  - Un premier mot sans dossier est considéré comme le programme (`cmd /c …`).
  - Sinon, la séparation se fait au début le plus court qui se termine par une extension de programme.

  Un chemin sans guillemets contenant des espaces peut exécuter un autre programme (`C:\Program.exe`) s'il en existe un.
- **`GlobalFlag` détermine `Enabled` pour `VerifierDlls` et `MonitorProcess`.** `VerifierDlls` n'est chargé
  qu'avec `0x100`. `MonitorProcess` ne s'exécute qu'avec `0x200` et le `0x1` de `ReportingMode` de
  `SilentProcessExit\<program>`. Un mode `2` (un vidage) ou `4` (une notification) n'exécute rien, et
  `Details` nomme les indicateurs du mode.
- **`KeyLastWrittenUTC` est la dernière écriture de la clé**, et non celle de la valeur. Plusieurs valeurs la partagent.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

Le résumé dénombre les enregistrements par mécanisme. Il liste aussi les entrées `StartupApproved\Run` et `Run32`
restées sans leur valeur `Run` (les fichiers du dossier Démarrage n'en ont aucune à perdre), ainsi que les classes COM
et commandes utilisateur qui remplacent celles de la machine.

Comme toutes les commandes `parse-registry`, elle refuse un `-o` désignant l'un des fichiers qu'elle lit, même
avec `-C`. Cela couvre la ruche, ses journaux de transactions et les ruches SYSTEM et SOFTWARE du système,
par chemin, lien symbolique ou lien physique. Lorsqu'elle ne peut pas le déterminer, elle n'écrit rien.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Contexte de l'hôte, ruche source et son type. |
| `User`, `UserSID` | Pour les ruches utilisateur : le dossier de profil et le SID. Le SID provient du nom de la clé racine de `UsrClass.dat`, sinon du `ProfileList` du système. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | Technique MITRE ATT&CK, lorsqu'une technique correspond (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | La clé sous la racine de la ruche (noms tels qu'écrits) et la valeur (vide : la valeur par défaut). |
| `Data` | Le texte de la valeur, ou un élément d'une liste. Vide lorsqu'il est illisible, ou lorsque seul `StartupApproved` nomme l'entrée. |
| `ImagePath`, `Args` | Le programme ou la DLL, et les arguments, lorsqu'ils peuvent être distingués (voir ci-dessus). |
| `Enabled`, `DisabledAtUTC` | Ce qu'indiquent `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (avec `ReportingMode`), `UseFilter`, `IsInstalled`, le `Enabled` du fournisseur de temps ou `ScreenSaveActive`. Vide lorsque rien ne l'indique. |
| `Details` | Informations supplémentaires : l'effet d'un renommage en attente, le nom de la classe COM, `ReportingMode`, `FilterFullPath`, un `ShellComponent` d'Active Setup, … |
| `OverridesMachineRegistration` | Pour une classe COM ou une commande d'un utilisateur : indique si la machine enregistre la même. Vide sans la ruche `SOFTWARE` du système. |
| `WOW64` | Sous `Wow6432Node` : pour les programmes 32 bits. |
| `KeyLastWrittenUTC` | La dernière écriture de la clé. |
| `DataRaw` | Les octets de la valeur (hexadécimal), lorsqu'ils ne sont pas du texte ou pas de l'UTF-16 valide. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Types de valeur incorrects, données illisibles, entrées `StartupApproved` de taille incorrecte. |

## Stratégie d'investigation { #investigation-strategy }

1. Commencez par le résumé :
   - les classes COM et commandes qui remplacent celles de la machine ;
   - les entrées `StartupApproved` sans leur valeur `Run` ;
   - les mécanismes qui ont rarement des entrées (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Comparez les paquets LSA, `BootExecute`, `KnownDLLs`, les moniteurs d'impression et les assistants netsh avec une installation
   saine de la même version de Windows.
3. Résolvez `ImagePath` par rapport au système de fichiers ($MFT, empreintes et signatures), et vérifiez les chemins situés dans les
   profils utilisateur, `ProgramData` et `Temp`.
4. Corrélez `KeyLastWrittenUTC` avec les événements du Registre de Security et de Sysmon, la création de processus,
   Prefetch et Amcache.
5. Utilisez `parse-registry read --recover-deleted` pour trouver les démarrages automatiques supprimés de ces clés.

## Plus de détails { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft : clés de Registre Run et RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
