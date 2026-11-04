# Redes { #networks }

## Qué son { #what-they-are }

Windows recuerda las redes a las que se unió un sistema y cómo se configuró su red.

- **`SOFTWARE`:**
  - **Perfiles de red** (`NetworkList\Profiles`): uno por cada red a la que se conectó el sistema,
    con su nombre (el SSID de una red Wi-Fi), su categoría (pública, privada, dominio), el tipo de
    conexión (cableada, inalámbrica, VPN, banda ancha móvil) y cuándo se conectó el sistema a ella
    por primera y por última vez.
  - **Firmas** (`NetworkList\Signatures`): lo que identifica cada red, como la dirección MAC de la
    puerta de enlace y el sufijo DNS.
  - **Tarjetas de red** (`NetworkCards`): los adaptadores.
- **`SYSTEM`** (por conjunto de control):
  - **Interfaces TCP/IP:** direcciones, servidor y concesión DHCP, servidores DNS, dominio.
  - **Recursos compartidos** que ofrece el sistema (`LanmanServer\Shares`).
  - **Proxies de puertos** (`PortProxy`): los túneles que configura
    `netsh interface portproxy`, que los atacantes usan para pivotar y para llegar a RDP a través
    de otro puerto.
  - **Rutas persistentes.**
  - **La zona horaria**, que convierte las horas locales de los perfiles a UTC.

<span class="evidence-strength">Mejor conclusión:</span> el sistema se conectó a una red con este
nombre y esta puerta de enlace, por primera y por última vez a estas horas. Tenía esta dirección
asignada por este servidor DHCP y ofrecía estos recursos compartidos y túneles cuando se guardó el
hive. El nombre de una red por sí solo no identifica dónde estaba: muchas redes se llaman "Network"
o comparten un SSID.

## Ubicación { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

Recopile ambos hives con sus registros de transacciones. El hive SYSTEM es necesario para
convertir las horas de los perfiles a UTC.

## Precauciones para el analista { #analyst-cautions }

- **Las horas de los perfiles son locales.** `DateCreated` y `DateLastConnected` se escriben en la
  hora local del sistema.
  - `FirstConnectedLocal` y `LastConnectedLocal` las conservan tal como se encontraron.
  - `FirstConnectedUTC` y `LastConnectedUTC` las convierten con la zona horaria del hive SYSTEM del
    sistema (`TimeZone`) y su regla de horario de verano.
  - Una hora anterior a un cambio de zona horaria, o de un año con otra regla de horario de verano
    (fechas de EE. UU. anteriores a 2007), puede estar desfasada en esa diferencia.
  - Sin un hive SYSTEM legible y un conjunto de control actual válido, solo se proporcionan horas locales. Los saltos del horario de verano y las horas ambiguas repetidas no tienen una única marca de tiempo UTC demostrable: UTC se deja vacío y `Problems` explica la limitación. Los desplazamientos o fechas no válidos tampoco se convierten.
- **Solo la primera y la última.** Cada perfil tiene una primera y una última conexión, no todas las
  conexiones. La última escritura de la clave del perfil suele coincidir con la última conexión.
- **Los nombres se repiten.** Windows denomina las redes cableadas no administradas "Network",
  "Network 2", etc. Los SSID de Wi-Fi no son únicos. Utilice `GatewayMac` y `DnsSuffix` para
  distinguir las redes, y correlaciónelos con los registros DHCP y el inventario de activos. La MAC de una puerta de enlace no es necesariamente el BSSID de un punto de acceso Wi-Fi y por sí sola no determina la ubicación física.
- **Los recursos compartidos, los proxies y las rutas** son configuración: un recurso compartido o
  un proxy de puertos puede configurarse y no usarse nunca. Ningún valor tiene una hora propia; la
  última escritura de la clave es el momento en que cambió cualquiera de ellos.
- **Interfaces:** una clave de interfaz bajo un GUID sin direcciones es un adaptador que no está
  configurado. Con DHCP, las direcciones son las de la última concesión. Una dirección estática
  que quedó de antes de DHCP no se muestra como en uso.
- **Conjuntos de control:** de forma predeterminada se lee el conjunto de control que Windows usó
  por última vez. `--all-control-sets` los lee todos, lo que puede mostrar un recurso compartido o
  un proxy que se eliminó del actual.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

El resumen enumera las últimas redes a las que se conectó (de la más reciente a la más antigua) y
todos los recursos compartidos, proxies de puertos y rutas persistentes.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Host y el hive. |
| `ControlSet`, `IsCurrentControlSet` | Para los registros de SYSTEM: el conjunto de control y si es el que Windows usó por última vez. |
| `Source` | `network_profile`, `network_signature` (una firma sin perfil), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | El nombre de un perfil (el SSID de una red Wi-Fi); la descripción de una firma. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (o, en caso contrario, el número). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` y `DateLastConnected` convertidos a UTC con `TimeZone`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Lo mismo, tal como se encontró (hora local, con precisión de milisegundos). |
| `TimeZone` | La zona horaria utilizada para la conversión. |
| `GatewayMAC`, `DNS-Suffix` | La dirección MAC de la puerta de enlace y el sufijo DNS de las firmas del perfil (`|` entre varios). |
| `InterfaceGUID` | El GUID de una interfaz; el `ServiceName` de una tarjeta de red. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Las direcciones en uso de una interfaz, el servidor DHCP y la concesión. |
| `ShareName`, `SharePath` | Un recurso compartido y la carpeta que ofrece. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Un proxy de puertos: dónde escucha y hacia dónde reenvía. |
| `Route` | Una ruta persistente tal como se almacena: `destination,netmask,gateway,metric`. |
| `Details` | Otros valores, como `Name=Value; …` (CSV) o un objeto (JSON): la descripción de un perfil, `Managed`, el primer nombre y las claves de firma; el adaptador de una interfaz (de `NetworkCards`), máscaras, puertas de enlace, servidores DNS y dominio; el tipo, el comentario y otros campos de un recurso compartido; el tipo de un proxy; las partes de una ruta. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | La clave, el valor de un recurso compartido, proxy o ruta, y la última escritura de la clave. |
| `DataRaw` | Bytes que no se decodifican: una hora que no es un `SYSTEMTIME` válido, un recurso compartido que no es una lista de cadenas. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Valores de tipo o tamaño incorrecto, fechas imposibles y nombres que no tienen la forma esperada. |

## Estrategia de investigación { #investigation-strategy }

1. Ordene los perfiles por `LastConnectedUTC` y revise las redes en torno al incidente: redes Wi-Fi
   desconocidas, puntos de acceso móviles, VPN.
2. Utilice `GatewayMac` y `DnsSuffix` para distinguir redes con el mismo nombre y para relacionar
   una red entre hosts.
3. Revise todos los proxies de puertos y recursos compartidos. Un proxy de puertos hacia el 3389 o
   el 445 de un host interno, o un recurso compartido de una carpeta de preparación
   (`C:\ProgramData`, `C:\Users\Public`), es una pista. Correlacione con los registros de eventos
   (creación del proceso `netsh`, eventos de recursos compartidos 5142/5144) y con las horas de las
   claves.
4. Utilice las direcciones y las horas de concesión de las interfaces para situar el host en la red,
   y relaciónelas con los registros del cortafuegos, DHCP y proxy.
5. Utilice `--all-control-sets` y `parse-registry read --recover-deleted` para los recursos
   compartidos, proxies y perfiles eliminados.

## Más información { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
