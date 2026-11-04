# Agradecimentos do Persistence Hunter { #persistence-hunter-acknowledgements }

O comando `persistence-hunter` do Juntei se baseia em duas gerações de caça comunitária à persistência.
Agradecemos aos autores e colaboradores que disponibilizaram suas técnicas, implementações e pesquisas
aos investigadores.

## PyrsistenceSniper: o projeto em Python { #pyrsistencesniper-the-python-project }

O [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), da
[Hexastrike](https://github.com/Hexastrike), é a referência direta deste recurso. O Juntei porta
seu catálogo de 130 verificações para Rust nativo, junto com a lista de permissões padrão, o snapshot do LOLBAS e o
comportamento de classificação. A versão de referência está fixada no
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

O projeto em Python é licenciado sob a MIT. O Juntei mantém seu aviso de copyright e de licença em
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
A implementação do Juntei é executada offline, sem um interpretador Python. Seu tratamento de coletas,
seus relatórios de cobertura e sua interpretação conservadora de evidências incompletas estão documentados no
[guia do comando](../commands/persistence-hunter.md).

## PersistenceSniper: a ferramenta original em PowerShell { #persistencesniper-the-original-powershell-tool }

O [**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) é o projeto original em PowerShell
de caça à persistência que inspirou o PyrsistenceSniper. O PyrsistenceSniper dá crédito a
Federico Lagrasta; o repositório original também dá crédito a Riccardo Ancarani e a outros
colaboradores. Agradecemos ao projeto original e a seus colaboradores por tornarem os mecanismos de persistência
do Windows mais fáceis de investigar.

A referência direta da portabilidade do Juntei é o projeto em Python. A ferramenta original em PowerShell recebe crédito
por essa linhagem. O Juntei também implementa de forma independente os mecanismos propostos por
[hongson11698](https://github.com/hongson11698) no
[PR nº 31 de resolução de COM em tarefas agendadas](https://github.com/last-byte/PersistenceSniper/pull/31)
e no [PR nº 32 de bibliotecas/pastas CLSID do Menu Iniciar](https://github.com/last-byte/PersistenceSniper/pull/32) do projeto original.
Eles complementam o catálogo de referência em Python; isso não significa que todos os recursos para sistemas ativos de qualquer um dos projetos upstream
sejam implementados pelo comando offline do Juntei.

## Pesquisas e ferramentas de apoio { #supporting-research-and-tools }

- O [MITRE ATT&CK](https://attack.mitre.org/) fornece os identificadores de técnicas incluídos no
  catálogo. Um mapeamento de técnica descreve um uso possível de um mecanismo; um valor do Registro
  ou um arquivo correspondente não é, por si só, um ataque confirmado pelo ATT&CK.
- O [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  também é reconhecido pelo PyrsistenceSniper como inspiração para examinar as inicializações automáticas do Windows.
- O [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) documenta binários legítimos do Windows que podem
  ser abusados. Reconhecer um desses binários ajuda a interpretar um comando, mas não torna esse
  comando automaticamente malicioso.

Para as bases de análise de Registro, NTFS, Shell Link, logs de eventos e outras usadas em todo o Juntei,
consulte os [agradecimentos gerais do projeto](../acknowledgements.md).

Volte ao [guia de mecanismos de persistência](index.md).
