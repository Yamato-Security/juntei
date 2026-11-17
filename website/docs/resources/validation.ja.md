# パーサーの検証 { #parser-validation }

Juntei が維持するパーサー クレートは、公的証拠と独立した実装に対してテストされています。これらの比較は、回帰や解釈の違いを検出するのに役立ちます。パーサーを確実なものにするわけではありません。ソース証拠を保存し、各レコードの問題または損傷フィールドを確認し、結果として得られた発見を別のアーティファクトまたは実装で検証します。

以下のステートメントは、Juntei のメイン README に以前保存されていた検証情報を保存します。現在のテスト フィクスチャと実装の詳細については、リンクされたパーサー リポジトリに従ってください。

## Amcache { #amcache }

Juntei 用途 [ys-amcache-パーサー](https://github.com/Yamato-Security/ys-amcache-parser)、アパッチ-2.0 Security Ronin のフォーク [amcache-フォレンジック](https://github.com/SecurityRonin/amcache-forensic)。その [出力ドキュメント](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md) すべての実行可能ファイル、デバイス、検索結果、および概要フィールドを定義します。

## マスターファイルテーブル { #master-file-table }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) の mft クレートは、Omer Ben-Amram の [mft](https://github.com/omerbenamram/mft) クレートの保守フォークです。 8 つの `$MFT` ファイル (クレートのサンプル、NIST ハッキング ケース、6 つの DFIR Artifact Museum ファイル) 全体で、1,048,733 CSV の行と列が順番に参照パーサーと一致しました。比較では、デフォルトの出力、`--all-times`、`--short-names`、および `--cutoff` を対象としました。タイムスタンプの表示は意図的に異なっていました。

## NTFS トランザクション ログ ($LogFile) { #ntfs-transaction-log-logfile }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) のログファイル クレートは、[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) から維持されます。その検証では、6 つの完全な DFIR Artifact Museum ログ、パブリック バージョン 1.1 および 2.0 フィクスチャ、デコードされた REDO/UNDO メタデータ、および個別に解析された NTFS データと比較されたジャーナル書き込みが対象となります。詳細な範囲と既知の違いについては、パーサー リポジトリを参照してください。

Juntei は、パブリック go-ntfs テスト ボリューム (2,129 レコード) 上のすべての CSV フィールドを JSON および JSONL と比較し、個別の REDO/UNDO データと生のペイロード長を検証し、破損したログをテストし、証拠の上書きを拒否し、最新の状態を維持する CLI テストを追加します。次の再利用レコード番号からの MFT パス。回帰ケースでは、影響を受ける MFT レコードを、そのレコードが書き込むオープン属性から区別し、フラットなディレクトリ エントリ/データ属性メタデータをチェックし、クロスボリューム MFT ペアリングを防ぎます。マルチボリューム `auto-parse` テストは、個別の CSV ファイルと、出所付きの単一の JSONL ファイルを対象としています。

このコマンドは、プライベートに収集された 6 つの Windows ログでもチェックされました。708,465 でエクスポートされたすべての CSV レコードは、フィールドごとに JSONL フィールドと一致し、ホストごとの `auto-parse` エクスポートでは同じレコード数が保持されました。ソース証拠はリポジトリとともに配布されません。これらのチェックでは、出力の一貫性が検証されますが、ログに記録されたすべてのトランザクションが完了したユーザー アクションを表しているかどうかは検証されません。

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) は、ForensicRS の [frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) の保守フォークです。デコードされた出力は、libyal の独立した Prefetch 実装である [libscca](https://github.com/libyal/libscca) と比較されました。

## USN ジャーナル { #usn-journal }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) の usn クレートは、[Sootmark usn](https://github.com/Sootmark/usn) の保守フォークです。 6 つの実際のジャーナルでは、Juntei の時間認識親パスは、パーサーがパスを解決した場合はどこでも、独立したパーサーと一致しました。 Juntei は、比較で未解決のままになっていた 4,321 レコードも解決しました。そのカーバーは、`$MFT` データの 880 MB に対してテストしたときに、誤ったレコードを生成しませんでした。

## ディレクトリインデックス ($I30) { #directory-indexes-i30 }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) の mft クレートの `i30` モジュールは、INDX レコードとそのスラックを読み取ります。 ntfs-3g で構築されたボリュームの 2 つの `$I30` ストリーム (Juntei の `test_files/i30/`) と、Windows で作成された 3 つのパブリック `$I30` サンプルでは、使用中のエントリとスラック中のエントリが Maxim Suhanov のエントリと一致しました。 [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs)、名前と時間が同じです。 dfir_ntfs が 32 ビット (親に上書きされたバイト数) を超えた親参照を報告する場合、Juntei は親を空のままにします。

## ブートセクタ ($Boot) { #boot-sector-boot }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) のブート クレートは Yamato Security 独自のものです。 DFIR Artifact Museum の 5 つの公開 `$Boot` ファイル (Windows 10、11、およびサーバー 2012 R2) では、すべてのフィールドが参照パーサーと一致しました。 512、1,024、および 4,096 バイトのセクターと 4 KiB ～ 64 KiB クラスターを備えた `mkntfs` で作成されたボリュームのサイズ、`$MFT` および `$MFTMirr`クラスターとシリアル番号は The Sleuth Kit の `fsstat` と一致しました。博物館の Windows 11 `$Boot` には、同じホストの LNK ファイルに記録されているボリューム シリアル番号があり、5 つのファイルは同じブート コードを持っています。

## セキュリティ記述子 ($Secure:$SDS) { #security-descriptors-securesds }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) の安全なクレートは Yamato Security 独自のものです。博物館の 5 つの `$SDS` ストリーム (7,808 記述子、60,781 ACE) 全体で、セキュリティ ID、ハッシュ、オフセット、所有者、グループ、制御フラグ、および ACE カウントが参照パーサーと一致し、すべての ACE のタイプ、フラグ、マスク、 SID とオブジェクト タイプが `dissect.ntfs` に一致しました。記録されたすべてのハッシュはその記述子のものであり、すべてのミラー コピーはそのエントリと一致し、すべての記述子には 3,384 コールバック ACE の条件を含む SDDL があります。

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) は、Sootmark の [ese](https://github.com/Sootmark/ese) および [srum](https://github.com/Sootmark/srum) クレートに基づいています。 11 の実データベース全体で、45,255 が出力したエントリのすべての列は、`dissect.esedb` によって公開された値の独立したデコードと一致しました。

## ユーザーアクセスログ { #user-access-logging }

`parse-ese ual` は、同じ ESE リーダーで UAL データベースを読み取ります。16 の実データベース (308 行、うちクライアント行 145 行) 全体で、すべての値が `dissect.esedb` によって公開された値の独立したデコードと一致しました。すべてのクライアント行で、要求数のある日は `InsertDate` の UTC 日に始まって `LastAccess` の UTC 日に終わり、その合計は `TotalAccesses` と一致しました。

## Web ブラウザ { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) は、Sootmark の browser、sqlite、plist クレートの保守フォークで、同じデータベースを対象に別の Rust ライブラリ、および Plaso、Velociraptor (SQLiteHunter)、Hindsight と比較したうえで選定しました。`parse-sqlite browser` は、38 の公開データベースで、各データベースのコピーを `-wal` ファイルとともに読み取った `sqlite3` ライブラリの結果と比較しました。対象は、Plaso のテスト ファイル (Chrome 8 ～ 68、Firefox 2 ～ 118)、Chromium 150 と Firefox 140 で作成したプロファイル、および ys-browser-parser のフォーム入力履歴のテスト データベースです。293 件の閲覧、21 件のダウンロード、692 件の Cookie、41 件のフォーム入力がすべて一致しました。削除済みエントリが復元されたのは、実際にエントリが削除されていた箇所だけでした。すなわち、Firefox 140 の履歴から削除された 2 つのページと 2 件の閲覧 (時刻、URL、タイトルを含む) と、テスト データベースの削除されたフォーム入力です。

## Internet Explorer と従来の Edge (WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` は、[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) を使用して `WebCacheV01.dat` を読み取ります。Plaso の 2 つの公開 WebCache データベース (Internet Explorer 11 のものと、従来の Edge の閲覧履歴、ダウンロード、Cookie を含む Windows 10 のダーティなもの) で、`dissect.database` の ESE リーダーと比較しました。4,156 行すべてが値ごとに一致しました。内訳は、44 のコンテナー、182 件の履歴エントリ、6 件のダウンロード、170 件の Cookie、3,754 件のその他のエントリです。比較側では、ダウンロード レコードを独自にデコードしました。データベースが削除済みとしてフラグを付けているレコードは、どちらからも除外しました。

## Outlook { #outlook }

`parse-file outlook` は、[ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser) を使用して PST、OST、`.msg`、`.oft` ファイルを読み取ります。その PST と OST のリーダーは Microsoft の [outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) のフォークで、`.msg` のリーダーは Microsoft のオープン仕様に基づいて作成されています。

- **PST と OST**: 66 の公開 PST および OST ファイル (Apache Tika、artemis、Aspose.Email のサンプル、DFIR Artifact Museum、Enron、GroupDocs のサンプル、java-libpst、libyal のテスト データ、outlook-pst-rs、pst-extractor) を、libpff とプロパティ単位で比較しました。対象は、すべてのストア、フォルダー、メッセージ、受信者、添付ファイルのすべてのプロパティと、すべての添付ファイルのデータです。324,175 個の値が一致し、説明のつかない差異はありませんでした。残りは、ys-outlook-parser だけが読み取るもの (libpff が読み取りに失敗するメッセージ、受信者、添付ファイル、および libpff が開けないストアの 15,846 個の値と、非表示アイテムの 8,223 個の値)、行の情報では存在しないとされているのに libpff が空またはゼロとして返す受信者のセル (49,077 個)、MAPI と同じ方法で算出したフォルダーのエントリ ID と種類 (2,004 個)、libpff が生データとして返すオブジェクトと埋め込みメッセージ (480 個) です。うち 2 つのファイルは、どちらのリーダーもまったく読み取れません。
- **`.msg` と `.oft`**: 2,404 のメッセージ、受信者、添付ファイルと 41,002 のプロパティを含む 214 の公開ファイルを extract-msg と比較しました。110,114 個の値が一致し、異なる 791 個にはすべて説明がつきます。601 個は extract-msg の時刻の誤り、35 個は extract-msg が誤ったコード ページでデコードする ANSI ファイルに由来し、残りは extract-msg が省略または追加するものです。
- **損傷した入力**: 260,000 を超える変異させた `.msg` ファイル、160,000 を超える変異させた PST、および細工したファイルを、クラッシュ、ハング、メモリの際限ない消費を起こすことなく読み取りました。

## LNK のショートカット { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) は、Sootmark の [shell](https://github.com/Sootmark/shell) クレートの保守フォークです。 1,118 パブリック ショートカット間で比較されました。参照パーサーと共有される 30 CSV 列のうち、16 の個々の値は異なります。破損した、切り捨てられた、プレースホルダー、MTP、またはコントロール パネル項目を含む 13 ターゲット パス。不明なブロックまたは切り捨てられたブロックを含む 2 つの追加データ ブロック リスト。もう 1 つは、参照パーサーが異なるバイトから読み取った MFT 参照です。 Juntei は、構造をサイレントに正規化するのではなく、損傷した構造を報告します。

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) は、`ys-lnk-parser` によってデコードされた埋め込みリンクを使用して、Sootmark の複合ファイルとジャンプ リストの作業を使用します。 77 パブリック Jump Lists 全体で、100 ns までのタイムスタンプを含むすべての 13,057 共有比較セルが参照パーサーと一致しました。参照が拒否した 6 つのファイル (空のカスタム リスト、欠落しているフッター、またはジャンプ リストではないファイル) はデコードされたか、Juntei によって明示的に報告されました。

## ごみ箱 { #recycle-bin }

`parse-file recycle-bin` は Juntei 独自のパーサーです。[rifiuti2](https://github.com/abelcheung/rifiuti2) の公開サンプルでは、rifiuti2 が読み取るすべてのレコードで、インデックス、削除日時 (rifiuti2 の表示どおり秒単位)、削除状態、サイズ、パスが一致しました。対象は 92 レコードで、Windows 95 から Server 2003 の INFO2 ファイル 11 個の 67 レコードと、Windows Vista と Windows 10 の `$I` ファイルのフォルダー 3 つの 25 レコードです。INFO2 の ANSI パスも、rifiuti2 自身のテストが使うコード ページで一致しました。Juntei は rifiuti2 が拒否するもの、つまり両方の `$I` バージョンが混在するフォルダー、パスのない `$I` ヘッダー、切り詰められたレコードも読み取り、それぞれの破損を報告します。Plaso のテスト データにある `$I` ファイルと INFO2 では、Plaso のテストが期待する日時、サイズ、パス、インデックス、ドライブが得られます。Digital Corpora が公開している M57-Jean ディスク イメージ (Windows XP) を `auto-parse --e01` で読み取ると、INFO2 のレコードは rifiuti2 と一致し、その横にある復元対象のファイル `Dc1.jpg` は Juntei が付ける名前と同じでした。

## RDP ビットマップキャッシュ { #rdp-bitmap-cache }

`parse-file rdp-cache` は Juntei 独自のパーサーです。CODE BLUE 2026 CTF のコレクションに含まれる 6 台のホストの 1 つである kofu では、3 つの `Cache????.bin` ファイルにある 1,911 個のタイルのすべてが、[bmc-tools](https://github.com/ANSSI-FR/bmc-tools) が作成する画像とピクセル単位で一致します。ApoorvCTF 2025 の「Phantom Connection」チャレンジで公開されている `Cache0000.bin` の 1,980 個のタイルも同様で、そのうち 105 個は 64 × 48 ピクセルです。bmc-tools はすべてのタイルを幅 64 ピクセルとして読み取るため、画面の右端にある幅の狭いタイルは歪んで出力されます。Juntei は各タイルをそれぞれの幅で読み取ります。

タイルを含む公開の `.bmc` キャッシュは見つかりませんでした。上記のサンプルは、ほかの Windows 10 以降のクライアントと同様に、空の `bcache24.bmc` しか持っていません。そのため `.bmc` の読み取りは、形式どおりに作成したファイルでテストしました。4 種類すべての色深度の非圧縮タイル、小さいタイルの後に以前のタイルの末尾が続くもの、破損したスロット、そして MS-RDPBCGR 3.1.9 のすべてのオーダーを使う、RLE 圧縮されたランダムな 16 ビット タイル 3,000 個です。この 3,000 個では、タイルの最初の行の終わりを越えて続くオーダーはありません。Juntei と bmc-tools は 3,000 個すべてを同じピクセルにデコードします。オーダーが最初の行で始まり、その行を越えて続く場合は、両者の結果が異なります。Juntei は仕様と FreeRDP のデコーダーに従ってオーダー全体で最初の行の規則を使い続けますが、bmc-tools はオーダーの途中で規則を切り替えます。

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) は、Security Ronin の [shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) の保守フォークであり、レジストリ ハイブとトランザクション ログをサポートしています。 20 パブリック `SYSTEM` ハイブ (24 コントロール セットおよび 7,290 エントリ) と 17 スタンドアロン AppCompatCache 値では、すべてのエントリが参照パーサーによって出力されたパス、時間、実行フラグ、フラグ、位置と一致しました。

## Microsoft Defender 隔離 { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) は、Sootmark の [defender](https://github.com/Sootmark/defender) クレートの保守フォークです。パーサーのテスト ツールによって Defender の形式で書き込まれた隔離フォルダーは、独立した `dissect.target` 隔離リーダーによって一貫してデコードされます。

## レジストリ由来のアーティファクト { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) は、割り当てられたキー、値、ダーティ ハイブ トランザクション リプレイ、フリーセル リカバリ、セル スラック、ハイブ ビンを超えたデータ、およびリプレイによって削除または置換されたレコードについて [yarp](https://github.com/msuhanov/yarp) と比較されます。アーティファクト固有の比較には次のものが含まれます。

- Windows 7 およびダーティ Windows 11 `SYSTEM` ハイブからのサービス、値、および失敗アクション。
- RecentDocs は 9 つのパブリック `NTUSER.DAT` ハイブにわたって yarp と libfwsi でデコードされました。
- スケジュールされたタスクは、Windows 7 ～ 11、サーバー 2012 R2、およびサーバー 2022 をカバーする 7 つのパブリック `SOFTWARE` ハイブ全体で yarp を使用して個別にデコードされます。そして
- 7 つのパブリック `SAM` ハイブにおける SAM アカウント構造と `dissect.target` の比較。

参照ツール、フォーマット調査、仕様、および公開証拠ソースの完全なリストについては、[Acknowledgements](../acknowledgements.md) を参照してください。
