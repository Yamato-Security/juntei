# Varredura YARA { #yara-scan }

Analise arquivos coletados com um mecanismo Rust portátil. Windows, macOS e Linux são compatíveis. Os arquivos são lidos como dados e nunca executados. Este comando é independente da análise automática de artefatos.

## Escolher regras { #choose-rules }

Core é o padrão e prioriza precisão e desempenho. Extended amplia a busca. Full oferece a maior abrangência, mas pode gerar mais falsos positivos.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: Use um arquivo ou diretório local de regras UTF-8, sem download. Diretórios são percorridos recursivamente. Inclusões ficam dentro do diretório escolhido e são expandidas uma vez, na ordem de dependência. Regras inválidas ou incompatíveis interrompem a compilação em vez de serem descartadas silenciosamente.

## Limites e escopo { #limits-and-scope }

Todos os arquivos regulares são elegíveis, inclusive conteúdos recuperados da quarentena. Links simbólicos e arquivos especiais são ignorados. Arquivos compactados e imagens de disco não são extraídos; memória em execução não é analisada. Use arquivos extraídos estáveis e um novo diretório de saída fora das evidências.

- `-T, --threads <N>`: Varreduras paralelas; padrão: quantidade de CPUs, limitada a 8.
- `--timeout <SECONDS>`: Tempo máximo de avaliação por arquivo em segundos; padrão: 60.
- `--max-file-mib <MIB>`: Tamanho máximo em MiB; padrão: 64. Arquivos maiores são registrados como ignorados. Mais trabalhadores e buffers maiores aumentam o uso de memória.
- `--rules-release YYYYMMDD`: Data da versão Forge ou latest por padrão. As fontes baixadas e seus hashes são salvos para reprodução.

## Resultados { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Correspondências entre regras e arquivos, com caminhos relativos, nomes dos computadores de origem, hashes, metadados e posições limitadas. Os bytes correspondentes não são exportados.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Cobertura por arquivo, incluindo itens ignorados, erros de leitura, limites de tempo e resultados parciais.
- `Summary.json`: Estado final, configurações, contagens, versão do mecanismo e hashes das fontes.
- `Rules.yar`: Regras baixadas exatas ou uma cópia das regras locais com inclusões resolvidas.
- `CompilationWarnings.txt` / `CompilationError.txt`: Avisos de compilação ou erro quando a compilação falha.

Códigos de saída: 0 significa conclusão, 3 cobertura incompleta, 2 falha de configuração/saída e 130 cancelamento. Uma correspondência é uma pista, não prova de comprometimento; a ausência de correspondências não prova segurança. Avalie o título da regra e o contexto do artefato. Descriptografe a quarentena antes de identificar seu conteúdo.

## Mais detalhes { #further-details }

- [Mecanismo: Boreal](https://github.com/vthib/boreal)
- [Pacotes de regras: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Uso detalhado e referência de campos (inglês)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Uso detalhado e referência de campos (japonês)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
