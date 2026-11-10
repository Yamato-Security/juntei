# Junteiをインストールする { #install-juntei }

## バイナリをリリースする { #release-binaries }

[Juntei リリース ページ](https://github.com/Yamato-Security/juntei/releases) から、お使いのシステム用の zip をダウンロードしてください。リリース バイナリがまだ公開されていない場合は、以下の説明に従ってソースからコンパイルします。別のアナリストが実行を再現できるように、アーカイブまたはソースのコミットとチェックサムをケースメモと一緒に保管してください。

| システム | zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` または `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` または `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple シリコン) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

各 zip に入っているのは、バージョンとプラットフォーム名の付いたバイナリ (例: `juntei-1.0.0-win-x64.exe`) だけです。Juntei は他のファイルを必要とせず、Windows 版は Visual C++ 再頒布可能パッケージも不要です。Linux の musl 版は静的リンクされており、どのディストリビューションでも動作します。gnu 版には新しい glibc が必要です。macOS では、ダウンロードしたバイナリを「システム設定」>「プライバシーとセキュリティ」で許可するか、`xattr -d com.apple.quarantine juntei-<version>-mac-*` で隔離属性を削除してください。

```bash
juntei --version
juntei --help
```

例ではバイナリを `juntei` と表記しています。名前を変更するか、フルネームで実行してください。

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
