# Redes { #networks }

## O que são { #what-they-are }

O Windows guarda as redes às quais um sistema se conectou e como sua rede foi configurada.

- **`SOFTWARE`:**
  - **Perfis de rede** (`NetworkList\Profiles`): um por rede à qual o sistema se conectou,
    com seu nome (o SSID de uma rede Wi-Fi), categoria (pública, privada, domínio), tipo de conexão
    (cabeada, sem fio, VPN, banda larga móvel) e quando o sistema se conectou a ela pela primeira e pela última vez.
  - **Assinaturas** (`NetworkList\Signatures`): o que identifica cada rede, como o
    endereço MAC do gateway e o sufixo DNS.
  - **Placas de rede** (`NetworkCards`): os adaptadores.
- **`SYSTEM`** (por conjunto de controles):
  - **Interfaces TCP/IP:** endereços, servidor e concessão DHCP, servidores DNS, domínio.
  - **Compartilhamentos** oferecidos pelo sistema (`LanmanServer\Shares`).
  - **Proxies de porta** (`PortProxy`): os túneis configurados por `netsh interface portproxy`, que
    os atacantes usam para pivotar e para alcançar o RDP por outra porta.
  - **Rotas persistentes.**
  - **O fuso horário**, que converte os horários locais dos perfis em UTC.

<span class="evidence-strength">Melhor conclusão:</span> o sistema se conectou a uma rede com
este nome e gateway, pela primeira e pela última vez nestes horários. Ele tinha este endereço obtido deste servidor DHCP
e oferecia estes compartilhamentos e túneis quando o hive foi salvo. O nome de uma rede, sozinho, não
identifica onde ela estava: muitas redes se chamam "Network" ou compartilham um SSID.

## Localização { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

Colete os dois hives com seus logs de transação. O hive SYSTEM é necessário para converter os horários
dos perfis em UTC.

## Cuidados para o analista { #analyst-cautions }

- **Os horários dos perfis são locais.** `DateCreated` e `DateLastConnected` são gravados no horário
  local do sistema.
  - `FirstConnectedLocal` e `LastConnectedLocal` os mantêm como foram encontrados.
  - `FirstConnectedUTC` e `LastConnectedUTC` os convertem com o fuso horário do hive
    SYSTEM do sistema (`TimeZone`) e sua regra de horário de verão.
  - Um horário anterior a uma mudança de fuso horário, ou de um ano com outra regra de horário
    de verão (datas dos EUA anteriores a 2007), pode estar deslocado pela diferença.
  - Sem um hive SYSTEM legível e um conjunto de controles atual válido, apenas os horários locais são fornecidos. Lacunas do horário de verão e horas repetidas ambíguas não têm um único carimbo de data/hora UTC comprovado: o UTC fica vazio e `Problems` explica a limitação. Deslocamentos ou datas inválidos também permanecem sem conversão.
- **Apenas a primeira e a última.** Cada perfil tem uma primeira e uma última conexão, não todas as
  conexões. A última gravação da chave do perfil geralmente coincide com a última conexão.
- **Os nomes se repetem.** O Windows nomeia redes cabeadas não gerenciadas como "Network", "Network 2" e assim por diante.
  Os SSIDs de Wi-Fi não são únicos. Use `GatewayMac` e `DnsSuffix` para diferenciar as redes e
  correlacione-os com logs de DHCP e com o inventário de ativos. Um MAC de gateway não é necessariamente o BSSID de um ponto de acesso Wi-Fi e, sozinho, não estabelece a localização física.
- **Compartilhamentos, proxies e rotas** são configuração: um compartilhamento ou um proxy de porta pode ser configurado e
  nunca usado. Nenhum valor tem horário próprio; a última gravação da chave indica quando qualquer um deles
  mudou.
- **Interfaces:** uma chave de interface sob um GUID sem endereços é um adaptador que não está
  configurado. Com DHCP, os endereços são os da última concessão. Um endereço estático remanescente de antes do DHCP
  não é exibido como em uso.
- **Conjuntos de controles:** por padrão, é lido o conjunto de controles que o Windows usou por último. `--all-control-sets`
  lê todos, o que pode revelar um compartilhamento ou um proxy que foi removido do atual.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

O resumo lista as redes conectadas por último (as mais recentes primeiro) e todos os compartilhamentos, proxies de porta e
rotas persistentes.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Host e o hive. |
| `ControlSet`, `IsCurrentControlSet` | Para registros do SYSTEM: o conjunto de controles e se ele é o que o Windows usou por último. |
| `Source` | `network_profile`, `network_signature` (uma assinatura que nenhum perfil tem), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | O nome de um perfil (o SSID de uma rede Wi-Fi); a descrição de uma assinatura. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (caso contrário, o número). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` e `DateLastConnected` convertidos para UTC com `TimeZone`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Os mesmos, como encontrados (horário local, com precisão de milissegundos). |
| `TimeZone` | O fuso horário usado na conversão. |
| `GatewayMAC`, `DNS-Suffix` | O endereço MAC do gateway e o sufixo DNS das assinaturas do perfil (`|` entre vários). |
| `InterfaceGUID` | O GUID de uma interface; o `ServiceName` de uma placa de rede. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Os endereços em uso de uma interface, o servidor DHCP e a concessão. |
| `ShareName`, `SharePath` | Um compartilhamento e a pasta que ele oferece. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Um proxy de porta: onde ele escuta e para onde encaminha. |
| `Route` | Uma rota persistente como armazenada: `destination,netmask,gateway,metric`. |
| `Details` | Outros valores, como `Name=Value; …` (CSV) ou um objeto (JSON): a descrição de um perfil, `Managed`, o primeiro nome e as chaves de assinatura; o adaptador de uma interface (de `NetworkCards`), máscaras, gateways, servidores DNS e domínio; o tipo, o comentário e outros campos de um compartilhamento; o tipo de um proxy; as partes de uma rota. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | A chave, o valor de um compartilhamento, proxy ou rota, e a última gravação da chave. |
| `DataRaw` | Bytes que não são decodificados: um horário que não é um `SYSTEMTIME` válido, um compartilhamento que não é uma lista de cadeias de caracteres. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Valores do tipo ou tamanho errado, datas impossíveis e nomes que não seguem o formato esperado. |

## Estratégia de investigação { #investigation-strategy }

1. Ordene os perfis por `LastConnectedUTC` e verifique as redes próximas ao incidente: redes
   Wi-Fi desconhecidas, hotspots móveis, VPNs.
2. Use `GatewayMac` e `DnsSuffix` para diferenciar redes com o mesmo nome e para associar uma
   rede entre hosts.
3. Verifique todos os proxies de porta e compartilhamentos. Um proxy de porta para a porta 3389 ou 445 de um host interno, ou um compartilhamento
   de uma pasta de preparação (`C:\ProgramData`, `C:\Users\Public`), é uma pista. Correlacione com os logs
   de eventos (criação do processo `netsh`, eventos de compartilhamento 5142/5144) e com os horários das chaves.
4. Use os endereços e os horários de concessão das interfaces para situar o host na rede e compare-os
   com os logs de firewall, DHCP e proxy.
5. Use `--all-control-sets` e `parse-registry read --recover-deleted` para encontrar compartilhamentos, proxies
   e perfis removidos.

## Mais detalhes { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
