---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, diosa de la ciencia forense digital](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> es el analizador multiplataforma de Yamato Security para artefactos forenses de Windows
que <em>no</em> son registros de eventos. Convierte las evidencias recopiladas en registros CSV, JSON y JSONL
coherentes, y conserva el contexto que un investigador necesita para evitar conclusiones erróneas.
</p>

<div class="jt-cta" markdown>
[Empiece aquí :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Conozca los artefactos :material-archive-search:](artifacts/index.md){ .md-button }
[Referencia de comandos :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## Del artefacto sin procesar a una conclusión defendible { #from-raw-artifact-to-defensible-conclusion }

Juntei puede indicarle lo que registra un artefacto; este sitio explica lo que ese registro **demuestra y lo que no
demuestra**. Empiece por la página de un artefacto aunque nunca haya realizado DFIR en Windows.
Cada guía explica su finalidad, la ubicación en disco, las salvedades de retención y recopilación, el comando de análisis,
los campos de salida, la interpretación, las correlaciones y lecturas adicionales.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __Historial del sistema de archivos__

    ---

    Utilice la `$MFT` y el Diario USN para reconstruir la existencia, los nombres, el movimiento, la eliminación,
    los cambios de metadatos y los flujos de datos alternativos de los archivos.

-   :material-play-circle-outline:{ .lg .middle } __Evidencias de programas__

    ---

    Prefetch puede respaldar con solidez la ejecución. Amcache y ShimCache añaden contexto de inventario, ruta, hash,
    compatibilidad e historial, pero requieren una interpretación cuidadosa.

-   :material-account-clock:{ .lg .middle } __Actividad del usuario__

    ---

    Los archivos LNK, Jump Lists, RecentDocs y SRUM revelan elementos abiertos, actividad de aplicaciones,
    ubicaciones extraíbles o de red, uso de recursos y contexto histórico del usuario.

-   :material-shield-bug:{ .lg .middle } __Evidencias de seguridad__

    ---

    La cuarentena de Defender puede conservar etiquetas de amenazas, rutas originales, orígenes de descarga, hashes y
    propietarios, e incluso recuperar de forma segura los bytes en cuarentena para su análisis aislado.

-   :material-database-search:{ .lg .middle } __Estado del Registro__

    ---

    Lea datos del Registro asignados y recuperables, reproduzca los registros de transacciones e inspeccione servicios,
    tareas programadas, cuentas locales y el estado de los documentos recientes.

-   :material-link-variant:{ .lg .middle } __Correlacione, no suponga__

    ---

    Ningún artefacto cuenta por sí solo toda la historia. Las guías muestran cómo combinar fuentes independientes
    en una cronología respaldada por evidencias.

</div>

!!! warning "La salida forense es evidencia, no un veredicto"
    Una ruta que parece sospechosa es una pista. Una marca de tiempo puede describir los metadatos de un archivo en lugar de
    una ejecución. La ausencia de un registro no demuestra que una acción no ocurriera. Conserve las evidencias de origen,
    registre las versiones de las herramientas y los comandos, y corrobore las conclusiones importantes.
