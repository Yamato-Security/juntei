# Junteiをインストールする { #install-juntei }

## バイナリをリリースする { #release-binaries }

オペレーティング システム用のバイナリについては、[Juntei リリース ページ ](https://github.com/Yamato-Security/juntei/releases) を確認してください。リリース バイナリがまだ公開されていない場合は、以下の説明に従ってソースからコンパイルします。別のアナリストが実行を再現できるように、アーカイブまたはソースのコミットとチェックサムをケースメモと一緒に保管してください。

```bash
juntei --version
juntei --help
```

Windows では、実行可能ファイルの名前は `juntei.exe` である可能性があります。例では、読みやすくするために `juntei` を使用しています。

## ソースからコンパイルする { #compile-from-source }

現在、Juntei には、`Cargo.toml` で宣言された Rust バージョンが必要です (このページの作成時点では Rust 1.99)。

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` は、`Cargo.lock` に記録されている正確な依存関係リビジョンを使用します。

## 基本的なコマンドの形 { #basic-command-shape }

すべてのパーサーは、1 つのファイル (`-f`) またはディレクトリ コレクション (`-d`) を受け入れます。

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

レポートから古いコマンドをコピーするのではなく、現在のオプションを表示します。

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

安全なワークフローを実現するには、[First Investigation](first-investigation.md) に進みます。
