# 애플리케이션 기록 { #application-history }

## 도구가 남기는 흔적 { #what-tools-leave-behind }

많은 서드파티 도구는 설정과 기록을 사용자의 `NTUSER.DAT`에 보관합니다. 그중 일부는 공격자가 사용하거나
데이터를 반출하기 전에 준비(staging)하는 데 사용하는 도구입니다. Sysinternals 도구, 압축 프로그램,
클라우드 동기화 클라이언트 및 원격 액세스 도구가 여기에 해당합니다. Juntei는 다음을 읽습니다.

| 애플리케이션 | `HKCU\Software` 아래의 키 | 보여 주는 내용 |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | 사용자가 라이선스에 동의한(`EulaAccepted`) 각 도구: PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | 연 압축 파일, 대화 상자에 입력한 압축 파일 이름과 압축 해제 폴더 |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | 만든 압축 파일, 압축 해제 폴더, 파일 관리자에서 탐색하고 복사 대상으로 사용한 폴더 |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | File 메뉴에 표시된 압축 파일, 압축 해제 폴더, 최근 압축 파일 |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | 각 계정의 이메일 주소, 로컬 폴더 및 이름, 동기화된 라이브러리(`ScopeIdToMountPointPathCache`)와 폴더(`Tenants\<tenant>`) |
| Adobe Acrobat 및 Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | 최근에 연 PDF 파일과 폴더(파일 이름, 크기 및 날짜 포함) |
| TeamViewer | `TeamViewer` (및 이전 버전의 `Version<N>`) | 모든 값: 계정 및 사용자 이름, 파일 전송 시작 폴더(`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | 모든 값: 정리 대상으로 설정된 항목, `UpdateKey`, … |

<span class="evidence-strength">가장 타당한 결론:</span> 이 사용자가 도구를 실행했거나(또는 라이선스에
동의했거나), 도구가 이 압축 파일과 폴더를 열었거나 만들었거나 압축 해제했거나, 또는 이 클라우드 계정이
해당 사용자용으로 설정되었습니다. 대부분의 기록에는 항목별 시간이 없습니다. 가장 최근 항목만 목록 키의
마지막 쓰기 시간을 갖습니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

모든 사용자의 `NTUSER.DAT`를 트랜잭션 로그와 함께 수집하고, 사용자 SID 확인을 위해 시스템의
`SOFTWARE`도 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **Sysinternals EULA 동의.** `FirstUsedUTC`는 키의 유일한 값이 `1`로 설정된 읽을 수 있는
  `EulaAccepted`일 때, 키의 마지막 쓰기 시간으로부터 추정한 최초 사용 시간입니다. 거부되었거나 손상된
  EULA에는 이 추정값이 부여되지 않습니다. 레지스트리 설정은 미리 구성되거나 복사되거나 편집될 수도
  있으므로, 실행 여부와 그 시간을 다른 아티팩트로 확인하십시오.
  해당 위치에 설정을 저장하는 도구(Process Explorer, Autoruns, …)는 나중에 키를 다시 씁니다.
  이 경우 `FirstUsedUTC`는 비어 있으며 `KeyLastWrittenUTC`는 마지막 설정 변경 시간일 뿐입니다.
  `-accepteula`로 실행하거나 다른 사용자로 실행한 도구(PsExec의 원격 측은 SYSTEM으로 실행됨)는
  해당 사용자의 키에 기록합니다.
- **시간이 아닌 위치.** 목록 항목에는 순서가 있지만(`MruPosition` 0이 가장 최근), 목록 키의 마지막
  쓰기 시간은 가장 최근 항목의 날짜만 알려 줍니다. 이전 항목은 그보다 앞선, 알 수 없는 시점에 사용되었습니다.
- **입력된 이름.** WinRAR의 `ArcName`은 압축 파일을 만들 때 입력한 이름으로, 폴더가 없는 파일 이름인
  경우가 많습니다.
- **Adobe 경로**는 Adobe가 기록한 형식 그대로 보존됩니다(`/C/Users/alice/a.pdf`는
  `C:\Users\alice\a.pdf`입니다). `Details`에 있는 `sDate`는 PDF 날짜 문자열이며 기록된 그대로 보존됩니다.
- **CCleaner의 `UpdateKey`는** 시간대가 없는 로컬 시간이며, 기록된 그대로 보존됩니다.
- **TeamViewer.** 연결 로그는 레지스트리 값이 아니라 파일(`Connections_incoming.txt`,
  `TeamViewer*_Logfile.log`)입니다. 이 파일들도 수집하십시오.
- **부재.** 사용자와 도구는 이러한 기록을 지울 수 있습니다(CCleaner 자체도 지울 수 있습니다).

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

요약에는 가장 최근의 Sysinternals 최초 사용 추정값, 각 목록에서 마지막으로 사용된 압축 파일과 폴더,
그리고 OneDrive 계정이 표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트와 `NTUSER.DAT`입니다. |
| `User`, `UserSID` | 프로필 폴더의 이름과, 시스템의 `ProfileList`에서 얻은 해당 SID입니다. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | Sysinternals 도구, 목록(`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), OneDrive 계정 또는 동기화된 폴더의 키, 또는 값의 이름입니다. |
| `MRU-Position` | 목록 내 항목의 위치이며, 0이 가장 최근입니다. |
| `Path` | 압축 파일, 폴더 또는 문서입니다. |
| `Value` | `EulaAccepted`, OneDrive 계정의 이메일 주소 또는 동기화된 라이브러리의 ID, TeamViewer 또는 CCleaner 값입니다. |
| `FirstUsedUTC` | EULA 동의로부터 추정한 최초 사용 시간입니다(위의 주의 사항 참조). |
| `Details` | OneDrive 계정의 `DisplayName`, `UserCID`, `ConfiguredTenantId`, …, Adobe 문서의 `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate`입니다. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | 레코드의 출처와 해당 키가 마지막으로 변경된 시간입니다. |
| `DataRaw` | 디코딩되지 않는 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 디코딩하거나 읽을 수 없었던 데이터입니다. |

## 조사 전략 { #investigation-strategy }

1. 일반 사용자가 실행하지 않는 Sysinternals 도구(PsExec, ProcDump, PsLoggedOn, SDelete)와 그 EULA
   타임스탬프를 찾으십시오.
2. 인시던트 전후의 압축 파일 이름과 압축 해제 폴더를 검토하십시오. 데이터가 시스템을 떠나기 전에
   `ProgramData`, `Temp` 또는 사용자 폴더에서 만들어진 압축 파일이 그 예입니다.
3. 조직의 계정이 아닌 OneDrive 계정과 TeamViewer 계정 이름 및 파일 전송 폴더를 확인하십시오.
4. Prefetch, Amcache, BAM 및 UserAssist(도구 실행), `$MFT`와 USN 저널(압축 파일), LNK 파일과
   Jump Lists(연 문서), 그리고 이벤트 로그와 상관 분석하십시오.

## 추가 정보 { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
