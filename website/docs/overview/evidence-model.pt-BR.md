# Evidências e interpretação { #evidence-and-interpretation }

A forense digital não se resume a ler campos. Trata-se de decidir qual observação o campo sustenta,
quão forte é esse respaldo e o que poderia produzir a mesma observação de forma inocente.

## Quatro categorias úteis de evidências { #four-useful-evidence-categories }

| Categoria | Significado | Exemplos |
|---|---|---|
| Presença | Um caminho ou arquivo era conhecido pelo Windows | Registro de arquivo do Amcache, entrada do ShimCache, registro do `$MFT` |
| Execução | O Windows registrou a inicialização de um programa ou um estado de execução | Horário de execução do Prefetch; flag de execução do ShimCache, quando compatível |
| Interação do usuário | Um usuário ou aplicativo referenciou/abriu um item | RecentDocs, Jump List, LNK da pasta Recent |
| Atividade do sistema | O Windows registrou uso de recursos ou configuração | Contadores do SRUM, configuração de serviços, ação do TaskCache |

Um artefato pode sustentar mais de uma categoria, mas nunca o promova automaticamente. Por exemplo,
uma entrada do Amcache com um caminho suspeito sustenta **presença**. Um horário de execução correspondente no Prefetch acrescenta
uma evidência mais forte de **execução**. Um registro de criação no USN e o caminho no `$MFT` podem situar o arquivo no disco, e
a quarentena do Defender pode explicar por que ele desapareceu.

## Carimbos de data/hora são afirmações, não verdades universais { #timestamps-are-statements-not-universal-truth }

Pergunte o que causou cada carimbo de data/hora:

- Os horários de `$STANDARD_INFORMATION` e `$FILE_NAME` descrevem metadados NTFS e podem divergir.
- Um horário de execução do Prefetch descreve uma inicialização registrada, mas apenas os horários de execução recentes permanecem.
- Um horário do ShimCache costuma ser o horário da última modificação do arquivo em cache — não o horário de execução.
- O horário da última gravação de uma chave do Registro se aplica à chave como um todo, não a um valor individual.
- O RecentDocs atribui o horário da chave ao item mais recente; itens mais antigos têm ordem, mas geralmente não têm horário de abertura.
- Os carimbos de data/hora do SRUM frequentemente identificam um intervalo de agregação, e não a operação de rede exata.

Use UTC durante a análise, mantenha o valor original quando disponível e documente qualquer conversão.

## Evidências positivas e negativas { #positive-and-negative-evidence }

A presença de um registro bem compreendido pode ser uma forte evidência positiva. A ausência geralmente é
mais fraca, pois os artefatos são sobrescritos, os caches têm limite de tamanho, a coleta pode estar incompleta, as configurações variam
e existem ferramentas de limpeza. Diga “não observado no artefato coletado” em vez de “não aconteceu”.

## Escada de corroboração { #corroboration-ladder }

1. Valide o arquivo de origem e o caminho de coleta.
2. Confirme que o analisador não relatou danos relevantes nem logs de transação não aplicados.
3. Interprete o campo de acordo com a página do artefato.
4. Encontre um artefato independente que sustente a mesma ação.
5. Verifique logs de eventos, telemetria de rede, memória ou conteúdo de arquivos, quando disponíveis.
6. Registre a versão exata do Juntei, o comando, os filtros e o hash da origem.

!!! tip "Escreva conclusões na medida da força da evidência"
    Prefira “o Prefetch registra `tool.exe` em execução às 10:02:11Z” a “o atacante executou o
    malware exatamente às 10:02:11Z”. A segunda afirmação acrescenta identidade e intenção que o Prefetch
    sozinho não consegue estabelecer.
