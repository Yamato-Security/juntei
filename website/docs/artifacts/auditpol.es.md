# Directiva de auditoría { #audit-policy }

## Qué es { #what-it-is }

La directiva de auditoría indica qué eventos de seguridad registra Windows en el registro de eventos Security. Se configura
por subcategoría en Windows Vista y versiones posteriores (59 en Windows 10 y 11), y por categoría en
Windows XP y Server 2003. Cada una registra los aciertos, los errores, ambos o nada.

LSA guarda la directiva vigente en el hive `SECURITY`, tanto si procede de `auditpol`, de la directiva de seguridad
local o de la directiva de grupo.

<span class="evidence-strength">Mejor conclusión:</span> el sistema estaba configurado para registrar estos eventos (o
no) cuando se escribió el hive por última vez. Cuando una subcategoría no se audita, la ausencia de un evento no dice
nada sobre si la actividad se produjo.

## Ubicación { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista y las versiones posteriores guardan un encabezado de 12 bytes, después una configuración por subcategoría, categoría por
categoría, y después cuántas subcategorías tiene cada categoría. Las versiones solo difieren en esos recuentos:
cada una añade subcategorías al final de una categoría (56 en los hives de Windows 8.x y Server 2012 R2
leídos, 59 en Windows 10, 11 y Server 2022). Windows XP y Server 2003 guardan si la auditoría está
activada (un byte, seguido de relleno que Windows no borra) y nueve configuraciones de categoría. Esa estructura se
comprobó con la imagen pública de Windows XP del Hacking Case de CFReDS del NIST, que no audita nada; los
valores de sus configuraciones de categoría siguen la documentación de Microsoft.

Recopile `SECURITY` junto con sus registros de transacciones, y el hive `SYSTEM` del sistema, que indica el nombre del
host.

## Precauciones para el analista { #analyst-cautions }

- **Los datos desconocidos siguen siendo desconocidos.** Las revisiones no compatibles se conservan con diagnósticos. Los indicadores
  de configuración desconocidos dejan `Success` y `Failure` vacíos y se excluyen de los recuentos de configuraciones auditadas.
  Una tabla de recuentos dañada no puede convertir la palabra reservada en otra configuración de auditoría.
  Las capturas pueden contener posiciones adicionales más allá del catálogo de nombres y GUID. Sus configuraciones y el
  `SubcategoryId` numérico se conservan con nombres y GUID vacíos; una posición sin correspondencia no es por sí misma
  una evidencia de daños.


- **El orden es el del propio Windows.** Dentro de una categoría, las configuraciones siguen el orden de los
  identificadores de mensaje de las subcategorías, no el de sus GUID. Inicio de sesión especial (`%%12548`, índice
  4 de Inicio y cierre de sesión) va antes que Modo rápido de IPsec. Las herramientas que las leen en orden de GUID informan de que el valor
  predeterminado de Windows 10 es "No Auditing" para Inicio de sesión especial y "Success" para Modo rápido de IPsec. `SubcategoryId` es el
  mismo identificador que indica el evento 4719 (System audit policy was changed), por lo que ambos se pueden hacer coincidir.
- **La directiva vigente, no su origen.** La directiva de grupo y la directiva local escriben el mismo valor.
  El `audit.csv` de la directiva de grupo (en SYSVOL y en `C:\Windows\System32\GroupPolicy`) muestra lo que se
  pretendía aplicar (`Machine\Microsoft\Windows NT\Audit\audit.csv` de una directiva).
- **Cambios a lo largo del tiempo:** `KeyLastWrittenUTC` indica aproximadamente cuándo cambió la directiva por última vez. El evento 4719
  registra cada cambio (siempre que el registro Security lo conserve). Un atacante puede desactivar la auditoría
  y volver a activarla después.
- **Subcategorías frente a categorías.** En Windows Vista y versiones posteriores, Windows utiliza las subcategorías
  (salvo que la opción de seguridad `SCENoApplyLegacyAuditPolicy` esté desactivada y una directiva heredada por
  categorías las invalide).
- **Windows XP y 2003:** con la auditoría desactivada (`AuditingEnabled` False), no se audita ninguna categoría,
  sea cual sea su configuración.
- **Eventos:** `Events` enumera los eventos principales de cada subcategoría, según la documentación de Microsoft
  sobre la configuración de la directiva de auditoría avanzada. Son los que suele buscar una investigación, no
  todos los eventos que registra la subcategoría.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Para cada hive, el resumen indica su estructura, cuántas configuraciones se auditan y cuándo se escribió la directiva
por última vez. Enumera lo que se audita para aciertos y errores, solo aciertos y solo
errores, y después las subcategorías que no se auditan, con sus eventos principales: esos eventos están
deshabilitados en la directiva capturada; esto no establece cuál era la directiva durante actividades anteriores.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Host (según el hive `SYSTEM` del sistema) y el hive `SECURITY`. |
| `Layout` | `subcategories` (Windows Vista y posteriores) o `categories` (XP y 2003); vacío cuando el valor no se puede decodificar. |
| `Category`, `Subcategory` | Tal como los denomina `auditpol`. |
| `SubcategoryGUID`, `SubcategoryID` | GUID de la subcategoría y su identificador de mensaje tal como lo indica el evento 4719 (`%%12548`). |
| `Setting` | `none`, `success`, `failure` o `success_and_failure`. |
| `Success`, `Failure` | Si se registran los aciertos y los errores. |
| `AuditingEnabled` | Windows XP y 2003: si la auditoría está activada en absoluto. |
| `Events` | Identificadores de los eventos principales de la subcategoría. |
| `KeyLastWrittenUTC` | Última escritura de `PolAdtEv`: aproximadamente cuándo cambió la directiva por última vez. |
| `DataRaw` | Bytes del valor (en hexadecimal) cuando no se decodifica por completo. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Una estructura o un tamaño desconocidos, recuentos que no cuadran y valores superiores a 3. |

## Estrategia de investigación { #investigation-strategy }

1. Antes de concluir que algo no ocurrió, compruebe si su subcategoría se auditaba:
   Creación de procesos (4688), Inicio de sesión (4624, 4625), Inicio de sesión especial (4672), Otros eventos de acceso a objetos
   (tareas programadas, 4698), Extensión del sistema de seguridad (instalación de servicios, 4697), Administración de cuentas de usuario y
   de grupos de seguridad (4720, 4732), Recurso compartido de archivos (5140), Kerberos (4768, 4769).
2. Compare la directiva con el registro Security: la presencia de eventos de subcategorías que ahora no se auditan
   significa que la directiva cambió. Busque el evento 4719 y un `KeyLastWrittenUTC` reciente.
3. Compare con el `audit.csv` de la directiva de grupo para distinguir una directiva de todo el dominio de un cambio local.
4. Utilice `parse-registry read --recover-deleted` y los registros de transacciones para obtener versiones anteriores de
   `PolAdtEv`.

## Más información { #further-details }

- [Microsoft: configuración de la directiva de auditoría de seguridad avanzada](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: constantes de auditoría (GUID de subcategorías)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
