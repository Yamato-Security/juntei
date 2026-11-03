# 一般的な出力パターン { #common-output-patterns }

## 1 つのアーティファクトまたはコレクション { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` は、そのコマンドに適切なファイルを再帰的に検索します。ソースパス列は、結合された出力内のホストとアーティファクトを区別します。

## CSVを保存 { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

一部のパーサーは複数の CSV テーブルを出力するため、ディレクトリが必要です。

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## ストリーム JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

メッセージと進行状況は標準エラーに送られます。標準出力は引き続き解析可能です。

## 意図的に上書きする { #overwrite-intentionally }

Juntei は、デフォルトで既存の出力の上書きを拒否します。ターゲットを確認し、重要な以前の結果を保存してから、`-C` / `--clobber` を追加します。

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## 静かなモードとノーカラーモード { #quiet-and-no-color-modes }

自動化には `-q` を使用し、端末のカラー コードが不適切な場合には `-K` を使用します。

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## ケースとパス { #case-and-paths }

シェルの必要に応じて、スペース、`$`、シェル メタ文字、またはバックスラッシュを含む Windows パスを引用符で囲みます。 Bash のようなシェルでは、一重引用符によって `$MFT` および `$J` が変数になるのを防ぎます。

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
