# Dados de configuração de inicialização (BCD) { #boot-configuration-data-bcd }

## O que é o repositório BCD { #what-the-bcd-store-is }

O Windows Vista e versões posteriores mantêm sua configuração de inicialização em um hive do Registro, o repositório BCD:
`\Boot\BCD` em sistemas BIOS, `\EFI\Microsoft\Boot\BCD` na partição de sistema EFI de sistemas
UEFI. O `bcdedit` e o instalador do Windows o editam; o gerenciador de inicialização o lê a cada inicialização.

O repositório contém objetos, cada um nomeado por um GUID:

- o **gerenciador de inicialização do Windows** (`{bootmgr}`), com a entrada padrão, a ordem de exibição e o
  tempo limite do menu; em sistemas UEFI, também o **gerenciador de inicialização do firmware** (`{fwbootmgr}`) com a
  ordem de inicialização do firmware;
- um **carregador de inicialização do Windows** por instalação do Windows (incluindo o ambiente de recuperação), um
  **aplicativo de retomada** (hibernação) e o **testador de memória**;
- objetos de **configurações herdadas** (`{globalsettings}`, `{bootloadersettings}`, …) cujas configurações os
  demais herdam;
- objetos de **dispositivo** (opções de ramdisk do ambiente de recuperação).

Cada objeto tem elementos: uma descrição, um dispositivo e um caminho, e configurações como
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` e `safeboot`.

Dois tipos de ataque os alteram:

- **Ransomware** desativa a recuperação para que o sistema não possa ser reparado:
  `bcdedit /set {default} recoveryenabled No` e
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Algumas famílias definem `safeboot` para
  que o Windows reinicie no modo de segurança, no qual os produtos de segurança não são executados.
- **Rootkits e drivers não assinados** podem usar `testsigning on` ou `nointegritychecks on` para enfraquecer as verificações durante a inicialização. Essas configurações, por si sós, não provam que um rootkit foi executado.

<span class="evidence-strength">Melhor conclusão:</span> no momento da coleta, esta entrada de inicialização
tinha esta configuração. O horário da última gravação da chave do elemento é quando a configuração foi gravada pela última vez.

## Localização { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

O tipo de um elemento tem 8 dígitos hexadecimais: sua classe (`1` todos os aplicativos, `2` um tipo de
aplicativo, `3` dispositivos), seu formato (`1` dispositivo, `2` cadeia de caracteres, `3` objeto, `4` lista de objetos, `5`
inteiro, `6` booleano, `7` lista de inteiros) e seu número. O comando nomeia os elementos como
a Microsoft os documenta (`BcdLibraryBoolean_AutoRecoveryEnabled`).

As ferramentas de coleta muitas vezes não copiam o repositório: a partição de sistema EFI não está montada, e `\Boot`
é oculto. Colete-o explicitamente ou monte a partição. `-d` também lê `BCD-Template`
(`HiveType` indica qual é); o modelo é o ponto de partida do Windows, não o repositório com o qual ele inicializa.

## Cuidados para o analista { #analyst-cautions }

- **As configurações são herdadas.** Um carregador de inicialização herda dos objetos em sua
  `BcdLibraryObjectList_InheritedObjects` (geralmente `{bootloadersettings}`, que herda
  `{globalsettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` e
  `BootStatusPolicy` são valores armazenados resolvidos (próprios ou, na falta deles, herdados), e `Inherited` indica quais
  vêm de outro objeto. A herança é seguida por no máximo oito níveis, com ciclos limitados. Um objeto referenciado pode estar ausente de um repositório parcial. Campos vazios não estabelecem um padrão efetivo do Windows. O firmware e o Secure Boot também podem impedir que uma opção armazenada tenha efeito.
- **Não definido não é desativado.** Um `RecoveryEnabled` vazio significa que o elemento não existe; aplica-se o
  padrão do Windows.
- **Os horários são por elemento.** O horário da chave de cada elemento (`key_last_written` em `Elements`) é quando ele
  foi gravado pela última vez. O Windows regrava alguns elementos em atualizações e upgrades; compare com os horários
  dos outros elementos.
- **Os dispositivos são parcialmente decodificados.** Uma partição GPT é indicada pelos GUIDs da partição e do disco;
  um ramdisk, pelo seu objeto de opções. Outros dispositivos mantêm apenas seus bytes (`data_raw`).
- **Elementos não documentados** (`0x15000065`, a classe `0x4…`) não têm nome e mantêm seus bytes.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

O resumo lista a entrada padrão de cada repositório e todas as configurações de assinatura de teste, verificação de integridade, recuperação,
política de status de inicialização (qualquer `Ignore…`) e modo de segurança, com o horário em que foram definidas ou o objeto
do qual são herdadas.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, o hive e `BCD` ou `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | O GUID do objeto, seu tipo (`0x10200003`) e o que ele é (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | A descrição, o caminho do aplicativo e seu dispositivo (uma partição e um disco GPT, ou o objeto de opções de um ramdisk). |
| `IsDefault`, `DisplayOrder` | Se o gerenciador de inicialização do Windows o inicializa por padrão e sua posição no menu de inicialização. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, conforme resolvidos a partir do repositório. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) e `bootstatuspolicy` (`IgnoreAllFailures`, …), com o número. |
| `Inherited` | Quais dessas configurações vêm de outro objeto, e de qual. |
| `Elements` | Todos os elementos: tipo, nome, valor, bytes (para dispositivos, tipos não documentados e dados que não são decodificados) e horário da chave. |
| `KeyLastWrittenUTC` | A última gravação da chave do objeto. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Objetos sem tipo, elementos cujos dados não correspondem ao seu formato. |

## Estratégia de investigação { #investigation-strategy }

1. Verifique a entrada padrão e todos os carregadores de inicialização quanto a `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, uma `BootStatusPolicy` `Ignore…` e `SafeBoot`.
2. Use os horários dos elementos para datar a alteração e correlacione com `bcdedit.exe` no Prefetch,
   no Amcache, nos eventos de criação de processos (4688, Sysmon 1) e no log de eventos System.
3. Procure entradas de inicialização ou dispositivos que não deveriam existir (outro caminho de carregador, um ramdisk) e
   uma entrada padrão ou ordem de exibição alteradas.
4. Compare o repositório com `BCD-Template` e com outro sistema da mesma build.

## Mais detalhes { #further-details }

- [Microsoft: referência de opções de inicialização do BCD](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (tipos de objeto)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
