# Glossário DFIR { #dfir-glossary }

| Termo | Significado em linguagem simples |
|---|---|
| Alocado (Allocated) | Registro/objeto atualmente em uso, de acordo com o artefato. |
| Fluxo de dados alternativo (Alternate Data Stream, ADS) | Dados NTFS nomeados anexados a um arquivo além do seu conteúdo normal sem nome, como `Zone.Identifier`. |
| Artefato | Dados persistentes produzidos por um sistema operacional/aplicativo que podem responder a perguntas forenses. |
| Carving | Localização de registros pela estrutura/assinatura, sem depender de seu contêiner/índice normal. |
| Conjunto de controles (Control set) | Uma versão da configuração do SYSTEM (`ControlSet001` etc.); `Select\Current` indica o ativo. |
| Corroboração | Evidência independente que sustenta a mesma conclusão. |
| Banco de dados/hive sujo (dirty) | O arquivo primário não estava totalmente sincronizado quando foi coletado; os logs de transação podem conter estado necessário. |
| Droid | Identificador semelhante a GUID do rastreamento de links distribuídos, usado para acompanhar arquivos/volumes ao longo de movimentações. |
| ESE / JET Blue | Formato de banco de dados Extensible Storage Engine da Microsoft, usado pelo SRUM e por outros dados do Windows. |
| FILETIME | Contagem de intervalos de 100 nanossegundos desde 1601-01-01 UTC; um número só faz sentido como horário quando seu esquema assim o define. |
| Referência de arquivo (File reference) | Número da entrada NTFS mais o número de sequência, que identificam uma geração de um registro do MFT. |
| Hive | Arquivo binário que contém uma árvore do Registro do Windows. |
| Registro INDX | Bloco (geralmente de 4 KiB) de um índice de diretório NTFS (`$I30`) que lista os arquivos do diretório; seu slack pode preservar possíveis evidências de nomes de arquivos excluídos ou renomeados. |
| LNK | Shell Link/atalho do Windows que contém um destino e metadados abrangentes. |
| MRU | Ordenação Most Recently Used (usados mais recentemente); geralmente uma ordem relativa, e não um horário para cada item. |
| MFT | Master File Table (Tabela Mestre de Arquivos) do NTFS, que contém os registros e atributos de arquivos/diretórios. |
| Hash do Prefetch | Identificador do Windows usado na nomenclatura dos `.pf`; não é um hash criptográfico do conteúdo. |
| Última gravação da chave do Registro | Horário em que a chave mudou; o Windows normalmente não informa qual valor mudou. |
| Dados residentes (Resident data) | Bytes de um arquivo/fluxo armazenados diretamente em um atributo do MFT, e não em data runs externos. |
| RID | Relative Identifier (identificador relativo), o componente final de conta/grupo sob uma autoridade/um domínio de SID do Windows. |
| SDDL | Security Descriptor Definition Language: um descritor de segurança escrito como texto, como `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`. |
| Descritor de segurança | O proprietário de um arquivo, seu grupo primário, a DACL (quem pode fazer o quê) e a SACL (auditoria e rótulo de integridade). O NTFS mantém cada descritor distinto uma única vez em `$Secure:$SDS`, sob um ID de segurança. |
| SID | Security Identifier (identificador de segurança) do Windows para uma conta, serviço, grupo ou entidade de segurança. |
| Slack | Bytes não utilizados dentro/depois de uma estrutura alocada que podem reter dados antigos. |
| SRUM | Banco de dados do System Resource Usage Monitor sobre o uso de recursos por aplicativos/usuários. |
| Timestomping | Modificação intencional de carimbos de data/hora para enganar a investigação; heurísticas não são prova. |
| Log de transação | Alterações pendentes/confirmadas usadas para recuperar um Registro ou banco de dados a um estado consistente. |
| USN | Update Sequence Number, usado pelos registros do diário de alterações do NTFS e por metadados relacionados. |
| UTC | Tempo Universal Coordenado; base de tempo comum preferida para linhas do tempo entre sistemas. |
| Número de série do volume | Número que o NTFS atribui a um volume quando ele é formatado (`$Boot`); seus 32 bits inferiores são o que os arquivos LNK, Jump Lists e Prefetch registram sobre o volume de um arquivo. |
