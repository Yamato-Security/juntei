---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, deusa da forense digital](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> é o analisador multiplataforma da Yamato Security para artefatos forenses do Windows
que <em>não</em> são logs de eventos. Ele transforma as evidências coletadas em registros CSV, JSON e JSONL
consistentes, preservando o contexto de que o investigador precisa para evitar conclusões falsas.
</p>

<div class="jt-cta" markdown>
[Comece aqui :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Conheça os artefatos :material-archive-search:](artifacts/index.md){ .md-button }
[Referência de comandos :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## Do artefato bruto a uma conclusão defensável { #from-raw-artifact-to-defensible-conclusion }

O Juntei pode dizer o que um artefato registra; este site explica o que esse registro **prova e o que
não prova**. Comece por uma página de artefato, mesmo que você nunca tenha feito DFIR no Windows antes.
Cada guia aborda a finalidade, a localização em disco, as ressalvas de retenção e coleta, o comando de análise,
os campos de saída, a interpretação, as correlações e leituras complementares.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __Histórico do sistema de arquivos__

    ---

    Use o `$MFT` e o Diário USN para reconstruir a existência de arquivos, nomes, movimentações, exclusões,
    alterações de metadados e fluxos de dados alternativos.

-   :material-play-circle-outline:{ .lg .middle } __Evidências de programas__

    ---

    O Prefetch pode sustentar fortemente a execução. O Amcache e o ShimCache acrescentam inventário, caminho, hash,
    compatibilidade e contexto histórico — mas exigem interpretação cuidadosa.

-   :material-account-clock:{ .lg .middle } __Atividade do usuário__

    ---

    Arquivos LNK, Jump Lists, RecentDocs e SRUM revelam itens abertos, atividade de aplicativos,
    locais removíveis ou de rede, uso de recursos e contexto histórico do usuário.

-   :material-shield-bug:{ .lg .middle } __Evidências de segurança__

    ---

    A quarentena do Defender pode preservar rótulos de ameaças, caminhos originais, origens de download, hashes,
    proprietários e até recuperar com segurança os bytes em quarentena para análise isolada.

-   :material-database-search:{ .lg .middle } __Estado do Registro__

    ---

    Leia dados do Registro alocados e recuperáveis, reproduza logs de transação e inspecione serviços,
    tarefas agendadas, contas locais e o estado de documentos recentes.

-   :material-link-variant:{ .lg .middle } __Correlacione, não adivinhe__

    ---

    Nenhum artefato isolado conta a história inteira. Os guias mostram como combinar fontes independentes
    em uma linha do tempo sustentada por evidências.

</div>

!!! warning "A saída forense é evidência, não veredito"
    Um caminho que parece suspeito é uma pista. Um horário pode descrever os metadados de um arquivo, e não
    a execução. A ausência de um registro não prova que uma ação não aconteceu. Preserve as evidências de origem,
    registre as versões das ferramentas e os comandos, e corrobore as conclusões importantes.
