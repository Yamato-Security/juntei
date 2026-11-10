# Dispositivos USB { #usb-devices }

## O que é o histórico de USB { #what-the-usb-history-is }

O Windows registra todos os dispositivos que instala. Para armazenamento USB, outros dispositivos USB e dispositivos
portáteis (telefones, câmeras, reprodutores de mídia), o hive `SYSTEM` mantém uma chave por dispositivo com seu nome,
número de série e os horários em que o Windows o instalou, o viu chegar pela última vez e o viu ser removido pela última vez.
`MountedDevices` registra os GUIDs de volume e as letras de unidade atribuídos a seus volumes. O hive
`SOFTWARE` mantém os nomes que o Explorer deu a dispositivos portáteis e volumes, e o registro do ReadyBoost
dos números de série de volumes. O `NTUSER.DAT` de cada usuário registra os volumes que esse usuário tinha
montado.

`parse-registry usb` os une em um registro por dispositivo e volume: qual dispositivo era, quando
foi instalado e conectado, qual letra de unidade e qual volume ele tinha, e quais usuários tinham esse
volume montado.

<span class="evidence-strength">Melhor conclusão:</span> um dispositivo com este fabricante, produto e
número de série foi conectado a este sistema, pela primeira e pela última vez nestes horários; seu volume tinha esta
letra de unidade; estes usuários o tinham montado. Os registros do Registro não mostram quais arquivos foram copiados.

## Localização { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Propriedade do dispositivo | Campo | Significado (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: quando seu driver foi instalado pela última vez |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: quando o dispositivo foi instalado pela primeira vez |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: quando foi conectado pela última vez (Windows 8 e posteriores) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: quando foi removido pela última vez (Windows 8 e posteriores) |

O Windows 8 e posteriores mantêm cada horário como o valor padrão da chave `0064` … `0067`. O Windows 7
o mantém em `00000064\00000000\Data`. O Juntei lê ambos.

Muitas referências de DFIR rotulam `0064` como a primeira instalação e `0065` como a instalação. O Juntei segue
as definições de `devpkey.h` da Microsoft acima, e nomeia as colunas de acordo com o que cada horário registra.

Forneça a `-f` ou `-d` o hive `SYSTEM`. O hive `SOFTWARE` do sistema e os hives `NTUSER.DAT` de seus usuários
são encontrados ao lado dele, como uma coleta os mantém (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), ou no mesmo diretório e um nível abaixo quando os hives foram
exportados juntos. Colete todos eles com seus logs de transação.

## Como os registros são unidos { #how-the-records-are-joined }

- **Dispositivos:** um registro por chave de dispositivo: `usbstor`, `usb` e `wpdbusenum` (`DeviceClass`). Um
  dispositivo de armazenamento USB obtém seus IDs de fabricante e de produto do dispositivo USB com o mesmo
  container ID (ou, na falta dele, o mesmo número de série).
- **Volumes:** os volumes de um dispositivo de armazenamento USB são os valores `\??\Volume{GUID}` de
  `MountedDevices` cujos dados são um caminho que nomeia o dispositivo. Um dispositivo com dois volumes tem dois
  registros. Suas letras de unidade são os valores `\DosDevices\X:` com os mesmos dados.
- **Nomes e números de série:** os nomes de volume vêm de `Windows Portable Devices` e dos
  dispositivos portáteis que nomeiam o mesmo dispositivo. Os rótulos e números de série de volume vêm de chaves
  `EMDMgmt` que o nomeiam.
- **Usuários:** os usuários cujo `MountPoints2` tem o GUID do volume, com o horário da última gravação dessa chave.
- **Volumes sem chave de dispositivo** (`volume`): um volume de `MountedDevices` que nenhuma chave de dispositivo nomeia (o de um
  disco interno, ou o de um dispositivo removido cuja chave `Enum` não existe mais), uma letra de unidade com a qual nenhum valor de GUID de volume
  compartilha dados, e um volume que somente o `MountPoints2` dos usuários tem.
- **Dispositivos que somente o `SOFTWARE` nomeia** (`portable_device`, `readyboost`).

`SourceKeys` lista todas as chaves e valores a partir dos quais um registro foi unido, com seu hive.

## Cuidados para o analista { #analyst-cautions }

- **As letras de unidade são reutilizadas.** `MountedDevices` mantém o último dispositivo que recebeu cada letra. Uma
  letra é mostrada apenas no dispositivo cujos dados ainda são os dessa letra.
- **Os números de série podem ser inventados.** Quando o segundo caractere de uma instância é `&`, o Windows
  o inventou porque o dispositivo não informa número de série; `Serial` fica vazio nesse caso. Essa
  instância pode ser diferente em outro computador.
- **As instâncias de armazenamento USB terminam com a unidade lógica** (`&0`). `Serial` a omite; `Instance`
  a mantém.
- **Os horários variam conforme a versão do Windows.** O Windows 7 tem apenas os horários de instalação. `LastArrivalUTC` e
  `LastRemovalUTC` são apenas a última conexão, não todas as conexões. Os logs de eventos
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) e o
  `setupapi.dev.log` fornecem mais.
- **Conjuntos de controle:** por padrão, é lido o conjunto de controle que o Windows usou por último. `--all-control-sets`
  lê todos os conjuntos de controle; um mais antigo ainda pode listar um dispositivo que o atual não tem mais.
- **O horário da última gravação de uma chave não é um horário de conexão.** O Windows regrava as chaves de dispositivos por muitos
  motivos.
- **Dispositivos removidos deixam vestígios.** Um registro de volume com fabricante e produto, mas sem chave de dispositivo,
  é um dispositivo cuja chave `Enum` foi removida (por uma desinstalação ou por ferramentas de limpeza) e cujo volume
  o `MountedDevices` ainda tem.
- **Os números de série do ReadyBoost** são o número de série do volume (`VolumeSerials`, como o `dir` o exibe),
  que arquivos LNK e Jump Lists também registram.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

O resumo lista os dispositivos de armazenamento USB que chegaram por último (os mais recentes primeiro), com suas letras
de unidade, e quantos volumes pertencem a dispositivos de armazenamento USB sem chave de dispositivo.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host` | O nome do computador, a partir do `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` ou, sem chave de dispositivo, `volume`, `portable_device` ou `readyboost`. |
| `ControlSet` | O conjunto de controle da chave do dispositivo. |
| `Vendor`, `Product`, `Revision` | Armazenamento USB: a partir do nome do dispositivo (`Disk&Ven_…&Prod_…&Rev_…`), também para um dispositivo que uma chave de volume ou do `SOFTWARE` nomeia. |
| `Vid`, `PID` | Os IDs USB de fabricante e de produto. |
| `Serial`, `Instance` | O número de série e o nome da chave de instância conforme gravado. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Os valores da chave do dispositivo, conforme gravados (`DeviceDesc` costuma ser `@<inf>,%…%;<text>`). O `FriendlyName` de um dispositivo portátil geralmente é o rótulo ou a unidade de seu volume. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Os horários das propriedades do dispositivo (veja acima). |
| `DriveLetters` | As letras de unidade cujos dados em `MountedDevices` são os do volume. |
| `VolumeGUID`, `VolumeData` | O GUID do volume e onde seus dados em `MountedDevices` dizem que ele está: um caminho de dispositivo, uma assinatura de disco MBR e o deslocamento da partição, ou uma partição GPT. |
| `VolumeNames`, `VolumeSerials` | Nomes que o Explorer deu ao dispositivo ou a seu volume, rótulos do ReadyBoost e números de série de volume (`1234-ABCD`). |
| `User` | Os usuários cujo `MountPoints2` tem o volume, com a última gravação dessa chave (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | Onde foi encontrado (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`) e todas as chaves e valores a partir dos quais foi unido. |
| `KeyPath`, `KeyLastWrittenUTC` | A chave de instância do dispositivo e sua última gravação. |
| `SourcePath`, `SoftwarePath` | O hive `SYSTEM` e o hive `SOFTWARE`, quando ele forneceu algo. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro dos hives a partir dos quais o registro foi unido. |
| `Problems` | Chaves que não puderam ser lidas e propriedades de dispositivo que não são FILETIMEs (com seus bytes). |

## Estratégia de investigação { #investigation-strategy }

1. Ordene os dispositivos de armazenamento USB por `LastArrivalUTC` (ou, na falta dele, `DriverLastInstalledUTC`) e separe os
   leitores e hubs do próprio computador.
2. Anote o fabricante, o produto, o número de série, o GUID do volume, a letra de unidade e o número de série do volume
   de cada dispositivo, e procure-os em arquivos LNK, Jump Lists, ShellBags e no `$MFT` (caminhos nessa
   letra de unidade ou nesse volume).
3. Verifique `User` para saber quem tinha o volume montado, e quando (a última gravação da chave `MountPoints2`).
4. Procure registros `volume` que nomeiem um dispositivo de armazenamento USB: dispositivos cujas chaves foram removidas.
5. Correlacione com os logs de eventos para todas as conexões, não apenas a última.
6. Use `parse-registry read --recover-deleted` para chaves de dispositivos excluídas.

## Mais detalhes { #further-details }

- [Microsoft: Propriedades de dispositivos (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Suporte a solicitações do gerenciador de montagem](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (como nos cabeçalhos do Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): os números das propriedades de dispositivos
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) e
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

As junções de usuários usam a identidade de origem de cada hive, de modo que hives exportados com o mesmo nome de perfil permanecem separados. `SourceKeys` mantém os caminhos dos hives de usuário que contribuíram. A descoberta de usuários associados ignora links simbólicos, de forma consistente com a descoberta recursiva de hives. A correlação de VID/PID exige um container ID utilizável correspondente; sem ele, a correspondência por número de série deve produzir IDs inequívocos. Candidatos conflitantes permanecem desconhecidos, com um problema registrado.
