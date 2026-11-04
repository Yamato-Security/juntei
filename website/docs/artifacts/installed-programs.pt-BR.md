# Programas instalados { #installed-programs }

## O que o Windows mantém { #what-windows-keeps }

O Windows e os instaladores registram o software instalado em várias listas do Registro. Cada lista tem uma chave
por programa, produto, distribuição ou pacote; o Juntei gera um registro por chave.

| Fonte | Hive e chave | O que contém |
|---|---|---|
| `uninstall` | `SOFTWARE` e `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, e `SOFTWARE\Wow6432Node\…` | O que Configurações e *Programas e Recursos* listam: nome, versão, editor, data de instalação, pastas, comandos de desinstalação, tamanho, se está oculto |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Produtos do Windows Installer: nome e versão do produto, o nome do arquivo do pacote e a pasta a partir da qual foi instalado |
| `app_path` | `SOFTWARE` e `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Programas que o Windows inicia pelo nome (`Win+R`, `start tool`): o caminho do programa e sua pasta |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Distribuições do Subsistema do Windows para Linux: nome, onde estão seus arquivos ou disco, WSL 1 ou 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Aplicativos empacotados (da Store): provisionados para todos os usuários, do próprio Windows, preparados (staged), desprovisionados e os de cada usuário |

<span class="evidence-strength">Melhor conclusão:</span> este programa, produto ou pacote foi
instalado (ou registrado) no sistema, para todos os usuários ou para este usuário. A data de instalação é
o que o instalador gravou; a última gravação da chave é quando a entrada foi alterada pela última vez.

## Localização { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Colete o `SOFTWARE`, todos os `NTUSER.DAT` e seus logs de transação. O `SYSTEM` nomeia o host, e
o `ProfileList` do `SOFTWARE` fornece o SID de cada usuário.

## Cuidados para o analista { #analyst-cautions }

- **Data de instalação e horário da chave.** `InstallDate` é uma data (sem hora) gravada pelo instalador, no
  horário local; alguns instaladores não a gravam, e outros gravam em outro formato (mantido como gravado em
  `InstallDateRaw`). Uma atualização pode regravar a entrada sem alterar a data, ou alterar a data
  sem que o programa mude muito: compare `InstallDate` com `KeyLastWrittenUTC`.
- **Software desinstalado.** Um programa removido de forma limpa perde suas chaves. Use `parse-registry read
  --recover-deleted` para chaves `Uninstall` excluídas, e o Amcache, o Prefetch e o `$MFT` para os vestígios
  deixados.
- **Registros por usuário.** As entradas do `NTUSER.DAT` identificam software registrado para um perfil.
  Esse escopo não estabelece quem executou o instalador nem se foram usados direitos de administrador.
- **Entradas ocultas.** `SystemComponent` oculta uma entrada de *Programas e Recursos*; entradas
  sem `DisplayName` (atualizações, componentes) também não são exibidas. Todas são relatadas.
- **Códigos de produto.** Uma chave `Uninstall` com formato de GUID pode ser associada a uma chave
  `msi_product` compactada por meio de `ProductCode`. O nome em formato de GUID, por si só, não prova que o
  Windows Installer criou a entrada.
- **App Paths.** São registros usados para localizar programas, não registros de
  execução. O valor `Path` pode conter vários diretórios de pesquisa e não é necessariamente o
  diretório de instalação.
- **Editor.** São metadados do Registro fornecidos pelo instalador, não informações de assinatura
  verificadas.
- **Arquitetura.** `Wow6432Node` contém programas de 32 bits no Windows de 64 bits (`Architecture` `x86`,
  `Wow64` `True`); as demais entradas são da arquitetura própria do sistema (vazio). A arquitetura de um pacote
  é a dele próprio (`x64`, `neutral`, …).
- **WSL.** `Version` é a versão do WSL obtida das `Flags` da distribuição (`0x8`: ela é executada na
  máquina virtual do WSL 2). O próprio valor `Version` da distribuição (em `Details`) é a versão do seu sistema de arquivos,
  2 também em distribuições WSL 1. `BasePath` contém seu `ext4.vhdx` (WSL 2) ou `rootfs`
  (WSL 1): colete-o para examinar seu conteúdo.
- **Pacotes.** As listas `<SID>` são os pacotes registrados para cada usuário; a maioria é do próprio
  Windows. Pacotes `Deprovisioned` foram removidos para novos usuários; os de `EndOfLife\<SID>`, para aquele usuário.
- **Valores danificados.** Números malformados, tipos de valor incorretos e UTF-16 inválido são relatados em
  `Problems` e mantidos em `DataRaw`, incluindo uma string hex vazia para um valor presente com
  zero bytes. Uma string vazia sem problemas em um campo numérico opcional é tratada como ausente. A versão de um pacote
  deve conter quatro componentes decimais, cada um dentro do intervalo de 16 bits sem sinal.
  Os avisos no nível do hive permanecem no stderr quando `-t` transmite os registros para o stdout.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

O resumo lista os programas instalados por último (por `InstallDate`), o que os usuários instalaram para
si mesmos e as distribuições WSL.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, arquivo do hive e `SOFTWARE` ou `NTUSER.DAT`. |
| `User`, `UserSID` | O usuário de um hive de usuário (sua pasta de perfil e o SID do `ProfileList`) ou o usuário para o qual um pacote está listado. |
| `Source`, `Scope` | A lista (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) e `machine` ou `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; `ProductName` e `Version` de um produto; o nome do arquivo do programa de um App Path; o nome e a versão do WSL de uma distribuição; o nome e a versão de um pacote. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` como data quando gravado como `YYYYMMDD`; caso contrário, como foi gravado. |
| `KeyLastWrittenUTC` | Quando a chave da entrada foi alterada pela última vez. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (o `Path` de um App Path, o `BasePath` de uma distribuição, o `Path` de um pacote); `InstallSource` (a última pasta de origem usada por um produto). |
| `Program` | O programa de um App Path. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` em KiB; se está oculto; se o Windows Installer o instalou. |
| `ProductCode` | O código de produto do Windows Installer (`{GUID}`). |
| `Architecture`, `WOW64` | `x86` sob `Wow6432Node` ou a arquitetura de um pacote; se a chave está sob `Wow6432Node`. |
| `Details` | Mais informações: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (a conta que executou um instalador Inno Setup); `PackageCode` e `PackageName` de um produto; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName` e `Default` de uma distribuição; `List`, `ResourceId` e `PublisherId` de um pacote. |
| `KeyPath` | A chave da entrada. |
| `DataRaw` | Os bytes (hex) de valores que não são o que seus nomes indicam. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Valores que não puderam ser lidos ou que não são o que seus nomes indicam. |

## Estratégia de investigação { #investigation-strategy }

1. Ordene por `InstallDate` e `KeyLastWrittenUTC` em torno do incidente; examine ferramentas de acesso
   remoto, clientes de tunelamento e de transferência de arquivos, compactadores, runtimes de scripts e ferramentas de segurança.
2. Revise os registros com `Scope` `user`: software instalado sem direitos de administrador.
3. Verifique `InstallLocation`, `InstallSource` e `UninstallCommand` em busca de pastas graváveis pelo usuário
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Procure distribuições WSL e colete seu `BasePath`.
5. Correlacione com o Amcache e o Prefetch (o instalador e o programa em execução), o `$MFT` (a criação
   da pasta de instalação) e o log de eventos Application (eventos do MsiInstaller).

## Mais detalhes { #further-details }

- [Microsoft: chave do Registro Uninstall](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: registro de aplicativos (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: visão geral da identidade de pacote](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
