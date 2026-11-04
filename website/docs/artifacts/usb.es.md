# Dispositivos USB { #usb-devices }

## Qué es el historial de USB { #what-the-usb-history-is }

Windows registra todos los dispositivos que instala. Para el almacenamiento USB, otros dispositivos USB y los dispositivos
portátiles (teléfonos, cámaras, reproductores multimedia), el hive `SYSTEM` guarda una clave por dispositivo con su nombre,
su número de serie y las horas en que Windows lo instaló, lo vio llegar por última vez y lo vio retirar por última vez.
`MountedDevices` registra los GUID de volumen y las letras de unidad asignados a sus volúmenes. El hive
`SOFTWARE` guarda los nombres que el Explorador dio a los dispositivos portátiles y a los volúmenes, y el registro de
números de serie de volumen de ReadyBoost. El `NTUSER.DAT` de cada usuario registra los volúmenes que ese usuario tenía
montados.

`parse-registry usb` los une en un registro por dispositivo y volumen: qué dispositivo era, cuándo
se instaló y se conectó, qué letra de unidad y qué volumen tenía, y qué usuarios tenían montado ese
volumen.

<span class="evidence-strength">Mejor conclusión:</span> un dispositivo con este fabricante, este producto y este
número de serie se conectó a este sistema, por primera y por última vez a estas horas; su volumen tenía esta
letra de unidad; estos usuarios lo tenían montado. Los registros del Registro no muestran qué archivos se copiaron.

## Ubicación { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Propiedad del dispositivo | Campo | Significado (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: cuándo se instaló su controlador por última vez |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: cuándo se instaló el dispositivo por primera vez |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: cuándo se conectó por última vez (Windows 8 y posteriores) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: cuándo se retiró por última vez (Windows 8 y posteriores) |

Windows 8 y las versiones posteriores guardan cada hora como valor predeterminado de la clave `0064` … `0067`. Windows 7
la guarda en `00000064\00000000\Data`. Juntei lee ambas.

Muchas referencias de DFIR etiquetan `0064` como la primera instalación y `0065` como la instalación. Juntei sigue
las definiciones de `devpkey.h` de Microsoft indicadas arriba, y nombra las columnas según lo que registra cada hora.

Proporcione a `-f` o `-d` el hive `SYSTEM`. El hive `SOFTWARE` del sistema y los hives `NTUSER.DAT`
de sus usuarios se localizan junto a él, tal como los guarda una recopilación (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), o en el mismo directorio y un nivel por debajo cuando los hives se
exportaron juntos. Recopílelos todos junto con sus registros de transacciones.

## Cómo se unen los registros { #how-the-records-are-joined }

- **Dispositivos:** un registro por clave de dispositivo: `usbstor`, `usb` y `wpdbusenum` (`DeviceClass`). Un
  dispositivo de almacenamiento USB toma sus identificadores de fabricante y de producto del dispositivo USB con el mismo
  identificador de contenedor (o, en su defecto, con el mismo número de serie).
- **Volúmenes:** los volúmenes de un dispositivo de almacenamiento USB son los valores `\??\Volume{GUID}` de
  `MountedDevices` cuyos datos son una ruta que nombra el dispositivo. Un dispositivo con dos volúmenes tiene dos
  registros. Sus letras de unidad son los valores `\DosDevices\X:` con los mismos datos.
- **Nombres y números de serie:** los nombres de volumen proceden de `Windows Portable Devices` y de los
  dispositivos portátiles que nombran el mismo dispositivo. Las etiquetas y los números de serie de volumen proceden de las claves
  `EMDMgmt` que lo nombran.
- **Usuarios:** los usuarios cuyo `MountPoints2` contiene el GUID del volumen, con la hora de última escritura de esa clave.
- **Volúmenes sin clave de dispositivo** (`volume`): un volumen de `MountedDevices` que ninguna clave de dispositivo nombra (el de un
  disco interno, o el de un dispositivo retirado cuya clave `Enum` ya no existe), una letra de unidad con cuyos datos no coincide ningún valor
  de GUID de volumen, y un volumen que solo tienen los `MountPoints2` de los usuarios.
- **Dispositivos que solo nombra `SOFTWARE`** (`portable_device`, `readyboost`).

`SourceKeys` enumera todas las claves y valores a partir de los cuales se unió un registro, con su hive.

## Precauciones para el analista { #analyst-cautions }

- **Las letras de unidad se reutilizan.** `MountedDevices` conserva el último dispositivo al que se asignó cada letra. Una
  letra solo se muestra en el dispositivo cuyos datos siguen siendo los de esa letra.
- **Los números de serie pueden ser inventados.** Cuando el segundo carácter de una instancia es `&`, Windows
  lo inventó porque el dispositivo no informa de ningún número de serie; en ese caso `Serial` está vacío. Una instancia así
  puede ser distinta en otro equipo.
- **Las instancias de almacenamiento USB terminan con la unidad lógica** (`&0`). `Serial` la omite; `Instance`
  la conserva.
- **Las horas varían según la versión de Windows.** Windows 7 solo tiene las horas de instalación. `LastArrivalUTC` y
  `LastRemovalUTC` son solo la última conexión, no todas las conexiones. Los registros de eventos
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) y
  `setupapi.dev.log` proporcionan más información.
- **Conjuntos de controles:** de forma predeterminada se lee el último conjunto de controles que utilizó Windows. `--all-control-sets`
  lee todos los conjuntos de controles; uno anterior puede seguir enumerando un dispositivo que el actual ya no tiene.
- **La hora de última escritura de una clave no es una hora de conexión.** Windows reescribe las claves de dispositivo por muchos
  motivos.
- **Los dispositivos retirados dejan rastros.** Un registro de volumen con fabricante y producto pero sin clave de dispositivo
  es un dispositivo cuya clave `Enum` se eliminó (por una desinstalación o por herramientas de limpieza) y cuyo volumen
  sigue figurando en `MountedDevices`.
- **Los números de serie de ReadyBoost** son el número de serie del volumen (`VolumeSerials`, tal como lo muestra `dir`),
  que también registran los archivos LNK y las Jump Lists.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

El resumen enumera los dispositivos de almacenamiento USB que llegaron en último lugar (los más recientes primero), con sus letras
de unidad, y cuántos volúmenes pertenecen a dispositivos de almacenamiento USB sin clave de dispositivo.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host` | Nombre del equipo, según `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` o, sin clave de dispositivo, `volume`, `portable_device` o `readyboost`. |
| `ControlSet` | Conjunto de controles de la clave del dispositivo. |
| `Vendor`, `Product`, `Revision` | Almacenamiento USB: según el nombre del dispositivo (`Disk&Ven_…&Prod_…&Rev_…`), también para un dispositivo que nombra un volumen o una clave de `SOFTWARE`. |
| `Vid`, `PID` | Identificadores USB de fabricante y de producto. |
| `Serial`, `Instance` | El número de serie y el nombre de la clave de instancia tal como se escribió. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Valores de la clave del dispositivo, tal como se escribieron (`DeviceDesc` suele ser `@<inf>,%…%;<text>`). El `FriendlyName` de un dispositivo portátil suele ser la etiqueta o la unidad de su volumen. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Horas de las propiedades del dispositivo (consulte más arriba). |
| `DriveLetters` | Letras de unidad cuyos datos de `MountedDevices` son los del volumen. |
| `VolumeGUID`, `VolumeData` | GUID del volumen y en qué se encuentra según sus datos de `MountedDevices`: una ruta de dispositivo, una firma de disco MBR y el desplazamiento de la partición, o una partición GPT. |
| `VolumeNames`, `VolumeSerials` | Nombres que el Explorador dio al dispositivo o a su volumen, etiquetas de ReadyBoost y números de serie de volumen (`1234-ABCD`). |
| `User` | Usuarios cuyo `MountPoints2` contiene el volumen, con la última escritura de esa clave (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | Dónde se encontró (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`) y todas las claves y valores a partir de los cuales se unió. |
| `KeyPath`, `KeyLastWrittenUTC` | Clave de instancia del dispositivo y su última escritura. |
| `SourcePath`, `SoftwarePath` | El hive `SYSTEM`, y el hive `SOFTWARE` cuando aportó algo. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro de los hives a partir de los cuales se unió el registro. |
| `Problems` | Claves que no se pudieron leer y propiedades de dispositivo que no son FILETIME (con sus bytes). |

## Estrategia de investigación { #investigation-strategy }

1. Ordene los dispositivos de almacenamiento USB por `LastArrivalUTC` (o, en su defecto, `DriverLastInstalledUTC`) y aparte los
   lectores y concentradores propios del equipo.
2. Anote el fabricante, el producto, el número de serie, el GUID de volumen, la letra de unidad y el número de serie de volumen
   de cada dispositivo, y búsquelos en los archivos LNK, las Jump Lists, los ShellBags y la `$MFT` (rutas en esa
   letra de unidad o en ese volumen).
3. Compruebe `User` para saber quién tenía montado el volumen y cuándo (la última escritura de la clave `MountPoints2`).
4. Busque registros `volume` que nombren un dispositivo de almacenamiento USB: dispositivos cuyas claves se eliminaron.
5. Correlacione con los registros de eventos para cada conexión, no solo para la última.
6. Utilice `parse-registry read --recover-deleted` para las claves de dispositivo eliminadas.

## Más información { #further-details }

- [Microsoft: propiedades de dispositivo (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: compatibilidad con las solicitudes del administrador de montaje](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (tal como figura en los encabezados de Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): los números de las propiedades de dispositivo
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) y
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Las uniones de usuario utilizan la identidad de origen de cada hive, por lo que los hives exportados con el mismo nombre de perfil se mantienen separados. `SourceKeys` conserva las rutas de los hives de usuario que contribuyeron. La detección de usuarios complementarios omite los vínculos simbólicos, de forma coherente con la detección recursiva de hives. La correlación de VID/PID requiere un identificador de contenedor utilizable coincidente; sin él, la coincidencia por número de serie debe producir identificadores inequívocos. Los candidatos contradictorios quedan como desconocidos y se notifica un problema.
