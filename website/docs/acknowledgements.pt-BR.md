# Agradecimentos { #acknowledgements }

O Juntei existe porque as comunidades de DFIR e de código aberto publicaram analisadores, pesquisas
sobre formatos, saídas de referência, especificações e evidências de teste. Os projetos abaixo
orientaram a implementação, a validação, a nomenclatura de campos ou a saída esperada pelo analista.
Constar desta lista não significa que o código do projeto esteja incorporado ao Juntei; consulte
cada projeto para conhecer sua licença.

## Bases de analisadores mantidas pela Yamato Security { #parser-foundations-maintained-by-yamato-security }

| Área do Juntei | Analisador mantido | Upstream/base |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| Diário USN | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| `$LogFile` do NTFS | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Índices de diretórios (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`, módulo `i30`) | Pesquisa sobre slack de índice creditada abaixo |
| `$Boot`, `$Secure:$SDS` do NTFS | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crates `boot` e `secure`) | MS-DTYP da Microsoft e a pesquisa sobre NTFS creditada abaixo |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) e [ese](https://github.com/Sootmark/ese) |
| Navegadores web / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite) e [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Trabalho do Sootmark shell sobre arquivos compostos e Jump Lists |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| Imagens de disco VHDX (`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) e [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), expostos via [fuser](https://github.com/cberner/fuser) | MS-VHDX da Microsoft |
| E01 / Ex01 (`mount`, `auto-parse --e01`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) | [ewf-image](https://github.com/ebrig/ewf-image) (Apache-2.0); [libewf](https://github.com/libyal/libewf) (EWF1/EWF2) |
| Registro | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Pesquisa sobre o formato do Registro e recuperação creditada abaixo |

## Ferramentas de referência e validação independente { #reference-tools-and-independent-validation }

- O [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), da Hexastrike,
  forneceu o catálogo de verificações de persistência, a lista de permissões padrão, o snapshot do
  LOLBAS e o comportamento de referência para o comando nativo em Rust `persistence-hunter`. Sua
  licença MIT e seu aviso de copyright são mantidos em `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`.
  O [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) original inspirou o projeto
  em Python; consulte os [agradecimentos do Persistence Hunter](persistence-hunter/acknowledgements.md)
  para conhecer a linhagem completa.

- A **especificação regf** e o **yarp** de Maxim Suhanov orientaram as estruturas do Registro, a
  reprodução de logs de transação e a recuperação de registros excluídos; o **dfir_ntfs**, também
  dele, foi a referência para registros INDX e para a recuperação (carving) em slack de índice.
- O **INDXRipper** de Harel Segev é mais uma referência para a recuperação de nomes de arquivo a
  partir do slack de índice.
- O **dfir_ntfs**, o **LogFileParser** de Joakim Schicht e o **ntfs-linker** da Stroz Friedberg
  orientaram a validação e a interpretação do `$LogFile`; o **go-ntfs** da Velocidex forneceu
  evidências de teste públicas de NTFS.
- Os projetos libyal de Joachim Metz — **libregf**, **libfsntfs**, **libscca**, **libfwsi** e
  **libesedb** — oferecem pesquisa detalhada sobre formatos e referências de análise independentes.
- O **dissect.esedb**, o **dissect.ntfs** e o **dissect.target** da Fox-IT forneceram interpretações
  independentes de SRUM, UAL, `$Secure:$SDS`, SAM e Defender usadas durante a validação.
- O `fsstat` do **The Sleuth Kit** foi a referência para os setores de inicialização NTFS de volumes
  criados com `mkntfs`.
- Algumas das ferramentas de Eric Zimmerman estabeleceram nomes de campos e expectativas dos
  examinadores usados nas comparações.
- O **regipy**, o **python-registry**, o **Notatin** e o **artemis** forneceram implementações do
  Registro, testes ou interpretações de amostras.
- O **LnkParse3**, o **lnk_parser** e o **jumplist_parser** de AbdulRhmanAlfaifi, e o JLECmd forneceram
  comportamento independente de LNK/Jump Lists e casos de teste públicos.
- O **prefetch-forensic** da Security Ronin e o projeto **forensicnomicon** orientaram as observações
  de alto valor do Juntei sobre Prefetch/Amcache/ShimCache.
- O **defender-dump** e o dissect.target oferecem implementações separadas para entender as
  estruturas de quarentena do Defender e o comportamento de extração segura.
- O **Plaso**, o **Chainsaw** e o **Velociraptor** forneceram comparações de analisadores, evidências
  de teste ou comportamento de referência orientado à coleta para vários artefatos.
- O **Hindsight**, da Obsidian Forensics, e as definições do **SQLiteHunter**, da Velocidex, serviram de
  referência para histórico, downloads, cookies e histórico de formulários de navegadores.
- O **rifiuti2**, de Abel Cheung, serviu de referência para os arquivos `$I` e o INFO2 da Lixeira, junto com a
  documentação **dtformats** da libyal sobre os dois formatos.

## Especificações e pesquisas { #specifications-and-research }

Agradecemos à Microsoft por publicar as Open Specifications para Shell Links, Compound Binary Files,
protocolos NTFS/SAM, estruturas de Registro/serviços/tarefas e a documentação de APIs. Agradecemos
também aos autores das pesquisas forenses sobre AppCompatCache, Amcache, SRUM, Prefetch, Registro e
NTFS que tornaram esses artefatos compreensíveis para os profissionais da área.

## Evidências de teste { #test-evidence }

As evidências públicas do
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) de Andrew Rathbun, do NIST CFReDS, do Plaso,
do rifiuti2, do Regipy, do Artemis, de repositórios de teste de analisadores e de outras coleções devidamente
licenciadas tornam possíveis os testes de regressão com dados reais do Windows. O Juntei mantém notas
de atribuição/licença ao lado dos arquivos de teste incluídos.

## Colaboradores { #contributors }

Agradecemos a todos os colaboradores, revisores e relatores de problemas do Juntei e dos analisadores
da Yamato Security, e aos usuários que forneceram casos-limite. Os colaboradores atuais são listados
pelo GitHub na [página de colaboradores do Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors).

## Licença { #license }

O Juntei é distribuído sob a
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5), com copyright da Shirofune Security Co., Ltd. Seu código-fonte foi publicado sob a
AGPLv3 antes da versão 1.0.0. As licenças dos crates e de outros códigos e dados de terceiros com
os quais ele é construído estão em
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
