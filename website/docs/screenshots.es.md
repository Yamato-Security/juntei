# Capturas de pantalla { #screenshots }

Explore la línea de comandos de Juntei, el análisis automático y los resultados organizados para la investigación.
Haga clic en una captura de pantalla para verla a tamaño completo.

## 1. Inicio y comandos { #1-startup-and-commands }

<figure markdown>

[![Pantalla de inicio de Juntei con la versión CODE BLUE y los comandos disponibles](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>La pantalla de inicio de Juntei enumera el análisis automático, los analizadores específicos de cada fuente y Persistence Hunter.</figcaption>

</figure>

## 2. Análisis automático en menos de 20 segundos { #2-automatic-parsing-in-under-20-seconds }

En un **MacBook Pro M4**, la ejecución que se muestra a continuación analizó **8,7 GiB de datos de Windows recopilados**
de **seis sistemas** y guardó **4,4 GiB de salida en menos de 20 segundos**.
El comando [auto-parse](commands/auto-parse.md) ejecutó 240 tareas de análisis con 16 procesos
en paralelo y generó 482 archivos de resultados. Hubo **210 tareas correctas, 30 omitidas
porque los artefactos no se habían recopilado y ningún fallo**.

<figure markdown>

[![auto-parse de Juntei completando seis sistemas, 8,7 GiB de entrada y 4,4 GiB de salida en menos de 20 segundos](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Detección automática de hosts, análisis en paralelo y un resumen final con el tamaño de entrada, el tamaño de salida, los resultados de las tareas y el tiempo transcurrido.</figcaption>

</figure>

## 3. Resultados organizados para la investigación { #3-results-organized-for-investigation }

<figure markdown>

[![Carpetas de salida de un sistema agrupadas en categorías DFIR, con los CSV de cuentas y sus registros del analizador](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Los resultados de cada equipo se agrupan por categoría DFIR. Los CSV de los artefactos y sus archivos <code>-logs.txt</code> correspondientes se mantienen juntos, lo que facilita encontrar tanto los registros como los detalles del análisis.</figcaption>

</figure>
