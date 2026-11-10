# セキュリティ記述子 (`$Secure:$SDS`) { #security-descriptors-securesds }

## `$SDS`とは { #what-sds-is }

NTFS 上のすべてのファイルとディレクトリには、**セキュリティ記述子** があります。つまり、その所有者、プライマリ グループ、**DACL** (誰がそれを使用して何を行うことができるか)、および**SACL** (監査されるアクセスとその整合性ラベル)。 NTFS は、`$Secure` ファイルの `$SDS` ストリーム (MFT エントリ 9) に個別の各記述子を 1 回保持し、それに **security ID** を与えます。ファイルの `$STANDARD_INFORMATION` には、セキュリティ ID ([`parse-ntfs mft`](mft.md) の `SID` 列) のみが記録されます。 `$SDS` はそれを所有者と ACL に変更します。

`$SDS` は 256 KiB ブロック内にあり、それぞれの後にそのコピー (ミラー) が続きます。各エントリにはその記述子のハッシュがあり、NTFS は既存の記述子 (`$SDH`) を検索するために使用し、セキュリティ ID (`$SII` はインデックスを作成します) を持ちます。

<span class="evidence-strength">この証拠から判断できること:</span> `SID` でこの記述子を参照するファイルには、その `$MFT` レコードの最終書き込み時点で、この所有者と ACL が設定されていました。通常、所有者はファイルを作成したアカウントですが、管理者、インストーラ、所有権を取得するプログラムによって変更される場合があります。

## 保存場所と収集 { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

raw 対応のフォレンジック ツールを使用して収集します。 `:` をファイル名に含めることはできないため、コレクションでは通常、`$Secure_$SDS` という名前が付けられます。同じボリュームの `$MFT` も一緒に収集します。

## 分析時の注意点 { #analyst-cautions }

- 所有者は SID です。既知の SID (`NT AUTHORITY\SYSTEM`、`BUILTIN\Administrators`、`NT SERVICE\TrustedInstaller` など) と、マシンまたはドメインの組み込みアカウント (`-500` `Administrator` など) のみが名前付けされます。その他は、ホストの `SAM` ([SAM アカウント](sam.md))、そのプロファイル リスト ([System Information](system-info.md))、またはドメインを使用して解決します。
- 記述子は、同じ所有者と ACL を持つすべてのファイルで共有され、通常、それらのファイルが削除された後も `$SDS` に残ります。 `$MFT` 内にファイルのない記述子は、それ自体では削除されたファイルではありません。
- ACEはアクセス履歴ではありません。 DACL は、誰がファイルにアクセスできるかを示します。イベント ログやその他のアーティファクトには、誰が実行したかが記録されています。 SACL の監査エントリは、ログに記録された内容ではなく、構成について説明します。
- ACE は、ACL にその順序でリストされます。 Juntei は有効なアクセスを計算しません。拒否エントリ、継承、グループ メンバーシップ、特権、および所有者の暗黙的権限はすべてカウントされます。
- `DACL` および `SACL` は、`icacls` と同様に、それぞれ ACE を書き込みます。 `SDDL` は、サポートされているアクセス許可を 16 進数のマスクと SID としてのアカウントで表します。これは、バイナリ記述子のバイトごとのコピーではありません。表現できない条件付きテキストは空のままになり、診断されます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

概要では、記述子をカウントし、その所有者、それぞれが所有する記述子の数、NULL DACL (まったく制限なし) を持つ記述子、および損傷をリストします。

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 解釈 |
|---|---|
| `SourcePath` | 入力ファイル。 |
| `SecurityID` | セキュリティ ID: 同じボリュームの [`parse-ntfs mft`](mft.md) の `SID` と結合します。 |
| `Hash` | 記述子とともに記録されたハッシュ。 |
| `EntryOffset` | ストリーム内のエントリの場所。 |
| `FromMirror` | `True` は、ミラー ブロックから読み取られたコピーの場合、そのエントリ (その後にリストされている) と異なるもの、または破損したエントリのコピーです。 |
| `Size` | エントリのサイズ (20 バイトのヘッダーを含む)。 |
| `Owner`, `OwnerName` | 所有者の SID、およびよく知られている場合はその名前。 |
| `Group`, `GroupName` | プライマリ グループの SID、およびよく知られている場合はその名前。 Windows はアクセスチェックには使用しません。 |
| `Control` | 記述子の制御フラグ: `SE_DACL_PRESENT`、`SE_DACL_PROTECTED` (継承ブロック)、`SE_DACL_AUTO_INHERITED`、… `|` が結合。 |
| `DACL-ACE-Count`, `DACL` | `; ` で区切られた DACL のエントリ。 DACL がない場合は空です。 `NULL DACL: no restriction` 記述子が全員にすべてを許可する場合。 |
| `SACL-ACE-Count`, `SACL` | SACL のエントリ: 監査エントリ、必須整合性ラベル、プロセス信頼ラベル、およびリソース属性。 |
| `SDDL` | SDDL の記述子全体。内容がデコードされていない場合にのみ空になります。 |
| `Problems` | 記述子のものではないハッシュ、デコードできない部分、タイプが不明な ACE。 |

`DACL` および `SACL` の各エントリは、`icacls` が書き込むのと同じように書き込まれます。アカウント、許可エントリでない場合はタイプ、継承フラグ、権利です。

| パート | 意味 |
|---|---|
| アカウント | 有名な名前、つまり SID。 |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | ACE タイプ (許可エントリではない場合)。必須ラベルにはタイプが表示されません。 |
| `(I)` | 親ディレクトリから継承されます。 |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | ファイルによって継承される (オブジェクト継承)、ディレクトリによって継承される (コンテナ継承)、このオブジェクトではなくディレクトリのみによって継承される (継承のみ)、および 1 レベル下のみに継承される (伝播なし)。 |
| `(SA)`, `(FA)` | アクセスの成功または失敗に関する監査エントリ。 |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | フルコントロール、変更、読み取りと実行、読み取り、書き込み。 |
| `(RD,WD,AD,…)` | それ以外の特定の権限: データの読み取り、データの書き込み、データの追加、拡張属性の読み取りおよび書き込み (`REA`、`WEA`)、実行 (`X`)、子の削除 (`DC`)、属性の読み取りおよび書き込み (`RA`、`WA`)、削除(`DE`)、読み取り制御 (`RC`)、書き込み DAC (`WDAC`)、書き込み所有者 (`WO`)、同期 (`S`)、および一般権限 `GA`、 `GR`、`GW`、`GE`。 `N` はアクセスできません。 |
| `(NW)`, `(NR)`, `(NX)` | 必須ラベルのポリシー: 書き込み禁止、読み取り禁止、実行禁止。 |
| `if (…)` | コールバック ACE の条件 (`(WIN://SYSAPPID Contains "…")` など): エントリは、それが true であるプロセスにのみ適用されます。 Windows は、アプリ コンテナー パッケージへのアクセスを許可するためにそれらを使用します。 |

プロセスの信頼ラベルとリソース属性はマスクを 16 進数で示し、リソース属性はそのクレーム (`("IMAGELOAD",TU,0x0,1)`) を示します。

JSON/JSONL には同じフィールドと名前があり、数値は数値、`null` は空の値、`Problems` はリストとなります。

## 調査戦略 { #investigation-strategy }

1. 同じボリュームの `$MFT` を解析し、その `SID` をこの出力と結合します。各ファイルはその所有者と ACL を取得します。
2. ユーザー アカウント (`S-1-5-21-…`) である所有者を確認します。疑わしいアカウントがどのファイルを所有しているか、またどのアカウントがシステム ディレクトリ内のファイルを所有しているかを調べます。
3. プログラムが実行またはロードされるディレクトリで、寛容な ACL を探します: `Everyone`、`Authenticated Users`、または `User` (`(F)`、`(M)` または `(W)`、および NULL DACL)。
4. 1 つのファイルのみで使用される記述子、または通常とは異なるファイルのセットで使用される記述子は、通常のインストール以外で作成または変更されたファイルを示す可能性があります。
5. セキュリティ イベント ログ エントリ (4670、4663 と SACL) および [USN Journal](usn-journal.md) の `SECURITY_CHANGE` レコードを使用して、所有権とアクセス許可の変更を確認します。

## さらに詳しい情報 { #further-details }

- [Microsoft: セキュリティ記述子](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft オープン仕様: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (セキュリティ記述子、ACE、条件式)
- [Microsoft: セキュリティ記述子文字列形式 (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS 形式のドキュメント](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`secure` クレート)

標準出力の構造化出力 (`-o` なしの `-t`) では、ストリームレベルの回復警告が標準エラー出力に送られるため、パイプは破損または回復された領域を隠すことなく、読み取り可能な CSV/JSON/JSONL を保持します。
