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

## LNK のショートカット { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) は、Sootmark の [shell](https://github.com/Sootmark/shell) クレートの保守フォークです。 1,118 パブリック ショートカット間で比較されました。参照パーサーと共有される 30 CSV 列のうち、16 の個々の値は異なります。破損した、切り捨てられた、プレースホルダー、MTP、またはコントロール パネル項目を含む 13 ターゲット パス。不明なブロックまたは切り捨てられたブロックを含む 2 つの追加データ ブロック リスト。もう 1 つは、参照パーサーが異なるバイトから読み取った MFT 参照です。 Juntei は、構造をサイレントに正規化するのではなく、損傷した構造を報告します。

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) は、`ys-lnk-parser` によってデコードされた埋め込みリンクを使用して、Sootmark の複合ファイルとジャンプ リストの作業を使用します。 77 パブリック Jump Lists 全体で、100 ns までのタイムスタンプを含むすべての 13,057 共有比較セルが参照パーサーと一致しました。参照が拒否した 6 つのファイル (空のカスタム リスト、欠落しているフッター、またはジャンプ リストではないファイル) はデコードされたか、Juntei によって明示的に報告されました。

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
