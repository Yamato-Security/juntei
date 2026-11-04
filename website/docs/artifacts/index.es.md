# Guía de artefactos { #artifact-guide }

Windows conserva rastros que se solapan en el sistema de archivos, el Registro, las bases de datos de compatibilidad,
el historial de la interfaz de usuario, la contabilidad de recursos y los productos de seguridad. Juntei analiza estas fuentes,
pero el investigador sigue teniendo que interpretar lo que significa cada una.

## Elegir un artefacto según la pregunta { #choose-an-artifact-by-question }

| Pregunta de la investigación | Empiece aquí | Corrobore con |
|---|---|---|
| ¿Se ejecutó este ejecutable? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), registros de eventos |
| ¿Estuvo presente alguna vez este archivo? | [Amcache](amcache.md), [$MFT](mft.md) | [Diario USN](usn-journal.md), [$I30](i30.md), ShimCache |
| ¿Cuándo se creó, renombró o eliminó un archivo? | [Diario USN](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), cuarentena de Defender |
| ¿Qué nombre o marca de tiempo existía antes de un cambio de metadatos? | [$LogFile](logfile.md) | [$MFT](mft.md), [Diario USN](usn-journal.md), [$I30](i30.md) |
| ¿Quién es el propietario de un archivo y quién puede modificarlo? | [$Secure:$SDS](secure-sds.md) con [$MFT](mft.md) | [SAM](sam.md), registros de eventos Security |
| ¿En qué volumen estaba el destino de un acceso directo o de un archivo Prefetch? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [dispositivos USB](usb.md) |
| ¿Qué abrió un usuario? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| ¿Transfirió un programa una gran cantidad de datos? | [SRUM](srum.md) | Telemetría de cortafuegos, proxy, DNS, paquetes o EDR |
| ¿Se configuró persistencia? | [Servicios](services.md), [TaskCache](taskcache.md), [Registro](registry.md) | Registros de eventos, Prefetch, `$MFT` |
| ¿Qué eliminó Defender? | [Cuarentena de Defender](defender-quarantine.md) | Registros de eventos, `$MFT`, Diario USN |
| ¿Se añadieron o modificaron cuentas locales? | [SAM](sam.md) | Registros de eventos Security e historial de transacciones del Registro |

## Leer las advertencias antes que los campos { #read-warnings-before-fields }

Todas las páginas de artefactos siguen la misma secuencia:

1. **Qué es** y a qué preguntas puede responder.
2. **Dónde encontrarlo** y cómo le afecta la recopilación.
3. **Qué demuestra y qué no demuestra.**
4. **Cómo analizarlo** con Juntei.
5. **Cómo interpretar cada campo de salida.**
6. **Cómo correlacionarlo** con evidencias independientes.
7. **Más información** para la investigación a bajo nivel.

!!! danger "No equipare un hallazgo del analizador con malicia"
    Los hallazgos de Juntei identifican patrones que merece la pena revisar, como un nombre de binario del sistema fuera de
    `System32`. Los instaladores, los administradores, las capas de compatibilidad y las actualizaciones de software pueden generar
    registros inusuales. Valídelos con contexto y evidencias independientes.
