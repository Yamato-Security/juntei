# Sobre o Juntei { #about-juntei }

O Juntei é um analisador de artefatos forenses do Windows escrito em Rust, uma linguagem com segurança de memória. Ele complementa o
[Hayabusa](https://github.com/Yamato-Security/hayabusa): o Hayabusa analisa os logs de eventos do Windows,
enquanto o Juntei analisa artefatos persistentes de sistema de arquivos, Registro, execução, atividade do usuário, uso de recursos
e produtos de segurança.

O mesmo binário funciona no Windows, macOS e Linux. Analistas podem examinar evidências coletadas do Windows e imagens de disco com uma interface consistente, em vez de usar um programa diferente para cada artefato.

## Objetivos de design { #design-goals }

- **Análise defensável:** preservar os valores brutos e relatar danos em vez de inventar dados silenciosamente.
- **Útil para todos:** o uso padrão deve ser fácil para iniciantes, mas também personalizável e completo
  em recursos para profissionais experientes.
- **Binário autônomo:** não requer nenhuma biblioteca de tempo de execução para ser executado. Simplesmente funciona
  de imediato.
- **Extremamente rápido:** tem como objetivo ser o analisador mais rápido sem exigir uma grande quantidade de
  memória.
- **Uso multiplataforma:** analisar evidências do Windows sem exigir uma estação de trabalho Windows.
- **Automação:** CSV para planilhas e ferramentas de linha do tempo; JSON/JSONL para `jq`, scripts e data lakes.
- **Falha segura:** registros malformados são relatados e ignorados sempre que possível.
- **Origem transparente:** o Juntei identifica os analisadores de código aberto e as ferramentas de referência por trás de cada implementação.

## O que o Juntei não faz { #what-juntei-does-not-do }

O Juntei não adquire um disco em funcionamento nem substitui a preservação de evidências, e as únicas imagens de
disco que ele monta são imagens VHDX e E01, somente leitura. Ele
não combina automaticamente todos os artefatos em uma única narrativa do incidente. Ele analisa as
evidências que você fornece e produz registros adequados para investigação e correlação.

O Juntei também não fará a análise por você. É necessário um especialista em DFIR para realizar a análise das evidências.

## Licença { #license }

O Juntei é distribuído sob a
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5), com copyright da Shirofune Security Co., Ltd. Seu código-fonte foi publicado sob a
AGPLv3 antes da versão 1.0.0. As licenças dos crates e de outros códigos e dados de terceiros com os quais ele
é construído estão em
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
Os projetos nos quais o Juntei se baseia e com os quais foi comparado estão listados em
[Agradecimentos](../acknowledgements.md).
