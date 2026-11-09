# Acerca de Juntei { #about-juntei }

Juntei es un analizador de artefactos forenses de Windows escrito en Rust, un lenguaje con seguridad de memoria. Complementa a
[Hayabusa](https://github.com/Yamato-Security/hayabusa): Hayabusa analiza los registros de eventos de Windows,
mientras que Juntei analiza artefactos persistentes del sistema de archivos, del Registro, de ejecución, de actividad del usuario, de uso de recursos
y de productos de seguridad.

El mismo binario funciona en Windows, macOS y Linux. Los analistas pueden examinar pruebas de Windows recopiladas e imágenes de disco con una interfaz coherente, en lugar de usar un programa distinto para cada artefacto.

## Objetivos de diseño { #design-goals }

- **Análisis defendible:** conservar los valores sin procesar e informar de los daños en lugar de inventar datos de forma silenciosa.
- **Útil para todos:** el uso predeterminado debe ser sencillo para los principiantes, pero también personalizable y completo
  para los usuarios veteranos.
- **Binario independiente:** no requiere ninguna biblioteca de tiempo de ejecución para funcionar. Funciona tal cual, sin
  configuración adicional.
- **Extremadamente rápido:** aspira a ser el analizador más rápido sin necesitar por ello una gran cantidad de
  memoria.
- **Uso multiplataforma:** analizar evidencias de Windows sin necesidad de una estación de trabajo con Windows.
- **Automatización:** CSV para hojas de cálculo y herramientas de línea de tiempo; JSON/JSONL para `jq`, scripts y data lakes.
- **Fallo seguro:** los registros con formato incorrecto se notifican y se omiten siempre que sea posible.
- **Linaje transparente:** Juntei identifica los analizadores de código abierto y las herramientas de referencia en que se basa cada implementación.

## Lo que Juntei no hace { #what-juntei-does-not-do }

Juntei no adquiere un disco en vivo ni sustituye la preservación de evidencias, y las únicas imágenes de disco que
monta son imágenes VHDX y E01, en modo de solo lectura. No
combina automáticamente todos los artefactos en una única narrativa del incidente. Analiza las
evidencias que se le proporcionan y produce registros adecuados para la investigación y la correlación.

Juntei tampoco realizará el análisis por usted. Se requiere un especialista en DFIR para analizar las evidencias.

## Licencia { #license }

Juntei se publica bajo la
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) y su copyright pertenece a Shirofune Security Co., Ltd. Su código fuente se publicó bajo la
AGPLv3 antes de la versión 1.0.0. Las licencias de los crates y demás código y datos de terceros con los que
se compila figuran en
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
Los proyectos en los que se basa Juntei y con los que se ha contrastado se enumeran en
[Agradecimientos](../acknowledgements.md).
