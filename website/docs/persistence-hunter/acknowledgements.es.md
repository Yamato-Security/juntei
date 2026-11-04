# Agradecimientos de Persistence Hunter { #persistence-hunter-acknowledgements }

El comando `persistence-hunter` de Juntei se basa en dos generaciones de trabajo comunitario sobre la
búsqueda de persistencia. Gracias a los autores y colaboradores que pusieron sus técnicas,
implementaciones e investigaciones a disposición de los investigadores.

## PyrsistenceSniper: el proyecto en Python { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), de
[Hexastrike](https://github.com/Hexastrike), es la referencia directa de esta funcionalidad. Juntei
porta a Rust nativo su catálogo de 130 comprobaciones, junto con la lista de permitidos
predeterminada, la instantánea de LOLBAS y el comportamiento de clasificación. La versión de
referencia está fijada en el
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

El proyecto en Python tiene licencia MIT. Juntei conserva su aviso de copyright y de licencia en
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
La implementación de Juntei se ejecuta sin conexión y sin intérprete de Python. Su tratamiento de
las recopilaciones, sus informes de cobertura y su interpretación conservadora de las evidencias
incompletas se documentan en la [guía del comando](../commands/persistence-hunter.md).

## PersistenceSniper: la herramienta original en PowerShell { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) es el proyecto original en
PowerShell para la búsqueda de persistencia que inspiró PyrsistenceSniper. PyrsistenceSniper
reconoce a Federico Lagrasta; el repositorio original también reconoce a Riccardo Ancarani y a
otros colaboradores. Agradecemos al proyecto original y a sus colaboradores que hayan facilitado la
investigación de los mecanismos de persistencia de Windows.

La referencia directa para el porte de Juntei es el proyecto en Python. La herramienta original en
PowerShell se reconoce por ese linaje. Juntei también implementa de forma independiente los
mecanismos propuestos por [hongson11698](https://github.com/hongson11698) en el proyecto original:
la [PR #31 de resolución COM de tareas programadas](https://github.com/last-byte/PersistenceSniper/pull/31)
y la [PR #32 de bibliotecas y carpetas CLSID del menú Inicio](https://github.com/last-byte/PersistenceSniper/pull/32).
Estos mecanismos amplían el catálogo de referencia en Python; ello no implica que el comando sin conexión de Juntei implemente todas
las funcionalidades para sistemas en vivo de cualquiera de los dos proyectos originales.

## Investigación y herramientas de apoyo { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) proporciona los identificadores de técnicas incluidos en
  el catálogo. La asignación de una técnica describe un posible uso de un mecanismo; un valor del
  Registro o un archivo coincidente no constituye, por sí solo, un ataque confirmado según ATT&CK.
- PyrsistenceSniper también reconoce
  [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  como fuente de inspiración para examinar los inicios automáticos de Windows.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) documenta binarios legítimos de Windows de los
  que se puede abusar. Reconocer uno de estos binarios ayuda a interpretar un comando, pero no
  convierte automáticamente ese comando en malicioso.

Para conocer las bases de los analizadores del Registro, NTFS, Shell Link, registros de eventos y
otros que se usan en todo Juntei, consulte los
[agradecimientos generales del proyecto](../acknowledgements.md).

Volver a la [guía de mecanismos de persistencia](index.md).
