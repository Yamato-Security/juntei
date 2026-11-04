# Persistence Hunter { #persistence-hunter }

La persistance est un moyen de faire en sorte que du code s'exécute à nouveau : au démarrage de Windows, lorsqu'un utilisateur se connecte, lorsqu'une application
s'ouvre, lorsqu'une heure planifiée arrive ou lorsqu'un autre événement se produit. Les attaquants peuvent détourner les mêmes paramètres
que ceux utilisés par les logiciels légitimes pour les mises à jour, la prise en charge des périphériques, la sécurité et l'administration.

`juntei persistence-hunter` examine hors ligne des preuves Windows collectées au moyen de **131 vérifications** couvrant
ces mécanismes ainsi que les anomalies associées liées aux comptes et aux protections. Il lit les ruches du Registre, les fichiers de tâches,
les scripts, les paramètres d'applications et une sélection d'enregistrements de bases de données.

Cette page explique chaque vérification implémentée. Pour l'ensemble des options, des champs de sortie, des profils et du
traitement des horodatages, ouvrez le [guide de la commande](../commands/persistence-hunter.md). Les
[remerciements](acknowledgements.md) renvoient vers le projet Python PyrsistenceSniper et vers l'outil
PowerShell d'origine, PersistenceSniper.

## Démarrer une investigation { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

Utilisez `--min-severity info` pour obtenir l'inventaire complet des candidats. La commande manuelle utilise par défaut
`medium` ; son profil peut classer des correspondances partielles avec la liste d'autorisation comme `LOW`, même lorsque l'absence de preuves sur la charge utile ou sur
le signataire empêche une comparaison complète. Ni une correspondance avec la liste d'autorisation ni une entrée inconnue ne permettent de trancher
si un élément est malveillant. `auto-parse` conserve déjà tous les niveaux de gravité et écrit les résultats
dans la catégorie `persistence/` de chaque ordinateur.

Pour vous concentrer sur un mécanisme, copiez son **ID de vérification** ci-dessous :

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

Par défaut, la commande utilise un pool de workers partagé. Ajoutez `-T 4` pour sélectionner quatre workers, ou `-T 1`
pour une analyse monothread. L'entrée peut contenir plusieurs ordinateurs ; leurs preuves restent séparées.

## Lire les résultats comme des preuves { #read-findings-as-evidence }

| Question | Éléments à examiner |
|---|---|
| Qu'est-ce qui a été configuré, et où ? | `CheckId`, `Path` et la `Value` complète, y compris les arguments. Suivez `SourceFile` jusqu'à la ruche, au fichier ou à la base de données collectés. |
| La cible existe-t-elle dans la collecte ? | `ResolvedPath`, `ResolutionStatus` et `Exists`. Les références non collectées, ambiguës ou non prises en charge sont inconnues ; Juntei ne substitue pas un fichier provenant de la machine de l'analyste. |
| La configuration est-elle effective ? | Le déclencheur réel, les indicateurs d'activation, le jeu de contrôles sélectionné, la version de Windows/de l'application, l'utilisateur et l'architecture. Certaines vérifications conservent délibérément une configuration inactive. |
| S'agit-il d'une attaque ? | `Severity` et `PolicyReason` expliquent la correspondance avec le profil, et non un verdict. Comparez avec les logiciels attendus, la commande complète, les empreintes des fichiers, des preuves d'exécution indépendantes et la chronologie environnante. |
| Qui pouvait l'exécuter ? | `AccessGained` est une indication de portée/de contexte. Une configuration à l'échelle de la machine indique souvent `SYSTEM`, mais cela ne prouve pas que le processus lancé s'est exécuté en tant que SYSTEM. Les principaux des tâches et les SID des propriétaires BITS fournissent un contexte plus précis lorsqu'ils sont disponibles. |
| Quand cela a-t-il changé ? | `KeyLastWrite` date la clé de Registre entière. `LastChange`, `ChangeSource` et `ChangeEvidence` décrivent une corrélation MFT/EVTX distincte, lorsqu'elle est disponible. Aucun de ces champs ne date automatiquement l'exécution de la charge utile. |
| Quelle est la fiabilité de l'état récupéré ? | Lisez `RecordStatus`, `Coverage.csv` et les avertissements relatifs aux preuves rejouées, récupérées par carving, endommagées, manquantes ou limitées. Un enregistrement historique récupéré peut ne plus être actif. |
| Un éditeur nommé est-il digne de confiance ? | `Signer` et `SignerStatus` sont des métadonnées descriptives de PE/certificat/catalogue. Juntei ne valide ni les signatures Authenticode ni la confiance dans les certificats. |

L'absence de résultats ne prouve pas l'absence de persistance. Une ligne de couverture `complete` signifie
que la source collectée applicable a été lue, et non que l'acquisition contient tous les fichiers pertinents,
tous les utilisateurs ou tous les mécanismes de persistance Windows possibles. Les autres états de couverture expliquent les vérifications manquantes,
partielles, désactivées ou en échec. Le hunter n'analyse ni la mémoire vive, ni le réseau, ni les identités cloud.

## Catalogue et notation des emplacements { #catalogue-and-location-notation }

Le catalogue est fondé sur PyrsistenceSniper, auquel s'ajoute la vérification native `start_menu_com` ;
les descriptions ci-dessous documentent
le comportement natif de Juntei, y compris les limites de ses preuves. Il existe **116 vérifications fondées sur le Registre
et 15 vérifications de fichiers/de référentiels**. Certaines vérifications de fichiers croisent également des données du Registre ; il s'agit donc de décomptes
d'implémentation plutôt que d'exigences d'acquisition mutuellement exclusives. Les mécanismes
sont regroupés ci-dessous selon leur finalité d'investigation, puis classés par ordre alphabétique d'ID de vérification au sein de chaque groupe.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` et `HKLM\SAM` désignent les ruches machine collectées, normalement
  issues de `Windows\System32\config`, avec les journaux de transactions lorsqu'ils sont disponibles.
- `HKU\<user>` désigne la ruche utilisateur collectée concernée. La plupart des paramètres utilisateur proviennent de NTUSER.DAT ;
  les enregistrements de classes peuvent provenir de `AppData\Local\Microsoft\Windows\UsrClass.dat`, comme indiqué.
- `<ControlSet>` désigne le jeu de contrôles actif sélectionné dans SYSTEM. Les emplacements historiques supplémentaires
  de la vérification DSRM sont indiqués explicitement ci-dessous. Consultez `SYSTEM\Select` lorsque vous les examinez.
- `<name>` est un composant de chemin variable. Dans les tableaux de cibles du Registre, `<child>` désigne les enregistrements
  enfants immédiats ; il ne s'agit pas d'un parcours récursif illimité. Les vérifications explicitement récursives
  décrivent leur parcours séparément. Les crochets dans les chemins Office indiquent une version facultative.
- Les tableaux répertorient les vues machine et utilisateur effectivement sélectionnées par la vérification. Une vue 32 bits est indiquée
  lorsqu'elle est implémentée ; ne supposez pas que chaque vérification examine toutes les architectures/vues possibles.
- Les vérifications génériques de valeurs émettent les entrées configurées non vides, y compris les éléments de chaînes multiples et
  le zéro numérique le cas échéant. Elles n'exigent pas, à elles seules, une charge utile malveillante, un
  déclencheur activé ou une exécution réussie. Les prédicats plus sélectifs sont indiqués pour chaque vérification.

Les ID ATT&CK sont des correspondances héritées du catalogue et des liens permettant d'approfondir. Ils décrivent un
détournement potentiel, et non le constat qu'un attaquant a mis en œuvre cette technique ; les vérifications de paramètres de protection et de diagnostic
peuvent décrire des conditions favorables plutôt qu'un mécanisme de persistance autonome.

| Groupe de mécanismes | Vérifications |
|---|---:|
| [Démarrage, connexion et démarrage automatique](#boot-sign-in-and-startup) | 32 |
| [Services et extensions du système](#services-and-system-extensions) | 13 |
| [Tâches planifiées](#scheduled-tasks) | 3 |
| [WMI, BITS et autres déclencheurs d'événements](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorateur, COM et gestionnaires de fichiers ou d'URL](#explorer-com-and-file-or-url-handlers) | 15 |
| [Applications et compléments Office](#office-applications-and-add-ins) | 7 |
| [Débogueurs, gestionnaires de plantage et shims de compatibilité](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [Runtime .NET et paramètres de profileur](#net-runtime-and-profiler-settings) | 7 |
| [Substitutions de la recherche de DLL et d'exécutables](#dll-and-executable-lookup-overrides) | 18 |
| [Comptes et visibilité à la connexion](#accounts-and-sign-in-visibility) | 4 |
| [Composants d'authentification et protection des identifiants](#authentication-components-and-credential-protection) | 8 |
| **Total** | **131** |

## Démarrage, connexion et démarrage automatique { #boot-sign-in-and-startup }

Ces paramètres font en sorte qu'une commande ou une bibliothèque s'exécute pendant le démarrage, la connexion ou l'initialisation du bureau. Vérifiez la version de Windows et l'utilisateur concerné : un paramètre peut être obsolète, désactivé ou destiné à une borne (kiosque) plutôt qu'à un attaquant.

### Active Setup {#active_setup}

**ID de vérification :** `active_setup` · **ATT&CK du catalogue :** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Ce que Juntei vérifie :** lit `StubPath` dans les enregistrements de composants Active Setup à l'échelle de la machine, dans les vues native et 32 bits. Windows peut utiliser ces commandes lors de la préparation du profil d'un utilisateur ; la vérification ignore les valeurs de substitution littérales `U` et `/UserInstall`.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` et l'arborescence `Wow6432Node` correspondante.

**Comment investiguer :** comparez chaque composant avec le produit installé correspondant et examinez les arguments de la commande. Un enregistrement à l'échelle de la machine peut entraîner une exécution dans la session d'un utilisateur qui se connecte ; l'étiquette `SYSTEM` de la sortie décrit la portée de la configuration, et ne prouve pas le jeton du processus résultant.

### Boot Execute {#boot_execute}

**ID de vérification :** `boot_execute` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit la chaîne multiple `BootExecute` du Session Manager, en conservant chaque entrée non vide. Elle enregistre les commandes configurées pour une phase précoce du démarrage.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**Comment investiguer :** `autocheck autochk *` est une valeur par défaut familière, et les outils de disque peuvent ajouter des entrées légitimes. Examinez les commandes ajoutées et leurs fichiers sans supposer que chaque entrée non par défaut s'est exécutée ou est malveillante.

### Programme de vérification du démarrage (Boot Verification Program) {#boot_verification_program}

**ID de vérification :** `boot_verification_program` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit l'`ImagePath` de `BootVerificationProgram`, un emplacement d'exécutable associé à la vérification du démarrage.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**Comment investiguer :** vérifiez si ce mécanisme ancien s'applique à la version de Windows et à la configuration de démarrage collectées. Résolvez le fichier cible et comparez sa finalité avec les logiciels de gestion du démarrage attendus.

### Valeur Load de l'Explorateur {#explorer_load}

**ID de vérification :** `explorer_load` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit la valeur `Load` associée à la configuration d'initialisation de la connexion Windows, dans les ruches machine et utilisateur.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**Comment investiguer :** examinez les commandes inconnues, les destinations accessibles en écriture par l'utilisateur et les arguments qui récupèrent ou décodent du contenu. Confirmez séparément l'applicabilité et l'exécution ; un résultat vide ne couvre pas les autres mécanismes de connexion.

### Pilotes de polices (Font Drivers) {#font_drivers}

**ID de vérification :** `font_drivers` · **ATT&CK du catalogue :** [T1547](https://attack.mitre.org/techniques/T1547/)

**Ce que Juntei vérifie :** énumère les valeurs non vides de la clé de Registre historique `Font Drivers` en tant qu'enregistrements potentiels de bibliothèques.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | Toutes les valeurs non vides |

**Comment investiguer :** considérez-la comme une surface de configuration historique inhabituelle. Vérifiez la prise en charge par la version de Windows et la bibliothèque référencée avant de conclure qu'elle pouvait être chargée, et corroborez l'utilisation effective avec des preuves de processus ou de chargement d'image.

### DLL d'extension de stratégie de groupe {#gp_extension_dlls}

**ID de vérification :** `gp_extension_dlls` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `DllName` dans chaque enregistrement immédiat d'extension de stratégie de groupe sous Winlogon. Ces DLL étendent le traitement des stratégies.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**Comment investiguer :** les produits de gestion et de sécurité peuvent installer des extensions. Faites correspondre le GUID de l'extension et la DLL au produit attendu, et examinez les preuves de traitement des stratégies et de chargement d'image avant d'affirmer une exécution.

### Scripts de stratégie de groupe {#gp_scripts}

**ID de vérification :** `gp_scripts` · **ATT&CK du catalogue :** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Ce que Juntei vérifie :** analyse les paires commande/paramètres des fichiers d'enregistrement de la stratégie de groupe locale `scripts.ini` et `psscripts.ini` collectés, y compris les magasins de stratégie locale par utilisateur. Les sections de démarrage/arrêt et d'ouverture/fermeture de session conservent leurs commandes et arguments configurés, les sections machine étant distinguées des sections utilisateur.

**Preuves examinées :**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` et les fichiers `GroupPolicy\User\Scripts` équivalents.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**Comment investiguer :** seuls les emplacements d'enregistrement de stratégie locale implémentés sont examinés ; il ne s'agit pas d'un audit complet de SYSVOL/des GPO. Un fichier de script situé ailleurs dans un répertoire de stratégie ne suffit pas. Corrélez son enregistrement avec les preuves de traitement des stratégies et d'exécution.

### Scripts d'ouverture de session (UserInitMprLogonScript) {#logon_scripts}

**ID de vérification :** `logon_scripts` · **ATT&CK du catalogue :** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Ce que Juntei vérifie :** lit la valeur d'environnement `UserInitMprLogonScript` de l'utilisateur, qui peut désigner un script exécuté lors de la connexion de cet utilisateur.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**Comment investiguer :** comparez avec les scripts d'ouverture de session approuvés et examinez le script effectif. La valeur ne prouve pas une connexion ou une exécution ultérieure, et les scripts fournis par le domaine nécessitent des preuves de stratégie supplémentaires.

### Platform Execute {#platform_execute}

**ID de vérification :** `platform_execute` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit chaque entrée `PlatformExecute` non vide du Session Manager. Elle identifie les programmes sélectionnés pour l'initialisation du démarrage liée à la plateforme.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**Comment investiguer :** des processus matériels ou de provisionnement peuvent expliquer une entrée. Déterminez si le chemin de démarrage de cette build de Windows l'utilise et examinez l'image référencée plutôt que de déduire une attaque du nom de la valeur.

### DLL d'extension du client RDP {#rdp_clx_dll}

**ID de vérification :** `rdp_clx_dll` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit `ClxDllPath` dans la configuration utilisateur par défaut de Terminal Server, en tant qu'emplacement possible d'une extension liée à RDP.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**Comment investiguer :** vérifiez la version de Windows/de RDP et les extensions installées. Une DLL configurée est une piste pour l'analyse des chargements d'image ; la valeur de Registre seule ne montre pas qu'une connexion RDP ou un chargement de DLL a eu lieu.

### DLL de canal virtuel RDP {#rdp_virtual_channel}

**ID de vérification :** `rdp_virtual_channel` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** énumère les valeurs situées sous les enregistrements `Default\AddIns` du Terminal Server Client à l'échelle de la machine. Les valeurs binaires sont décodées en UTF-16LE lorsque c'est possible, ce qui préserve les chemins d'extension exploitables.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>` : toutes les valeurs.

**Comment investiguer :** les clients RDP et les produits de redirection de périphériques enregistrent légitimement des compléments. Examinez la DLL ou la commande effective et la session client concernée ; toutes les valeurs situées sous un complément ne sont pas des chemins d'exécutables.

### Programmes de démarrage RDP WDS {#rdp_wds_startup}

**ID de vérification :** `rdp_wds_startup` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit `StartupPrograms` dans la configuration du pilote de station Windows RDP. Elle identifie les programmes configurés pour le démarrage des sessions à distance.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**Comment investiguer :** `rdpclip` est une entrée familière. Examinez les ajouts et la configuration RDP de l'hôte, et utilisez les enregistrements de sessions/de processus pour déterminer si le programme configuré a effectivement démarré.

### Clés Run du Registre {#run_keys}

**ID de vérification :** `run_keys` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** énumère les valeurs Run, RunOnce, RunEx, RunOnceEx et Run de la stratégie Explorer dans les ruches machine/utilisateur, ainsi que les clés machine 32 bits listées. Les sections RunEx/RunOnceEx sont parcourues avec des limites et des contrôles de cycles.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Toutes les valeurs non vides |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Toutes les valeurs non vides |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Toutes les valeurs non vides |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Toutes les valeurs non vides |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Toutes les valeurs non vides |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | Toutes les valeurs non vides |

**Comment investiguer :** les outils de mise à jour de logiciels et les applications de la zone de notification apparaissent fréquemment ici. Comparez l'exécutable, les arguments de la commande et le contexte utilisateur. Une entrée RunOnce peut disparaître après utilisation ; l'état collecté ne constitue pas un historique complet des connexions antérieures.

### RunServices (historique) {#run_services}

**ID de vérification :** `run_services` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** énumère les valeurs de la clé historique `RunServices` à l'échelle de la machine.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | Toutes les valeurs non vides |

**Comment investiguer :** il s'agit d'une surface de configuration historique, sans garantie qu'une build moderne de Windows l'exécute. Examinez la cible et l'applicabilité au système d'exploitation, puis recherchez des preuves d'exécution indépendantes.

### RunServicesOnce (historique) {#run_services_once}

**ID de vérification :** `run_services_once` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** énumère les valeurs de la clé historique `RunServicesOnce` à l'échelle de la machine.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | Toutes les valeurs non vides |

**Comment investiguer :** considérez les valeurs non vides comme des pistes de configuration. Vérifiez si la version de Windows source prend en charge le mécanisme et si un logiciel de déploiement ou de compatibilité explique l'entrée.

### Commande initiale S0 {#s0_initial_command}

**ID de vérification :** `s0_initial_command` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit `S0InitialCommand` dans le Session Manager en tant que commande potentielle d'initialisation de la session 0.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**Comment investiguer :** établissez l'applicabilité à la build de Windows et à l'état de démarrage collectés. Une commande configurée n'est pas un enregistrement horodaté d'une exécution au démarrage ; examinez son fichier et corrélez avec la télémétrie de démarrage/de processus.

### Execute du Session Manager {#session_manager_execute}

**ID de vérification :** `session_manager_execute` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit les entrées `Execute` non vides du Session Manager, ce qui révèle les programmes sélectionnés pour l'initialisation de la session.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**Comment investiguer :** comparez avec la référence d'images attendue et examinez les exécutables natifs ou chemins inhabituels. Établissez qu'un démarrage/une session pertinent(e) a eu lieu après la modification avant d'affirmer une exécution.

### SubSystems du Session Manager {#session_manager_subsystems}

**ID de vérification :** `session_manager_subsystems` · **ATT&CK du catalogue :** [T1547](https://attack.mitre.org/techniques/T1547/)

**Ce que Juntei vérifie :** lit la valeur `Windows` de la clé `SubSystems` du Session Manager, qui configure la commande du sous-système Windows.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**Comment investiguer :** la commande normale est longue et dépend de la version. Comparez la valeur complète et ses arguments avec une référence fiable de la même build ; modifier un paramètre critique du sous-système peut également empêcher le démarrage.

### Setup Execute {#setup_execute}

**ID de vérification :** `setup_execute` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit les entrées `SetupExecute` du Session Manager, qui couvrent les commandes liées à l'installation configurées pour le démarrage.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**Comment investiguer :** les activités d'installation, de mise à niveau et de récupération peuvent créer des entrées légitimes. Comparez avec les journaux d'installation et l'image référencée avant d'attribuer la configuration à une persistance.

### Redirection du dossier Startup dans Shell Folders {#shell_folders_startup}

**ID de vérification :** `shell_folders_startup` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** compare `Startup` et `Common Startup`, dans Shell Folders comme dans User Shell Folders, aux emplacements Startup par défaut. Les redirections non par défaut sont signalées et le contenu collecté de leurs dossiers est examiné.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` et `User Shell Folders` : `Common Startup`.
- Paramètres `Software\Microsoft\Windows\CurrentVersion\Explorer` équivalents dans NTUSER.DAT : `Startup`, ainsi que les fichiers collectés dans les emplacements redirigés.

**Comment investiguer :** un dossier Startup redirigé peut dissimuler des démarrages automatiques, mais la redirection peut être légitime. Le chemin doit être résolu au sein des preuves collectées ; une destination non collectée est inconnue, et non la preuve que le dossier était vide.

### Substitution du lanceur de shell {#shell_launcher}

**ID de vérification :** `shell_launcher` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit la valeur de stratégie `Shell` et le mappage Shell de la section boot de `system.ini` dans SOFTWARE. Ce sont des surfaces alternatives de sélection du shell.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**Comment investiguer :** les bornes et les appliances applicatives peuvent remplacer intentionnellement l'Explorateur. Comparez le shell sélectionné avec le rôle de la machine et confirmez que le paramètre était effectif pour la session concernée.

### Dossier Startup {#startup_folder}

**ID de vérification :** `startup_folder` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** examine les fichiers situés directement dans les dossiers Startup par défaut, propres à chaque utilisateur et communs, en suivant la première redirection disponible du dossier Startup. Les cibles des raccourcis sont récupérées lorsqu'un `.lnk` collecté peut être analysé.

**Preuves examinées :**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` et `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Les redirections `Common Startup` / `Startup` applicables dans les paramètres Shell Folders et User Shell Folders de la machine/de l'utilisateur.

**Comment investiguer :** les liens et fichiers de démarrage peuvent lancer des applications légitimes à la connexion. Examinez les arguments des cibles ainsi que les noms de fichiers. Le dossier commun est un emplacement de configuration partagé ; il ne prouve pas que le programme lancé a utilisé un jeton SYSTEM.

### Programme initial des services Terminal Server {#ts_initial_program}

**ID de vérification :** `ts_initial_program` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit la valeur de stratégie Terminal Services `InitialProgram` ainsi que la configuration Run/RunOnce/RunOnceEx de l'installation Terminal Server de la machine, y compris les sections RunOnceEx imbriquées, dans certaines limites.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | Toutes les valeurs non vides |

**Comment investiguer :** les déploiements d'applications à distance peuvent utiliser ces paramètres intentionnellement. Établissez les sessions concernées et suivez les commandes sélectionnées jusque dans les preuves de processus ; l'enregistrement seul ne montre pas une ouverture de session à distance.

### Winlogon AppSetup {#winlogon_appsetup}

**ID de vérification :** `winlogon_appsetup` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit `AppSetup` dans Winlogon, en séparant les entrées délimitées par des virgules en candidats distincts.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**Comment investiguer :** comparez chaque composant avec les logiciels d'installation/de connexion Windows attendus. Une entrée de liste ne constitue pas une exécution observée distincte, et sa pertinence peut dépendre de la version de Windows et de l'état du déploiement.

### Winlogon GinaDLL {#winlogon_gina_dll}

**ID de vérification :** `winlogon_gina_dll` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit le paramètre historique de remplacement `GinaDLL` de Winlogon. GINA était un ancien modèle d'extension de la connexion Windows.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**Comment investiguer :** ceci concerne principalement les systèmes antérieurs à Vista. Une clé résiduelle sur un système plus récent ne prouve pas l'existence d'une extension d'ouverture de session effective ; examinez la version source et toute DLL référencée.

**Plus de détails :** [documentation Microsoft](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**ID de vérification :** `winlogon_mpnotify` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit le paramètre de commande/bibliothèque `mpnotify` de Winlogon, une surface d'extension liée à la connexion.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**Comment investiguer :** comparez la cible et les arguments avec une référence Windows correspondante. Vérifiez le comportement propre à la version et corrélez avec une connexion ultérieure avant d'affirmer que du code s'est exécuté.

### Packages Notify de Winlogon {#winlogon_notify_packages}

**ID de vérification :** `winlogon_notify_packages` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit `DllName` dans chaque enregistrement immédiat de package `Notify` de Winlogon.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**Comment investiguer :** les packages de notification sont un mécanisme d'extension historique. Déterminez si la version de Windows collectée les charge et distinguez les logiciels d'authentification approuvés des bibliothèques inexpliquées.

### Winlogon Shell {#winlogon_shell}

**ID de vérification :** `winlogon_shell` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit les valeurs `Shell` de Winlogon à l'échelle de la machine et par utilisateur, afin de révéler la sélection du shell du bureau.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**Comment investiguer :** l'Explorateur est courant, mais les bornes peuvent utiliser un autre shell. Examinez les commandes ajoutées ou les chemins inhabituels et déterminez quelle configuration utilisateur effective s'appliquait à la connexion.

### Winlogon System {#winlogon_system}

**ID de vérification :** `winlogon_system` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit la valeur `System` de Winlogon et sépare les entrées délimitées par des virgules en candidats individuels.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**Comment investiguer :** considérez ceci comme une piste de configuration de connexion sensible à la version. Examinez chaque composant référencé et corroborez avec des preuves d'exécution plutôt que de traiter la liste comme un historique d'exécution.

### Winlogon Taskman {#winlogon_taskman}

**ID de vérification :** `winlogon_taskman` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit les valeurs `Taskman` de Winlogon de la machine/de l'utilisateur, à la recherche d'une commande alternative de gestion des tâches.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**Comment investiguer :** déterminez quel comportement de Windows utiliserait ce paramètre et si la cible est attendue. Un remplacement configuré ne prouve pas qu'un utilisateur a invoqué cette fonctionnalité.

### Winlogon Userinit {#winlogon_userinit}

**ID de vérification :** `winlogon_userinit` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit les valeurs `Userinit` de Winlogon de la machine/de l'utilisateur, en séparant les programmes délimités par des virgules et en ignorant les éléments finaux vides.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**Comment investiguer :** une entrée normale pointe souvent vers `userinit.exe`. Examinez les entrées supplémentaires et les chemins modifiés. Confirmez l'utilisateur concerné et une connexion ultérieure ; `KeyLastWrite` appartient à la clé entière, et non spécifiquement à cette valeur.

### Winlogon VMApplet {#winlogon_vmapplet}

**ID de vérification :** `winlogon_vmapplet` · **ATT&CK du catalogue :** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Ce que Juntei vérifie :** lit le paramètre de commande `VMApplet` de Winlogon, qui couvre une ancienne surface de lancement d'applet système.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**Comment investiguer :** vérifiez l'applicabilité au système d'exploitation et l'applet attendu. Une commande inattendue mérite d'être examinée, mais n'établit pas que l'interface déclenchante a été ouverte.

## Services et extensions du système { #services-and-system-extensions }

Les services Windows et les composants système extensibles offrent de nombreux moyens légitimes de charger des exécutables ou des DLL. Comparez la cible configurée avec le produit attendu, le compte de service, les paramètres de démarrage et l'historique d'installation étayé par les sources.

### DLL DiagTrack {#diagtrack_dll}

**ID de vérification :** `diagtrack_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit l'`ImagePath` du service DiagTrack. Malgré le nom du catalogue, orienté DLL, cette vérification examine la commande d'image configurée du service.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**Comment investiguer :** comparez avec la référence du service et avec les vérifications générales des services. Une même configuration peut produire plusieurs résultats de vérification ; le nombre de candidats n'est pas un nombre d'attaques distinctes.

### DLL DiagTrack Listener {#diagtrack_listener_dll}

**ID de vérification :** `diagtrack_listener_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `FileName` dans la configuration de l'autologger DiagTrack-Listener. Il s'agit d'un paramètre d'autologger ETW, et non d'une preuve du chargement d'une DLL.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**Comment investiguer :** examinez la valeur brute et la configuration de journalisation de Windows. Le catalogue classe ceci comme une technique DLL, mais la recherche ne valide pas une DLL chargeable et ne prouve pas une exécution par l'intermédiaire de cette valeur.

### DLL d'assistance Netsh {#netsh_helper}

**ID de vérification :** `netsh_helper` · **ATT&CK du catalogue :** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Ce que Juntei vérifie :** énumère les valeurs non vides sous la clé d'enregistrement des assistants `NetSh` de la machine. Les assistants peuvent étendre `netsh.exe` avec des commandes fondées sur des DLL.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | Toutes les valeurs non vides |

**Comment investiguer :** des produits réseau peuvent installer des assistants. Faites correspondre chaque valeur à un produit attendu et examinez la bibliothèque résolue, puis recherchez les exécutions de netsh ou les chargements d'image pertinents.

### Moniteurs d'impression {#print_monitors}

**ID de vérification :** `print_monitors` · **ATT&CK du catalogue :** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Ce que Juntei vérifie :** lit `Driver` sous chaque enregistrement immédiat de Print Monitor dans le jeu de contrôles actif.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**Comment investiguer :** les logiciels d'impression et les moniteurs de ports étendent légitimement le spouleur. Comparez la DLL et les métadonnées du produit avec l'inventaire des imprimantes, et confirmez le chargement à l'aide de preuves du spouleur/de chargement d'image.

### Processeurs d'impression {#print_processors}

**ID de vérification :** `print_processors` · **ATT&CK du catalogue :** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Ce que Juntei vérifie :** lit `Driver` sous les arborescences d'enregistrement des processeurs d'impression x64 et x86.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**Comment investiguer :** les suites d'impression tierces peuvent ajouter des processeurs au-delà des valeurs par défaut familières de Windows. Examinez les deux architectures, la DLL effective et les logiciels d'impression installés avant d'attribuer une intention malveillante.

### Extension du Service Control Manager {#scm_extension}

**ID de vérification :** `scm_extension` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** énumère les valeurs non vides de `ServiceControlManagerExtension`, une surface d'extension du gestionnaire de services.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | Toutes les valeurs non vides |

**Comment investiguer :** examinez les bibliothèques nommées et déterminez si cette build de Windows utilise le paramètre. Corrélez avec l'activité de démarrage/des services ; la vérification inventorie des enregistrements plutôt que de prouver le chargement d'une extension.

### DLL de plugin de niveau serveur DNS {#server_level_plugin_dll}

**ID de vérification :** `server_level_plugin_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit le paramètre `ServerLevelPluginDll` du service DNS dans le jeu de contrôles SYSTEM actif.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**Comment investiguer :** ceci concerne les systèmes exécutant le rôle Serveur DNS, y compris certains contrôleurs de domaine. Examinez la DLL, le compte de service et les preuves de redémarrage/de chargement du service ; un rôle inactif ou absent modifie la conclusion.

### Commande en cas d'échec de service {#service_failure_command}

**ID de vérification :** `service_failure_command` · **ATT&CK du catalogue :** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Ce que Juntei vérifie :** lit la valeur `FailureCommand` de chaque service dans le jeu de contrôles actif, ce qui révèle une commande de récupération configurée.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**Comment investiguer :** la commande ne s'exécute que lorsque les actions de récupération du service la sélectionnent et que l'échec correspondant se produit. Examinez la configuration de récupération et les enregistrements d'échecs/de processus ; la valeur seule n'établit pas une action de récupération active.

### DLL d'agent d'extension SNMP {#snmp_extension_agent}

**ID de vérification :** `snmp_extension_agent` · **ATT&CK du catalogue :** [T1574](https://attack.mitre.org/techniques/T1574/)

**Ce que Juntei vérifie :** énumère les valeurs sous la clé d'enregistrement `ExtensionAgents` du service SNMP. Les valeurs peuvent désigner un autre emplacement du Registre plutôt qu'un chemin direct de DLL.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | Toutes les valeurs non vides |

**Comment investiguer :** suivez manuellement un enregistrement indirect jusqu'à sa bibliothèque et vérifiez que SNMP est installé et utilisé. Une chaîne non résolue dans cette sortie ne prouve pas que la DLL est absente ou malveillante.

### Fournisseurs de temps {#time_providers}

**ID de vérification :** `time_providers` · **ATT&CK du catalogue :** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Ce que Juntei vérifie :** lit `DllName` dans les enregistrements `TimeProviders` immédiats du service Temps Windows.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**Comment investiguer :** examinez séparément les paramètres d'activation des fournisseurs et l'état du service Temps Windows ; la vérification du chemin de DLL n'exige pas un fournisseur activé. Des produits de synchronisation de l'heure peuvent fournir des remplacements légitimes.

### Service Windows (ServiceDll) {#windows_service_dll}

**ID de vérification :** `windows_service_dll` · **ATT&CK du catalogue :** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Ce que Juntei vérifie :** lit `Parameters\ServiceDll` sous les services du jeu de contrôles SYSTEM actif et joint les preuves d'horodatage propres au service lorsqu'elles sont disponibles.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**Comment investiguer :** un hôte partagé tel que `svchost.exe` n'est pas la DLL de la charge utile. Examinez la bibliothèque, le compte de service, l'état de démarrage et les preuves correspondantes d'installation du service/de chargement d'image ; un enregistrement ne prouve pas que le service s'est exécuté.

**Plus de détails :** [documentation Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Service Windows (ImagePath) {#windows_service_image_path}

**ID de vérification :** `windows_service_image_path` · **ATT&CK du catalogue :** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Ce que Juntei vérifie :** lit les commandes `ImagePath` des services dans le jeu de contrôles actif. Les pilotes de type 1, 2 ou 8 sans ImagePath conservent une cible implicite `System32\drivers\<service>.sys`.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>` : `ImagePath`, `Type` et les métadonnées de service associées.

**Comment investiguer :** examinez le compte de service et les paramètres de démarrage indépendamment de l'étiquette de portée machine de la sortie. Les enregistrements désactivés ou à démarrage à la demande font tout de même partie de l'inventaire. Pour les horodatages, les preuves fondées sur le nom du service sont préférées aux chemins d'exécutables partagés ambigus.

**Plus de détails :** [documentation Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### DLL de démarrage du service Windows Update {#wu_service_startup_dll}

**ID de vérification :** `wu_service_startup_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit la valeur `Parameters\ServiceDll` du service Windows Update dans le jeu de contrôles actif.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**Comment investiguer :** comparez la bibliothèque avec la même build de Windows et l'état de maintenance installé. Cette vérification recoupe l'inventaire générique ServiceDll et ne représente pas une seconde compromission distincte.

## Tâches planifiées { #scheduled-tasks }

Le Planificateur de tâches stocke la configuration à la fois dans des fichiers XML et dans la ruche SOFTWARE. Lisez ces sources ensemble. Une tâche peut être enregistrée sans avoir été exécutée, et une collecte incomplète peut ressembler à une tâche cachée ou fantôme.

### Tâche planifiée fantôme {#ghost_task}

**ID de vérification :** `ghost_task` · **ATT&CK du catalogue :** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Ce que Juntei vérifie :** compare les enregistrements `Tree` et `Tasks` du TaskCache de SOFTWARE et, lorsque des fichiers de tâches existent dans la collecte, recherche les fichiers XML de tâche correspondants.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` et `TaskCache\Tasks`.
- Les fichiers `Windows\System32\Tasks\<task path>` collectés, pour la comparaison avec le système de fichiers.

**Comment investiguer :** des contreparties manquantes peuvent refléter des tâches supprimées/obsolètes, des enregistrements intégrés ou une collecte incomplète. L'absence de XML collecté signifie que la comparaison avec le système de fichiers n'est pas disponible. Examinez la couverture et l'écart exact avant de qualifier une tâche de cachée ou de malveillante.

### Tâche planifiée cachée {#hidden_scheduled_task}

**ID de vérification :** `hidden_scheduled_task` · **ATT&CK du catalogue :** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Ce que Juntei vérifie :** contrôle les enregistrements Tree du TaskCache à la recherche d'une valeur `SD` manquante ou d'un descripteur de sécurité binaire vide, conditions associées au fait de dissimuler une tâche à l'énumération ordinaire.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>` : présence/données de `SD`, avec les métadonnées de la contrepartie TaskCache.

**Comment investiguer :** un SD manquant/vide est une anomalie de configuration spécifique, et non la preuve que la tâche s'est exécutée ou qu'elle est malveillante. Le paramètre XML `Hidden` et les problèmes d'ACL arbitraires sont des conditions différentes et ne constituent pas le prédicat utilisé ici.

**Plus de détails :** [documentation Microsoft](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Tâche planifiée (fichiers XML) {#scheduled_task_files}

**ID de vérification :** `scheduled_task_files` · **ATT&CK du catalogue :** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Ce que Juntei vérifie :** analyse les fichiers XML de tâches collectés pour en extraire les commandes/arguments Exec ainsi que les ID de classe et les données des gestionnaires COM. Elle sélectionne le principal d'après `Actions Context`/`Principal id`, conserve séparément l'identité de l'utilisateur et celle du groupe, et signale SYSTEM, USER ou UNKNOWN. Les actions COM non résolues restent visibles.

**Preuves examinées :**

- Les fichiers XML `Windows\System32\Tasks\<task path>`, y compris Exec, ComHandler et les champs de principal disponibles.
- Le `UsrClass.dat` et le `Software\Classes` de NTUSER collectés pour l'utilisateur correspondant, ainsi que `Classes\CLSID` du SOFTWARE de la machine, dans les vues native et 32 bits.
- Le `ProfileList` de SOFTWARE collecté pour les principaux exprimés sous forme de SID. Les principaux nommés privilégient un dossier `user.DOMAIN` correspondant unique, puis les suffixes d'hôte local collectés, et enfin le dossier `user` simple. Les correspondances sont explicitement étiquetées comme heuristiques ; une véritable ambiguïté reste non résolue et est signalée dans le résultat et dans la couverture. Les clés de Registre volatiles ne sont pas nécessaires.
- Le `CodeBase` managé (y compris les sous-clés de version), les associations AppID et l'`ImagePath` du service SYSTEM pour un `LocalService` d'AppID.

Les candidats de serveurs COM indiquent leur source dans le Registre dans `Description`. Les associations AppID
ne comportant qu'un nom d'exécutable restent non résolues au lieu de se voir attribuer un répertoire deviné. Les URL
`file:` locales sont décodées ; les URL distantes sont conservées sans accès réseau. Les substitutions de classes utilisateur
sont limitées à ce profil, et un CLSID utilisateur incomplet ne se rabat pas sur l'enregistrement machine
qu'il masque. Les tâches définies uniquement pour un groupe conservent les candidats machine, la limitation relative aux utilisateurs individuels
étant indiquée dans leur description, sans ajouter d'avis de routine au résumé des erreurs de couverture.

**Comment investiguer :** examinez les déclencheurs, l'état d'activation, le principal et l'historique d'exécution dans le XML source et les journaux d'événements. Une tâche créée peut ne jamais s'exécuter. Un gestionnaire COM non résolu reste un candidat, et des données de principal manquantes ou limitées à un groupe ne doivent pas être interprétées comme signifiant SYSTEM.

## WMI, BITS et autres déclencheurs d'événements { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) prend en charge la gestion du système. Background Intelligent Transfer Service (BITS) prend en charge les transferts en arrière-plan. Tous deux peuvent conserver des commandes déclenchées par des événements. Certains mécanismes attendent un événement de gestion, une notification de transfert, le lancement d'une application ou une action de l'utilisateur. Un déclencheur stocké est une preuve de configuration utile, mais sa présence n'établit pas que le déclencheur s'est activé.

### Porte dérobée des fonctionnalités d'accessibilité {#accessibility_tools}

**ID de vérification :** `accessibility_tools` · **ATT&CK du catalogue :** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Ce que Juntei vérifie :** compare sept binaires d'accessibilité de System32 collectés à l'aide du champ PE OriginalFilename et des métadonnées de certificat/catalogue, après avoir exigé des métadonnées de catalogue Microsoft exploitables provenant d'un binaire de référence collecté.

**Preuves examinées :**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` et `DisplaySwitch.exe`.
- Les métadonnées de référence de `Windows\System32\winlogon.exe`, `cmd.exe` ou `notepad.exe` collectés, ainsi que les catalogues collectés applicables.

**Comment investiguer :** les métadonnées peuvent suggérer un remplacement renommé, mais Juntei ne valide ni les signatures ni la confiance dans les certificats. Une collecte partielle dépourvue des catalogues nécessaires produit une couverture de comparaison incomplète, plutôt que de traiter comme porte dérobée chaque fichier paraissant non signé.

### Fonctionnalités d'accessibilité {#assistive_technology}

**ID de vérification :** `assistive_technology` · **ATT&CK du catalogue :** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Ce que Juntei vérifie :** lit les commandes d'accessibilité enregistrées `StartExe` et `StartParams` dans les vues native/32 bits de SOFTWARE, ainsi que les entrées d'accessibilité `Configuration` activées dans les ruches utilisateur et DEFAULT. Les valeurs de substitution StartExe numériques/non textuelles sont ignorées.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>` : `StartExe` et `StartParams` ; ainsi que la vue machine `Wow6432Node`.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` dans NTUSER.DAT et le paramètre équivalent dans la ruche DEFAULT.

**Comment investiguer :** les logiciels d'accessibilité sont légitimes. Distinguez le nom d'une fonctionnalité activée de l'enregistrement de son exécutable effectif, puis examinez la cible et les preuves d'exécution à la connexion/sur le bureau. Ceci est distinct du remplacement de l'exécutable d'accessibilité lui-même.

**Plus de détails :** [référence de la technique (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### Commande de notification de tâche BITS {#bits_notify_command}

**ID de vérification :** `bits_notify_command` · **ATT&CK du catalogue :** [T1197](https://attack.mitre.org/techniques/T1197/)

**Ce que Juntei vérifie :** récupère les enregistrements de tâches BITS sous forme de chaînes préfixées par leur longueur dans les fichiers `qmgr.db`, `qmgr0.dat` ou `qmgr1.dat` collectés, en conservant les programmes de notification non vides, les paramètres, les ID de tâche et les SID des propriétaires.

**Preuves examinées :**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` et `qmgr1.dat`.

**Comment investiguer :** il s'agit d'enregistrements récupérés par carving : l'état alloué/actuel des tâches n'est pas validé, et des tâches obsolètes/supprimées peuvent apparaître. L'heure d'écriture MFT de la base de données est une preuve faible portant sur le conteneur, et non l'heure d'exécution de la notification. Les SID des propriétaires distinguent SYSTEM, le service local, le service réseau et les utilisateurs.

### AutoRun du processeur de commandes {#cmd_autorun}

**ID de vérification :** `cmd_autorun` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit les commandes `AutoRun` du Command Processor de la machine/de l'utilisateur, un paramètre qui peut affecter le démarrage de l'invite de commandes.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**Comment investiguer :** les outils de développement et la personnalisation du shell peuvent expliquer une valeur. `cmd.exe /d` contourne AutoRun ; la configuration seule ne prouve donc pas qu'elle s'est exécutée à chaque apparition de cmd.exe. Examinez le contenu de la commande et les arguments du processus déclencheur.

**Plus de détails :** [documentation Microsoft](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### Persistance via ErrorHandler.cmd {#error_handler_cmd}

**ID de vérification :** `error_handler_cmd` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** signale la présence du script `Windows\System32\ErrorHandler.cmd` collecté en tant que candidat de persistance par gestionnaire d'erreurs.

**Preuves examinées :**

- `Windows\System32\ErrorHandler.cmd`.

**Comment investiguer :** la présence seule ne prouve pas que le chemin de récupération/d'erreur Windows applicable l'a invoqué. Lisez le script, examinez sa provenance et corrélez avec les preuves d'installation, d'erreur et de processus.

### Flux Power Automate Desktop {#power_automate}

**ID de vérification :** `power_automate` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** recherche les répertoires de flux/de scripts collectés dans les magasins `flows` et `scripts` de Power Automate Desktop de chaque utilisateur, en signalant chaque flux découvert une seule fois.

**Preuves examinées :**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` et `scripts\<flow>\...`.

**Comment investiguer :** ceci identifie une automatisation stockée, et non chaque déclencheur planifié ni un flux exécuté. Examinez le contenu du flux et la configuration de lancement externe avant de le considérer comme une persistance automatique ou malveillante.

### Profil PowerShell {#powershell_profiles}

**ID de vérification :** `powershell_profiles` · **ATT&CK du catalogue :** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Ce que Juntei vérifie :** recherche les fichiers de type profile.ps1 dans les répertoires d'installation/de profil reconnus de Windows PowerShell et de PowerShell, y compris les Documents de l'utilisateur, les Documents OneDrive découverts et les redirections de Documents collectées.

**Preuves examinées :**

- Les fichiers se terminant par `profile.ps1` directement dans `Windows\System32\WindowsPowerShell\v1.0` et `Windows\SysWOW64\WindowsPowerShell\v1.0`.
- `Documents\WindowsPowerShell` et `Documents\PowerShell` de chaque utilisateur, y compris les redirections de Documents collectées et les dossiers Documents OneDrive découverts.
- Les fichiers se terminant par `profile.ps1` directement dans les répertoires de version sous `Program Files\PowerShell` ou `Program Files (x86)\PowerShell`.

**Comment investiguer :** les profils peuvent contenir une personnalisation inoffensive. Lisez le script et suivez les charges utiles invoquées. Lancer PowerShell avec `-NoProfile` les ignore ; la seule présence d'un fichier ne prouve ni l'exécution du profil ni un contenu malveillant.

### Détournement de l'écran de veille {#screensaver}

**ID de vérification :** `screensaver` · **ATT&CK du catalogue :** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Ce que Juntei vérifie :** lit la valeur Desktop `SCRNSAVE.EXE` de chaque utilisateur, qui identifie l'image d'écran de veille configurée.

**Preuves examinées :**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` dans NTUSER.DAT.

**Comment investiguer :** comparez avec les écrans de veille approuvés et examinez la cible. Les autres paramètres du bureau, l'état de la session et le délai d'inactivité déterminent s'il peut se déclencher ; un chemin enregistré ne prouve pas qu'un écran de veille a démarré.

### Commande du TelemetryController {#telemetry_controller}

**ID de vérification :** `telemetry_controller` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit `Binary`, `Command` et `MaintenanceCommand` dans TelemetryController et ses enregistrements enfants immédiats.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` et ses enfants immédiats : `Binary`, `Command`, `MaintenanceCommand`.

**Comment investiguer :** comparez l'exécutable sélectionné et ses arguments avec la même build de Windows et les logiciels de maintenance. Une commande dans le Registre ne démontre pas qu'un déclencheur de maintenance s'est activé.

### Profils personnalisés de Windows Terminal {#windows_terminal}

**ID de vérification :** `windows_terminal` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** analyse les paramètres JSON/JSONC de Windows Terminal (versions empaquetée, préliminaire et non empaquetée) et signale les paramètres `commandline` non vides des profils et des valeurs par défaut.

**Preuves examinées :**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` et l'équivalent WindowsTerminalPreview.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json` : anciens tableaux de profils, champs commandline de `profiles.list` et de `profiles.defaults`.

**Comment investiguer :** les shells ordinaires apparaissent naturellement ici. Déterminez le profil sélectionné/par défaut et examinez les commandes ou arguments de lancement inhabituels. Le simple enregistrement d'un profil ne le lance pas.

### Abonnement aux événements WMI {#wmi_event_subscription}

**ID de vérification :** `wmi_event_subscription` · **ATT&CK du catalogue :** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Ce que Juntei vérifie :** récupère par carving les instances de consommateurs d'événements reconnues et les références de liaison filtre-consommateur dans le référentiel WMI OBJECTS.DATA collecté. Les classes reconnues incluent les consommateurs de ligne de commande, de script actif, de fichier journal, de journal d'événements et SMTP.

**Preuves examinées :**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` et `Repository\FS\OBJECTS.DATA`.

**Comment investiguer :** une occurrence brute dans le référentiel peut être obsolète, non liée ou historique ; la sortie conserve cette réserve. Le carving ne reconstitue pas un graphe complet des abonnements actifs et ne prouve pas que le déclencheur s'est activé. L'heure d'écriture du fichier du référentiel n'est pas une heure de création/d'exécution de l'abonnement.

## Explorateur, COM et gestionnaires de fichiers ou d'URL { #explorer-com-and-file-or-url-handlers }

Le Component Object Model (COM) permet aux applications d'activer des composants logiciels enregistrés. Le shell Windows utilise des gestionnaires enregistrés pour ouvrir des fichiers, des URL, des dossiers et des objets spéciaux. Les identifiants COM (CLSID) identifient les composants ; les ProgID donnent à certains composants des noms lisibles. Suivez chaque enregistrement jusqu'à la commande ou à la DLL qu'il sélectionne.

### Détournement d'App Paths {#app_paths}

**ID de vérification :** `app_paths` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit les valeurs par défaut sous les enregistrements `App Paths` à l'échelle de la machine, qui peuvent rediriger la recherche d'applications lors des lancements effectués par le shell.

**Preuves examinées :**

- Valeur par défaut de `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`.

**Comment investiguer :** un enregistrement App Paths est courant pour les applications installées et ne constitue pas une réécriture générale de chaque lancement de processus. Comparez le nom et l'image enregistrés avec les logiciels attendus, puis examinez les preuves de shell/de processus pertinentes.

### Détournement d'objet COM {#com_treat_as}

**ID de vérification :** `com_treat_as` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** recherche les redirections COM `TreatAs` et les enregistrements InprocServer32/LocalServer32 par utilisateur qui masquent un CLSID machine, dans les magasins de classes native/32 bits pris en charge. Les cibles TreatAs sont résolues à l'aide des classes du même utilisateur et des classes de la machine.

**Preuves examinées :**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` et la vue machine `Classes\Wow6432Node`.
- `Software\Classes\CLSID` par utilisateur dans NTUSER.DAT et `CLSID` dans UsrClass.dat, y compris leurs vues `Wow6432Node` implémentées : valeurs par défaut de `TreatAs`, `InprocServer32` et `LocalServer32`.

**Comment investiguer :** les substitutions par utilisateur peuvent ne nécessiter aucun droit d'administrateur, mais des logiciels légitimes peuvent également les enregistrer. Suivez la classe et la DLL/l'exécutable sélectionnés, évaluez l'architecture et le contexte d'activation, et corroborez de manière indépendante l'activation COM effective.

### Détournement de gestionnaire du Nettoyage de disque {#disk_cleanup_handler}

**ID de vérification :** `disk_cleanup_handler` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** lit les CLSID des gestionnaires VolumeCaches du Nettoyage de disque et suit les enregistrements COM InprocServer32 de la machine pouvant être résolus jusqu'à leurs bibliothèques.

**Preuves examinées :**

- CLSID par défaut de `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` et `Classes\CLSID\<CLSID>\InprocServer32`.

**Comment investiguer :** le gestionnaire doit être invoqué par le processus de nettoyage concerné pour s'exécuter. Les gestionnaires non résolus ne sont pas émis par cette vérification particulière ; utilisez donc le parseur de Registre pour examiner leurs enregistrements bruts si la collecte est incomplète.

### Substitution AppKey de l'Explorateur {#explorer_app_key}

**ID de vérification :** `explorer_app_key` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit les valeurs `ShellExecute` et `Association` sous les enfants immédiats `AppKey` de l'Explorateur, qui couvrent les commandes/actions sélectionnées par les touches d'application.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>` : `ShellExecute` et `Association`.

**Comment investiguer :** les utilitaires de clavier et les outils OEM peuvent définir ces valeurs. Un nom d'association peut nécessiter un autre pivot dans le Registre au lieu d'être un chemin de fichier. Vérifiez l'action de l'utilisateur qui le déclencherait.

### Browser Helper Objects {#explorer_bho}

**ID de vérification :** `explorer_bho` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** lit les enregistrements de CLSID des Browser Helper Objects et résout les bibliothèques InprocServer32 de la machine lorsqu'elles sont disponibles ; les ID de classe non résolus restent visibles.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` et la valeur par défaut `Classes\CLSID\<CLSID>\InprocServer32` de la machine.

**Comment investiguer :** ceci est lié au comportement des navigateurs compatibles/de l'Explorateur et concerne particulièrement les environnements anciens. Des extensions de navigateur/de sécurité approuvées peuvent apparaître. Un BHO enregistré ne prouve pas qu'un navigateur actuel l'a chargé.

### Détournement de CLSID du bureau de l'Explorateur {#explorer_clsid_hijack}

**ID de vérification :** `explorer_clsid_hijack` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** lit les commandes d'ouverture du shell configurées pour les trois CLSID spéciaux de l'espace de noms de l'Explorateur listés, dans les magasins de classes machine/utilisateur.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**Comment investiguer :** comparez les gestionnaires de commandes du Bureau, du Poste de travail et de Mes documents avec une référence correspondante. Cette vérification contrôle des CLSID et des chemins sélectionnés, et non tous les détournements possibles de l'espace de noms de l'Explorateur.

### Gestionnaires de menu contextuel de l'Explorateur {#explorer_context_menu}

**ID de vérification :** `explorer_context_menu` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** énumère les gestionnaires de menu contextuel pour Directory, Directory Background, Folder, Drive, AllFilesystemObjects et `*`, en suivant les ID de classe jusqu'aux enregistrements de DLL collectés lorsque c'est possible.

**Preuves examinées :**

- Magasins de classes de la machine `HKLM\SOFTWARE\Classes` et de l'utilisateur UsrClass.dat : valeur par défaut de `<subject>\shellex\ContextMenuHandlers\<handler>` et enregistrements de bibliothèques COM correspondants.

**Comment investiguer :** les extensions de clic droit des outils d'archivage, des produits de sécurité et des clients cloud sont courantes. Examinez le gestionnaire effectif et l'interaction déclenchante. Les GUID non résolus restent des pistes utiles, et non la preuve que la DLL n'existait pas.

### Détournement d'association de fichiers {#file_association_hijack}

**ID de vérification :** `file_association_hijack` · **ATT&CK du catalogue :** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Ce que Juntei vérifie :** examine les enregistrements `shell\open\command` des extensions directes et des ProgID résolus pour `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` et `.ps1`.

**Preuves examinées :**

- Magasins de classes de la machine `HKLM\SOFTWARE\Classes` et de l'utilisateur UsrClass.dat : valeurs par défaut des `<extension>` sélectionnées et valeurs par défaut de `<extension-or-ProgID>\shell\open\command`.

**Comment investiguer :** comparez avec l'application attendue et examinez les arguments de la commande. Il s'agit d'un ensemble défini d'extensions, et non de tous les types de fichiers ni de la résolution UserChoice effective complète. Une commande enregistrée ne prouve pas qu'un document a été ouvert.

### Détournement de gestionnaire de protocole {#protocol_handler_hijack}

**ID de vérification :** `protocol_handler_hijack` · **ATT&CK du catalogue :** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Ce que Juntei vérifie :** lit les commandes d'ouverture d'URL pour http, https, mailto, ms-msdt et ms-officecmd, ainsi que les enregistrements `URL Protocol` découverts et les substitutions utilisateur applicables.

**Preuves examinées :**

- Magasins de classes de la machine `HKLM\SOFTWARE\Classes` et de l'utilisateur UsrClass.dat : marqueurs `URL Protocol` et valeurs par défaut de `<scheme>\shell\open\command`.

**Comment investiguer :** les navigateurs et les programmes d'installation d'applications enregistrent légitimement des schémas. Suivez la commande et ses arguments, évaluez la finalité attendue du schéma et corrélez avec les preuves d'ouverture d'URL/de processus.

### Gestionnaire d'extension COM de la Corbeille {#recycle_bin_com_extension}

**ID de vérification :** `recycle_bin_com_extension` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** lit les substitutions de commandes open/empty/explore du CLSID de la Corbeille listées, ainsi que les enregistrements immédiats de gestionnaires de menu contextuel/de glisser-déposer.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**Comment investiguer :** comparez avec les extensions du shell approuvées. Certaines valeurs signalées sont des identifiants COM plutôt qu'une bibliothèque directement résolue ; examinez la classe référencée et l'action de l'utilisateur avant d'affirmer une exécution de code.

### Détournement du gestionnaire de protocole de recherche {#search_protocol_handler}

**ID de vérification :** `search_protocol_handler` · **ATT&CK du catalogue :** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Ce que Juntei vérifie :** lit le gestionnaire par défaut `search-ms\shell\open\command` de la machine/de l'utilisateur.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**Comment investiguer :** une modification du gestionnaire peut changer ce qui se passe à l'ouverture d'une URL search-ms, mais sa présence ne prouve pas la livraison d'un exploit. Comparez avec les valeurs par défaut de Windows et examinez l'activité d'URL/de processus qui l'utiliserait.

### SharedTaskScheduler {#shared_task_scheduler}

**ID de vérification :** `shared_task_scheduler` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** énumère les valeurs `SharedTaskScheduler` non vides de l'Explorateur en tant que composants du shell enregistrés.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | Toutes les valeurs non vides |

**Comment investiguer :** les valeurs peuvent référencer des classes COM plutôt qu'un chemin d'exécutable. Suivez leurs enregistrements de serveur et vérifiez le comportement du système d'exploitation, la provenance de l'extension et les preuves pertinentes de chargement par le shell.

### ShellExecuteHooks {#shell_execute_hooks}

**ID de vérification :** `shell_execute_hooks` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** énumère les valeurs `ShellExecuteHooks` non vides de l'Explorateur, qui couvrent les enregistrements historiques de hooks d'exécution du shell.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | Toutes les valeurs non vides |

**Comment investiguer :** résolvez tout CLSID référencé et vérifiez si la build de Windows et la stratégie autorisent le hook. Un enregistrement historique peut être inutilisé ; la valeur elle-même n'est pas une exécution observée par le shell.

### Persistance COM via les bibliothèques et dossiers CLSID du menu Démarrer {#start_menu_com}

**ID de vérification :** `start_menu_com` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** recherche les fichiers XML de bibliothèque `.library-ms` dont l'URL `simpleLocation` référence
un CLSID COM, ainsi que les répertoires nommés `<name>.{CLSID}`, sous
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. L'interaction de l'Explorateur avec ces
objets peut activer du code COM. La vérification inclut les répertoires vides collectés ; elle ne suit pas
les liens symboliques. Les fichiers de bibliothèque ordinaires situés hors du menu Démarrer et les GUID figurant dans des commentaires XML ne sont pas
signalés par cette vérification.

**Preuves examinées :**

- Les fichiers `.library-ms` et les entrées de répertoires nommées d'après un CLSID dans le menu Démarrer de chaque utilisateur.
- Les enregistrements de classes collectés de l'utilisateur correspondant et le SOFTWARE de la machine, séparément pour chaque architecture.
- La configuration CodeBase managé/AppID/service, au moyen du même résolveur que pour les tâches planifiées.

**Comment investiguer :** `Path` et `SourceFile` identifient l'artefact déclencheur de l'activation. `Value`
contient la commande de serveur candidate, ou le CLSID si la résolution n'est pas disponible. `Description`
relie le déclencheur au CLSID et à la source du Registre collectée. `ResolvedPath`/`Sha256` ne décrivent
qu'une charge utile résolue et collectée. Une association AppID ne comportant qu'un nom d'exécutable reste non résolue.
Un déclencheur peut être légitime ; son existence ne prouve ni une connexion, ni une activation, ni une exécution, ni une attaque.
Examinez l'URL de la bibliothèque, l'enregistrement de classe, la charge utile, le contexte utilisateur et des journaux d'exécution indépendants.

**Précautions relatives à la collecte :** les collectes limitées aux fichiers peuvent omettre les répertoires vides. Les chemins standard du menu Démarrer
de l'utilisateur sont examinés ; les menus Démarrer redirigés et les autres emplacements de l'espace de noms de l'Explorateur ne sont pas couverts
par cette vérification. Les ruches manquantes ou les enregistrements non résolus produisent une couverture partielle tout en conservant
le candidat. L'architecture de la tâche, l'élévation et les stratégies peuvent modifier le comportement effectif de la recherche COM.

**Plus de détails :** [proposition originale et crédit du chercheur](https://github.com/last-byte/PersistenceSniper/pull/32),
[vues fusionnées COM par utilisateur/machine](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE : détournement COM](https://attack.mitre.org/techniques/T1546/015/).

### Détournement COM via TypeLib {#typelib_hijack}

**ID de vérification :** `typelib_hijack` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** lit les enregistrements TypeLib GUID/version/paramètres régionaux win32/win64 du UsrClass.dat de chaque utilisateur, en conservant les chemins et les valeurs de moniker de script. Les index de ressource numériques finaux ne sont supprimés que pour la résolution du fichier cible.

**Preuves examinées :**

- Valeurs par défaut de `TypeLib\<GUID>\<version>\<locale>\win32` et `win64` dans chaque UsrClass.dat collecté.

**Comment investiguer :** une bibliothèque de types peut pointer vers une DLL légitime, un autre fichier ou un moniker de script. La vérification inventorie ces enregistrements ; évaluez l'application concernée et corroborez le chargement de la bibliothèque de types plutôt que de supposer que chaque chemin non par défaut est une attaque.

## Applications et compléments Office { #office-applications-and-add-ins }

Office prend en charge des compléments, des modèles et des extensions propres aux applications légitimes. Un enregistrement suspect peut dépendre d'une version d'Office, d'une application, d'un paramètre de chargement ou d'une interaction de l'utilisateur particuliers. La configuration seule n'établit pas l'exécution d'une macro.

### Enregistrement de compléments Office {#office_addins}

**ID de vérification :** `office_addins` · **ATT&CK du catalogue :** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Ce que Juntei vérifie :** énumère les compléments Word, Excel, PowerPoint, Outlook et Access dans les racines Office sans version et 14.0/15.0/16.0, y compris le magasin Click-to-Run de la machine implémenté et les racines utilisateur. Elle conserve les valeurs explicites Manifest/FileName/Path ou résout les entrées éligibles fondées sur COM, avec le contexte LoadBehavior.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` et l'équivalent Click-to-Run de la machine implémenté.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` dans NTUSER.DAT ; classes COM machine/utilisateur correspondantes issues de SOFTWARE, NTUSER.DAT et UsrClass.dat.

**Comment investiguer :** les compléments sont des logiciels métier courants. Examinez LoadBehavior, l'application/la version, les manifestes, les charges utiles effectives et l'éditeur attendu. La vérification ne garantit pas que chaque format de complément installé ou chaque vue d'enregistrement a été collecté ou chargé.

### Détournement de complément IA d'Office {#office_ai_hijack}

**ID de vérification :** `office_ai_hijack` · **ATT&CK du catalogue :** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Ce que Juntei vérifie :** énumère les valeurs de l'emplacement d'enregistrement Common AI de Office 16.0 Click-to-Run.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI` : toutes les valeurs.

**Comment investiguer :** comparez les valeurs et chemins inhabituels avec la build d'Office installée. La présence d'une valeur n'indique pas quelle fonctionnalité d'Office l'a utilisée et ne prouve pas qu'elle a sélectionné du code exécutable.

### Substitution de DLL Office {#office_dll_override}

**ID de vérification :** `office_dll_override` · **ATT&CK du catalogue :** [T1137](https://attack.mitre.org/techniques/T1137/)

**Ce que Juntei vérifie :** lit `WwlibtDll` et `PPCoreTDLL` sous les racines Word/PowerPoint pour les paramètres sans version et 14.0/15.0/16.0, y compris les racines Click-to-Run et utilisateur implémentées.

**Preuves examinées :**

- Paramètres `Microsoft\Office\[<version>\]Word` et `PowerPoint` de la machine/de l'utilisateur, ainsi que l'arborescence Office Click-to-Run de la machine implémentée : `WwlibtDll` et `PPCoreTDLL`.

**Comment investiguer :** déterminez quelle application/build reconnaît la substitution, examinez la bibliothèque et sa provenance, et corrélez avec un lancement d'Office ou un chargement d'image. Une substitution configurée ne constitue pas à elle seule une preuve d'exécution.

### Modèles Office par défaut {#office_templates}

**ID de vérification :** `office_templates` · **ATT&CK du catalogue :** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Ce que Juntei vérifie :** inventorie les fichiers Normal.dotm/NormalEmail.dotm collectés et les fichiers situés directement dans les dossiers Word Startup et Excel XLSTART reconnus, y compris les dossiers de démarrage de l'installation d'Office.

**Preuves examinées :**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` et `NormalEmail.dotm`.
- Les fichiers situés directement dans `Microsoft\Word\STARTUP` et `Microsoft\Excel\XLSTART` de l'utilisateur, sous AppData Roaming.
- Les fichiers situés directement dans les dossiers Office reconnus `<version>\STARTUP` / `XLSTART` et `root\<version>\STARTUP` / `XLSTART` sous Program Files et Program Files (x86).

**Comment investiguer :** un modèle peut être légitime ou ne contenir aucun contenu exécutable. Juntei n'extrait ni n'exécute de macros dans cette vérification. Examinez la structure interne du document, la stratégie relative aux macros et l'activité de l'application ; un chemin d'installation machine n'implique pas un jeton de processus SYSTEM.

### DLL Office Test {#office_test_dll}

**ID de vérification :** `office_test_dll` · **ATT&CK du catalogue :** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Ce que Juntei vérifie :** énumère les valeurs de la clé Office Test Special Perf dans les vues machine/utilisateur et machine 32 bits listées.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | Toutes les valeurs non vides |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | Toutes les valeurs non vides |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | Toutes les valeurs non vides |

**Comment investiguer :** considérez les substitutions inattendues comme une piste nécessitant une validation de la version d'Office et de la charge utile. La vérification signale une configuration plutôt que de tester si une installation d'Office particulière charge la valeur.

### Attaque par la page d'accueil d'Outlook {#outlook_home_page}

**ID de vérification :** `outlook_home_page` · **ATT&CK du catalogue :** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Ce que Juntei vérifie :** lit `URL` sous les enfants WebView d'Outlook de l'utilisateur, pour les versions d'Office 14.0, 15.0 et 16.0.

**Preuves examinées :**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` dans NTUSER.DAT.

**Comment investiguer :** une page d'accueil de dossier peut être légitime, et le comportement varie selon la version d'Outlook et les mises à jour de sécurité. Examinez l'URL/le contenu et la stratégie effective, puis corrélez avec les preuves d'Outlook, web et de processus.

### Détournement de DLL de moniteur VBA {#vba_monitors}

**ID de vérification :** `vba_monitors` · **ATT&CK du catalogue :** [T1137](https://attack.mitre.org/techniques/T1137/)

**Ce que Juntei vérifie :** lit les valeurs InprocServer32 des deux CLSID liés au moniteur VBA implémentés, dans les magasins de classes de la machine et de chaque utilisateur.

**Preuves examinées :**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` et `{0002E157-0000-0000-C000-000000000046}` dans le SOFTWARE de la machine, ainsi que les enregistrements équivalents dans le UsrClass.dat de l'utilisateur.

**Comment investiguer :** comparez les bibliothèques sélectionnées avec l'environnement Office/VBA installé. Il s'agit d'une vérification ciblée de CLSID, et non d'un inventaire de toutes les extensions VBA ni d'une preuve qu'une macro s'est exécutée.

## Débogueurs, gestionnaires de plantage et shims de compatibilité { #debuggers-crash-handlers-and-compatibility-shims }

Un débogueur ou un gestionnaire d'erreurs peut s'exécuter lorsqu'une application démarre, plante, se bloque ou se termine. Les bases de données de compatibilité des applications peuvent également affecter le démarrage. Examinez les paramètres prérequis et le programme déclencheur en même temps que la commande signalée.

### Détournement du débogueur AeDebug {#ae_debug}

**ID de vérification :** `ae_debug` · **ATT&CK du catalogue :** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Ce que Juntei vérifie :** lit les commandes `Debugger` d'AeDebug, natives et 32 bits, utilisées par la configuration du débogage juste-à-temps en cas de plantage.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**Comment investiguer :** des logiciels de développement/de débogage peuvent définir ceci légitimement. Examinez la commande complète du débogueur et les conditions préalables au déclenchement par plantage, puis corrélez avec les événements d'erreur d'application et de processus.

### Débogueur AeDebug pour processus protégés {#ae_debug_protected}

**ID de vérification :** `ae_debug_protected` · **ATT&CK du catalogue :** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Ce que Juntei vérifie :** lit les commandes `Debugger` d'AeDebugProtected, natives et 32 bits, pour la surface de débogage des processus protégés.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**Comment investiguer :** déterminez si la build du système d'exploitation et la stratégie de protection peuvent utiliser la commande configurée. Un débogueur enregistré ne prouve pas qu'un processus protégé a planté ni que la commande s'est exécutée.

### Débogueur Image File Execution Options {#ifeo_debugger}

**ID de vérification :** `ifeo_debugger` · **ATT&CK du catalogue :** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Ce que Juntei vérifie :** recherche les valeurs `Debugger` dans les enregistrements Image File Execution Options natifs/32 bits, y compris les clés de filtre imbriquées.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, y compris les clés de filtre imbriquées et la vue machine `Wow6432Node`.

**Comment investiguer :** un débogueur peut être sélectionné lorsque l'image nommée est lancée. Examinez la portée de l'image/du filtre et les arguments du débogueur ; les outils de développement peuvent également utiliser IFEO. Retracez le lancement de la cible pour établir l'exécution.

### NTDLL délégué par IFEO {#ifeo_delegated_ntdll}

**ID de vérification :** `ifeo_delegated_ntdll` · **ATT&CK du catalogue :** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Ce que Juntei vérifie :** recherche les substitutions `VerifierDlls` dans les enregistrements Image File Execution Options natifs/32 bits, y compris les clés imbriquées.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, y compris les clés imbriquées et la vue machine `Wow6432Node`.

**Comment investiguer :** le prédicat de la vérification est la valeur VerifierDlls configurée ; elle ne prouve pas que toutes les conditions préalables du vérificateur/de la délégation sont activées. Examinez les indicateurs applicables, la version de Windows et la charge utile avant de déduire un chemin d'injection fonctionnel.

### Moniteur de sortie silencieuse de processus {#ifeo_silent_process_exit}

**ID de vérification :** `ifeo_silent_process_exit` · **ATT&CK du catalogue :** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Ce que Juntei vérifie :** recherche `MonitorProcess` dans les enregistrements SilentProcessExit natifs/32 bits, y compris les clés imbriquées.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, y compris les clés imbriquées et la vue machine `Wow6432Node`.

**Comment investiguer :** la surveillance des sorties silencieuses nécessite également la configuration de rapport/IFEO correspondante. Examinez ces paramètres et les événements de sortie de l'image surveillée ; des outils de diagnostic légitimes peuvent expliquer le moniteur.

### Shims d'application {#installed_sdb}

**ID de vérification :** `installed_sdb` · **ATT&CK du catalogue :** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Ce que Juntei vérifie :** lit `DatabasePath` dans les enregistrements InstalledSDB immédiats des bases de données de compatibilité des applications installées.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**Comment investiguer :** les correctifs de compatibilité peuvent être légitimes. Examinez le contenu du SDB, les exécutables ciblés et l'historique d'installation. Un enregistrement de base de données n'identifie pas ses shims et ne prouve pas qu'une application correspondante s'est exécutée.

### Détournement du débogueur LSM {#lsm_debugger}

**ID de vérification :** `lsm_debugger` · **ATT&CK du catalogue :** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Ce que Juntei vérifie :** lit `MonitorProcess` spécifiquement sous SilentProcessExit pour `lsm.exe`, dans les vues machine native et 32 bits.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**Comment investiguer :** ceci recoupe l'inventaire SilentProcessExit plus large. Confirmez les indicateurs prérequis et une sortie LSM pertinente avant de traiter la configuration comme une persistance exécutée ; comptez une seule fois la même entrée lors de l'évaluation des résultats distincts.

### DLL auxiliaires MiniDump {#minidump_auxiliary_dlls}

**ID de vérification :** `minidump_auxiliary_dlls` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** énumère les valeurs nommées et non vides de MiniDumpAuxiliaryDlls, en conservant le nom de la valeur comme contexte du module déclencheur et les données de la valeur comme cible sélectionnée.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls` : noms de valeurs des modules déclencheurs et données cibles.

**Comment investiguer :** examinez à la fois le module déclencheur et la DLL auxiliaire. Des outils de plantage/de minidump peuvent expliquer une entrée, et un enregistrement ne montre pas qu'une opération de vidage a chargé la bibliothèque.

### Shims d'application {#shim_custom}

**ID de vérification :** `shim_custom` · **ATT&CK du catalogue :** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Ce que Juntei vérifie :** lit les valeurs nommées `.sdb` sous les clés d'application Custom immédiates d'AppCompatFlags, en conservant l'association entre application et base de données.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>` : noms de valeurs se terminant par `.sdb`.

**Comment investiguer :** associez ces associations à InstalledSDB et à la base de données de compatibilité effective. Une association de nom de fichier ne révèle pas à elle seule le comportement de la base de données et ne démontre pas le lancement d'une application correspondante.

### Détournement du débogueur WER {#wer_debugger}

**ID de vérification :** `wer_debugger` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit les valeurs `Debugger` natives/32 bits de Windows Error Reporting.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**Comment investiguer :** des logiciels de diagnostic légitimes peuvent configurer un débogueur. Examinez la commande complète et les événements de plantage pertinents ; l'enregistrement ne prouve pas que Windows Error Reporting l'a lancé.

### Détournement du débogueur WER Hangs {#wer_hangs}

**ID de vérification :** `wer_hangs` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit les valeurs `Debugger` natives/32 bits de Windows Error Reporting Hangs.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**Comment investiguer :** vérifiez si un processus de traitement des blocages invoquerait ce paramètre et examinez le programme cible. Corrélez avec les enregistrements de blocage d'application et de processus plutôt que de traiter une écriture dans le Registre comme une exécution de débogueur.

### Détournement de ReflectDebugger de WER {#wer_reflect_debugger}

**ID de vérification :** `wer_reflect_debugger` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit les valeurs `ReflectDebugger` natives/32 bits de Windows Error Reporting.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**Comment investiguer :** comparez avec la configuration de diagnostic approuvée et établissez l'applicabilité selon la version de Windows/le déclencheur. Examinez les arguments de la commande et recherchez une corroboration par les plantages/processus avant d'attribuer une exécution.

### Modules d'exception d'exécution WER {#wer_runtime_exception}

**ID de vérification :** `wer_runtime_exception` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** énumère les noms de valeurs sous RuntimeExceptionHelperModules natif/32 bits ; ici, le chemin de la bibliothèque figure dans le nom de la valeur, et pas nécessairement dans ses données.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` et la vue machine `Wow6432Node` : noms de valeurs.

**Comment investiguer :** examinez le module nommé et le contexte de l'application/du runtime. Une petite valeur numérique dans les données n'est pas le chemin de la charge utile, et l'enregistrement seul ne prouve pas que l'assistant a été chargé.

## Runtime .NET et paramètres de profileur { #net-runtime-and-profiler-settings }

Les runtimes managés peuvent charger des profileurs, des assemblys de démarrage ou des bibliothèques de débogage avant le code de l'application. Ces paramètres sont également utilisés par les développeurs et les produits de supervision. Vérifiez le runtime, l'architecture du processus, les indicateurs d'activation et si un processus nouvellement démarré pourrait hériter du paramètre.

### Injection AppDomainManager {#appdomain_manager}

**ID de vérification :** `appdomain_manager` · **ATT&CK du catalogue :** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Ce que Juntei vérifie :** lit les variables d'environnement `APPDOMAIN_MANAGER_ASM` et `APPDOMAIN_MANAGER_TYPE` de la machine/de l'utilisateur, qui couvrent la sélection de l'assembly et du type d'un gestionnaire AppDomain .NET personnalisé.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**Comment investiguer :** des applications peuvent utiliser intentionnellement des gestionnaires personnalisés. Résolvez ensemble l'assembly et le type, et vérifiez la compatibilité du runtime et l'héritage par les processus ; ni une variable isolée ni l'horodatage de sa clé ne prouvent qu'une application managée l'a chargé.

### Détournement du profileur CLR .NET {#cor_profiler}

**ID de vérification :** `cor_profiler` · **ATT&CK du catalogue :** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Ce que Juntei vérifie :** lit les paramètres d'environnement de profilage COR de la machine/de l'utilisateur ainsi que les entrées nommées correspondantes dans les chaînes multiples Environment de chaque service. Elle conserve les indicateurs d'activation, les identifiants de profileur et les chemins propres à chaque architecture.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` et l'`Environment` du NTUSER.DAT de chaque utilisateur collecté.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment` : paramètres COR nommés dans la chaîne multiple.

**Comment investiguer :** les profileurs prennent en charge une supervision légitime. Un indicateur d'activation, un CLSID et un chemin de DLL sont des éléments de configuration distincts ; confirmez l'activation effective et le processus .NET concerné plutôt que de traiter chaque ligne comme un profileur exécuté.

### Détournement du profileur CLR .NET Core {#coreclr_profiler}

**ID de vérification :** `coreclr_profiler` · **ATT&CK du catalogue :** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Ce que Juntei vérifie :** lit les paramètres d'environnement de profilage CORECLR correspondants de la machine/de l'utilisateur ainsi que les entrées Environment de services correspondantes, y compris les chemins propres à chaque architecture.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` et l'`Environment` du NTUSER.DAT de chaque utilisateur collecté.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment` : paramètres CORECLR nommés dans la chaîne multiple.

**Comment investiguer :** validez le runtime .NET, l'architecture, l'indicateur d'activation et la bibliothèque. Des produits d'observabilité approuvés peuvent configurer ces paramètres, et un processus existant peut ne pas hériter d'un environnement nouvellement modifié.

### Détournement de DbgManagedDebugger .NET {#dotnet_dbg_managed_debugger}

**ID de vérification :** `dotnet_dbg_managed_debugger` · **ATT&CK du catalogue :** [T1546](https://attack.mitre.org/techniques/T1546/)

**Ce que Juntei vérifie :** lit les paramètres `DbgManagedDebugger` de `.NETFramework` de la machine/de l'utilisateur listés, y compris le chemin machine 32 bits explicite.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**Comment investiguer :** déterminez si le runtime .NET/la stratégie de débogage applicable utilise cette commande. Des outils de développement peuvent l'expliquer ; corrélez avec les plantages managés et la création de processus pour établir l'exécution.

### Clé de Registre COR_PROFILER du .NET Framework {#dotnet_framework_profiler}

**ID de vérification :** `dotnet_framework_profiler` · **ATT&CK du catalogue :** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Ce que Juntei vérifie :** lit `COR_PROFILER`, `COR_PROFILER_PATH` et `COR_ENABLE_PROFILING` dans les vues de Registre `.NETFramework` listées. Ceci complète les vérifications de profileur fondées sur l'environnement.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**Comment investiguer :** évaluez les valeurs ensemble et déterminez si le runtime tient compte de cet emplacement du Registre. Un outil de profilage configuré peut être légitime ; n'interprétez pas un indicateur d'activation isolé comme le chargement identifié d'une DLL.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**ID de vérification :** `dotnet_startup_hooks` · **ATT&CK du catalogue :** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Ce que Juntei vérifie :** lit les valeurs d'environnement `DOTNET_STARTUP_HOOKS` de la machine/de l'utilisateur, qui peuvent sélectionner des assemblys de hook de démarrage pour les runtimes .NET pris en charge.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` et `Environment\DOTNET_STARTUP_HOOKS` du NTUSER.DAT de chaque utilisateur.

**Comment investiguer :** examinez la valeur entière à la recherche de plusieurs assemblys et vérifiez la prise en charge par le runtime. Cette vérification n'énumère pas les environnements de hooks de démarrage propres aux services et ne prouve pas qu'un processus a hérité de la valeur et l'a utilisée.

### DLL de débogage managé connues {#known_managed_debugging_dlls}

**ID de vérification :** `known_managed_debugging_dlls` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit les enregistrements `KnownManagedDebuggingDlls` de `.NETFramework` en tant que substitutions possibles de bibliothèques du débogueur managé.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**Comment investiguer :** comparez avec les outils de développement/de runtime installés et déterminez le débogueur déclencheur. L'entrée de Registre ne prouve ni une session de débogage ni le chargement d'une bibliothèque.

## Substitutions de la recherche de DLL et d'exécutables { #dll-and-executable-lookup-overrides }

Ces vérifications examinent des emplacements de bibliothèques explicites ou des paramètres qui influencent la manière dont un composant trouve du code. Une substitution de chemin ou une stratégie de recherche affaiblie est une piste d'investigation ; une charge utile et un composant déclencheur compatible restent nécessaires pour une exécution.

### AppCert DLLs {#appcert_dlls}

**ID de vérification :** `appcert_dlls` · **ATT&CK du catalogue :** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Ce que Juntei vérifie :** énumère les enregistrements AppCertDlls non vides du Session Manager, une surface d'extension de bibliothèques associée aux API de création de processus.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | Toutes les valeurs non vides |

**Comment investiguer :** examinez chaque bibliothèque et la provenance de son installation. Déterminez séparément l'applicabilité et le chargement à l'exécution ; une création de processus normale dans une chronologie ne prouve pas à elle seule que la DLL configurée a été chargée.

### AppInit DLLs {#appinit_dlls}

**ID de vérification :** `appinit_dlls` · **ATT&CK du catalogue :** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Ce que Juntei vérifie :** lit `AppInit_DLLs` natif/32 bits, sépare sa liste de bibliothèques et ajoute à chaque candidat le contexte disponible de `LoadAppInit_DLLs` et de `RequireSignedAppInit_DLLs`.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` : `AppInit_DLLs`, `LoadAppInit_DLLs` et `RequireSignedAppInit_DLLs` ; ainsi que la vue machine `Wow6432Node`.

**Comment investiguer :** un indicateur LoadAppInit_DLLs à zéro est explicitement marqué comme inactif. La version de Windows, le démarrage sécurisé (Secure Boot) et les exigences de signature affectent également le chargement. Examinez les paramètres effectifs avant d'affirmer qu'une DLL AppInit s'est exécutée.

**Plus de détails :** [documentation Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### Substitution d'AutodialDLL {#autodial_dll}

**ID de vérification :** `autodial_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit la substitution `AutodialDLL` des Parameters de WinSock2.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**Comment investiguer :** vérifiez la fonctionnalité réseau et la version de Windows source qui utilisent ce paramètre. Examinez la DLL effective et l'activité réseau/de processus ; une substitution seule n'établit pas un chargement actif ou malveillant.

### DLL d'assistance CHM {#chm_helper_dll}

**ID de vérification :** `chm_helper_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `Location` sous HtmlHelp Author, une surface de substitution de bibliothèque/d'emplacement liée à l'aide.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**Comment investiguer :** déterminez si le composant d'aide concerné utilise la valeur sur ce système et examinez le fichier sélectionné. Une substitution dans le Registre ne prouve pas qu'un document CHM a été ouvert.

### Substitution de DLL ContentIndex NL6 {#content_index_dll}

**ID de vérification :** `content_index_dll` · **ATT&CK du catalogue :** [T1574](https://attack.mitre.org/techniques/T1574/)

**Ce que Juntei vérifie :** lit `DLLOverridePath` dans les enregistrements ContentIndex Language immédiats du jeu de contrôles actif.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**Comment investiguer :** les composants de langue/de recherche peuvent différer selon la build de Windows. Examinez la bibliothèque et le composant installé correspondant ; suivez le processus d'indexation effectif pour corroborer le chargement.

### DLL Crypto ExpoOffload {#crypto_expo_offload}

**ID de vérification :** `crypto_expo_offload` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit la valeur `ExpoOffload` de Cryptography Offload en tant que candidat de substitution de bibliothèque cryptographique.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**Comment investiguer :** confirmez l'applicabilité selon la fonctionnalité/la build et examinez l'origine de la bibliothèque sélectionnée. Ne considérez pas la présence d'une valeur peu courante comme une preuve de vol d'identifiants ou d'exécution de code.

### DLL du rastériseur logiciel Direct3D {#direct3d_dll}

**ID de vérification :** `direct3d_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit la substitution de bibliothèque `SoftwareRasterizer` de Direct3D Drivers.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**Comment investiguer :** les logiciels graphiques et de compatibilité peuvent influencer la configuration du rastériseur. Examinez la DLL et l'activité d'une application Direct3D compatible ; l'enregistrement du chemin ne prouve pas qu'elle a été chargée.

### Affaiblissement de l'ordre de recherche des DLL {#dll_search_mode}

**ID de vérification :** `dll_search_mode` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** signale `SafeDllSearchMode` ou `CWDIllegalInDllSearch` uniquement lorsque la valeur active du Session Manager s'analyse comme zéro. Il s'agit de conditions de stratégie de recherche, et non de charges utiles sélectionnées.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` : `SafeDllSearchMode` et `CWDIllegalInDllSearch`.

**Comment investiguer :** une stratégie de recherche affaiblie peut faciliter une attaque distincte sur l'ordre de recherche. Elle n'identifie pas une DLL implantée et ne prouve pas une exécution. Les valeurs non nulles ou absentes ne sont pas émises par cette vérification ; évaluez séparément les valeurs par défaut effectives et la stratégie.

**Plus de détails :** [documentation Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### Exclusion de KnownDLLs {#exclude_from_known_dlls}

**ID de vérification :** `exclude_from_known_dlls` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit les entrées `ExcludeFromKnownDlls` non vides du Session Manager en tant que modifications du comportement de recherche des DLL protégées/connues.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**Comment investiguer :** une exclusion identifie une condition de recherche, et non la bibliothèque de remplacement elle-même. Trouvez l'application concernée et les emplacements de recherche de DLL potentiels, puis corroborez un chargement d'image non standard.

**Plus de détails :** [documentation Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### Substitution de DLL hhctrl.ocx {#hhctrl_ocx_dll}

**ID de vérification :** `hhctrl_ocx_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit les substitutions InprocServer32 du CLSID implémenté du contrôle HTML Help dans les magasins de classes machine natif/32 bits et utilisateur pris en charge.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**Comment investiguer :** déterminez quelle application et quelle architecture activent le contrôle. Examinez la DLL enregistrée et l'activité liée à l'aide ; un enregistrement COM ne prouve pas que HTML Help a affiché du contenu.

### Known DLLs {#known_dlls}

**ID de vérification :** `known_dlls` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** énumère les valeurs nommées de la clé KnownDLLs du Session Manager actif.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs` : valeurs nommées.

**Comment investiguer :** de nombreuses entrées sont des bibliothèques Windows attendues, et certaines valeurs sont des éléments de configuration plutôt qu'un chemin de charge utile. Comparez la clé complète avec une build correspondante fiable ; cette vérification n'analyse pas toutes les arborescences KnownDLLs alternatives propres à une architecture.

### DLL d'extensions LSA {#lsa_extensions}

**ID de vérification :** `lsa_extensions` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `Extensions` dans la configuration LsaSrv de LsaExtensionConfig.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**Comment investiguer :** des logiciels de sécurité peuvent étendre les composants d'authentification. Examinez les bibliothèques effectives et leur compatibilité avec la version de Windows source, puis recherchez des chargements d'image dans LSASS ou d'autres preuves d'utilisation indépendantes.

### Substitution du chemin de DLL MAPI32 {#mapi32_dll_path}

**ID de vérification :** `mapi32_dll_path` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `DLLPathEx` et `DLLPath` sous les enregistrements immédiats de clients de messagerie (Mail), dans les vues machine native et 32 bits.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**Comment investiguer :** les clients de messagerie enregistrent des fournisseurs MAPI légitimes. Déterminez quel client/fournisseur était sélectionné et examinez la bibliothèque effective ; un fournisseur installé ne prouve pas qu'une application a utilisé MAPI pendant l'incident.

### DLL XA de MSDTC {#msdtc_xa_dll}

**ID de vérification :** `msdtc_xa_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `OracleXaLib` et `OracleOciLib` dans la configuration MTxOCI de MSDTC.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**Comment investiguer :** l'intégration de bases de données peut légitimement nécessiter ces bibliothèques. Vérifiez la configuration du coordinateur de transactions et les logiciels de bases de données installés, et corrélez avec l'utilisation du service/des modules plutôt que de supposer qu'un chemin configuré s'est exécuté.

### Substitution de DLL NLDP {#nldp_dll}

**ID de vérification :** `nldp_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit la valeur de configuration `DllOverridePath` de NlsData.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**Comment investiguer :** examinez la bibliothèque cible, la build de Windows et le composant de langage naturel susceptible d'utiliser la substitution. La valeur ne montre pas qu'une indexation ou un traitement linguistique a eu lieu.

### DLL RDP TestDVCPlugin {#rdp_test_dvc_plugin}

**ID de vérification :** `rdp_test_dvc_plugin` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit le paramètre de bibliothèque `TestDVCPlugin` du Terminal Server Client de la machine.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**Comment investiguer :** examinez tout plugin de test inconnu et le comportement propre à la version. Une session client compatible et un chargement de bibliothèque sont nécessaires pour établir l'exécution ; la valeur de Registre seule ne montre ni l'un ni l'autre.

### Substitution de DLL de l'indexeur de recherche {#search_indexer_dll}

**ID de vérification :** `search_indexer_dll` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit la substitution `DllPath` de Windows Search dans SOFTWARE.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**Comment investiguer :** déterminez le composant/la version de Search correspondant et examinez le fichier cible. Un service Windows Search inactif ou un paramètre inutilisé est différent d'un chargement observé de bibliothèque malveillante.

### DLL Winsock AutoProxy {#winsock_auto_proxy}

**ID de vérification :** `winsock_auto_proxy` · **ATT&CK du catalogue :** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Ce que Juntei vérifie :** lit `LibraryPath` sous les enregistrements immédiats Catalog_Entries et Catalog_Entries64 de NameSpace_Catalog5.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**Comment investiguer :** les fournisseurs réseau peuvent être légitimes. Examinez les deux architectures, la provenance du fournisseur et les preuves pertinentes d'application réseau/de chargement d'image. Cette recherche ciblée de fournisseurs d'espace de noms ne couvre pas toutes les extensions Winsock possibles.

## Comptes et visibilité à la connexion { #accounts-and-sign-in-visibility }

Les enregistrements de comptes locaux et les paramètres de visibilité à la connexion peuvent révéler une persistance fondée sur l'identité. Ces vérifications examinent les ruches SAM et SOFTWARE collectées ; elles ne constituent pas un inventaire des comptes Active Directory, des appartenances aux groupes du domaine ou des ouvertures de session passées.

### Appartenance au groupe Administrateurs {#admin_group_membership}

**ID de vérification :** `admin_group_membership` · **ATT&CK du catalogue :** [T1098](https://attack.mitre.org/techniques/T1098/)

**Ce que Juntei vérifie :** décode la liste des membres de l'alias local Administrateurs à partir de SAM. Elle signale les SID de la machine locale qui désignent un compte caché, ou qui ne désignent aucun compte/groupe dans le SAM collecté.

**Preuves examinées :**

- `SAM\Domains\Builtin\Aliases\00000220\C` ainsi que le SID de la machine et les enregistrements d'utilisateurs/de groupes situés sous `SAM\Domains\Account` dans la ruche SAM.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` pour la corrélation des noms cachés.

**Comment investiguer :** il s'agit d'une vérification ciblée d'anomalies, et non d'une liste de tous les administrateurs. Les SID externes/de domaine ne sont pas évalués comme des utilisateurs locaux manquants. Des comptes supprimés, le moment de la collecte ou des ruches incomplètes nécessitent une investigation avant de conclure à une appartenance malveillante.

### Compte local caché {#hidden_account}

**ID de vérification :** `hidden_account` · **ATT&CK du catalogue :** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Ce que Juntei vérifie :** signale les valeurs numériques nulles dans le UserList de SpecialAccounts de Winlogon, qui identifient les noms masqués sur l'écran de connexion ordinaire.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` : valeurs de noms de comptes.

**Comment investiguer :** des comptes de service ou de maintenance peuvent être masqués intentionnellement, et la valeur n'établit pas que le compte nommé existe ni qu'il est administrateur. Croisez avec les enregistrements de comptes SAM et l'activité d'ouverture de session.

### Détournement de RID {#rid_hijacking}

**ID de vérification :** `rid_hijacking` · **ATT&CK du catalogue :** [T1098](https://attack.mitre.org/techniques/T1098/)

**Ce que Juntei vérifie :** compare le RID hexadécimal d'une sous-clé de compte SAM avec le RID stocké à l'offset 0x30 de sa valeur F, et signale les divergences.

**Preuves examinées :**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` dans la ruche SAM.

**Comment investiguer :** une divergence est une anomalie grave de l'enregistrement de compte, mais peut également nécessiter une analyse de corruption/de collecte. Examinez les octets source, l'état des journaux de transactions et les enregistrements de comptes/d'ouverture de session ; elle ne prouve pas quel acteur a modifié l'enregistrement.

### RID Suborner (compte administrateur caché) {#rid_suborner}

**ID de vérification :** `rid_suborner` · **ATT&CK du catalogue :** [T1098](https://attack.mitre.org/techniques/T1098/)

**Ce que Juntei vérifie :** signale les comptes SAM dont le RID de sous-clé n'est pas 500 mais dont le RID de la valeur F est 500, un schéma plus spécifique de substitution de l'identité de l'administrateur.

**Preuves examinées :**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` dans la ruche SAM.

**Comment investiguer :** ceci recoupe la vérification générale des divergences de RID. Malgré le titre du catalogue, elle n'exige pas en soi un paramètre de connexion masqué ; confirmez séparément la visibilité du compte, ses privilèges et son utilisation effective.

## Composants d'authentification et protection des identifiants { #authentication-components-and-credential-protection }

L'autorité de sécurité locale (LSA) gère les fonctions de sécurité par l'intermédiaire de LSASS et des composants associés. Les packages d'authentification, les filtres de mots de passe et les fournisseurs réseau peuvent être des logiciels de sécurité légitimes ou une voie d'accès persistant aux identifiants. Deux vérifications inventorient des paramètres de protection plutôt que des démarrages automatiques d'exécutables. Interprétez-les en fonction du rôle de l'hôte, de la version de Windows et de la stratégie effective.

### DLL de fournisseur AMSI {#amsi_providers}

**ID de vérification :** `amsi_providers` · **ATT&CK du catalogue :** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Ce que Juntei vérifie :** énumère les CLSID des fournisseurs AMSI de la machine et résout leurs bibliothèques InprocServer32 natives/32 bits lorsqu'elles sont disponibles. Les enregistrements non résolus conservent l'ID de classe.

**Preuves examinées :**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` et les enregistrements InprocServer32 `Classes\CLSID` / `Classes\Wow6432Node\CLSID` pouvant être résolus.

**Comment investiguer :** les antivirus et produits de sécurité enregistrent légitimement des fournisseurs AMSI. Examinez la provenance du fournisseur/de la bibliothèque et son chargement effectif ; un enregistrement ne prouve pas qu'AMSI a été contourné ni que le fournisseur était malveillant.

### Packages d'authentification {#authentication_packages}

**ID de vérification :** `authentication_packages` · **ATT&CK du catalogue :** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Ce que Juntei vérifie :** lit les entrées individuelles non vides de `Authentication Packages` de la LSA dans le jeu de contrôles actif.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**Comment investiguer :** comparez avec la build de Windows source et les logiciels d'authentification installés. Un package non par défaut peut être légitime. Résolvez la DLL et corroborez le chargement/l'activité liée aux identifiants avant de conclure à une interception.

### Comportement d'ouverture de session de l'administrateur DSRM {#dsrm_backdoor}

**ID de vérification :** `dsrm_backdoor` · **ATT&CK du catalogue :** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Ce que Juntei vérifie :** signale la valeur numérique `DsrmAdminLogonBehavior=2` dans les emplacements implémentés ControlSet001, ControlSet002 ou CurrentControlSet. Ce paramètre peut permettre l'utilisation du compte DSRM en dehors de son contexte de récupération habituel.

**Preuves examinées :**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, ainsi que ControlSet002 et CurrentControlSet lorsqu'ils sont présents.

**Comment investiguer :** concerne principalement les contrôleurs de domaine. Déterminez quel jeu de contrôles est actif à l'aide de Select dans SYSTEM ; une correspondance dans un jeu de contrôles historique n'est pas nécessairement effective. La valeur ne prouve pas qu'un attaquant connaissait le mot de passe DSRM ni qu'il a ouvert une session.

### Configuration de Credential Guard {#lsa_cfg_flags}

**ID de vérification :** `lsa_cfg_flags` · **ATT&CK du catalogue :** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Ce que Juntei vérifie :** lit la valeur `LsaCfgFlags` configurée de la LSA, y compris les valeurs d'activation ordinaires ainsi que zéro. Ceci inventorie la configuration de Credential Guard plutôt qu'un démarrage automatique d'exécutable.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**Comment investiguer :** interprétez la valeur en tenant compte de la version de Windows, des prérequis de firmware/de sécurité et de la stratégie effective. Une valeur zéro stockée ou une valeur absente n'établit pas à elle seule une modification par un attaquant, l'état effectif de la protection ou un vol d'identifiants.

### Filtre de mots de passe LSA {#lsa_password_filter}

**ID de vérification :** `lsa_password_filter` · **ATT&CK du catalogue :** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Ce que Juntei vérifie :** lit `Notification Packages` de la LSA. Les noms de bibliothèques seuls sont résolus par rapport au répertoire Windows System32 collecté, en ajoutant `.dll` le cas échéant.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**Comment investiguer :** les produits de stratégie de mots de passe installent fréquemment des filtres. Examinez la bibliothèque et les preuves de chargement, et distinguez les filtres configurés d'une preuve qu'un changement de mot de passe ou une interception d'identifiants a eu lieu.

### État de la protection PPL de LSASS {#lsa_run_as_ppl}

**ID de vérification :** `lsa_run_as_ppl` · **ATT&CK du catalogue :** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Ce que Juntei vérifie :** lit la valeur `RunAsPPL` configurée de la LSA, y compris les valeurs non nulles. Il s'agit d'un inventaire des paramètres de protection, et non d'un test établissant que LSASS s'est exécuté sans protection.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**Comment investiguer :** la protection LSA effective dépend de la version de Windows, de la stratégie, du verrouillage par le firmware et de l'état de redémarrage. L'absence de ligne pour une valeur absente n'est pas un résultat de protection mesuré. Utilisez des preuves système/d'exécution indépendantes pour évaluer la protection effective.

### DLL de fournisseur réseau {#network_provider_dll}

**ID de vérification :** `network_provider_dll` · **ATT&CK du catalogue :** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Ce que Juntei vérifie :** lit `NetworkProvider\ProviderPath` sous les services du jeu de contrôles SYSTEM actif.

**Preuves examinées :**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**Comment investiguer :** les fournisseurs d'ouverture de session réseau peuvent être légitimes. Vérifiez l'ordre/l'applicabilité des fournisseurs, les logiciels réseau/de sécurité installés et le chargement effectif de la bibliothèque ; une valeur ProviderPath seule ne prouve pas une interception d'identifiants.

### Packages de sécurité {#security_packages}

**ID de vérification :** `security_packages` · **ATT&CK du catalogue :** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Ce que Juntei vérifie :** lit les entrées individuelles non vides de `Security Packages` de la LSA, qui couvrent les fournisseurs de prise en charge de la sécurité (SSP) configurés.

**Preuves examinées :**

| Clé de Registre | Valeur examinée |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**Comment investiguer :** comparez avec la référence attendue de Windows/des produits de sécurité et examinez chaque bibliothèque. La configuration ne prouve pas qu'un fournisseur a été chargé, a capturé des identifiants ou a été ajouté pendant l'incident.

## Passer d'un candidat à une conclusion { #work-from-a-candidate-to-a-conclusion }

1. Ouvrez la source d'origine identifiée par `SourceFile`. Confirmez la valeur complète, l'action XML,
   le script ou l'enregistrement de base de données récupéré, ainsi que l'utilisateur/l'hôte exact auquel il appartient.
2. Déterminez si le mécanisme pouvait fonctionner : vérifiez les indicateurs d'activation, la version du runtime ou de Windows,
   l'état du service/de la tâche, le déclencheur requis, l'utilisateur, l'architecture et le jeu de contrôles actif.
3. Examinez le programme, la DLL, le script ou l'assembly sélectionné. Calculez l'empreinte des octets collectés et examinez les arguments
   et le contenu. Un lanceur Windows reconnaissable ou une chaîne d'éditeur ne suffit pas à établir la légitimité.
4. Établissez ce qui s'est réellement passé à l'aide de preuves indépendantes. Par exemple : la création de processus,
   les chargements d'image Sysmon, les enregistrements d'exécution du Planificateur de tâches, l'activité des services, Prefetch et les fichiers associés.
   Un événement de modification de configuration peut prouver que le paramètre a changé sans prouver que sa charge utile s'est exécutée.
5. Construisez une chronologie à l'aide des horodatages de la source d'origine et de la provenance de corrélation enregistrée.
   Conservez les écritures de clés, les modifications d'installation, les écritures de fichiers/de conteneurs et les exécutions comme des faits distincts.
6. Consignez les limites aussi bien que les résultats : ruches manquantes, fichiers cibles manquants, principaux de tâches inconnus,
   métadonnées de signataire non vérifiées, enregistrements historiques récupérés par carving ou déclencheur qui ne peut pas être corroboré.

Pour la syntaxe de la commande, les profils, chaque champ de sortie et les états de couverture, revenez au
[guide de la commande Persistence Hunter](../commands/persistence-hunter.md). Les
[guides des artefacts](../artifacts/index.md) expliquent les sources sous-jacentes, et les
[remerciements](acknowledgements.md) identifient les outils de persistance en amont.
