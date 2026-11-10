# Configuración de seguridad { #security-settings }

## Qué son { #what-they-are }

Los atacantes cambian la configuración de seguridad de Windows para permanecer ocultos y acceder a credenciales. Desactivan
Microsoft Defender o excluyen sus herramientas de él, hacen que Windows conserve contraseñas en texto no cifrado,
abren Escritorio remoto, desactivan el firewall y silencian el registro de PowerShell. El Registro conserva
esta configuración:

- **`SOFTWARE`:** Defender, sus exclusiones y directivas; UAC; las directivas de Escritorio remoto y del
  firewall; la directiva de ejecución de PowerShell, sus directivas de registro y los proveedores de AMSI; Restaurar sistema;
  los canales del registro de eventos.
- **`SYSTEM`** (por conjunto de controles): `WDigest`, la protección de LSA, Credential Guard, Restricted Admin,
  NTLM; Escritorio remoto; los perfiles del firewall; SMBv1; las horas de último acceso; el tamaño de cada registro de eventos.
- **`NTUSER.DAT`:** la directiva de ejecución y las directivas de registro de PowerShell del usuario, y AMSI para
  Windows Script Host.

`parse-registry security-settings` notifica como registro cada ajuste que encuentra. Cada registro contiene el
valor del ajuste, lo que significa el valor y si **debilita la configuración predeterminada**: si desactiva una
protección o una fuente de evidencias, o la reduce por debajo de lo que hace Windows actual (10 y 11)
sin el valor.

<span class="evidence-strength">Mejor conclusión:</span> este ajuste tenía este valor cuando se guardó el
hive, y la clave se escribió por última vez a esta hora. Un ajuste debilitado es una pista, no una prueba de
compromiso. Los administradores, la directiva de grupo, otros productos de seguridad y las versiones anteriores de Windows también establecen
algunos de estos valores, y el significado lo indica cuando es habitual.

## Ubicación { #location }

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

Recopile los tres hives con sus registros de transacciones. Un ajuste modificado poco antes de la recopilación
puede encontrarse únicamente en los registros de transacciones.

## Precauciones para el analista { #analyst-cautions }

- **La configuración almacenada no es el estado efectivo de la protección.** Se notifican los valores del Registro recopilados y las filas de diagnóstico de las listas de valores
  dañadas. La ausencia de datos puede reflejar valores predeterminados, fuentes no recopiladas u otro mecanismo de
  administración; no prueba que la protección esté habilitada o deshabilitada. `WeakensDefault` marca
  los ajustes que pueden reducir la referencia indicada cuando se respetan.
- **Los indicadores heredados de Defender pueden ignorarse.** `DisableAntiSpyware` y `DisableAntivirus` no
  prueban que Defender se detuviera. Su aplicabilidad depende de la versión de la plataforma, la incorporación y la protección contra
  alteraciones. [Microsoft explica estos ajustes heredados](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Las exclusiones administradas modernas pueden no estar disponibles en las capturas del Registro.** A partir de la versión de plataforma
  4.18.25110.6, los dispositivos que utilizan la administración de la configuración de Defender for Endpoint requieren cmdlets de PowerShell
  de Defender compatibles para obtener la configuración de exclusiones. Por tanto, su ausencia en el Registro sin conexión
  no establece que no existan exclusiones.
  [Guía de configuración de Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Debilitado no significa malicioso.** Ejemplos:
  - Windows también establece `DisableAntiSpyware` = 1 mientras hay otro antivirus registrado.
  - Escritorio remoto se habilita deliberadamente en los servidores.
  - Windows 7 conservaba registros de eventos más pequeños y no actualizaba las horas de último acceso.
- **Los valores predeterminados dependen de la versión.** Algunos valores eran los predeterminados anteriormente:
  - `NtfsDisableLastAccessUpdate` = 1 era el valor predeterminado antes de Windows 10 1803. Se notifica
    sin veredicto, pero `0x80000001`, establecido manualmente en una versión posterior, se considera debilitado.
  - `RunAsPPL`, `LsaCfgFlags` y `UseLogonCredential` se comparan con Windows actual.
- **Exclusiones de Defender:** cada ruta, extensión, proceso o dirección es un registro. Toda exclusión
  debilita Defender. `TemporaryPaths`, que Defender añade por sí mismo, no tiene veredicto. Busque carpetas con permisos de escritura para el usuario,
  extensiones `.ps1`/`.exe` y LOLBins (`rundll32.exe`, `powershell.exe`).
- **Las claves de directiva** (`Policy` = True) las escriben la directiva de grupo, las herramientas y los atacantes que
  saben que las directivas prevalecen sobre la configuración local. Correlaciónelas con los GPO del dominio.
- **Canales del registro de eventos:** solo se notifican 25 canales relevantes para la seguridad, indicando si Windows
  activa cada uno de forma predeterminada. `TaskScheduler/Operational` y `DNS-Client/Operational`, por
  ejemplo, están desactivados de forma predeterminada. Utilice `parse-registry read` para los demás.
- **Horas de las claves:** `KeyLastWrittenUTC` es la hora de la clave, por lo que no es necesariamente el momento en que cambió este
  valor. Es habitual que se escriba toda una clave `Policies` de una vez.
- **Conjuntos de controles:** de forma predeterminada, se lee el conjunto de controles que Windows utilizó por última vez. `--all-control-sets`
  los lee todos.

## Análisis con Juntei { #parse-with-juntei }

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

El resumen cuenta los ajustes por área. Enumera los ajustes que debilitan la configuración predeterminada y todas
las exclusiones de Defender.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, el hive y su tipo (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | Para `SYSTEM`: el conjunto de controles y si es el que Windows utilizó por última vez; desconocido cuando `Select\Current` falta o no es válido. |
| `User`, `UserSID` | Para `NTUSER.DAT`: el nombre de la carpeta del perfil y el SID del usuario según el `ProfileList` del sistema. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | Qué es: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | El valor: un número en decimal, una cadena o la propia exclusión. Vacío cuando el valor no es del tipo del ajuste. |
| `Meaning` | Lo que significa el valor. |
| `WeakensDefault` | `True` cuando el valor debilita lo que hace Windows actual sin él, `False` cuando no lo hace, y vacío cuando no procede (un tamaño, un puerto, un proveedor de AMSI) o no se puede determinar. |
| `Policy` | Si la clave se encuentra bajo una clave `Policies`. |
| `KeyPath`, `ValueName` | La clave y el valor. El nombre del valor de una exclusión es el elemento excluido; un proveedor de AMSI no tiene ninguno. |
| `KeyLastWrittenUTC` | La última escritura de la clave. |
| `DataRaw` | Los bytes del valor (hexadecimal), cuando no son del tipo del ajuste. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Valores del tipo incorrecto y claves que no se pueden leer. |

## Estrategia de investigación { #investigation-strategy }

1. Ejecute con `--weakened-only` en todos los hosts y compárelos entre sí: un ajuste
   que solo tiene un host destaca.
2. Compare cada exclusión de Defender con las herramientas encontradas en el host y en los registros de Defender
   (el evento 5007 registra los cambios de configuración).
3. Correlacione las horas de las claves con los inicios de sesión, las instalaciones de servicios y la creación de procesos. Busque
   líneas de comandos de `reg.exe`, `Set-MpPreference`, `netsh advfirewall` y `powershell` (evento
   4688, Sysmon 1 y 13, PowerShell 4104).
4. Para los ajustes de credenciales (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`), busque posteriormente volcados de credenciales y pass-the-hash.
5. Utilice `parse-registry read --recover-deleted` para los valores eliminados con el fin de restaurar un valor predeterminado, y
   `--all-control-sets` para los conjuntos de controles anteriores.

## Más información { #further-details }

- [Microsoft: configurar y validar exclusiones para el Antivirus de Microsoft Defender](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: configuración del Control de cuentas de usuario](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: configurar la protección adicional de LSA](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
