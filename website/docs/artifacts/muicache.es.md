# MuiCache { #muicache }

## Qué es MuiCache { #what-muicache-is }

Cuando un usuario inicia un programa, Windows lee los recursos de versión del programa para obtener el nombre que
debe mostrar el shell (el `FileDescription`, por ejemplo "Windows Command Processor" para
`cmd.exe`) y lo almacena en caché por usuario en `MuiCache`. Windows 8 y posteriores también almacenan en caché la compañía
(`CompanyName`). La caché se indexa por la ruta completa del programa:

- **Windows 8 y posteriores:** dos valores por programa, `<path>.FriendlyAppName` y
  `<path>.AppCompany`.
- **Windows Vista y 7:** un valor por programa, `<path>`, que contiene el nombre descriptivo.
- **Windows XP:** la misma forma, en `NTUSER.DAT`.

MuiCache puede proporcionar pistas de ejecución de programas para la sesión de shell de un usuario, incluidos programas
en unidades extraíbles, recursos compartidos de red y carpetas que ya no existen, con el nombre y la compañía
que indicaba el propio archivo.

<span class="evidence-strength">Mejor conclusión:</span> este perfil de shell almacenó en caché la ruta,
la descripción y la compañía de un archivo. Correlaciónelo con otros artefactos de ejecución. El perfil no identifica
la cuenta con la que se ejecutó el proceso. MuiCache no tiene hora por programa ni recuento de ejecuciones.

## Ubicación { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (el idioma de las cadenas en caché) no es un programa y se omite. Los valores con nombre
`@<file>,-<number>` (`@shell32.dll,-22019`) son cadenas de recursos que el shell almacenó en caché para sus propios
menús y tipos de archivo, no programas; se omiten y se cuentan en el resumen. En Windows Vista
y posteriores, `NTUSER.DAT` no tiene MuiCache.

Recopile `UsrClass.dat` (y `NTUSER.DAT`) con sus registros de transacciones para cada perfil, así como los hives
`SYSTEM` y `SOFTWARE` del sistema para obtener los nombres del host y de los usuarios.

## Precauciones para el analista { #analyst-cautions }

- **El propietario del perfil no es necesariamente la cuenta del proceso.** Un programa iniciado con otra
  cuenta puede mostrar una ventana en la sesión de este usuario y dejar metadatos en la caché de este usuario.
  `User` y `UserSid` identifican al propietario del hive, no el contexto de seguridad del proceso.
- **Sin hora por programa.** `KeyLastWrittenUTC` es la última escritura de la clave: el cambio más reciente de
  cualquiera de sus programas, no cuándo se ejecutó uno en concreto. Prefetch, BAM, UserAssist y los registros de eventos proporcionan
  las horas.
- **No solo programas que el usuario inició manualmente.** Puede aparecer una DLL o un programa que cargó el propio shell (para
  `Open with`, una extensión del shell o un proceso secundario de un instalador), como `explorerframe.dll` o
  `shell32.dll`.
- **El nombre procede del archivo.** `FriendlyAppName` y `AppCompany` son lo que indicaban los recursos
  de versión del archivo cuando se ejecutó. El malware puede declarar cualquier descripción y compañía; una herramienta
  con otro nombre conserva su descripción original, lo que puede delatarla (un archivo llamado `svchost.exe` cuyo
  nombre descriptivo es una herramienta de acceso remoto).
- **Una entrada por ruta.** Volver a ejecutar el mismo programa no añade una entrada; una copia en otra
  ruta sí. La entrada de un programa puede permanecer después de que se elimine el archivo.
- **Ambas formas para una misma ruta.** Cuando una ruta tiene un nombre descriptivo en ambas formas (un perfil
  actualizado), cada una obtiene su propio registro, de modo que no se pierde ningún valor.
- **Valores dañados.** Un valor que no es una cadena, no es UTF-16 válido o no se puede leer completo conserva
  sus bytes (`FriendlyAppNameRaw`, `AppCompanyRaw`) y se notifica en `Problems`.
  El UTF-16 no válido también puede producir caracteres de sustitución en el texto mostrado; utilice la columna
  sin procesar para obtener los bytes originales.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

El resumen enumera los programas situados fuera de las carpetas Windows y Program Files (carpetas de usuario,
otras unidades, recursos compartidos de red) con sus usuarios, y el número de cadenas de recursos omitidas.
También se enumeran las carpetas Program Files con otros nombres (en Windows en otros idiomas).
Las rutas relativas y las rutas que salen de una carpeta de variable de entorno permanecen en la lista. Las rutas de unidad
normales se comprueban tras resolver los componentes `.` y `..` sin acceder al sistema de archivos del
examinador.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nombre de la carpeta del perfil, SID del usuario (lo indica la clave raíz de un `UsrClass.dat`; si no, el `ProfileList` del sistema) y el hive. |
| `HiveType` | `UsrClass.dat` o `NTUSER.DAT`. |
| `Path` | La ruta del programa: el nombre del valor sin `.FriendlyAppName` ni `.AppCompany`. |
| `FriendlyAppName` | La descripción del programa, de `<path>.FriendlyAppName` o `<path>`. |
| `AppCompany` | La compañía del programa, de `<path>.AppCompany` (Windows 8 y posteriores). |
| `KeyPath`, `KeyLastWrittenUTC` | La clave MuiCache y su última escritura: el cambio más reciente de cualquiera de sus programas. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Los bytes del valor (hexadecimal), cuando no son texto que se decodifique correctamente. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro: `replayed` cuando los registros de transacciones escribieron uno de los valores del programa. |
| `Problems` | Valores que no son cadenas, no son UTF-16 válido o no se pueden leer. |

## Estrategia de investigación { #investigation-strategy }

1. Fíjese en los programas situados en ubicaciones con permisos de escritura para el usuario (`\Users\`, `Downloads`, `Temp`, `ProgramData`), en
   otras unidades y en recursos compartidos de red, así como en las herramientas de administración, acceso remoto y compresión.
2. Compare el nombre del archivo con su nombre descriptivo y su compañía: una discrepancia puede revelar una herramienta
   renombrada o un archivo que se hace pasar por otro.
3. Correlacione con Prefetch, BAM y UserAssist (cuándo se ejecutó), Amcache y ShimCache (el hash
   y la presencia del archivo) y la `$MFT` (si el archivo sigue existiendo).
4. Utilice `parse-registry read --recover-deleted` sobre `UsrClass.dat` para recuperar los valores de MuiCache eliminados.

## Más información { #further-details }

- [Microsoft: recurso VERSIONINFO](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: experimentos con MuiCache, sesiones de GUI y limitaciones de la cuenta del proceso](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
