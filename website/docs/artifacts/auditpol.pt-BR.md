# Política de auditoria { #audit-policy }

## O que é { #what-it-is }

A política de auditoria define quais eventos de segurança o Windows registra no log de eventos Security. Ela é definida
por subcategoria no Windows Vista e posteriores (59 no Windows 10 e 11), e por categoria no
Windows XP e Server 2003. Cada uma registra sucessos, falhas, ambos ou nada.

A LSA mantém a política em vigor no hive `SECURITY`, quer ela tenha vindo do `auditpol`, da política de segurança
local ou da Política de Grupo.

<span class="evidence-strength">Melhor conclusão:</span> o sistema estava configurado para registrar esses eventos (ou
não) quando o hive foi gravado pela última vez. Quando uma subcategoria não é auditada, a ausência de um evento não diz
nada sobre se a atividade aconteceu.

## Localização { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

O Windows Vista e posteriores mantêm um cabeçalho de 12 bytes, depois uma configuração por subcategoria, categoria por
categoria, e então quantas subcategorias cada categoria tem. As versões diferem apenas nessas contagens:
cada uma acrescenta subcategorias ao final de uma categoria (56 nos hives do Windows 8.x e Server 2012 R2
lidos, 59 no Windows 10, 11 e Server 2022). O Windows XP e o Server 2003 mantêm se a auditoria está
ativada (um byte, seguido de preenchimento que o Windows não limpa) e nove configurações de categoria. Esse layout foi
verificado na imagem pública do Windows XP do Hacking Case do CFReDS do NIST, que não audita nada; os
valores de suas configurações de categoria seguem a documentação da Microsoft.

Colete o `SECURITY` com seus logs de transação, e o hive `SYSTEM` do sistema, que nomeia o
host.

## Cuidados para o analista { #analyst-cautions }

- **Dados desconhecidos permanecem desconhecidos.** Revisões não compatíveis são preservadas com diagnósticos. Flags de
  configuração desconhecidas deixam `Success` e `Failure` vazios e são excluídas das contagens de configurações auditadas.
  Uma tabela de contagens danificada não pode transformar a palavra reservada em outra configuração de auditoria.
  As capturas podem conter posições adicionais além do catálogo de nomes/GUIDs. Suas configurações e o
  `SubcategoryId` numérico são mantidos com nomes/GUIDs vazios; uma posição não mapeada não é, por si só,
  evidência de corrupção.


- **A ordem é a do próprio Windows.** Dentro de uma categoria, as configurações seguem a ordem dos
  IDs de mensagem das subcategorias, não de seus GUIDs. Special Logon (`%%12548`, índice
  4 de Logon/Logoff) vem antes de IPsec Quick Mode. Ferramentas que as leem na ordem dos GUIDs relatam o
  padrão do Windows 10 como Special Logon "No Auditing" e IPsec Quick Mode "Success". `SubcategoryId` é o
  mesmo ID que o evento 4719 (System audit policy was changed) informa, de modo que os dois podem ser correlacionados.
- **A política em vigor, não sua origem.** A Política de Grupo e a política local gravam o mesmo valor.
  O `audit.csv` da Política de Grupo (no SYSVOL e em `C:\Windows\System32\GroupPolicy`) mostra o que
  deveria ser aplicado (`Machine\Microsoft\Windows NT\Audit\audit.csv` de uma política).
- **Alterações ao longo do tempo:** `KeyLastWrittenUTC` indica aproximadamente quando a política foi alterada pela última vez. O evento 4719
  registra cada alteração (desde que o log Security a mantenha). Um atacante pode desativar a auditoria
  e depois reativá-la.
- **Subcategorias versus categorias.** No Windows Vista e posteriores, as subcategorias são o que
  o Windows usa (a menos que a opção de segurança `SCENoApplyLegacyAuditPolicy` esteja desativada e uma política
  de categoria legada as substitua).
- **Windows XP e 2003:** com a auditoria desativada (`AuditingEnabled` False), nenhuma categoria é auditada,
  qualquer que seja sua configuração.
- **Eventos:** `Events` lista os principais eventos de cada subcategoria, segundo a documentação da Microsoft
  sobre as configurações avançadas da política de auditoria. São os que uma investigação normalmente procura, não
  todos os eventos que a subcategoria registra.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Para cada hive, o resumo informa seu layout, quantas configurações são auditadas e quando a política
foi gravada pela última vez. Ele lista o que é auditado para sucesso e falha, somente sucesso e somente
falha, e depois as subcategorias que não são auditadas, com seus principais eventos: esses eventos estão
desativados na política capturada; isso não estabelece qual era a política durante atividades anteriores.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Host (a partir do hive `SYSTEM` do sistema) e o hive `SECURITY`. |
| `Layout` | `subcategories` (Windows Vista e posteriores) ou `categories` (XP e 2003); vazio quando o valor não é decodificado. |
| `Category`, `Subcategory` | Como o `auditpol` as nomeia. |
| `SubcategoryGUID`, `SubcategoryID` | O GUID da subcategoria e seu ID de mensagem conforme o evento 4719 o informa (`%%12548`). |
| `Setting` | `none`, `success`, `failure` ou `success_and_failure`. |
| `Success`, `Failure` | Se sucessos e falhas são registrados. |
| `AuditingEnabled` | Windows XP e 2003: se a auditoria está ativada. |
| `Events` | Os principais IDs de evento da subcategoria. |
| `KeyLastWrittenUTC` | A última gravação de `PolAdtEv`: aproximadamente quando a política foi alterada pela última vez. |
| `DataRaw` | Os bytes do valor (hex), quando ele não é decodificado por inteiro. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Um layout ou tamanho desconhecido, contagens que não fecham e valores acima de 3. |

## Estratégia de investigação { #investigation-strategy }

1. Antes de concluir que algo não aconteceu, verifique se sua subcategoria era auditada:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (tarefas agendadas, 4698), Security System Extension (instalações de serviços, 4697), User Account and
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. Compare a política com o log Security: eventos de subcategorias que agora não são auditadas
   indicam que a política mudou. Procure o evento 4719 e um `KeyLastWrittenUTC` recente.
3. Compare com o `audit.csv` da Política de Grupo para distinguir uma política de todo o domínio de uma alteração local.
4. Use `parse-registry read --recover-deleted` e os logs de transação para obter versões anteriores de
   `PolAdtEv`.

## Mais detalhes { #further-details }

- [Microsoft: Configurações avançadas da política de auditoria de segurança](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Constantes de auditoria (GUIDs de subcategorias)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
