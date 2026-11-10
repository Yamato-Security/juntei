# Acesso remoto (lado do usuário) { #remote-access-user-side }

## O que é { #what-it-is }

O `NTUSER.DAT` de um usuário guarda vestígios das conexões remotas que o usuário fez a partir do computador.
Eles são o lado de origem da movimentação lateral:

- **Área de Trabalho Remota (`mstsc`):**
  - os computadores digitados no cliente, os mais recentes primeiro;
  - uma chave por computador em que o usuário entrou, com o nome da conta usada e a impressão digital
    do certificado do servidor que o usuário aceitou;
  - os computadores aos quais o usuário permitiu usar dispositivos locais.
- **PuTTY:**
  - sessões salvas: host, porta, conta, protocolo, arquivo de chave, proxy e encaminhamentos de porta;
  - as sessões iniciadas por último;
  - todas as chaves de servidor SSH que o usuário aceitou.
- **WinSCP:** sessões salvas e chaves de servidor SSH aceitas, no mesmo formato das do PuTTY.
- **Unidades mapeadas:** cada unidade mapeada para ser restaurada ao entrar, e os caminhos digitados em
  *Mapear unidade de rede*.
- **Compartilhamentos:** os compartilhamentos que o Explorer montou (`MountPoints2`).

Eles mostram para onde uma conta foi em seguida e com quais credenciais, mesmo depois que os logs do próprio
computador remoto tenham desaparecido.

<span class="evidence-strength">Melhor conclusão:</span> o perfil deste usuário se conectou a este destino, ou salvou
uma conexão com ele. Uma sessão salva ou um nome digitado, por si sós, não provam que uma
conexão foi bem-sucedida. Uma chave de host aceita ou uma chave `Servers` com um `UsernameHint` é mais forte:
o cliente alcançou o servidor.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

Colete o `NTUSER.DAT` com seus logs de transação para todos os perfis. Colete também o `SOFTWARE`
do sistema: ele fornece o SID de cada usuário.

## Cuidados para o analista { #analyst-cautions }

- **Os horários das chaves são horários das listas.** `Default`, `SshHostKeys` e `Map Network Drive MRU` contêm listas,
  e a última gravação de sua chave é quando a lista mudou pela última vez. Esse horário pertence ao primeiro item
  (`MruPosition` 0). Para os demais, é apenas um limite superior. Uma chave `Servers\<host>`, de sessão,
  `Network\<letter>` ou de compartilhamento tem seu próprio horário.
- **Salvo não é usado.** Uma sessão do PuTTY ou do WinSCP pode ser salva sem nunca se conectar. O PuTTY
  regrava `Recent sessions` quando uma sessão salva é iniciada, e adiciona uma chave de host quando um servidor é
  alcançado pela primeira vez.
- **Senhas:**
  - O WinSCP pode armazenar senhas de sessão, de túnel e de proxy.
  - O PuTTY armazena apenas a senha de um proxy, em texto puro.
  - `PasswordStored` e o `StoredPasswords` de `Details` indicam quais valores estão definidos. Eles nunca são
    decodificados nem gravados na saída.
- **O WinSCP omite os padrões.** Uma sessão sem `FSProtocol` é SFTP, e uma sem
  `PortNumber` usa a porta padrão do protocolo. `Protocol` e `Port` ficam vazios nesse caso, em vez
  de adivinhar.
- **As impressões digitais de chaves de host** são as `SHA256:…` do OpenSSH, como `ssh-keygen -l`, `ssh` e o PuTTY 0.75
  e posteriores as exibem. O Juntei as calcula a partir da chave que o PuTTY e o WinSCP guardam. Compare-as com
  a própria chave do servidor, arquivos `known_hosts` e logs para identificar o servidor por trás de um nome ou endereço
  IP. Uma chave SSH-1 (`rsa`) não tem impressão digital.
- **Unidades mapeadas:** `RemoteUsername` é definido somente quando a unidade foi mapeada com outra
  conta; o Windows grava um `REG_DWORD` 0 quando usou a do próprio usuário.
- **Compartilhamentos:** `MountPoints2` também tem chaves para volumes e letras de unidade. Apenas compartilhamentos de rede
  (`##server#share`) são relatados aqui; `\\server@SSL@443\DavWWWRoot` é um compartilhamento WebDAV.
- **Remoção:** limpar as listas, ou usar um cliente portátil, não deixa vestígios aqui. Compare com
  Jump Lists, Prefetch, arquivos `.rdp`, o cache de bitmaps do RDP e os logs do destino.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

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
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

O resumo conta as conexões por cliente. Ele lista as conexões cujas chaves foram gravadas por último,
as mais recentes primeiro, e as sessões que guardam uma senha.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, o nome da pasta do perfil, o SID do usuário a partir do `ProfileList` do sistema (com o hive `SOFTWARE` do sistema) e o hive. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` ou `network_share`. |
| `Kind` | De onde o registro vem: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | O computador (nome do host ou endereço IP) e a porta, ou o compartilhamento (`\\server\share`). Uma porta só é informada quando o Windows ou o cliente gravou uma. |
| `RemoteUsername` | A conta usada no destino: `UsernameHint`, o `UserName` de uma sessão (ou o `user@` do `HostName` do PuTTY), o `UserName` de uma unidade mapeada. |
| `Protocol` | O `Protocol` do PuTTY (`ssh`, `telnet`, `raw`, …), o `FSProtocol` do WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), o provedor de uma unidade mapeada (`Microsoft Windows Network`). |
| `MRU-Position` | A posição em sua lista (0 é a mais recente): `MRU<n>`, `MRUList`, `Recent sessions` do PuTTY. |
| `SessionName` | O nome de uma sessão salva, com `%20` e outros escapes decodificados. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | O tipo de uma chave de host SSH aceita (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), sua impressão digital OpenSSH e seus números conforme armazenados. |
| `DriveLetter` | A letra de uma unidade mapeada. |
| `PasswordStored` | Se a sessão guarda uma senha; nunca decodificada. |
| `Details` | Outras configurações, como `Name=Value; …` (CSV) ou um objeto (JSON): o `CertHash` do certificado RDP, `LocalDevices`, o arquivo de chave de uma sessão, proxy, túnel SSH, encaminhamentos de porta, comando e diretórios remotos, `StoredPasswords`, o `Label` de um compartilhamento, o `ConnectionType` de uma unidade. |
| `KeyPath`, `ValueName` | A chave, e o valor para um registro que é um único valor. |
| `KeyLastWrittenUTC` | A última gravação da chave (veja os cuidados sobre listas). |
| `DataRaw` | Os bytes do valor (hex), quando não é uma string ou não é decodificado. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Valores do tipo errado (com seus bytes), chaves de host que não podem ser lidas, letras de `MRUList` que não existem e valores que ele não lista. |

## Estratégia de investigação { #investigation-strategy }

1. Liste os destinos de cada usuário por cliente e compare-os com os sistemas que esse usuário deveria
   acessar. Procure compartilhamentos administrativos (`C$`, `ADMIN$`) e endereços IP.
2. Anote as contas usadas (`RemoteUsername`): administradores de domínio e contas locais em outras
   máquinas.
3. Associe `HostKeyFingerprint` às chaves dos servidores, e `CertHash` aos certificados dos servidores
   RDP.
4. Correlacione com os eventos de logon dos destinos (4624 tipo 10 e 3, 1149), Jump Lists e arquivos LNK
   (`mstsc.exe`, `putty.exe`, arquivos abertos a partir de compartilhamentos), Prefetch e o cache de bitmaps do RDP.
5. Use `parse-registry read --recover-deleted` para entradas excluídas, e cópias mais antigas do
   `NTUSER.DAT` do perfil (Cópias de Sombra de Volume, backups) para listas limpas.

## Mais detalhes { #further-details }

- [Microsoft: Remover entradas da caixa Computador da Conexão de Área de Trabalho Remota](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [FAQ do PuTTY: Onde o PuTTY armazena seus dados?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Armazenamento da configuração](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
