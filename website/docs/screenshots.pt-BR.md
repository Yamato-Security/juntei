# Capturas de tela { #screenshots }

Explore a linha de comando do Juntei, a análise automática e os resultados de investigação organizados.
Clique em uma captura de tela para vê-la em tamanho real.

## 1. Inicialização e comandos { #1-startup-and-commands }

<figure markdown>

[![Tela de inicialização do Juntei mostrando a release CODE BLUE e os comandos disponíveis](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>A tela de inicialização do Juntei lista a análise automática, os analisadores específicos por fonte e o Persistence Hunter.</figcaption>

</figure>

## 2. Análise automática em menos de 20 segundos { #2-automatic-parsing-in-under-20-seconds }

Em um **MacBook Pro M4**, a execução mostrada abaixo analisou **8,7 GiB de dados coletados do Windows**
de **seis sistemas** e salvou **4,4 GiB de saída em menos de 20 segundos**.
O comando [auto-parse](commands/auto-parse.md) executou 240 tarefas de análise usando 16 processos
paralelos e produziu 482 arquivos de resultado. Foram **210 tarefas bem-sucedidas, 30 ignoradas
porque os artefatos não foram coletados, e nenhuma falha**.

<figure markdown>

[![Juntei auto-parse concluindo seis sistemas, 8,7 GiB de entrada e 4,4 GiB de saída em menos de 20 segundos](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Descoberta automática de hosts, análise em paralelo e um resumo de conclusão mostrando o tamanho da entrada, o tamanho da saída, os resultados das tarefas e o tempo decorrido.</figcaption>

</figure>

## 3. Resultados organizados para investigação { #3-results-organized-for-investigation }

<figure markdown>

[![Pastas de saída de um sistema agrupadas em categorias DFIR, com CSVs de contas e seus logs do analisador](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Os resultados de cada computador são agrupados por categoria DFIR. Os CSVs de artefatos e seus arquivos <code>-logs.txt</code> correspondentes ficam juntos, facilitando encontrar tanto os registros quanto os detalhes da análise.</figcaption>

</figure>
