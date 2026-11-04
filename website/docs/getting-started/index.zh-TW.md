# 安裝 Juntei { #install-juntei }

## 發行版二進位檔 { #release-binaries }

請從 [Juntei 發行頁面](https://github.com/Yamato-Security/juntei/releases)下載適用於您系統的 zip 檔。
若尚未發布發行版二進位檔，請依照下方說明從原始碼編譯。請將封存檔或原始碼的
commit 與總和檢查碼保存在案件筆記中，以便其他分析人員重現執行結果。

| 系統 | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` 或 `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` 或 `juntei-<version>-lin-aarch64-musl.zip` |
| macOS（Apple 晶片） | `juntei-<version>-mac-aarch64.zip` |
| macOS（Intel） | `juntei-<version>-mac-x64.zip` |

每個 zip 檔只包含以版本與平台命名的二進位檔（例如
`juntei-1.0.0-win-x64.exe`）：Juntei 不需要其他任何檔案，Windows 二進位檔也不需要
Visual C++ 可轉散發套件。musl 版的 Linux 二進位檔採靜態連結，可在任何
發行版上執行；gnu 版則需要較新的 glibc。在 macOS 上，請在
「系統設定 > 隱私權與安全性」中允許下載的二進位檔，或以
`xattr -d com.apple.quarantine juntei-<version>-mac-*` 移除其隔離屬性。

```bash
juntei --version
juntei --help
```

範例中以 `juntei` 代表二進位檔；請將其重新命名，或輸入其完整名稱。

## 從原始碼編譯 { #compile-from-source }

Juntei 目前需要 `Cargo.toml` 中宣告的 Rust 版本（撰寫本頁時為 Rust 1.99）。

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` 會使用 `Cargo.lock` 中所記錄的確切相依套件版本。

## 基本命令形式 { #basic-command-shape }

每個剖析器都接受單一檔案（`-f`）或目錄形式的收集資料（`-d`）：

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

請顯示目前的選項，而不要從報告中複製舊的命令：

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

請繼續閱讀[第一次調查](first-investigation.md)，了解安全的工作流程。
