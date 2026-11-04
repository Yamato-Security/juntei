# Agradecimientos { #acknowledgements }

Juntei existe porque las comunidades DFIR y de código abierto han publicado analizadores,
investigaciones sobre formatos, salidas de referencia, especificaciones y evidencias de prueba. Los
proyectos que se indican a continuación han servido de base para la implementación, la validación,
la denominación de campos o la salida esperada por los analistas. Figurar en esta lista no significa
que su código esté incluido en Juntei; consulte la licencia de cada proyecto.

## Bases de los analizadores mantenidas por Yamato Security { #parser-foundations-maintained-by-yamato-security }

| Área de Juntei | Analizador mantenido | Proyecto original/base |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| Diario USN | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| `$LogFile` de NTFS | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Índices de directorios (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`, módulo `i30`) | Investigación sobre el espacio residual de índices citada más abajo |
| `$Boot`, `$Secure:$SDS` de NTFS | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crates `boot` y `secure`) | MS-DTYP de Microsoft y la investigación sobre NTFS citada más abajo |
| SRUM / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) y [ese](https://github.com/Sootmark/ese) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Trabajo de Sootmark shell sobre archivos compuestos y Jump Lists |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| Registro | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Investigación sobre el formato y la recuperación del Registro citada más abajo |

## Herramientas de referencia y validación independiente { #reference-tools-and-independent-validation }

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), de Hexastrike,
  aportó el catálogo de comprobaciones de persistencia, la lista de permitidos predeterminada, la
  instantánea de LOLBAS y el comportamiento de referencia para el comando nativo en Rust
  `persistence-hunter`. Su licencia MIT y su aviso de copyright se conservan en
  `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`. El
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) original inspiró el proyecto
  en Python; consulte los [agradecimientos de persistencia](persistence-hunter/acknowledgements.md)
  para conocer el linaje completo.

- La **especificación regf** y **yarp** de Maxim Suhanov sirvieron de base para las estructuras del
  Registro, la reproducción de los registros de transacciones y la recuperación de registros
  eliminados; su **dfir_ntfs** fue la referencia para los registros INDX y la recuperación
  (carving) del espacio residual de índices.
- **INDXRipper**, de Harel Segev, es una referencia adicional para recuperar nombres de archivo del
  espacio residual de índices.
- **dfir_ntfs**, **LogFileParser** de Joakim Schicht y **ntfs-linker** de Stroz Friedberg sirvieron
  de base para la validación e interpretación de `$LogFile`; **go-ntfs** de Velocidex aportó
  evidencias de prueba públicas de NTFS.
- Los proyectos de Joachim Metz / libyal **libregf**, **libfsntfs**, **libscca**, **libfwsi** y
  **libesedb** ofrecen una investigación detallada de los formatos y referencias de análisis
  independientes.
- **dissect.esedb**, **dissect.ntfs** y **dissect.target** de Fox-IT aportaron interpretaciones
  independientes de SRUM, `$Secure:$SDS`, SAM y Defender que se utilizaron durante la validación.
- `fsstat` de **The Sleuth Kit** fue la referencia para los sectores de arranque NTFS de volúmenes
  creados con `mkntfs`.
- Algunas de las herramientas de Eric Zimmerman establecieron los nombres de campo y las
  expectativas de los examinadores que se usaron en las comparaciones.
- **regipy**, **python-registry**, **Notatin** y **artemis** aportaron implementaciones del
  Registro, pruebas o interpretaciones de muestras.
- **LnkParse3**, **lnk_parser** y **jumplist_parser** de AbdulRhmanAlfaifi, y JLECmd aportaron
  comportamientos independientes de LNK/Jump Lists y casos de prueba públicos.
- **prefetch-forensic** de Security Ronin y el proyecto **forensicnomicon** sirvieron de base para
  las observaciones de alto valor de Juntei sobre Prefetch/Amcache/ShimCache.
- **defender-dump** y dissect.target ofrecen implementaciones independientes para comprender las
  estructuras de cuarentena de Defender y un comportamiento de extracción seguro.
- **Plaso**, **Chainsaw** y **Velociraptor** aportaron comparaciones de analizadores, evidencias de
  prueba o comportamientos de referencia orientados a la recopilación en varios artefactos.

## Especificaciones e investigación { #specifications-and-research }

Agradecemos a Microsoft la publicación de las Open Specifications para Shell Links, Compound Binary
Files, los protocolos NTFS/SAM, las estructuras de Registro, servicios y tareas, y la documentación
de las API. También agradecemos a los autores de las investigaciones forenses sobre AppCompatCache,
Amcache, SRUM, Prefetch, el Registro y NTFS, que han hecho que estos artefactos resulten
comprensibles para los profesionales.

## Evidencias de prueba { #test-evidence }

Las evidencias públicas del
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) de Andrew Rathbun, NIST
CFReDS, Plaso, Regipy, Artemis, los repositorios de pruebas de analizadores y otras colecciones con
licencias adecuadas permiten realizar pruebas de regresión con datos reales de Windows. Juntei
conserva notas de atribución y licencia junto a los archivos de prueba incluidos.

## Colaboradores { #contributors }

Gracias a todos los colaboradores, revisores, personas que han informado de incidencias y usuarios
de Juntei y de los analizadores de Yamato Security que han aportado casos límite. GitHub muestra los
colaboradores actuales en la
[página de colaboradores de Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors).

## Licencia { #license }

Juntei se publica bajo la
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) y su copyright pertenece a Shirofune Security Co., Ltd. Su código fuente se publicó bajo
la AGPLv3 antes de la versión 1.0.0. Las licencias de los crates y demás código y datos de terceros
con los que se compila se encuentran en
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
