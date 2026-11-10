# Descriptores de seguridad (`$Secure:$SDS`) { #security-descriptors-securesds }

## Qué es `$SDS` { #what-sds-is }

Todos los archivos y directorios de NTFS tienen un **descriptor de seguridad**: su propietario, su grupo principal, una
**DACL** (quién puede hacer qué con él) y una **SACL** (qué accesos se auditan y su etiqueta de integridad).
NTFS guarda cada descriptor distinto una sola vez, en el flujo `$SDS` del archivo `$Secure` (entrada 9 de la MFT),
y le asigna un **identificador de seguridad**. El `$STANDARD_INFORMATION` de un archivo registra solo ese identificador de seguridad: la
columna `SID` de [`parse-ntfs mft`](mft.md). `$SDS` lo convierte en un propietario y unas ACL.

`$SDS` se organiza en bloques de 256 KiB, cada uno seguido de una copia (su réplica). Cada entrada tiene un hash de
su descriptor, que NTFS utiliza para encontrar un descriptor que ya existe (`$SDH`), y el
identificador de seguridad (que `$SII` indexa).

<span class="evidence-strength">Mejor conclusión:</span> los archivos cuyo `SID` designa un
descriptor tenían, cuando se escribió por última vez su registro de `$MFT`, ese propietario y esas ACL. El
propietario de un archivo suele ser la cuenta que lo creó, pero un administrador, un instalador o un programa
que tome posesión puede cambiarlo.

## Ubicación y recopilación { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Recopílelo con una herramienta forense capaz de leer datos sin procesar; las recopilaciones suelen nombrarlo `$Secure_$SDS`, ya que `:`
no puede figurar en un nombre de archivo. Recopile junto con él la `$MFT` del mismo volumen.

## Precauciones para el analista { #analyst-cautions }

- Un propietario es un SID. Solo se nombran los SID conocidos (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) y las cuentas integradas de un equipo o dominio (`-500`
  `Administrator`, …). Resuelva los demás con el `SAM` del host
  ([cuentas SAM](sam.md)), su lista de perfiles ([Información del sistema](system-info.md)) o el dominio.
- Un descriptor lo comparten todos los archivos con el mismo propietario y las mismas ACL, y normalmente permanece en `$SDS`
  después de que esos archivos se eliminen. Un descriptor sin archivos en la `$MFT` no es por sí solo un
  archivo eliminado.
- Una ACE no es un historial de accesos. La DACL indica quién *puede* acceder a un archivo; los registros de eventos y otros
  artefactos indican quién lo hizo. Las entradas de auditoría de la SACL describen la configuración, no lo que se registró.
- Las ACE se enumeran en su orden dentro de la ACL. Juntei no calcula el acceso efectivo: las entradas de
  denegación, la herencia, la pertenencia a grupos, los privilegios y los derechos implícitos del propietario influyen.
- `DACL` y `SACL` escriben cada ACE como lo hace `icacls`. `SDDL` representa los permisos admitidos, con las máscaras en hexadecimal y las cuentas como SID. No es una copia byte a byte del descriptor binario; el texto condicional que no se puede representar lo deja vacío y se diagnostica.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

El resumen cuenta los descriptores y enumera sus propietarios, con el número de descriptores que posee cada uno,
los descriptores con una DACL NULL (sin ninguna restricción) y cualquier daño.

## Campos de salida CSV { #csv-output-fields }

| Campo | Interpretación |
|---|---|
| `SourcePath` | Archivo de entrada. |
| `SecurityID` | El identificador de seguridad: combínelo con el `SID` de [`parse-ntfs mft`](mft.md) del mismo volumen. |
| `Hash` | El hash registrado con el descriptor. |
| `EntryOffset` | Posición de la entrada en el flujo. |
| `FromMirror` | `True` para una copia leída del bloque de réplica: una que difiere de su entrada (que aparece a continuación de ella) o la copia de una entrada dañada. |
| `Size` | El tamaño de la entrada, incluido su encabezado de 20 bytes. |
| `Owner`, `OwnerName` | El SID del propietario y su nombre, si es conocido. |
| `Group`, `GroupName` | El SID del grupo principal y su nombre, si es conocido. Windows no lo utiliza para las comprobaciones de acceso. |
| `Control` | Los indicadores de control del descriptor: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (herencia bloqueada), `SE_DACL_AUTO_INHERITED`, … unidos por `|`. |
| `DACL-ACE-Count`, `DACL` | Las entradas de la DACL, separadas por `; `. Vacío si no hay DACL; `NULL DACL: no restriction` si el descriptor concede todo a todos. |
| `SACL-ACE-Count`, `SACL` | Las entradas de la SACL: entradas de auditoría, la etiqueta de integridad obligatoria, las etiquetas de confianza de procesos y los atributos de recursos. |
| `SDDL` | El descriptor completo en SDDL. Vacío solo si algún elemento no se decodifica. |
| `Problems` | Un hash que no corresponde al descriptor, partes que no se pueden decodificar, ACE de tipo desconocido. |

Cada entrada de `DACL` y `SACL` se escribe como la escribe `icacls`: la cuenta, el tipo si no es
una entrada de permiso, los indicadores de herencia y los derechos.

| Parte | Significado |
|---|---|
| Cuenta | El nombre conocido o el SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | El tipo de ACE, cuando no es una entrada de permiso. Para una etiqueta obligatoria no se muestra el tipo. |
| `(I)` | Heredada del directorio principal. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Heredada por los archivos (herencia de objetos), por los directorios (herencia de contenedores), solo por ellos y no por este objeto (solo herencia), y solo un nivel por debajo (sin propagación). |
| `(SA)`, `(FA)` | Una entrada de auditoría para accesos correctos o con error. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Control total, modificar, leer y ejecutar, leer, escribir. |
| `(RD,WD,AD,…)` | Derechos específicos en los demás casos: leer datos, escribir datos, anexar datos, leer y escribir atributos extendidos (`REA`, `WEA`), ejecutar (`X`), eliminar subelementos (`DC`), leer y escribir atributos (`RA`, `WA`), eliminar (`DE`), control de lectura (`RC`), escribir DAC (`WDAC`), escribir propietario (`WO`), sincronizar (`S`) y los derechos genéricos `GA`, `GR`, `GW`, `GE`. `N` significa sin acceso. |
| `(NW)`, `(NR)`, `(NX)` | La directiva de una etiqueta obligatoria: sin escritura hacia arriba, sin lectura hacia arriba, sin ejecución hacia arriba. |
| `if (…)` | La condición de una ACE de devolución de llamada, como `(WIN://SYSAPPID Contains "…")`: la entrada solo se aplica a los procesos para los que es verdadera. Windows las utiliza para conceder acceso a los paquetes de contenedores de aplicaciones. |

Las etiquetas de confianza de procesos y los atributos de recursos muestran su máscara en hexadecimal, y un atributo
de recurso, su notificación (`("IMAGELOAD",TU,0x0,1)`).

JSON/JSONL tienen los mismos campos y nombres, con los números como números, `null` para los valores vacíos y
`Problems` como lista.

## Estrategia de investigación { #investigation-strategy }

1. Analice la `$MFT` del mismo volumen y combine su `SID` con el de esta salida: cada archivo obtiene su
   propietario y sus ACL.
2. Fíjese en los propietarios que son cuentas de usuario (`S-1-5-21-…`): qué archivos posee una cuenta sospechosa
   y qué cuentas poseen archivos en directorios del sistema.
3. Busque ACL permisivas en directorios desde los que se ejecutan o cargan programas: `Everyone`,
   `Authenticated Users` o `User` con `(F)`, `(M)` o `(W)`, y DACL NULL.
4. Un descriptor utilizado por un solo archivo, o por un conjunto inusual de archivos, puede revelar archivos que se
   crearon o modificaron fuera de una instalación normal.
5. Corrobore los cambios de propietario y de permisos con las entradas del registro de eventos Security (4670, 4663 con una
   SACL) y los registros `SECURITY_CHANGE` del [Diario USN](usn-journal.md).

## Más información { #further-details }

- [Microsoft: descriptores de seguridad](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (descriptores de seguridad, ACE, expresiones condicionales)
- [Microsoft: formato de cadena del descriptor de seguridad (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: documentación del formato NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (su crate `secure`)

Con salida estructurada en stdout (`-t` sin `-o`), las advertencias de recuperación a nivel de flujo se envían a stderr, de modo que las canalizaciones conservan CSV/JSON/JSONL legibles sin ocultar las regiones dañadas o recuperadas.
