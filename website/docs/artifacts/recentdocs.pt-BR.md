# RecentDocs { #recentdocs }

## O que é o RecentDocs { #what-recentdocs-is }

O Windows Explorer armazena listas de arquivos e pastas usados mais recentemente no `NTUSER.DAT` de cada usuário.
A lista raiz abrange itens recentes de todos os tipos; as subchaves agrupam extensões como `.docx`, além de uma
lista `Folder`. `MRUListEx` especifica a ordem dos valores numerados.

O RecentDocs é útil para identificar documentos com os quais um usuário interagiu, incluindo arquivos que foram
excluídos ou movidos posteriormente. Os valores também podem conter o nome do atalho que o Windows usou na pasta Recent
do usuário, o que permite a correlação com evidências de LNK.

<span class="evidence-strength">Melhor conclusão:</span> o Explorer registrou o item na lista RecentDocs
desse usuário. O horário da chave data apenas o item mais recente daquela lista específica.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Colete o `NTUSER.DAT` e seus logs de transação de cada perfil relevante, além do diretório de LNK Recent
do usuário.

## Cuidados para o analista { #analyst-cautions }

- `MRUListEx` fornece a ordem, não um horário para cada item.
- O horário de última gravação da chave corresponde à atualização mais recente; somente a posição MRU 0 recebe um
  `LastOpenedUTC` defensável derivado dele.
- Um item pode aparecer na lista raiz e em uma lista de extensão. Não conte duas vezes como duas aberturas.
- O comportamento dos aplicativos e do Explorer pode variar; alguns métodos de acesso podem não atualizar o RecentDocs.
- Ferramentas de limpeza/privacidade e os limites de tamanho da lista removem evidências mais antigas.
- Um nome de arquivo pode ser controlado pelo atacante e não é prova do tipo de conteúdo.
- A inferência do usuário vem dos caminhos de perfil ou dos dados de pastas do shell; revise nomes de usuário vazios/ambíguos.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host` | Host derivado do contexto da coleta/do sistema, quando disponível. |
| `User` | Dono do perfil inferido a partir do caminho ou dos caminhos de pastas do shell; preserve a incerteza quando vazio. |
| `SourcePath` | `NTUSER.DAT` de entrada. Essencial ao combinar usuários. |
| `Extension` | Sublista (`.docx`, `Folder` etc.); nulo para a lista RecentDocs raiz. |
| `MRU-Position` | Zero é o mais recente nesta lista. É uma ordem relativa, não tempo decorrido. |
| `ValueIndex` | Valor numerado do Registro referenciado por `MRUListEx`. Apenas pivô estrutural. |
| `Filename` | Nome do item decodificado. Correlacione o caminho completo por meio de LNK, Jump List, MFT ou dados de aplicativos. |
| `LinkName` | Nome do atalho codificado no item do shell, frequentemente encontrado no diretório Recent do usuário. |
| `KeyLastWrittenUTC` | Horário da última alteração da chave da lista. Aplica-se à lista, não a cada linha. |
| `LastOpenedUTC` | Preenchido somente para a posição MRU 0, em que o horário da chave sustenta essa interpretação. |
| `ValueDataRaw` | Bytes brutos quando a decodificação falha ou a evidência precisa de validação. |
| `TransactionLogStatus`, `LogsApplied` | Se os logs do Registro foram reproduzidos e quais. |
| `Problems` | Valores MRU ausentes, valores não listados, tipos incorretos ou danos na decodificação. |

## Estratégia de investigação { #investigation-strategy }

1. Separe as listas raiz e de extensão de cada usuário; desduplique o mesmo item com cuidado.
2. Use a posição MRU para ordenação e somente `LastOpenedUTC` para uma afirmação com horário.
3. Associe `LinkName`/`Filename` aos dados de LNK Recent, Jump List, `$MFT` e USN.
4. Revise nomes chamariz, extensões duplas, destinos de rede/removíveis e documentos próximos ao acesso inicial.
5. Correlacione com o Prefetch/SRUM do aplicativo e os logs de eventos quando a execução for relevante.

## Mais detalhes { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [RECmd e Registry Explorer de Eric Zimmerman](https://github.com/EricZimmerman/RECmd)
- [Pôster Windows Forensic Analysis da SANS](https://www.sans.org/posters/windows-forensic-analysis/)
