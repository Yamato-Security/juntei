# Investigaciones entre artefactos { #cross-artifact-workflows }

Ningún artefacto debería sostener por sí solo una conclusión importante. Estos flujos de trabajo muestran cómo convertir una pista
en una cadena de evidencias defendible.

## Ejecución de un ejecutable sospechoso { #suspicious-executable-execution }

1. **Prefetch:** establezca la hora de ejecución conservada, el contexto de ruta y hash, el recuento de ejecuciones y los archivos cargados.
2. **`$MFT`:** busque la ruta, el tamaño, las marcas de tiempo, los ADS, el estado de eliminación y la referencia MFT.
3. **Diario USN:** reconstruya la creación, las escrituras, los cambios de nombre y la eliminación.
4. **Amcache:** añada el SHA-1, los metadatos PE, la versión y la ruta histórica.
5. **ShimCache:** añada la ruta y el orden en la caché de compatibilidad y el indicador de ejecución compatible.
6. **SRUM:** busque el uso de recursos y de red por parte del mismo ejecutable o usuario.
7. **Defender:** compruebe la hora de cuarentena, la ruta original, el hash y el origen de la descarga.
8. **Registros de eventos/EDR:** identifique el proceso principal, la línea de comandos, la cuenta, la red y el contexto de seguridad.

La conclusión debe distinguir entre la presencia del archivo, la ejecución y la atribución a una persona o a una intrusión.

## Actividad de archivos de ransomware { #ransomware-file-activity }

1. Identifique las extensiones cifradas y las rutas afectadas en `$MFT`.
2. Utilice los motivos y el orden del USN para reconstruir escrituras, cambios de nombre y eliminaciones masivos.
3. Utilice Prefetch para identificar los posibles cifradores y sus horas de ejecución.
4. Utilice SRUM para detectar picos de CPU, disco o red del proceso candidato.
5. Compruebe los servicios y TaskCache en busca de despliegue o persistencia.
6. Compruebe la cuarentena de Defender y los registros de eventos en busca de detección o corrección.

La MFT y el USN pueden demostrar el impacto en el sistema de archivos aunque no sobreviva ningún artefacto de ejecución. En general,
no identifican el proceso responsable sin telemetría de procesos o correlación.

## Acceso inicial mediante un documento o acceso directo malicioso { #malicious-document-shortcut-initial-access }

1. RecentDocs y las Jump Lists identifican la interacción del usuario o de la aplicación y el contexto MRU.
2. Analice el LNK correspondiente de la carpeta Recent para obtener el destino, el volumen o recurso compartido, la referencia MFT y los argumentos.
3. Utilice el Zone.Identifier del ADS residente de `$MFT` o el contenido de Defender para obtener las URL y zonas de origen.
4. Utilice Prefetch y los registros de eventos para la aplicación del documento y los procesos secundarios.
5. Utilice el USN y la MFT para seguir la creación y la eliminación de la carga útil extraída.

## Persistencia mediante servicios { #service-persistence }

1. Decodifique el nombre del servicio, `ImagePath`, `ServiceDll`, la cuenta, el tipo de inicio y las acciones de error.
2. Resuelva las variables de entorno y el comportamiento de las rutas con y sin comillas.
3. Localice el ejecutable o la DLL en `$MFT` y en el historial del USN.
4. Compruebe Prefetch y los registros de eventos para confirmar el inicio real del servicio o del proceso.
5. Compare los conjuntos de controles no actuales y los registros del Registro recuperados para obtener la configuración anterior.

## Persistencia mediante tareas programadas { #scheduled-task-persistence }

1. Decodifique las acciones de TaskCache, la entidad de seguridad y el contexto, los metadatos de registro y ejecución y el estado oculto.
2. Compare con el XML de `C:\Windows\System32\Tasks`.
3. Correlacione el nombre o el GUID de la tarea con los eventos de Security y del registro Operational del Programador de tareas.
4. Localice el ejecutable o script y calcule su hash; inspeccione Prefetch y el historial del sistema de archivos.
5. Recupere los registros de TaskCache eliminados si la tarea se quitó.

## Sospecha de exfiltración { #suspected-exfiltration }

1. Utilice SRUM para identificar volúmenes inusuales de bytes enviados por aplicación, usuario o perfil, y el período de medición.
2. Confirme la identidad del ejecutable con Prefetch, Amcache y `$MFT`.
3. Identifique los archivos a los que se accedió o que se abrieron mediante Jump Lists, LNK, RecentDocs y datos de eventos o EDR.
4. Utilice la telemetría de proxy, cortafuegos, DNS, paquetes o EDR para el destino y el protocolo.

El volumen de SRUM respalda una hipótesis de exfiltración, pero no identifica el contenido transferido ni demuestra
por sí mismo una intención maliciosa.

## Claves de correlación que merece la pena conservar { #correlation-keys-worth-preserving }

| Clave | Artefactos |
|---|---|
| Ruta completa / nombre de archivo | Casi todos los artefactos |
| SHA-1 / SHA-256 | Amcache, Defender, archivos recuperados |
| Entrada + secuencia de la MFT | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, Diario USN |
| Número de serie del volumen / droids | Prefetch, LNK, Jump Lists, identificadores de objeto de la MFT |
| SID / nombre de usuario | SRUM, SAM, Registro, registros de eventos |
| GUID de tarea / nombre de tarea | TaskCache, XML de tareas, registros de eventos |
| Nombre de servicio | Servicios, registros de eventos, datos de procesos y controladores |
| EntryId / ScanId de Defender | Cuarentena, historial de Defender, registros de eventos |
