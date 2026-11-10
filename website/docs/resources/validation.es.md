# Validación de analizadores { #parser-validation }

Los crates de analizadores que mantiene Juntei se prueban con evidencias públicas e
implementaciones independientes. Estas comparaciones ayudan a detectar regresiones y diferencias
de interpretación, pero no hacen infalible a ningún analizador. Conserve la evidencia de origen,
revise los campos de problemas o daños de cada registro y valide los hallazgos relevantes con otro
artefacto u otra implementación.

Las afirmaciones siguientes conservan la información de validación que antes figuraba en el README
principal de Juntei. Consulte los repositorios de analizadores enlazados para conocer los datos de
prueba y los detalles de implementación actuales.

## Amcache { #amcache }

Juntei utiliza [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser), una
bifurcación con licencia Apache-2.0 de
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) de Security Ronin. Su
[documentación de salida](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
define todos los campos de ejecutables, dispositivos, hallazgos y resumen.

## Tabla maestra de archivos { #master-file-table }

El crate mft de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) es una
bifurcación mantenida del crate [mft](https://github.com/omerbenamram/mft) de Omer Ben-Amram. En
ocho archivos `$MFT` (la muestra del crate, el caso de hacking del NIST y seis archivos del DFIR
Artifact Museum), 1 048 733 filas CSV y sus columnas coincidieron en orden con un analizador de
referencia. Las comparaciones abarcaron la salida predeterminada, `--all-times`, `--short-names` y
`--cutoff`; la presentación de las marcas de tiempo fue la diferencia intencionada.

## Registro de transacciones NTFS ($LogFile) { #ntfs-transaction-log-logfile }

El crate logfile de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) se mantiene
a partir de [Sootmark ntfslog](https://github.com/Sootmark/ntfslog). Su validación abarca seis
registros completos del DFIR Artifact Museum, datos de prueba públicos de las versiones 1.1 y 2.0,
metadatos redo/undo decodificados y escrituras del diario comparadas con datos NTFS analizados de
forma independiente. Consulte el repositorio del analizador para conocer la cobertura detallada y
las diferencias conocidas.

Juntei añade pruebas de la CLI que comparan cada campo CSV con JSON y JSONL en el volumen de prueba
público de go-ntfs (2129 registros), verifican por separado los datos redo/undo y las longitudes de
las cargas útiles sin procesar, prueban registros dañados, rechazan sobrescribir evidencias y
evitan que las rutas actuales de la MFT sigan números de registro reutilizados. Los casos de
regresión distinguen los registros de la MFT afectados del atributo abierto a través del cual
escriben, comprueban los metadatos planos de entradas de directorio y atributos de datos, e impiden
emparejar MFT de volúmenes distintos. Las pruebas de `auto-parse` con varios volúmenes abarcan
archivos CSV separados y un único archivo JSONL con información de procedencia.

El comando también se comprobó con seis registros de Windows recopilados de forma privada: los
708 465 registros CSV exportados coincidieron campo por campo con JSONL, y las exportaciones de
`auto-parse` por host conservaron los mismos recuentos de registros. La evidencia de origen no se
distribuye con el repositorio. Estas comprobaciones verifican la coherencia de la salida, no que
cada transacción registrada represente una acción de usuario completada.

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) es una bifurcación
mantenida de [frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) de ForensicRS. Su salida
decodificada se comparó con [libscca](https://github.com/libyal/libscca), la implementación
independiente de Prefetch de libyal.

## Diario USN { #usn-journal }

El crate usn de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) es una
bifurcación mantenida de [Sootmark usn](https://github.com/Sootmark/usn). En seis diarios reales,
las rutas de directorio principal de Juntei, que tienen en cuenta el tiempo, coincidieron con las
de un analizador independiente siempre que este resolvió una ruta. Juntei también resolvió 4321
registros que la comparación dejó sin resolver. Su carver no produjo registros falsos al probarse
con 880 MB de datos de `$MFT`.

## Índices de directorios ($I30) { #directory-indexes-i30 }

El módulo `i30` del crate mft de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)
lee los registros INDX y su espacio residual. En dos flujos `$I30` de un volumen creado con ntfs-3g
(`test_files/i30/` de Juntei) y en tres muestras públicas de `$I30` escritas por Windows, sus
entradas en uso y en el espacio residual coincidieron con las de
[dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) de Maxim Suhanov, con los mismos nombres y horas.
Cuando dfir_ntfs informa de una referencia al directorio principal que supera los 32 bits (bytes
que sobrescribieron el directorio principal), Juntei deja vacío el directorio principal.

## Sector de arranque ($Boot) { #boot-sector-boot }

El crate boot de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) es propio de
Yamato Security. En los cinco archivos `$Boot` públicos del DFIR Artifact Museum (Windows 10, 11 y
Server 2012 R2), todos los campos coincidieron con un analizador de referencia. En volúmenes
creados con `mkntfs` con sectores de 512, 1024 y 4096 bytes y clústeres de 4 KiB a 64 KiB, los
tamaños, los clústeres de `$MFT` y `$MFTMirr` y los números de serie coincidieron con `fsstat` de
The Sleuth Kit. El `$Boot` de Windows 11 del Museum tiene el número de serie de volumen que registra
el archivo LNK del mismo host, y los cinco archivos tienen el mismo código de arranque.

## Descriptores de seguridad ($Secure:$SDS) { #security-descriptors-securesds }

El crate secure de [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) es propio de
Yamato Security. En los cinco flujos `$SDS` del Museum (7808 descriptores, 60 781 ACE), los ID de
seguridad, los hashes, los desplazamientos, los propietarios, los grupos, los indicadores de control
y los recuentos de ACE coincidieron con un analizador de referencia, y el tipo, los indicadores, la
máscara, el SID y los tipos de objeto de cada ACE coincidieron con `dissect.ntfs`. Cada hash
registrado corresponde a su descriptor, cada copia espejo coincide con su entrada y cada descriptor
tiene SDDL, incluidas las condiciones de sus 3384 ACE de devolución de llamada (callback).

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) se basa en los crates
[ese](https://github.com/Sootmark/ese) y [srum](https://github.com/Sootmark/srum) de Sootmark. En
11 bases de datos reales, todas las columnas de las 45 255 entradas generadas coincidieron con una
decodificación independiente de los valores que expone `dissect.esedb`.

## Registro de acceso de usuarios { #user-access-logging }

`parse-ese ual` lee las bases de datos de UAL con el mismo lector ESE. En 16 bases de datos reales
(308 filas, 145 de ellas filas de clientes), todos los valores coincidieron con una decodificación
independiente de los valores que expone `dissect.esedb`. En cada fila de cliente, el primer día con
recuento fue el día UTC de `InsertDate`, el último, el día UTC de `LastAccess`, y los recuentos sumaban
`TotalAccesses`.

## Navegadores web { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) es una bifurcación mantenida de los
crates browser, sqlite y plist de Sootmark, elegida tras compararla con otra biblioteca de Rust y con Plaso,
Velociraptor (SQLiteHunter) y Hindsight sobre las mismas bases de datos. `parse-sqlite browser` se comparó con la
biblioteca `sqlite3` leyendo una copia de cada base de datos con su archivo `-wal`, en 38 bases de datos públicas:
los archivos de prueba de Plaso (Chrome 8 a 68, Firefox 2 a 118), perfiles creados con Chromium 150 y Firefox 140,
y las bases de datos de prueba de historial de formularios de ys-browser-parser. Coincidieron las 293 visitas, las
21 descargas, las 692 cookies y las 41 entradas de formularios. Solo se recuperaron entradas eliminadas donde se
habían eliminado entradas: las dos páginas y las dos visitas eliminadas del historial de Firefox 140, con sus horas,
URL y títulos, y las entradas de formularios eliminadas de las bases de datos de prueba.

## Internet Explorer y Edge heredado (WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` lee `WebCacheV01.dat` con
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser). Se comparó con el lector ESE de
`dissect.database` en las dos bases de datos WebCache públicas de Plaso: una de Internet Explorer 11 y otra sucia de
Windows 10 con el historial, las descargas y las cookies del Edge heredado. Coincidieron las 4156 filas, valor por
valor: 44 contenedores, 182 entradas del historial, 6 descargas, 170 cookies y 3754 entradas más. La comparación
decodificó por su cuenta los registros de descarga. Los registros que las bases de datos marcan como eliminados se
excluyeron de ambos lados.

## Outlook { #outlook }

`parse-file outlook` lee archivos PST, OST, `.msg` y `.oft` con
[ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser), cuyo lector de PST y OST es un fork de
[outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) de Microsoft y cuyo lector de `.msg` está escrito a partir
de las especificaciones abiertas de Microsoft.

- **PST y OST**: 66 archivos PST y OST públicos (Apache Tika, artemis, los ejemplos de Aspose.Email, el DFIR Artifact
  Museum, Enron, los ejemplos de GroupDocs, java-libpst, los datos de prueba de libyal, outlook-pst-rs y pst-extractor)
  se compararon con libpff, propiedad por propiedad: cada propiedad de cada almacén, carpeta, mensaje, destinatario y
  adjunto, y los datos de cada adjunto. 324 175 valores coincidieron y ninguno difirió sin una explicación. Los demás
  son lo que solo lee ys-outlook-parser (15 846 valores de mensajes, destinatarios y adjuntos en los que falla libpff,
  y de un almacén que no puede abrir; 8223 de elementos ocultos), celdas de destinatarios que libpff da como vacías o
  cero aunque sus filas indican que no existen (49 077), identificadores de entrada y tipos de carpetas calculados como
  lo hace MAPI (2004), y objetos y mensajes incrustados que libpff da como sus datos sin procesar (480). Ninguno de
  los dos lectores puede leer en absoluto dos de los archivos.
- **`.msg` y `.oft`**: 214 archivos públicos, con 2404 mensajes, destinatarios y adjuntos y 41 002 propiedades, se
  compararon con extract-msg: 110 114 valores coincidieron, y cada uno de los 791 que difieren tiene explicación: 601
  son errores de extract-msg con las horas, 35 proceden de archivos ANSI que decodifica con la página de códigos
  equivocada, y los demás son lo que omite o añade.
- **Entradas dañadas**: más de 260 000 archivos `.msg` mutados y 160 000 PST mutados, además de archivos diseñados
  expresamente, se leyeron sin un fallo, sin un bloqueo y sin un uso de memoria ilimitado.

## Accesos directos LNK { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) es una bifurcación mantenida del
crate [shell](https://github.com/Sootmark/shell) de Sootmark. Se comparó con 1118 accesos directos
públicos. De las 30 columnas CSV compartidas con el analizador de referencia, 16 valores
individuales difirieron: 13 rutas de destino con elementos dañados, truncados, de marcador de
posición, MTP o del Panel de control; dos listas de bloques de datos adicionales con bloques
desconocidos o truncados; y una referencia de la MFT que el analizador de referencia leyó de bytes
distintos. Juntei informa de las estructuras dañadas en lugar de normalizarlas de forma silenciosa.

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) utiliza el trabajo
de Sootmark sobre archivos compuestos y Jump Lists, con los vínculos incrustados decodificados por
`ys-lnk-parser`. En 77 Jump Lists públicas, las 13 057 celdas de comparación compartidas, incluidas
las marcas de tiempo con precisión de 100 ns, coincidieron con el analizador de referencia. Seis
archivos que la referencia rechazó (listas personalizadas vacías, un pie ausente o un archivo que
no era una Jump List) fueron decodificados o notificados explícitamente por Juntei.

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) es una bifurcación
mantenida de
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) de Security Ronin, con
compatibilidad con hives del Registro y registros de transacciones. En 20 hives `SYSTEM` públicos
(24 conjuntos de control y 7290 entradas) más 17 valores AppCompatCache independientes, cada
entrada coincidió en ruta, hora, indicador de ejecución, indicadores y posición con lo generado por
el analizador de referencia.

## Cuarentena de Microsoft Defender { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) es una
bifurcación mantenida del crate [defender](https://github.com/Sootmark/defender) de Sootmark. Una
carpeta de cuarentena escrita en el formato de Defender por las herramientas de prueba del
analizador se decodifica de forma coherente con el lector de cuarentena independiente de
`dissect.target`.

## Artefactos derivados del Registro { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) se compara con
[yarp](https://github.com/msuhanov/yarp) en cuanto a claves y valores asignados, reproducción de
transacciones en hives sucios, recuperación de celdas libres, espacio residual de celdas, datos más
allá de los bins del hive y registros eliminados o sustituidos por la reproducción. Las
comparaciones específicas de cada artefacto incluyen:

- servicios, valores y acciones en caso de error de hives `SYSTEM` de Windows 7 y de Windows 11
  sucios;
- RecentDocs decodificado con yarp y libfwsi en nueve hives `NTUSER.DAT` públicos;
- tareas programadas decodificadas de forma independiente con yarp en siete hives `SOFTWARE`
  públicos que abarcan Windows 7–11, Server 2012 R2 y Server 2022; y
- estructuras de cuentas SAM comparadas con `dissect.target` en siete hives `SAM` públicos.

Consulte [Agradecimientos](../acknowledgements.md) para ver la lista completa de herramientas de
referencia, investigaciones sobre formatos, especificaciones y fuentes de evidencias públicas.
