# Contas locais SAM { #sam-local-accounts }

## O que o SAM contém { #what-the-sam-contains }

O hive do Security Account Manager (`SAM`) armazena os registros das contas locais. O Juntei decodifica os
nomes das contas, RIDs, campos de perfil/comentário, flags de controle da conta, carimbos de data/hora de último logon/senha, contadores,
dicas de senha, vínculo com conta Microsoft, caminhos de perfil e se o Windows oculta a conta
da interface de entrada.

Isso é útil para encontrar administradores locais recém-criados, contas inativas que foram habilitadas, alterações de
senha, contas ocultas e contas preparadas para movimentação lateral.

<span class="evidence-strength">Melhor conclusão:</span> o SAM local representava a conta com
o estado registrado. Ele não é um histórico completo de autenticação e não descreve contas de domínio.

## Localização e coleta { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Colete o hive e os logs com privilégios/acesso bruto suficientes. A identidade do host pode ser enriquecida a partir
do hive `SYSTEM` correspondente na coleta.

## Cuidados para o analista { #analyst-cautions }

- Este comando analisa contas **locais**, não usuários de domínio do Active Directory.
- O Juntei, intencionalmente, não extrai hashes de senha.
- O RID 500 é a conta interna Administrator, mesmo que tenha sido renomeada; o nome, sozinho, não basta.
- `LastLogon` e os contadores são estado do Registro, não uma trilha de auditoria completa por sessão.
- `HiddenFromLogonUI` oculta uma conta da tela de entrada; não desabilita a conta.
- As contas internas `DefaultAccount` e `WDAGUtilityAccount` podem estar ocultas legitimamente.
- Flags de conta nulos, devido a um valor `F` danificado/ausente, não devem ser interpretados como conta habilitada.
- `NamesKeyLastWrittenUTC` dá uma aproximação do contexto de registro do nome da conta, mas não é um horário de criação garantido.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Contexto do host e o hive SAM exato. |
| `RID` | Identificador relativo. Forte ponto de pivô para a identidade da conta; 500 é a conta interna Administrator. |
| `User` | Nome atual da conta local. Pode ser renomeado; correlacione pelo RID. |
| `FullName`, `Comment`, `UserComment` | Texto descritivo; controlável pelo atacante e útil para analisar disfarces (masquerading). |
| `AccountFlags` | Estados decodificados, como desabilitada, bloqueada, senha não exigida, conta normal, senha nunca expira. |
| `AccountFlagsValue` | Flags numéricos brutos para validação e para bits não suportados. |
| `CountryCode`, `CodePage` | Metadados de localidade/página de código da conta; geralmente contexto, não atribuição. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | O mesmo estado de último logon, com precisão total e em segundos inteiros para compatibilidade. Não é um histórico completo. |
| `PasswordLastSetUTC` | Horário registrado da alteração de senha. Investigue valores inesperadamente recentes/antigos. |
| `AccountExpirationUTC` | Expiração configurada, quando finita. Nulo pode significar nunca/não definida. |
| `LastFailedLogonUTC` | Último logon local com falha registrado no estado do SAM; confirme com os eventos de Security. |
| `FailedLogonCount`, `LogonCount` | Contadores cumulativos/de estado com comportamento de atualização específico do Windows; use como contexto, não como contagem exata de eventos. |
| `PasswordHint` | Dica sensível fornecida pelo usuário; pode expor padrões de senha. Trate-a adequadamente. |
| `InternetUserName` | Identidade da conta Microsoft vinculada, quando presente. |
| `HiddenFromLogonUI` | Estado de ocultação na interface. Analise RIDs altos/contas personalizadas inesperadas; não significa desabilitada. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Configuração de ambiente/logon da conta local; scripts ou caminhos UNC incomuns podem indicar persistência. |
| `KeyLastWrittenUTC` | Horário de alteração da chave da conta (RID). Em nível de chave, não específico de um campo. |
| `NamesKeyLastWrittenUTC` | Horário de alteração da chave de mapeamento do nome de usuário; aproximação útil do ciclo de vida da conta. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Valores FILETIME brutos de 64 bits mantidos para validação e para o tratamento de valores sentinela especiais. |
| `TransactionLogStatus`, `LogsApplied` | Contexto da reprodução do Registro. |
| `Problems` | Divergência entre F/V/Names, entradas ausentes, deslocamentos malformados ou danos na decodificação. |

## Estratégia de investigação { #investigation-strategy }

1. Compare RID, nome, flags, caminho do perfil e `NamesKeyLastWrittenUTC` com a linha de base do host.
2. Analise contas habilitadas com RID 1000 ou maior, com senha não exigida, que nunca expiram, ocultas ou alteradas recentemente.
3. Correlacione com os eventos 4720/4722/4724/4732 de Security, eventos de logon, criação de perfis e atividade remota.
4. Pesquise o histórico de recuperação do Registro se uma conta foi removida antes da coleta.
5. Diferencie contas legítimas de provisionamento/padrão de contas interativas criadas por atacantes.

## Mais detalhes { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Plugin SAM do dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
