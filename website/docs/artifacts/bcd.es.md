# Configuración de arranque (BCD) { #boot-configuration-data-bcd }

## Qué es el almacén BCD { #what-the-bcd-store-is }

Windows Vista y versiones posteriores guardan su configuración de arranque en un hive del Registro,
el almacén BCD: `\Boot\BCD` en sistemas BIOS y `\EFI\Microsoft\Boot\BCD` en la partición del
sistema EFI de los sistemas UEFI. `bcdedit` y el instalador de Windows lo modifican; el
administrador de arranque lo lee en cada arranque.

El almacén contiene objetos, cada uno con el nombre de un GUID:

- el **administrador de arranque de Windows** (`{bootmgr}`), con la entrada predeterminada, el orden
  de presentación y el tiempo de espera del menú; en sistemas UEFI, también el **administrador de
  arranque del firmware** (`{fwbootmgr}`) con el orden de arranque del firmware;
- un **cargador de arranque de Windows** por cada instalación de Windows (incluido el entorno de
  recuperación), una **aplicación de reanudación** (hibernación) y el **comprobador de memoria**;
- objetos de **configuración heredada** (`{globalsettings}`, `{bootloadersettings}`, …) cuyas
  opciones heredan los demás;
- objetos de **dispositivo** (opciones de ramdisk del entorno de recuperación).

Cada objeto tiene elementos: una descripción, un dispositivo y una ruta, y opciones como
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` y `safeboot`.

Dos tipos de ataque los modifican:

- El **ransomware** desactiva la recuperación para que el sistema no pueda repararse:
  `bcdedit /set {default} recoveryenabled No` y
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Algunas familias establecen
  `safeboot` para que Windows se reinicie en modo seguro, donde los productos de seguridad no se
  ejecutan.
- Los **rootkits y controladores sin firmar** pueden usar `testsigning on` o `nointegritychecks on` para debilitar las comprobaciones durante el arranque. Estas opciones por sí solas no prueban que se haya ejecutado un rootkit.

<span class="evidence-strength">Mejor conclusión:</span> en el momento de la recopilación, esta
entrada de arranque tenía esta opción. La hora de última escritura de la clave del elemento es el
momento en que se escribió la opción por última vez.

## Ubicación { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

El tipo de un elemento consta de 8 dígitos hexadecimales: su clase (`1` todas las aplicaciones, `2`
un tipo de aplicación, `3` dispositivos), su formato (`1` dispositivo, `2` cadena, `3` objeto, `4`
lista de objetos, `5` entero, `6` booleano, `7` lista de enteros) y su número. El comando nombra
los elementos tal como los documenta Microsoft (`BcdLibraryBoolean_AutoRecoveryEnabled`).

Las herramientas de recopilación a menudo no copian el almacén: la partición del sistema EFI no
está montada y `\Boot` está oculto. Recopílelo explícitamente o monte la partición. `-d` también
lee `BCD-Template` (`HiveType` indica cuál es); la plantilla es el punto de partida de Windows, no
el almacén con el que arranca.

## Precauciones para el analista { #analyst-cautions }

- **Las opciones se heredan.** Un cargador de arranque hereda de los objetos de su
  `BcdLibraryObjectList_InheritedObjects` (normalmente `{bootloadersettings}`, que hereda de
  `{globalsettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` y
  `BootStatusPolicy` son valores almacenados resueltos (propios o, en su defecto, heredados), e
  `Inherited` indica cuáles proceden de otro objeto. La herencia se sigue como máximo ocho niveles, con los ciclos acotados. Un objeto referenciado puede faltar en un almacén parcial. Los campos vacíos no establecen un valor predeterminado efectivo de Windows. El firmware y el arranque seguro (Secure Boot) también pueden impedir que una opción almacenada surta efecto.
- **No establecido no significa desactivado.** Un `RecoveryEnabled` vacío significa que el elemento
  no existe; se aplica el valor predeterminado de Windows.
- **Las horas son por elemento.** La hora de la clave de cada elemento (`key_last_written` en
  `Elements`) es el momento en que se escribió por última vez. Windows reescribe algunos elementos
  durante las actualizaciones; compárela con las horas de los demás elementos.
- **Los dispositivos se decodifican parcialmente.** Una partición GPT se indica mediante los GUID de
  su partición y de su disco; un ramdisk, mediante su objeto de opciones. Los demás dispositivos
  solo conservan sus bytes (`data_raw`).
- **Los elementos no documentados** (`0x15000065`, la clase `0x4…`) no tienen nombre y conservan
  sus bytes.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

El resumen enumera la entrada predeterminada de cada almacén y todas las opciones de firma de
prueba, comprobación de integridad, recuperación, directiva de estado de arranque (cualquier
`Ignore…`) y modo seguro, con la hora en que se establecieron o el objeto del que se heredan.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, el hive y `BCD` o `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | El GUID del objeto, su tipo (`0x10200003`) y qué es (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | La descripción, la ruta de la aplicación y su dispositivo (una partición y un disco GPT, o el objeto de opciones de un ramdisk). |
| `IsDefault`, `DisplayOrder` | Si el administrador de arranque de Windows lo arranca de forma predeterminada y su posición en el menú de arranque. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, tal como se resuelven a partir del almacén. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) y `bootstatuspolicy` (`IgnoreAllFailures`, …), con el número. |
| `Inherited` | Cuáles de estas opciones proceden de otro objeto y de cuál. |
| `Elements` | Todos los elementos: su tipo, nombre, valor, bytes (para dispositivos, tipos no documentados y datos que no se decodifican) y hora de la clave. |
| `KeyLastWrittenUTC` | Última escritura de la clave del objeto. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Objetos sin tipo y elementos cuyos datos no se ajustan a su formato. |

## Estrategia de investigación { #investigation-strategy }

1. Revise la entrada predeterminada y todos los cargadores de arranque en busca de `TestSigning`,
   `NoIntegrityChecks`, `RecoveryEnabled = false`, una `BootStatusPolicy` `Ignore…` y `SafeBoot`.
2. Utilice las horas de los elementos para fechar el cambio y correlaciónelas con `bcdedit.exe` en
   Prefetch, Amcache, los eventos de creación de procesos (4688, Sysmon 1) y el registro de eventos
   System.
3. Busque entradas de arranque o dispositivos que no deberían existir (otra ruta de cargador, un
   ramdisk) y cambios en la entrada predeterminada o en el orden de presentación.
4. Compare el almacén con `BCD-Template` y con otro sistema de la misma compilación.

## Más información { #further-details }

- [Microsoft: referencia de opciones de arranque de BCD](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (tipos de objeto)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
