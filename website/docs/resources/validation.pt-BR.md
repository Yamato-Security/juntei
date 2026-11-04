# Validação dos analisadores { #parser-validation }

Os crates de analisadores mantidos pelo Juntei são testados com evidências públicas e implementações
independentes. Essas comparações ajudam a detectar regressões e diferenças de interpretação; elas
não tornam nenhum analisador infalível. Preserve as evidências de origem, analise os campos de problemas ou
danos de cada registro e valide as constatações relevantes com outro artefato ou outra implementação.

As declarações abaixo preservam as informações de validação que antes ficavam no README principal
do Juntei. Consulte os repositórios dos analisadores indicados para conhecer as fixtures de teste e os detalhes de implementação atuais.

## Amcache { #amcache }

O Juntei usa o [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser), um
fork sob Apache-2.0 do
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) da Security Ronin. Sua
[documentação de saída](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
define todos os campos de executáveis, dispositivos, constatações e resumo.

## Tabela Mestre de Arquivos { #master-file-table }

O crate mft do [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) é um fork mantido do crate
[mft](https://github.com/omerbenamram/mft) de Omer Ben-Amram. Em oito arquivos `$MFT` — a amostra do
crate, o caso de invasão do NIST e seis arquivos do DFIR Artifact Museum —, 1.048.733 linhas e colunas CSV
coincidiram, na mesma ordem, com as de um analisador de referência. As comparações abrangeram a saída padrão, `--all-times`,
`--short-names` e `--cutoff`; a apresentação dos carimbos de data/hora foi a diferença intencional.

## Log de transações NTFS ($LogFile) { #ntfs-transaction-log-logfile }

O crate logfile do [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) é mantido a partir do
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog). Sua validação abrange seis logs completos do DFIR
Artifact Museum, fixtures públicas das versões 1.1 e 2.0, metadados de redo/undo decodificados e gravações
do diário comparadas com dados NTFS analisados de forma independente. Consulte o repositório do analisador para ver a
cobertura detalhada e as diferenças conhecidas.

O Juntei adiciona testes de CLI que comparam cada campo CSV com o JSON e o JSONL no volume de teste público do go-ntfs
(2.129 registros), verificam os dados de redo/undo separados e os comprimentos dos payloads brutos, testam logs danificados,
recusam a sobrescrita de evidências e impedem que os caminhos atuais do MFT sigam números de registro reutilizados.
Os casos de regressão distinguem os registros do MFT afetados do atributo aberto pelo qual eles são gravados,
verificam os metadados planos de entradas de diretório/atributos de dados e impedem o pareamento de MFT entre volumes.
Os testes de `auto-parse` com vários volumes abrangem arquivos CSV separados e um único arquivo JSONL com procedência.

O comando também foi verificado em seis logs do Windows coletados de forma privada: todos os 708.465 registros CSV
exportados coincidiram campo a campo com o JSONL, e as exportações de `auto-parse` por host mantiveram as mesmas contagens
de registros. As evidências de origem não são distribuídas com o repositório. Essas verificações confirmam a consistência
da saída, não que toda transação registrada represente uma ação do usuário concluída.

## Prefetch { #prefetch }

O [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) é um fork mantido do
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) da ForensicRS. Sua saída decodificada foi
comparada com a [libscca](https://github.com/libyal/libscca), a implementação independente de Prefetch
da libyal.

## Diário USN { #usn-journal }

O crate usn do [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) é um fork mantido
do [Sootmark usn](https://github.com/Sootmark/usn). Em seis diários reais, os caminhos do pai do Juntei,
que levam em conta o tempo, coincidiram com os de um analisador independente sempre que esse analisador resolveu um caminho. O Juntei
também resolveu 4.321 registros que a comparação deixou sem resolução. Seu carver não produziu nenhum registro
falso nos testes com 880 MB de dados do `$MFT`.

## Índices de diretórios ($I30) { #directory-indexes-i30 }

O módulo `i30` do crate mft do [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) lê os registros INDX
e seu slack. Em dois fluxos `$I30` de um volume criado com o ntfs-3g (o
`test_files/i30/` do Juntei) e em três amostras públicas de `$I30` gravadas pelo Windows, suas entradas em uso e no
slack coincidiram com as do [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) de Maxim Suhanov, com os
mesmos nomes e horários. Quando o dfir_ntfs relata uma referência ao pai acima de 32 bits (bytes que sobrescreveram
o pai), o Juntei deixa o pai vazio.

## Setor de inicialização ($Boot) { #boot-sector-boot }

O crate boot do [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) é de autoria própria da Yamato
Security. Nos cinco arquivos `$Boot` públicos do DFIR Artifact Museum (Windows 10, 11 e
Server 2012 R2), todos os campos coincidiram com os de um analisador de referência. Em volumes criados com `mkntfs`, com setores de 512,
1.024 e 4.096 bytes e clusters de 4 KiB a 64 KiB, os tamanhos, os clusters do `$MFT` e do `$MFTMirr`
e os números de série coincidiram com o `fsstat` do The Sleuth Kit. O `$Boot` do Windows 11 do Museum
tem o número de série do volume registrado pelo arquivo LNK do mesmo host, e os cinco arquivos têm o
mesmo código de inicialização.

## Descritores de segurança ($Secure:$SDS) { #security-descriptors-securesds }

O crate secure do [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) é de autoria própria da Yamato
Security. Nos cinco fluxos `$SDS` do Museum (7.808 descritores, 60.781 ACEs), os
IDs de segurança, hashes, deslocamentos, proprietários, grupos, flags de controle e contagens de ACEs coincidiram com os de um analisador
de referência, e o tipo, os flags, a máscara, o SID e os tipos de objeto de cada ACE coincidiram com o `dissect.ntfs`. Cada
hash registrado é o do seu descritor, cada cópia espelhada coincide com sua entrada, e todo descritor tem
SDDL, incluindo as condições de suas 3.384 ACEs de callback.

## SRUM { #srum }

O [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) se baseia nos crates
[ese](https://github.com/Sootmark/ese) e [srum](https://github.com/Sootmark/srum) do Sootmark. Em
11 bancos de dados reais, todas as colunas das 45.255 entradas emitidas coincidiram com uma decodificação independente dos
valores expostos pelo `dissect.esedb`.

## Atalhos LNK { #lnk-shortcuts }

O [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) é um fork mantido do crate
[shell](https://github.com/Sootmark/shell) do Sootmark. Ele foi comparado em 1.118 atalhos
públicos. Das 30 colunas CSV em comum com o analisador de referência, 16 valores individuais diferiram:
13 caminhos de destino envolvendo itens danificados, truncados, de espaço reservado, MTP ou do Painel de Controle; duas
listas de blocos de dados extras envolvendo blocos desconhecidos ou truncados; e uma referência MFT que o
analisador de referência leu a partir de bytes diferentes. O Juntei relata as estruturas danificadas em vez de
normalizá-las silenciosamente.

## Jump Lists { #jump-lists }

O [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) usa o trabalho do Sootmark sobre
arquivos compostos e Jump Lists, com os links incorporados decodificados pelo `ys-lnk-parser`. Em 77 Jump Lists
públicas, todas as 13.057 células de comparação em comum — incluindo carimbos de data/hora com precisão de 100 ns — coincidiram com o
analisador de referência. Seis arquivos que a referência recusou (listas personalizadas vazias, um rodapé ausente ou um arquivo
que não era uma Jump List) foram decodificados ou relatados explicitamente pelo Juntei.

## ShimCache { #shimcache }

O [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) é um fork mantido
do [shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) da
Security Ronin, com suporte a hives do Registro e
logs de transação. Em 20 hives `SYSTEM` públicos (24 conjuntos de controles e 7.290 entradas), além de 17
valores AppCompatCache avulsos, todas as entradas coincidiram com o caminho, o horário, o flag de execução, os flags e a
posição emitidos pelo analisador de referência.

## Quarentena do Microsoft Defender { #microsoft-defender-quarantine }

O [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) é um
fork mantido do crate
[defender](https://github.com/Sootmark/defender) do Sootmark. Uma pasta de quarentena gravada no formato do Defender
pelas ferramentas de teste do analisador é decodificada de forma consistente pelo leitor de quarentena independente do
`dissect.target`.

## Artefatos derivados do Registro { #registry-derived-artifacts }

O [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) é comparado com o
[yarp](https://github.com/msuhanov/yarp) quanto a chaves e valores alocados, reprodução de transações de hives sujos,
recuperação de células livres, slack de célula, dados além dos bins do hive e registros removidos ou substituídos pela reprodução.
As comparações específicas de artefatos incluem:

- serviços, valores e ações de falha de hives `SYSTEM` do Windows 7 e de um Windows 11 sujo;
- RecentDocs decodificado com o yarp e a libfwsi em nove hives `NTUSER.DAT` públicos;
- tarefas agendadas decodificadas de forma independente com o yarp em sete hives `SOFTWARE` públicos, abrangendo
  Windows 7–11, Server 2012 R2 e Server 2022; e
- estruturas de contas do SAM comparadas com o `dissect.target` em sete hives `SAM` públicos.

Consulte os [Agradecimentos](../acknowledgements.md) para ver a lista completa de ferramentas de referência, pesquisas sobre
formatos, especificações e fontes de evidências públicas.
