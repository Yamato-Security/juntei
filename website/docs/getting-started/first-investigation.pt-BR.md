# Primeira investigação { #first-investigation }

Este fluxo de trabalho pressupõe que você recebeu uma coleta de triagem offline em `evidence/`.

## 1. Preserve a origem { #1-preserve-the-source }

Trabalhe a partir de uma cópia verificada. Registre os hashes, o horário da coleta, o fuso horário, o host, o volume e a ferramenta
de coleta. O Juntei lê as evidências, mas opções como a extração de fluxos residentes ou de quarentena criam
novos arquivos de saída; direcione-os para um diretório de análise separado.

## 2. Faça o inventário do que você tem { #2-inventory-what-you-have }

Procure os caminhos indicados em [Coleta de evidências](collection.md). A ausência de artefatos é normal. Não
presuma que uma coleta contém logs de transação só porque contém um hive do Registro.

## 3. Execute os resumos primeiro { #3-run-summaries-first }

Muitos comandos imprimem um resumo conciso quando nenhum arquivo de saída é solicitado:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Leia os avisos de danos, de banco de dados sujo e de logs não aplicados antes de confiar nos registros.

Para analisar todos os artefatos compatíveis de uma coleta com vários computadores em uma única operação, use:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Consulte [Análise automática de coletas](../commands/auto-parse.md) para a descoberta de hosts, o paralelismo,
o layout da saída e o JSONL consolidado para ingestão em SIEM.

## 4. Salve a saída estruturada { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Prefira JSONL para scripts e valores aninhados, e CSV para uma planilha ou um visualizador de linha do tempo.

## 5. Parta de um identificador confiável { #5-pivot-from-a-reliable-identifier }

Comece com um nome de arquivo, um caminho completo, SHA-1/SHA-256, o SID de um usuário, uma entrada/sequência da MFT, o GUID de uma tarefa,
o nome de um serviço ou uma janela de tempo. Normalize maiúsculas e minúsculas apenas nos campos que o Windows trata sem distinção entre elas.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. Correlacione { #6-correlate }

Use as [Investigações entre artefatos](../resources/workflows.md). Uma sequência prática é:

1. Prefetch para execução e horários de execução recentes.
2. `$MFT` e Diário USN para criação, caminhos, renomeações e exclusão.
3. Amcache e ShimCache para presença histórica e metadados.
4. LNK, Jump Lists, RecentDocs e SRUM para o contexto do usuário/aplicativo.
5. Serviços do Registro e TaskCache para persistência.
6. Quarentena do Defender para detecção, remoção, origem e hashes.

## 7. Documente a reprodutibilidade { #7-document-reproducibility }

Registre o caminho de origem, o hash da origem, a versão do Juntei, o comando exato, o hash da saída, o fuso horário do sistema
e qualquer filtragem posterior. Nunca edite silenciosamente a saída de evidências gerada.
