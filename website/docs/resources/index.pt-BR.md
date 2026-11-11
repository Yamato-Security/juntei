# Artigos e referências { #articles-and-references }

Esta página reúne pontos de partida de alta qualidade para leitores que querem ir além dos guias de
artefatos. As seções **Mais detalhes** de cada artefato contêm os links de baixo nível mais específicos.

## Orientação inicial em DFIR { #beginner-dfir-orientation }

- [Pôster SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/) — um mapa visual dos artefatos comuns do Windows e de seus locais.
- [Pôster SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/) — pivôs práticos de caça a ameaças e lembretes para investigações no Windows.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — amostras públicas de artefatos para aprendizado e validação de analisadores.
- [Referência de artefatos do Velociraptor](https://docs.velociraptor.app/artifact_references/) — descrições orientadas à coleta e implementações em VQL para muitos artefatos do Windows.
- [Documentação do Plaso](https://plaso.readthedocs.io/) — conceitos de linha do tempo e analisadores para análise entre artefatos.

## Artefatos do sistema de arquivos { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: descritores de segurança](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) e [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — descritores de segurança, ACEs e expressões condicionais.
- [Documentação do formato libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Analisador mft de Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — analisador NTFS com recuperação (carving) de registros INDX e do slack de índices.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — recupera nomes de arquivos do slack de `$I30`.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — pesquisa detalhada e análise do log de transações NTFS.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — correlaciona o histórico de `$LogFile`, `$MFT` e USN.

## Artefatos de execução e inventário { #execution-and-inventory-artifacts }

- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — trabalho sobre o formato/biblioteca do Prefetch.
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## Artefatos de atividade do usuário { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registro e persistência { #registry-and-persistence }

- [Microsoft: tipos de valores do Registro](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Especificação do formato do Registro do Windows de Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) e [libregf](https://github.com/libyal/libregf) — referências de baixo nível para análise e recuperação de hives.
- [Microsoft: árvore do Registro de serviços](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: esquema do Agendador de Tarefas](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender e tratamento de malware { #defender-and-malware-handling }

- [Microsoft: restaurar arquivos em quarentena no Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [Plugin Defender do dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — uma implementação separada de pesquisa/extração.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — recursos de tratamento e análise.

## Projetos de analisadores do Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "Materiais externos mudam"
    Verifique as afirmações em relação à versão do Windows afetada e aos dados coletados. Um artigo útil pode
    descrever uma geração específica do sistema operacional, uma versão de analisador ou um experimento, e não uma
    regra universal.
