# Instalar o Juntei { #install-juntei }

## Binários de release { #release-binaries }

Baixe o zip para o seu sistema na
[página de releases do Juntei](https://github.com/Yamato-Security/juntei/releases). Se ainda não houver binário de release
publicado, compile a partir do código-fonte, conforme descrito abaixo. Guarde o arquivo compactado ou o commit do código-fonte e o
checksum junto com suas anotações do caso, para que outro analista possa reproduzir a execução.

| Sistema | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` ou `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` ou `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Cada zip contém apenas o binário, nomeado com a versão e a plataforma (por exemplo,
`juntei-1.0.0-win-x64.exe`): o Juntei não precisa de nenhum outro arquivo, e o binário para Windows não precisa do
Visual C++ Redistributable. Os binários Linux musl são vinculados estaticamente e funcionam em qualquer
distribuição; os gnu precisam de uma glibc recente. No macOS, permita o binário baixado em
Ajustes do Sistema > Privacidade e Segurança ou remova seu atributo de quarentena com
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

Os exemplos usam `juntei` como nome do binário; renomeie-o ou digite seu nome completo.

## Compilar a partir do código-fonte { #compile-from-source }

Atualmente, o Juntei requer a versão do Rust declarada em `Cargo.toml` (Rust 1.99 no momento em que esta
página foi escrita).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` usa exatamente as revisões das dependências registradas em `Cargo.lock`.

## Forma básica dos comandos { #basic-command-shape }

Todo analisador aceita um único arquivo (`-f`) ou uma coleta em diretório (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Exiba as opções atuais em vez de copiar um comando antigo de um relatório:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Continue com [Primeira investigação](first-investigation.md) para um fluxo de trabalho seguro.
