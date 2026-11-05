# Descritores de segurança (`$Secure:$SDS`) { #security-descriptors-securesds }

## O que é o `$SDS` { #what-sds-is }

Todo arquivo e diretório no NTFS tem um **descritor de segurança**: seu proprietário, seu grupo primário, uma
**DACL** (quem pode fazer o quê com ele) e uma **SACL** (quais acessos são auditados e seu rótulo de integridade).
O NTFS mantém cada descritor distinto uma única vez, no fluxo `$SDS` do arquivo `$Secure` (entrada 9 da MFT),
e lhe atribui um **ID de segurança**. O `$STANDARD_INFORMATION` de um arquivo registra apenas esse ID de segurança: a
coluna `SID` de [`parse-ntfs mft`](mft.md). O `$SDS` o converte em um proprietário e ACLs.

O `$SDS` é organizado em blocos de 256 KiB, cada um seguido de uma cópia de si mesmo (seu espelho). Cada entrada tem um hash do
seu descritor, que o NTFS usa para encontrar um descritor já existente (`$SDH`), e o
ID de segurança (indexado pelo `$SII`).

<span class="evidence-strength">Melhor conclusão:</span> os arquivos cujo `SID` nomeia um
descritor tinham, quando seu registro no `$MFT` foi gravado pela última vez, esse proprietário e essas ACLs. O proprietário
de um arquivo geralmente é a conta que o criou, mas um administrador, um instalador ou um programa
que assuma a propriedade pode alterá-lo.

## Localização e coleta { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Colete-o com uma ferramenta forense capaz de acesso bruto; as coletas normalmente o nomeiam `$Secure_$SDS`, pois `:`
não pode fazer parte de um nome de arquivo. Colete junto o `$MFT` do mesmo volume.

## Cuidados para o analista { #analyst-cautions }

- Um proprietário é um SID. Somente SIDs conhecidos (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) e as contas internas de uma máquina ou domínio (`-500`
  `Administrator`, …) são nomeados. Resolva os demais com o `SAM` do host
  ([contas SAM](sam.md)), sua lista de perfis ([Informações do sistema](system-info.md)) ou o domínio.
- Um descritor é compartilhado por todos os arquivos com o mesmo proprietário e as mesmas ACLs e geralmente permanece no `$SDS`
  depois que esses arquivos são excluídos. Um descritor sem arquivos no `$MFT` não é, por si só, um
  arquivo excluído.
- Uma ACE não é um histórico de acessos. A DACL diz quem *pode* acessar um arquivo; os logs de eventos e outros
  artefatos dizem quem acessou. As entradas de auditoria da SACL descrevem a configuração, não o que foi registrado.
- As ACEs são listadas na ordem em que aparecem na ACL. O Juntei não calcula o acesso efetivo: entradas de negação,
  herança, associação a grupos, privilégios e os direitos implícitos do proprietário são todos relevantes.
- `DACL` e `SACL` escrevem cada ACE como o `icacls` faz. `SDDL` representa as permissões suportadas, com máscaras em hexadecimal e contas como SIDs. Não é uma cópia byte a byte do descritor binário; texto condicional não representável o deixa vazio e é diagnosticado.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

O resumo conta os descritores e lista seus proprietários, com quantos descritores cada um possui,
os descritores com DACL NULL (nenhuma restrição) e quaisquer danos.

## Campos de saída CSV { #csv-output-fields }

| Campo | Interpretação |
|---|---|
| `SourcePath` | Arquivo de entrada. |
| `SecurityID` | O ID de segurança: faça a junção com o `SID` de [`parse-ntfs mft`](mft.md) do mesmo volume. |
| `Hash` | O hash registrado com o descritor. |
| `EntryOffset` | Onde a entrada está no fluxo. |
| `FromMirror` | `True` para uma cópia lida do bloco espelho: uma que difere de sua entrada (listada depois dela) ou a cópia de uma entrada danificada. |
| `Size` | O tamanho da entrada, incluindo seu cabeçalho de 20 bytes. |
| `Owner`, `OwnerName` | O SID do proprietário e seu nome, se for conhecido. |
| `Group`, `GroupName` | O SID do grupo primário e seu nome, se for conhecido. O Windows não o usa em verificações de acesso. |
| `Control` | As flags de controle do descritor: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (herança bloqueada), `SE_DACL_AUTO_INHERITED`, … unidas por `|`. |
| `DACL-ACE-Count`, `DACL` | As entradas da DACL, separadas por `; `. Vazio se não houver DACL; `NULL DACL: no restriction` se o descritor conceder tudo a todos. |
| `SACL-ACE-Count`, `SACL` | As entradas da SACL: entradas de auditoria, o rótulo de integridade obrigatória, rótulos de confiança de processo e atributos de recurso. |
| `SDDL` | O descritor inteiro em SDDL. Vazio somente se algo nele não for decodificado. |
| `Problems` | Um hash que não corresponde ao descritor, partes que não podem ser decodificadas, ACEs de tipo desconhecido. |

Cada entrada de `DACL` e `SACL` é escrita como o `icacls` a escreve: a conta, o tipo, se não for
uma entrada de permissão, as flags de herança e os direitos.

| Parte | Significado |
|---|---|
| Conta | O nome conhecido ou o SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | O tipo da ACE, quando não é uma entrada de permissão. Um rótulo obrigatório não tem tipo exibido. |
| `(I)` | Herdada do diretório pai. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Herdada por arquivos (herança de objeto), por diretórios (herança de contêiner), somente por eles e não por este objeto (somente herança) e apenas um nível abaixo (sem propagação). |
| `(SA)`, `(FA)` | Uma entrada de auditoria para acesso bem-sucedido ou com falha. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Controle total, modificação, leitura e execução, leitura, gravação. |
| `(RD,WD,AD,…)` | Direitos específicos, nos demais casos: ler dados, gravar dados, acrescentar dados, ler e gravar atributos estendidos (`REA`, `WEA`), executar (`X`), excluir filho (`DC`), ler e gravar atributos (`RA`, `WA`), excluir (`DE`), ler controle (`RC`), gravar DAC (`WDAC`), gravar proprietário (`WO`), sincronizar (`S`) e os direitos genéricos `GA`, `GR`, `GW`, `GE`. `N` significa nenhum acesso. |
| `(NW)`, `(NR)`, `(NX)` | A política de um rótulo obrigatório: sem gravação ascendente, sem leitura ascendente, sem execução ascendente. |
| `if (…)` | A condição de uma ACE de callback, como `(WIN://SYSAPPID Contains "…")`: a entrada se aplica somente aos processos para os quais ela é verdadeira. O Windows as usa para conceder acesso a pacotes de contêiner de aplicativo. |

Os rótulos de confiança de processo e os atributos de recurso mostram sua máscara em hexadecimal, e um atributo
de recurso mostra sua declaração (`("IMAGELOAD",TU,0x0,1)`).

JSON/JSONL têm os mesmos campos e nomes, com números como números, `null` para valores vazios e
`Problems` como uma lista.

## Estratégia de investigação { #investigation-strategy }

1. Analise o `$MFT` do mesmo volume e faça a junção do seu `SID` com o desta saída: cada arquivo recebe seu
   proprietário e suas ACLs.
2. Examine os proprietários que são contas de usuário (`S-1-5-21-…`): quais arquivos uma conta suspeita
   possui e quais contas possuem arquivos em diretórios do sistema.
3. Procure ACLs permissivas em diretórios a partir dos quais programas são executados ou carregados: `Everyone`,
   `Authenticated Users` ou `User` com `(F)`, `(M)` ou `(W)`, e DACLs NULL.
4. Um descritor usado por apenas um arquivo, ou por um conjunto incomum de arquivos, pode revelar arquivos que foram
   criados ou alterados fora de uma instalação normal.
5. Corrobore alterações de propriedade e de permissões com entradas do log de eventos Security (4670, 4663 com uma
   SACL) e os registros `SECURITY_CHANGE` do [Diário USN](usn-journal.md).

## Mais detalhes { #further-details }

- [Microsoft: descritores de segurança](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (descritores de segurança, ACEs, expressões condicionais)
- [Microsoft: formato de string do descritor de segurança (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: documentação do formato NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (seu crate `secure`)

Com saída estruturada no stdout (`-t` sem `-o`), os avisos de recuperação no nível do fluxo vão para o stderr, de modo que os pipes mantêm CSV/JSON/JSONL legíveis sem ocultar regiões danificadas ou recuperadas.
