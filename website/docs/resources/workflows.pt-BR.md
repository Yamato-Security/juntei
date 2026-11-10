# Investigações entre artefatos { #cross-artifact-workflows }

Nenhum artefato deve sustentar sozinho uma conclusão importante. Estes fluxos de trabalho mostram como transformar uma pista
em uma cadeia de evidências defensável.

## Execução de um executável suspeito { #suspicious-executable-execution }

1. **Prefetch:** estabeleça o horário de execução mantido, o contexto de caminho/hash, a contagem de execuções e os arquivos carregados.
2. **`$MFT`:** encontre o caminho, o tamanho, os carimbos de data/hora, os ADS, o estado de exclusão e a referência MFT.
3. **Diário USN:** reconstrua a criação, as gravações, a renomeação e a exclusão.
4. **Amcache:** acrescente o SHA-1, os metadados PE, a versão e o caminho histórico.
5. **ShimCache:** acrescente o caminho/a ordem no cache de compatibilidade e a flag de execução, quando compatível.
6. **SRUM:** procure o uso de recursos/rede pelo mesmo executável/usuário.
7. **Defender:** verifique o horário da quarentena, o caminho original, o hash e a origem do download.
8. **Logs de eventos/EDR:** identifique o processo pai, a linha de comando, a conta, a rede e o contexto de segurança.

A conclusão deve distinguir a presença do arquivo, a execução e a atribuição a uma pessoa ou intrusão.

## Atividade de arquivos de ransomware { #ransomware-file-activity }

1. Identifique as extensões criptografadas e os caminhos das vítimas no `$MFT`.
2. Use os motivos e a ordem do USN para reconstruir gravações/renomeações/exclusões em massa.
3. Use o Prefetch para identificar criptografadores candidatos e seus horários de execução.
4. Use o SRUM para picos de CPU/disco/rede do processo candidato.
5. Verifique os serviços e o TaskCache em busca de implantação/persistência.
6. Verifique a quarentena do Defender e os logs de eventos em busca de detecção/remediação.

O MFT/USN pode provar o impacto no sistema de arquivos mesmo quando nenhum artefato de execução sobrevive. Em geral, eles
não identificam o processo responsável sem telemetria de processos ou correlação.

## Acesso inicial por documento / atalho malicioso { #malicious-document-shortcut-initial-access }

1. RecentDocs e Jump Lists identificam a interação do usuário/aplicativo e o contexto MRU.
2. Analise o LNK correspondente da pasta Recent para obter o destino, o volume/compartilhamento, a referência MFT e os argumentos.
3. Use o Zone.Identifier dos ADS residentes do `$MFT` ou o conteúdo do Defender para obter URLs/zonas de origem.
4. Use o Prefetch/logs de eventos para o aplicativo do documento e os processos filhos.
5. Use o USN/MFT para acompanhar a criação e a exclusão da carga útil extraída.

## Persistência por serviço { #service-persistence }

1. Decodifique o nome do serviço, `ImagePath`, `ServiceDll`, a conta, o tipo de inicialização e as ações em caso de falha.
2. Resolva as variáveis de ambiente e o comportamento de caminhos com/sem aspas.
3. Localize o executável/a DLL no `$MFT` e no histórico do USN.
4. Verifique o Prefetch e os logs de eventos para a inicialização efetiva do serviço/processo.
5. Compare conjuntos de controle não atuais e registros do Registro recuperados para obter configurações anteriores.

## Persistência por tarefa agendada { #scheduled-task-persistence }

1. Decodifique as ações do TaskCache, o principal/contexto, os metadados de registro/execução e o estado oculto.
2. Compare com o XML em `C:\Windows\System32\Tasks`.
3. Correlacione o nome/GUID da tarefa com os eventos Security e Task Scheduler Operational.
4. Localize e calcule o hash do executável/script; inspecione o Prefetch e o histórico do sistema de arquivos.
5. Recupere registros excluídos do TaskCache se a tarefa tiver sido removida.

## Suspeita de exfiltração { #suspected-exfiltration }

1. Use o SRUM para identificar volumes incomuns de bytes enviados por aplicativo/usuário/perfil e o período de medição.
2. Confirme a identidade do executável com Prefetch, Amcache e `$MFT`.
3. Identifique os arquivos acessados/abertos por meio de Jump Lists, LNK, RecentDocs e dados de eventos/EDR.
4. Use telemetria de proxy, firewall, DNS, pacotes ou EDR para obter o destino/protocolo.

O volume do SRUM sustenta uma hipótese de exfiltração, mas não identifica o conteúdo transferido nem prova
intenção maliciosa por si só.

## Chaves de correlação que vale a pena preservar { #correlation-keys-worth-preserving }

| Chave | Artefatos |
|---|---|
| Caminho completo / nome do arquivo | Quase todos os artefatos |
| SHA-1 / SHA-256 | Amcache, Defender, arquivos recuperados |
| Entrada + sequência da MFT | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, Diário USN |
| Número de série do volume / droids | Prefetch, LNK, Jump Lists, object IDs da MFT |
| SID / nome de usuário | SRUM, SAM, Registro, logs de eventos |
| GUID / nome da tarefa | TaskCache, XML da tarefa, logs de eventos |
| Nome do serviço | Serviços, logs de eventos, dados de processos/drivers |
| EntryId / ScanId do Defender | Quarentena, histórico do Defender, logs de eventos |
