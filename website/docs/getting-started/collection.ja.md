# 証拠の収集 { #collecting-evidence }

Juntei は、Windows からすでに収集されたファイルを解析します。信頼できる取得ツールを使用し、可能な場合は元のメタデータを保存します。

## 共通の場所 { #common-locations }

| アーティファクト | 典型的な場所 |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS ボリューム メタデータ ファイル `C:\$MFT` |
| `$LogFile` | NTFS ボリューム メタデータ ファイル `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN ジャーナル | NTFS メタデータ ストリーム `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| ユーザーアクセスログ (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| LNK / 最近 | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` およびその他のユーザー/システムの場所 |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` および `CustomDestinations` |
| ShimCache | `SYSTEM` ハイブ: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender 隔離 | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| マシンレジストリ | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| ユーザーレジストリ | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## 一緒に移動する必要があるファイル { #files-that-must-travel-together }

- 隣接する `.LOG`、`.LOG1`、および `.LOG2` ファイルを含むレジストリ ハイブを収集します。 Juntei はレジストリ コマンドのログを再生でき、ハイブがダーティな場合は ShimCache を再生できます。
- SRUM の `SRUDB.dat` を一貫して収集します。現在、Juntei はダーティな ESE データベースを報告しますが、`SRU*.log` は再生しません。これらのログにのみ存在するレコードは利用できません。
- 可能な場合は、`$Extend/$J` の隣に `$MFT` を収集します。 USN レコードには、完全なパスではなく、親ファイルの参照が保存されます。 MFT はパスの解像度を向上させます。
- `$LogFile` を、同じボリュームの `$MFT` とできるだけ近いタイミングで収集します。 `$LogFile` は急速にラップします。後の収集では、以前の REDO/UNDO レコードが完全に失われる可能性があります。
- 一致する `SOFTWARE` ハイブを含む SRUM を収集し、アカウントと Wi-Fi プロファイル名を解決します。
- ユーザーアクセスログの `Sum` ディレクトリ全体を収集します。`SystemIdentity.mdb` が役割の名前を定め、年ごとの `{GUID}.mdb` データベースを列挙しています。Juntei は、UAL のトランザクション ログ `Svc*.log` も再生しません。
- Defender `Quarantine` ツリー全体を収集して、エントリ、ResourceData、およびリソースを照合できるようにします。

## ライブシステムに関する注意事項 { #live-system-cautions }

開いているファイルはコピー中に変更される可能性があります。レジストリ ハイブと ESE データベースが内部的にダーティである可能性があります。 ShimCache は、シャットダウンするまで最新のメモリ内エントリをフラッシュしない可能性があります。通常のファイル コピーよりも、ボリューム スナップショットまたは RAW アクセスを使用するフォレンジック収集ツールの方が適しています。

## 安全性 { #safety }

証拠には、マルウェア、エクスプロイト サンプル、悪意のある LNK ファイル、スプレッドシートの数式テキストが含まれる可能性があります。分離されたシステムで分析し、復元されたストリームを実行せず、危険なコンテンツを処理する場合にのみ `--extract` を使用してください。変更されていないソースは別に保存してください。
