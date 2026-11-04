# Glosario DFIR { #dfir-glossary }

| Término | Significado en lenguaje sencillo |
|---|---|
| Asignado (allocated) | El registro u objeto está actualmente en uso según el artefacto. |
| Flujo de datos alternativo (ADS) | Datos NTFS con nombre adjuntos a un archivo además de su contenido normal sin nombre, como `Zone.Identifier`. |
| Artefacto | Datos persistentes generados por un sistema operativo o una aplicación que pueden responder a preguntas forenses. |
| Carving | Localización de registros por su estructura o firma sin depender de su contenedor o índice habitual. |
| Conjunto de control (control set) | Una versión de la configuración de SYSTEM (`ControlSet001`, etc.); `Select\Current` indica cuál está activo. |
| Corroboración | Evidencia independiente que respalda la misma conclusión. |
| Base de datos/hive sucio (dirty) | El archivo principal no estaba completamente sincronizado cuando se recopiló; los registros de transacciones pueden contener el estado necesario. |
| Droid | Identificador similar a un GUID del seguimiento de vínculos distribuidos (distributed link tracking) que se usa para seguir archivos y volúmenes cuando se mueven. |
| ESE / JET Blue | Formato de base de datos Extensible Storage Engine de Microsoft, utilizado por SRUM y otros datos de Windows. |
| FILETIME | Número de intervalos de 100 nanosegundos desde el 1601-01-01 UTC; un número solo tiene sentido como hora cuando su esquema así lo indica. |
| Referencia de archivo (file reference) | Número de entrada NTFS más número de secuencia que identifica una generación de un registro de la MFT. |
| Hive | Archivo binario que contiene un árbol del Registro de Windows. |
| Registro INDX | Bloque (normalmente de 4 KiB) de un índice de directorio NTFS (`$I30`) que enumera los archivos del directorio; su espacio residual puede conservar posibles evidencias de nombres de archivo eliminados o renombrados. |
| LNK | Shell Link o acceso directo de Windows que contiene un destino y abundantes metadatos. |
| MRU | Orden Most Recently Used (usado más recientemente); normalmente es un orden relativo y no una hora para cada elemento. |
| MFT | Master File Table (tabla maestra de archivos) de NTFS, que contiene los registros y atributos de archivos y directorios. |
| Hash de Prefetch | Identificador de Windows que se usa en el nombre de los `.pf`; no es un hash criptográfico del contenido. |
| Última escritura de una clave del Registro | Hora en que cambió la clave; Windows normalmente no indica qué valor cambió. |
| Datos residentes | Bytes de un archivo o flujo almacenados directamente en un atributo de la MFT en lugar de en ejecuciones de datos (data runs) externas. |
| RID | Relative Identifier (identificador relativo), el componente final de cuenta o grupo bajo una autoridad o dominio de un SID de Windows. |
| SDDL | Security Descriptor Definition Language: un descriptor de seguridad escrito como texto, como `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`. |
| Descriptor de seguridad | El propietario de un archivo, su grupo principal, su DACL (quién puede hacer qué) y su SACL (auditoría y etiqueta de integridad). NTFS guarda cada descriptor distinto una sola vez en `$Secure:$SDS`, bajo un ID de seguridad. |
| SID | Security Identifier (identificador de seguridad) de Windows para una cuenta, un servicio, un grupo o una entidad de seguridad. |
| Espacio residual (slack) | Bytes no utilizados dentro o a continuación de una estructura asignada que pueden conservar datos más antiguos. |
| SRUM | Base de datos System Resource Usage Monitor sobre el uso de recursos por aplicaciones y usuarios. |
| Timestomping | Modificación intencionada de marcas de tiempo para confundir una investigación; las heurísticas no constituyen una prueba. |
| Registro de transacciones | Cambios pendientes o confirmados que se usan para recuperar un Registro o una base de datos hasta un estado coherente. |
| USN | Update Sequence Number (número de secuencia de actualización) utilizado por los registros del diario de cambios de NTFS y los metadatos relacionados. |
| UTC | Tiempo universal coordinado; base horaria común preferida para las líneas de tiempo entre sistemas. |
| Número de serie del volumen | Número que NTFS asigna a un volumen al formatearlo (`$Boot`); sus 32 bits inferiores son lo que los archivos LNK, Jump List y Prefetch registran sobre el volumen de un archivo. |
