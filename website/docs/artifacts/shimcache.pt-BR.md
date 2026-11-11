# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## O que é o ShimCache { #what-shimcache-is }

O subsistema de Compatibilidade de Aplicativos do Windows armazena em cache caminhos e metadados de executáveis enquanto avalia
o comportamento de compatibilidade/shim. O cache sobrevive a arquivos renomeados, removidos ou desconectados junto com
um volume removível. Seu layout e os campos disponíveis variam do Windows XP ao Windows 11.

O ShimCache é valioso para descobrir caminhos históricos e, quando o formato tem uma flag de execução
confiável, como apoio adicional à execução. O horário em cache normalmente é o **horário da última modificação do arquivo**,
não o momento em que o Windows o viu ou executou.

<span class="evidence-strength">Melhor conclusão:</span> o Windows observou o caminho para o processamento de
compatibilidade. Uma entrada, por si só, é evidência de presença/observação; use `Executed` somente onde for suportado.

## Localização e coleta { #location-and-collection }

Os bytes estão no hive do Registro `SYSTEM`, normalmente em:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Colete `SYSTEM`, `SYSTEM.LOG1` e `SYSTEM.LOG2`. O Juntei lê todos os conjuntos de controle e identifica
`Select\Current` quando possível.

!!! warning "As entradas mais recentes do sistema em execução podem não estar no hive"
    O Windows pode manter atualizações do ShimCache em memória e gravá-las durante o desligamento/a reinicialização. Um hive copiado
    de um sistema em execução pode não conter as observações mais recentes. O horário de última gravação do cache no Registro
    ajuda a descrever quando ele foi salvo, mas não é um horário de execução por entrada.

## Cuidados para o analista { #analyst-cautions }

- Uma entrada do cache, por si só, **não é prova de execução**.
- `Executed` pode ser `Yes`, `No` ou `NA`; a semântica disponível depende do formato/da versão.
- `LastModifiedTimeUTC` pertence ao arquivo referenciado, não à observação ou à execução.
- A ordem é útil, mas não é uma linha do tempo precisa universal; ocorrem duplicatas entre conjuntos de controle/slack.
- Entradas `FromSlack` foram recuperadas fora do cache ativo e exigem cautela adicional.
- Um `SYSTEM` sujo (dirty) sem logs utilizáveis pode omitir registros atuais. Leia o resumo da recuperação.
- Os achados indicam padrões suspeitos de nomes/caminhos, não vereditos de malware.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `ControlSet` | Conjunto de controle numérico que contém o cache. Compare `IsCurrent` antes de escolher um. |
| `CacheEntryPosition` | Posição/ordem dentro do cache decodificado. Útil para ordenação relativa, não é um horário. |
| `Path` | Caminho do executável/pacote em cache. Principal pivô de busca. |
| `LastModifiedTimeUTC` | Horário de modificação do arquivo armazenado na entrada — não o horário de execução/observação. |
| `Executed` | `Yes`, `No` ou `NA`, de acordo com as flags de formato suportadas. Use somente com o contexto da versão. |
| `Duplicate` | A mesma entrada já foi vista em um conjunto de controle ou slack anterior; ajuda a evitar contagem dupla. |
| `SourceFile` | Hive/valor bruto de entrada. Essencial entre hosts. |
| `FromSlack` | Entrada recuperada do slack do valor do cache, e não da lista ativa. Menor confiança e possivelmente mais antiga. |
| `CacheFormat` | Layout do Windows/do cache decodificado. Determina a disponibilidade e a semântica dos campos. |
| `CacheLastWrittenUTC` | Horário de gravação da chave do Registro/do cache. É um contexto compartilhado, não o horário de execução de cada entrada. |
| `InsertFlags`, `ShimFlags` | Flags de compatibilidade brutas; `Executed` é o resultado interpretado pelo Juntei, onde suportado. |
| `DataSize` | Tamanho dos dados desta entrada, útil para validação estrutural. |
| `Machine` | Arquitetura registrada pelos formatos mais novos (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Metadados em cache específicos do Windows XP, quando presentes. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Identidade de aplicativos empacotados nas entradas suportadas. O ID do editor é contexto, não prova de assinatura. |
| `Findings` | Códigos de anomalia do Juntei, como binário de sistema realocado ou caminho de preparação suspeito. Apenas pistas. |
| `RegistryPath`, `RegistryValueName` | Origem exata da chave/valor no hive. Preserve para validação. |
| `CacheValueSize` | Tamanho do valor inteiro do Registro, não o `DataSize` da entrada individual. |
| `IsCurrent` | Se o conjunto de controle desta entrada é igual a `SYSTEM\Select\Current`; vazio se isso não for conhecido com segurança. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Números brutos dos conjuntos de controle de `SYSTEM\Select`, para contexto e análise de recuperação. |

JSON/JSONL preserva adicionalmente os dados da entrada, tags, sub-registros de aplicativos empacotados, achados estruturados,
problemas do cache e detalhes da recuperação.

## Estratégia de investigação { #investigation-strategy }

1. Prefira o conjunto de controle atual, mas mantenha os outros conjuntos e o slack como evidência histórica.
2. Pesquise caminhos e achados; não use `LastModifiedTimeUTC` como horário de execução.
3. Onde `Executed == Yes`, corrobore com Prefetch, SRUM, logs de eventos ou outros vestígios de execução.
4. Associe os caminhos ao `$MFT`/USN para determinar o histórico de criação, exclusão e metadados.
5. Relate se os logs de transação foram reproduzidos e se o cache em execução pode não ter sido gravado em disco.

## Mais detalhes { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [shimcache-forensic da Security Ronin](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Especificação do formato do Registro do Windows de Maxim Suhanov](https://github.com/msuhanov/regf)
