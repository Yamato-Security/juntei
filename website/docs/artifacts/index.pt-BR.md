# Guia de artefatos { #artifact-guide }

O Windows preserva vestígios sobrepostos no sistema de arquivos, no Registro, em bancos de dados de compatibilidade,
no histórico da interface do usuário, na contabilização de recursos e em produtos de segurança. O Juntei analisa essas fontes,
mas o investigador ainda precisa interpretar o que cada fonte significa.

## Escolha um artefato pela pergunta { #choose-an-artifact-by-question }

| Pergunta da investigação | Comece por aqui | Corrobore com |
|---|---|---|
| Quais linhas de comando do PowerShell foram salvas? | [Histórico do PowerShell](powershell-history.md) | Logs de eventos do PowerShell, telemetria de processos, Prefetch |
| Este executável foi executado? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), logs de eventos |
| Este arquivo chegou a existir? | [Amcache](amcache.md), [$MFT](mft.md) | [Diário USN](usn-journal.md), [$I30](i30.md), [Índice do Windows Search](windows-search.md), ShimCache |
| Quais arquivos, e-mails e páginas estavam neste computador, e o que um documento dizia? | [Índice do Windows Search](windows-search.md) | [$MFT](mft.md), [Diário USN](usn-journal.md), [Outlook](outlook.md), [Navegadores web](browser.md) |
| Quando um arquivo foi criado, renomeado ou excluído? | [Diário USN](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), quarentena do Defender |
| O que um usuário enviou para a Lixeira, e quando? | [Lixeira](recycle-bin.md) | [Diário USN](usn-journal.md), [$MFT](mft.md), [LNK](lnk.md), [Jump Lists](jump-lists.md) |
| Que nome ou carimbo de data/hora existia antes de uma alteração de metadados? | [$LogFile](logfile.md) | [$MFT](mft.md), [Diário USN](usn-journal.md), [$I30](i30.md) |
| Quem é o proprietário de um arquivo, e quem pode alterá-lo? | [$Secure:$SDS](secure-sds.md) com [$MFT](mft.md) | [SAM](sam.md), logs de eventos Security |
| Em qual volume estava o destino de um atalho ou de um arquivo Prefetch? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [dispositivos USB](usb.md) |
| O que um usuário abriu? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Um programa transferiu uma grande quantidade de dados? | [SRUM](srum.md) | Telemetria de firewall, proxy, DNS, pacotes ou EDR |
| Quais sites um usuário visitou, e o que ele baixou? | [Navegadores web](browser.md), [Internet Explorer e Edge legado](webcache.md) | Logs de proxy e DNS, `$MFT` e Diário USN, Defender, SRUM |
| Com quem um usuário trocou e-mails, e o que estava anexado? | [Outlook](outlook.md) | Logs de servidores e gateways de e-mail, `$MFT` e Diário USN, Defender, LNK e Jump Lists |
| Quais clientes e contas usaram este servidor, e em quais dias? | [Registro de acesso do usuário (UAL)](ual.md) | Logs de eventos Security, logs de SMB e RDP, logs de firewall |
| O que um usuário viu nos computadores que acessou pela Área de Trabalho Remota? | [Cache de bitmaps do RDP](rdp-cache.md) | [Acesso remoto (lado do usuário)](remote-access.md), [Jump Lists](jump-lists.md), os eventos de logon dos computadores remotos e o [UAL](ual.md) |
| Foi configurada persistência? | [Serviços](services.md), [TaskCache](taskcache.md), [Registro](registry.md) | Logs de eventos, Prefetch, `$MFT` |
| O que o Defender detectou, e o que ele fez a respeito? | [Histórico de detecções do Defender](defender-history.md) | [Quarentena do Defender](defender-quarantine.md), logs de eventos |
| O que o Defender removeu? | [Quarentena do Defender](defender-quarantine.md) | Logs de eventos, `$MFT`, Diário USN |
| Contas locais foram adicionadas ou alteradas? | [SAM](sam.md) | Logs de eventos Security e histórico de transações do Registro |

## Leia os avisos antes dos campos { #read-warnings-before-fields }

Toda página de artefato segue a mesma sequência:

1. **O que é** e quais perguntas ele pode responder.
2. **Onde encontrá-lo** e como a coleta o afeta.
3. **O que ele prova — e o que não prova.**
4. **Como analisá-lo** com o Juntei.
5. **Como ler cada campo de saída.**
6. **Como correlacioná-lo** com evidências independentes.
7. **Mais detalhes** para pesquisa de baixo nível.

!!! danger "Não trate registros incomuns como prova de atividade maliciosa"
    O Juntei analisa evidências para interpretação por um analista. As verificações do Persistence Hunter continuam disponíveis, mas uma entrada de inicialização configurada também pode fazer parte de uma administração legítima. Correlacione as observações com o contexto e evidências independentes antes de classificá-las como maliciosas.
