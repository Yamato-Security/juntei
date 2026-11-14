# Lista de comandos { #command-list }

Los analizadores individuales se agrupan según su tipo de origen. Ejecute `juntei --help`,
`juntei <SOURCE> --help` o `juntei <SOURCE> <ARTIFACT> --help` para explorarlos:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Para una recopilación de evidencias de Windows de varios equipos, utilice
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). Invoca los mismos comandos agrupados
y organiza los resultados y los registros por equipo y por categoría de investigación.

Para obtener una línea de tiempo que abarque los artefactos analizados, utilice
[`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md). Expande las marcas
de tiempo y etiqueta su significado. La salida no se ordena de forma predeterminada; añada `--sort`
para obtener un orden cronológico y consolidar los duplicados. `auto-parse -j` sigue conservando los
registros nativos.

## [Imágenes de disco](mount.md) { #disk-images }

| Comando | Entrada | Guía detallada |
|---|---|---|
| `mount` | Imagen de disco VHDX / E01 / Ex01 | [Montar imágenes de disco](mount.md) |

## [Comandos ESE](user-activity.md) { #ese-commands }

| Comando | Artefacto de entrada | Guía detallada |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Registro de acceso de usuarios (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer y Edge heredado](../artifacts/webcache.md) |

## [Comandos de archivos](execution.md) { #file-commands }

| Comando | Artefacto de entrada | Guía detallada |
|---|---|---|
| `parse-file defender-quarantine` | Árbol o entrada de la cuarentena de Defender | [Cuarentena de Defender](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [Accesos directos LNK](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |
| `parse-file rdp-cache` | `Cache????.bin`, `bcache*.bmc` | [Caché de mapas de bits de RDP](../artifacts/rdp-cache.md) |
| `parse-file recycle-bin` | `$Recycle.Bin\<SID>\$I*`, `RECYCLER\<SID>\INFO2`, `RECYCLED\INFO2` | [Papelera de reciclaje](../artifacts/recycle-bin.md) |

## [Comandos NTFS](filesystem.md) { #ntfs-commands }

| Comando | Artefacto de entrada | Guía detallada |
|---|---|---|
| `parse-ntfs boot` | `$Boot` o una imagen de volumen NTFS | [Sector de arranque ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`) o una imagen de disco | [Índices de directorios ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | `$LogFile` de NTFS | [Registro de transacciones NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | `$MFT` de NTFS | [Tabla maestra de archivos](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Descriptores de seguridad ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [Diario USN](../artifacts/usn-journal.md) |

## [Comandos del Registro](security-registry.md) { #registry-commands }

| Comando | Artefacto de entrada | Guía detallada |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Redes](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [Secretos LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (y `BCD-Template`) | [Configuración de arranque (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Compatibilidad de aplicaciones](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Directiva de auditoría](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps y elementos anclados a la barra de tareas](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Historial de aplicaciones](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Listas de elementos usados recientemente](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (con `SAM` y `SOFTWARE` para los nombres de usuario) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` en Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Programas instalados](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Cuadros de diálogo Abrir y Guardar como](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Inicio automático en el Registro](../artifacts/persistence.md) |
| `parse-registry read` | Cualquier hive principal del Registro | [Hives del Registro](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (con `SOFTWARE` para los SID de usuario) | [Acceso remoto (lado del usuario)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [Cuentas locales SAM](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Configuración de seguridad](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Servicios de Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` o un valor AppCompatCache exportado | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` de un mismo sistema | [Información del sistema](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Caché del Programador de tareas](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, con `SOFTWARE` y `NTUSER.DAT` | [Dispositivos USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [Comandos SQLite](sqlite.md) { #sqlite-commands }

| Comando | Artefacto de entrada | Guía detallada |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Navegadores web](../artifacts/browser.md) |

## Tipos de origen y cambios de formato { #source-types-and-format-changes }

Los grupos describen los formatos que Juntei admite actualmente. `parse-registry amcache` lee
`Amcache.hve`; no implica compatibilidad con las bases de datos de inventario de Amcache en SQLite. Los crates de los analizadores
de artefactos se mantienen separados para que la compatibilidad con otro formato de almacenamiento pueda añadirse de forma independiente.

Los antiguos nombres planos de los analizadores se han sustituido por las rutas de origen y artefacto indicadas arriba. Mantenga las
opciones existentes del analizador después del nombre del artefacto. `auto-parse` y los subcomandos del Registro existentes
conservan sus nombres.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` busca en varios formatos de origen, por lo que es un comando de análisis
de nivel superior. Complementa el inventario completo de la configuración de `parse-registry persistence` con
131 comprobaciones, clasificación de permitidos y bloqueados, metadatos de las cargas útiles, cobertura y correlación de marcas de tiempo.

## Uso de nivel superior { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Las páginas de cada origen reproducen el uso de los comandos y ofrecen ejemplos breves. Las guías de artefactos explican
la evidencia, los campos de salida, las limitaciones y la estrategia de investigación.
