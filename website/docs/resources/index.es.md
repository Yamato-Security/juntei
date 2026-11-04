# Artículos y referencias { #articles-and-references }

Esta página reúne puntos de partida de alta calidad para los lectores que deseen ir más allá de las guías de
artefactos. Las secciones **Más detalles** de cada artefacto contienen los enlaces de bajo nivel más específicos.

## Orientación DFIR para principiantes { #beginner-dfir-orientation }

- [Póster SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/): un mapa visual de los artefactos de Windows más comunes y sus ubicaciones.
- [Póster SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/): pivotes prácticos de búsqueda de amenazas y recordatorios para investigaciones en Windows.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): muestras públicas de artefactos para el aprendizaje y la validación de analizadores.
- [Referencia de artefactos de Velociraptor](https://docs.velociraptor.app/artifact_references/): descripciones orientadas a la recopilación e implementaciones en VQL para muchos artefactos de Windows.
- [Documentación de Plaso](https://plaso.readthedocs.io/): conceptos de líneas de tiempo y analizadores para el análisis entre artefactos.

## Artefactos del sistema de archivos { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) y [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/): descriptores de seguridad, ACE y expresiones condicionales.
- [Documentación del formato de libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Analizador mft de Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): analizador de NTFS con recuperación (carving) de registros INDX y del espacio residual de los índices.
- [INDXRipper](https://github.com/harelsegev/INDXRipper): recupera nombres de archivo del espacio residual de `$I30`.
- [LogFileParser](https://github.com/jschicht/LogFileParser): investigación detallada y análisis del registro de transacciones de NTFS.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker): correlaciona el historial de `$LogFile`, `$MFT` y USN.

## Artefactos de ejecución e inventario { #execution-and-inventory-artifacts }

- [AmcacheParser de Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [amcache-forensic de Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [libscca de libyal](https://github.com/libyal/libscca): trabajo sobre el formato y la biblioteca de Prefetch.
- [PECmd de Eric Zimmerman](https://github.com/EricZimmerman/PECmd)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [AppCompatCacheParser de Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)

## Artefactos de actividad del usuario { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LECmd de Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
- [JLECmd de Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [SrumECmd de Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registro y persistencia { #registry-and-persistence }

- [Microsoft: Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Especificación del formato del Registro de Windows de Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) y [libregf](https://github.com/libyal/libregf): referencias de bajo nivel para el análisis y la recuperación de hives.
- [Documentación de Registry Explorer / RECmd de Eric Zimmerman](https://ericzimmerman.github.io/)
- [Microsoft: Services registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender y tratamiento de malware { #defender-and-malware-handling }

- [Microsoft: Restore quarantined files in Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [Plugin de Defender de dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump): una implementación independiente de investigación y extracción.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/): recursos de tratamiento y análisis.

## Proyectos de analizadores de Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "El material externo cambia"
    Verifique las afirmaciones con la versión de Windows afectada y con los datos recopilados. Un artículo útil puede
    describir una generación concreta del sistema operativo, una versión del analizador o un experimento, y no una
    regla universal.
