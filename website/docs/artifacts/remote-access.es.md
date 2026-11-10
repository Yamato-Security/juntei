# Acceso remoto (lado del usuario) { #remote-access-user-side }

## Qué es { #what-it-is }

El `NTUSER.DAT` de un usuario conserva rastros de las conexiones remotas que el usuario estableció desde el equipo.
Constituyen el lado de origen del movimiento lateral:

- **Escritorio remoto (`mstsc`):**
  - los equipos escritos en el cliente, los más recientes primero;
  - una clave por cada equipo en el que el usuario inició sesión, con el nombre de cuenta utilizado y la huella digital
    del certificado de servidor que aceptó el usuario;
  - los equipos a los que el usuario permitió usar dispositivos locales.
- **PuTTY:**
  - sesiones guardadas: host, puerto, cuenta, protocolo, archivo de clave, proxy y reenvíos de puertos;
  - las últimas sesiones iniciadas;
  - todas las claves de servidor SSH que aceptó el usuario.
- **WinSCP:** sesiones guardadas y claves de servidor SSH aceptadas, con el mismo formato que las de PuTTY.
- **Unidades asignadas:** cada unidad asignada para restaurarse al iniciar sesión, y las rutas escritas en
  *Conectar a unidad de red*.
- **Recursos compartidos:** los recursos compartidos que montó el Explorador (`MountPoints2`).

Muestran adónde fue una cuenta a continuación y con qué credenciales, incluso cuando los registros propios del equipo
remoto ya han desaparecido.

<span class="evidence-strength">Mejor conclusión:</span> el perfil de este usuario se conectó a este destino o guardó
una conexión a él. Una sesión guardada o un nombre escrito por sí solos no demuestran que una
conexión se completara. Una clave de host aceptada o una clave `Servers` con un `UsernameHint` es más sólida:
el cliente llegó al servidor.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

Recopile `NTUSER.DAT` junto con sus registros de transacciones para cada perfil. Recopile también el `SOFTWARE`
del sistema: proporciona el SID de cada usuario.

## Precauciones para el analista { #analyst-cautions }

- **Las horas de clave son horas de lista.** `Default`, `SshHostKeys` y `Map Network Drive MRU` contienen listas,
  y la última escritura de su clave es cuándo cambió la lista por última vez. Esa hora corresponde al primer elemento
  (`MruPosition` 0). Para los demás, es solo un límite superior. Una clave `Servers\<host>`, de sesión,
  `Network\<letter>` o de recurso compartido tiene su propia hora.
- **Guardado no significa usado.** Una sesión de PuTTY o WinSCP puede guardarse sin llegar a conectarse nunca. PuTTY
  reescribe `Recent sessions` cuando se inicia una sesión guardada, y añade una clave de host cuando se llega a un servidor
  por primera vez.
- **Contraseñas:**
  - WinSCP puede almacenar contraseñas de sesión, de túnel y de proxy.
  - PuTTY solo almacena la contraseña de un proxy, en texto sin cifrar.
  - `PasswordStored` y el `StoredPasswords` de `Details` indican qué valores están establecidos. Nunca se
    decodifican ni se escriben.
- **WinSCP omite los valores predeterminados.** Una sesión sin `FSProtocol` es SFTP, y una sin
  `PortNumber` utiliza el puerto predeterminado del protocolo. En ese caso, `Protocol` y `Port` se dejan vacíos en lugar
  de suponer un valor.
- **Las huellas digitales de las claves de host** son las `SHA256:…` de OpenSSH, tal como las muestran `ssh-keygen -l`, `ssh` y PuTTY 0.75
  y posteriores. Juntei las calcula a partir de la clave que guardan PuTTY y WinSCP. Compárelas con
  la propia clave del servidor, los archivos `known_hosts` y los registros para identificar el servidor que hay detrás de un nombre o una dirección
  IP. Una clave SSH-1 (`rsa`) no tiene ninguna.
- **Unidades asignadas:** `RemoteUsername` solo se establece cuando la unidad se asignó con otra
  cuenta; Windows escribe un `REG_DWORD` 0 cuando se usó la propia del usuario.
- **Recursos compartidos:** `MountPoints2` también tiene claves para volúmenes y letras de unidad. Aquí solo se informa de los recursos
  compartidos de red (`##server#share`); `\\server@SSL@443\DavWWWRoot` es un recurso compartido WebDAV.
- **Eliminación:** borrar las listas, o usar un cliente portátil, no deja rastro aquí. Compare con
  las Jump Lists, Prefetch, los archivos `.rdp`, la caché de mapas de bits de RDP y los registros del destino.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

El resumen cuenta las conexiones por cliente. Enumera las conexiones cuyas claves se escribieron en último lugar,
las más recientes primero, y las sesiones que conservan una contraseña.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nombre de la carpeta del perfil, SID del usuario según el `ProfileList` del sistema (con el hive `SOFTWARE` del sistema) y el hive. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` o `network_share`. |
| `Kind` | De dónde procede el registro: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | El equipo (nombre de host o dirección IP) y el puerto, o el recurso compartido (`\\server\share`). Solo se indica un puerto cuando Windows o el cliente escribieron uno. |
| `RemoteUsername` | La cuenta utilizada en el destino: `UsernameHint`, el `UserName` de una sesión (o el `user@` del `HostName` de PuTTY), el `UserName` de una unidad asignada. |
| `Protocol` | El `Protocol` de PuTTY (`ssh`, `telnet`, `raw`, …), el `FSProtocol` de WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), el proveedor de una unidad asignada (`Microsoft Windows Network`). |
| `MRU-Position` | Posición en su lista (0 es la más reciente): `MRU<n>`, `MRUList`, el `Recent sessions` de PuTTY. |
| `SessionName` | Nombre de una sesión guardada, con `%20` y otras secuencias de escape decodificadas. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | Tipo de una clave de host SSH aceptada (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), su huella digital de OpenSSH y sus números tal como se almacenan. |
| `DriveLetter` | Letra de una unidad asignada. |
| `PasswordStored` | Si la sesión conserva una contraseña; nunca se decodifica. |
| `Details` | Otras opciones, como `Name=Value; …` (CSV) o como objeto (JSON): el `CertHash` del certificado RDP, `LocalDevices`, el archivo de clave de una sesión, el proxy, el túnel SSH, los reenvíos de puertos, el comando y los directorios remotos, `StoredPasswords`, la `Label` de un recurso compartido, el `ConnectionType` de una unidad. |
| `KeyPath`, `ValueName` | La clave, y el valor en el caso de un registro que es un solo valor. |
| `KeyLastWrittenUTC` | Última escritura de la clave (consulte las precauciones sobre las listas). |
| `DataRaw` | Bytes del valor (en hexadecimal) cuando no es una cadena o no se puede decodificar. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Valores de tipo incorrecto (con sus bytes), claves de host que no se pueden leer, letras de `MRUList` que no existen y valores que no enumera. |

## Estrategia de investigación { #investigation-strategy }

1. Enumere los destinos de cada usuario por cliente y compárelos con los sistemas a los que ese usuario debería
   acceder. Busque recursos compartidos administrativos (`C$`, `ADMIN$`) y direcciones IP.
2. Anote las cuentas utilizadas (`RemoteUsername`): administradores del dominio y cuentas locales de otros
   equipos.
3. Haga coincidir `HostKeyFingerprint` con las claves de los servidores, y `CertHash` con los certificados de los servidores
   RDP.
4. Correlacione con los eventos de inicio de sesión de los destinos (4624 de tipo 10 y 3, 1149), las Jump Lists y los archivos LNK
   (`mstsc.exe`, `putty.exe`, archivos abiertos desde recursos compartidos), Prefetch y la caché de mapas de bits de RDP.
5. Utilice `parse-registry read --recover-deleted` para las entradas eliminadas, y copias anteriores del
   `NTUSER.DAT` del perfil (instantáneas de volumen, copias de seguridad) para las listas borradas.

## Más información { #further-details }

- [Microsoft: quitar entradas del cuadro Equipo de Conexión a Escritorio remoto](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [Preguntas frecuentes de PuTTY: ¿dónde almacena PuTTY sus datos?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: almacenamiento de la configuración](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
