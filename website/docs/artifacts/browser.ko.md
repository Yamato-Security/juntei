# 웹 브라우저 { #web-browsers }

## 브라우저 데이터베이스란 { #what-the-browser-databases-are }

브라우저는 방문 기록, 다운로드, 쿠키, 양식 입력 항목을 각 사용자 프로필의 SQLite 데이터베이스에 저장합니다.
Chromium 기반 브라우저(Chrome, Edge, Brave, Opera, Vivaldi)와 Firefox 기반 브라우저는 각각 고유한 파일과
테이블을 사용하지만, 기록하는 활동의 종류는 같습니다.

- **방문 기록**: 방문한 페이지. 각 방문에는 그 시각, 브라우저가 해당 페이지에 도달한 방식(직접 입력, 링크,
  리디렉션, 북마크), 그리고 Chromium의 경우 페이지가 전면에 표시되어 있던 시간이 기록됩니다.
- **다운로드**: 파일을 가져온 URL(및 그 과정의 리디렉션), 저장된 위치, 크기, 다운로드의 시작 및 종료 시각,
  완료 여부.
- **쿠키**: 쿠키를 설정한 사이트, 그리고 쿠키가 설정된 시각, 마지막으로 전송된 시각, 만료 시각.
- **양식 기록**: 웹 양식과 검색 상자에 입력한 값, 입력 횟수, 처음과 마지막 입력 시각.

브라우저 활동은 사용자가 무엇을 했는지 보여 주는 가장 직접적인 기록인 경우가 많습니다. 사용자가 따라간 피싱
페이지, 다운로드한 도구, 사용한 웹메일, 입력한 검색어 등이 여기에 남습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 브라우저 프로필이 기록된 시각에 해당 URL을 방문한 것을
기록했거나, 다운로드를 기록된 경로에 저장했습니다. 데이터베이스는 누가 키보드 앞에 있었는지는 알려 주지
않습니다.

## 위치와 수집 { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>`은 `Default`, `Profile 1`, `Profile 2` 등입니다. Chromium은 버전 96에서 `Cookies`를 `Network`로
옮겼으며, 이전 버전에서는 `History` 옆에 있습니다. Firefox는 버전 25까지 다운로드를 `downloads.sqlite`에
저장했고, 이후 버전에서는 `places.sqlite`에 저장합니다.

프로필의 모든 파일을 수집하고, 각 데이터베이스는 그 옆에 있는 같은 이름의 파일과 함께 수집하십시오.
`places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal` 등입니다.

!!! warning "미리 쓰기 로그"
    Firefox는 데이터베이스를 미리 쓰기 로그(WAL) 모드로 운영합니다. 최신 변경 사항은 브라우저가 이를
    데이터베이스에 복사할 때(체크포인트)까지 `-wal` 파일에만 존재합니다. Juntei는 브라우저와 마찬가지로 각
    데이터베이스를 그 `-wal` 파일과 함께 읽습니다. 이 파일이 없으면 최근의 방문, 쿠키, 양식 항목이 누락되고,
    로그에서 삭제된 항목이 여전히 남아 있는 항목으로 읽힙니다.

## 삭제된 항목 { #deleted-entries }

브라우저가 방문 기록을 삭제하면 행은 테이블에서 사라지지만, 그 레코드는 데이터베이스의 여유 공간이나
프리리스트, 또는 데이터베이스 페이지의 이전 버전에 남아 있을 수 있습니다. 이전 버전은 `-wal` 파일 안에, 또는
`-wal`이 대체한 페이지로서 데이터베이스 파일 안에 있습니다. Juntei는 이들로부터 삭제된 방문, 페이지, 다운로드,
양식 항목을 `Status`가 `Deleted`인 행으로 복구합니다.

- `Confidence`는 레코드와 그 테이블의 대응이 얼마나 확실한지를 나타냅니다. 모든 값이 해당 열의 유형과 일치하는
  온전한 레코드는 `High`, 손상된 레코드나 여러 테이블에 들어맞는 레코드는 `Medium` 또는 `Low`입니다.
- `RecoveredFrom`은 레코드가 있던 위치를 나타냅니다. 페이지 내 영역(셀, 프리블록, 미할당 공간, 빈 페이지),
  어느 페이지의 어느 버전인지, 그리고 페이지 내 레코드의 오프셋입니다. WAL 프레임은 0부터 셉니다.
- `LostColumns`는 손상된 레코드에 더 이상 남아 있지 않은 열을 나열합니다. 해당 필드는 비어 있습니다.

브라우저는 SQLite의 `secure_delete`를 사용하여 삭제하므로, 데이터베이스 내에서 해제된 공간은 0으로
덮어써집니다. 따라서 복구할 수 있는 데이터는 대부분 이전 페이지 버전에 있습니다. `-wal` 파일이 아직 보관하고
있는 버전과, 다음 체크포인트까지 데이터베이스 파일에 남아 있는, `-wal`이 대체한 페이지의 사본입니다. `-wal`
파일은 데이터베이스와 함께, 브라우저가 다시 실행되기 전에 복사하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 시각은 UTC입니다. Chromium과 Firefox는 마이크로초 단위로 기록하며, Chromium의 양식 기록은 초 단위입니다.
- 방문했다고 해서 사용자가 입력하거나 클릭했다는 뜻은 아닙니다. 리디렉션, 프레임, 다른 프로그램이 연 페이지도
  방문입니다. `Transition`, `Typed`, `Hidden`을 확인하십시오.
- Chromium은 쿠키 값을 암호화하므로 해당 쿠키의 `Value`는 비어 있습니다. Juntei는 복호화하지 않습니다.
- `Removed` 양식 기록 행은 Firefox의 `moz_deleted_formhistory`에서 가져온 것으로, 이 테이블은 Android용
  Firefox가 Sync를 위해 채웁니다. 데스크톱 Firefox에서는 보통 비어 있습니다.
- 신뢰도가 `Low`인 삭제된 항목은 다른 레코드의 조각일 수 있습니다. 활성 행 및 다른 증거와 대조하십시오.
- 삭제된 행이 없다고 해서 아무것도 삭제되지 않았다는 뜻은 아닙니다. 0으로 덮어쓴 공간과 체크포인트 이후에는
  복구할 것이 남지 않습니다. 삭제된 쿠키는 아직 복구되지 않습니다.
- 데이터베이스 내용으로 브라우저 계열을 판별합니다. 호환되는 프로필 경로가 있으면 `Browser`를 Chrome이나 Firefox 등으로 구체화하지만, 모순되는 경로가 계열을 바꾸지는 않습니다. 그 외에는 `Chromium-based`, `Firefox-based`, `Safari`이며 판별 불가 시 `Unknown`입니다. `User`와 `Profile`은 경로에서 가져오므로 복사 시 정보가 사라질 수 있습니다.
- 롤백 저널(`-journal`)은 읽지 않으며, Internet Explorer와 레거시 Edge의 `WebCacheV01.dat`도 읽지 않습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d`는 파일 이름으로 파일을 찾은 뒤 각 파일을 그 테이블로 식별하므로, 다른 프로그램의 `History` 파일은
건너뜁니다. `-f`는 이름과 관계없이 데이터베이스를 읽습니다.

## 출력 테이블 { #output-tables }

CSV는 네 개의 파일을 생성하며, 행이 없으면 헤더만 있는 파일이 됩니다. JSON 및 JSONL 행에는 `SourceFile` 다음에
`Table`(`History`, `Downloads`, `Cookies` 또는 `FormHistory`)이 포함됩니다.

| CSV 파일 | 내용 |
|---|---|
| `Browser-History.csv` | 활성 및 삭제된 방문. URL이 어떤 활성 페이지에도 없는 삭제된 페이지. |
| `Browser-Downloads.csv` | 활성 및 삭제된 다운로드. |
| `Browser-Cookies.csv` | 쿠키. |
| `Browser-FormHistory.csv` | 활성 및 삭제된 양식 항목. Firefox의 제거 레코드. |

## 필드 { #fields }

모든 테이블은 다음 필드로 시작합니다.

| 필드 | 해석 |
|---|---|
| `SourceFile` | 행을 읽어 온 데이터베이스. |
| `User` | 경로상의 프로필 소유자인 Windows 사용자(`Users` 다음 폴더). |
| `Browser` | 내용으로 판별한 브라우저 계열. 호환되는 프로필 경로로 제품명을 구체화합니다. |
| `Profile` | 브라우저 프로필. 즉, 데이터베이스가 있는 폴더. |
| `Status` | `Live`, `Deleted`(복구됨) 또는 `Removed`(Firefox의 제거 레코드). |

### 방문 기록 { #history }

| 필드 | 해석 |
|---|---|
| `RecordType` | `Visit`, 또는 방문까지 사라졌을 수 있는 삭제된 페이지의 경우 `Page`. |
| `VisitTime` | 방문이 시작된 시각(UTC). |
| `URL`, `Title` | 페이지와 그 제목(마지막으로 확인된 값). |
| `Transition` | 브라우저가 해당 페이지에 도달한 방식. Chromium: 핵심 유형 뒤에 한정자가 이어짐(`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD` 등과 `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR` 등). Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | 페이지가 전면에 있었던 시간(초)(Chromium. 측정되지 않은 경우 0). |
| `VisitCount` | 해당 페이지의 총 방문 횟수. |
| `Typed`, `TypedCount` | URL을 직접 입력한 적이 있는지 여부와 그 횟수(Chromium). |
| `Hidden` | 방문 기록 목록에서 숨겨진 항목: 하위 프레임, 리디렉션 출발지, 오류. |
| `Frecency` | 주소 표시줄 제안을 위해 Firefox가 매긴 페이지 순위. |
| `VisitID`, `FromVisitID` | 방문의 행 ID와, 이 방문의 출발점이 된 방문(참조 페이지, 리디렉션의 출발지)의 행 ID. 복구된 레코드에서 행 ID가 손실된 경우 방문의 행 ID는 비어 있습니다. |
| `LastVisitTime` | 삭제된 페이지의 마지막 방문(UTC). |
| `PageSource` | 삭제된 방문의 URL과 제목을 가져온 곳: `Live`(활성 페이지), `Recovered`(삭제된 페이지) 또는 `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | 삭제된 항목이 복구된 방식. [삭제된 항목](#deleted-entries)을 참조하십시오. |

### 다운로드 { #downloads }

| 필드 | 해석 |
|---|---|
| `StartTime`, `EndTime` | 다운로드가 시작되고 종료된 시각(UTC). |
| `URL`, `RedirectChain` | 파일을 가져온 곳과 그 과정의 모든 URL(Chromium. CSV에서는 URL 사이에 ` -> `). |
| `TargetPath`, `CurrentPath` | 파일이 저장된 위치(Firefox에서는 `file://` URI)와, 다운로드 중에 기록되던 위치. |
| `ReceivedBytes`, `TotalBytes` | 수신한 바이트 수와 파일 크기. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` 또는 `Dirty`(맬웨어로 차단됨). |
| `DangerType` | 파일에 대한 Chromium의 판정: 0 위험하지 않음, 1 위험한 파일 유형, 2 위험한 URL, 3 위험한 콘텐츠, 4 위험할 수 있음, 5 흔하지 않음, 6 사용자가 유지함, 7 위험한 호스트, 8 원치 않을 수 있는 프로그램, … |
| `InterruptReason` | 중단된 다운로드가 멈춘 이유(Chromium. 없으면 0). |
| `Referrer`, `TabURL` | 참조 페이지와, 다운로드를 시작한 탭의 페이지. |
| `MimeType`, `Opened` | 서버가 제공한 파일 유형과, 브라우저에서 파일을 열었는지 여부(Chromium). |
| `FileDeletedByUser` | 사용자가 브라우저의 다운로드 목록을 통해 다운로드한 파일을 디스크에서 삭제했는지 여부(Firefox). |
| `DownloadID` | 다운로드의 행 ID. 복구된 레코드에서 손실된 경우 비어 있습니다. |

### 쿠키 { #cookies }

| 필드 | 해석 |
|---|---|
| `Created`, `LastAccessed`, `Expires` | 쿠키가 설정된 시각, 마지막으로 전송된 시각, 만료 시각(UTC. 세션 쿠키는 `Expires`가 비어 있음). |
| `Host`, `Name`, `Path` | 쿠키의 대상 사이트, 쿠키의 이름과 경로. |
| `Value` | 저장된 그대로의 값. Chromium이 암호화한 경우 비어 있음. |
| `Secure`, `HttpOnly`, `Persistent` | HTTPS로만 전송됨, 스크립트에서 숨겨짐, 세션이 끝나도 유지됨(Chromium). |
| `GoogleAnalytics` | Google Analytics 쿠키(`__utma`, `__utmb`, `__utmt`, `__utmz`)를 디코딩한 필드: 방문 시각, 세션, 캠페인 소스와 검색어. |
| `RowID` | 쿠키의 행 ID. |

### 양식 기록 { #form-history }

| 필드 | 해석 |
|---|---|
| `FirstUsed`, `LastUsed` | 값이 처음과 마지막으로 입력된 시각(UTC). |
| `RemovedTime` | Firefox의 제거 레코드에 기록된, 항목이 제거된 시각(UTC). |
| `FieldName`, `Value` | 양식 필드(`email`, `q`, Firefox 검색 표시줄의 경우 `searchbar-history`)와 입력된 내용. |
| `Count` | 값이 입력된 횟수. |
| `GUID` | 제거 레코드가 가리키는, Firefox의 항목 식별자. |
| `Sources` | Firefox가 항목을 저장한 출처(이를 보관하는 버전의 경우. CSV에서는 세로 막대로 구분). |
| `RowID` | 항목의 행 ID. 복구 중 소실되면 빈 값/null입니다. 실제 기록된 0은 `0`으로 유지됩니다. |

## 자동 분석 { #automatic-parsing }

`auto-parse`는 테이블을 `BrowserActivity`에 `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv`, `Browser-FormHistory.csv`로 기록하며(`-t json` 또는 `-t jsonl`을 사용하면 하나의
`Browser.json` 또는 `Browser.jsonl` 파일), `BrowserActivity/Browser-logs.txt`도 함께 생성합니다. 결합된 `-j`
타임라인에는 모든 행이 `Comp`와 `Artifact` 식별자 `browser`와 함께 포함됩니다. `-Z`는 `VisitTime`,
`LastVisitTime`, `StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed`,
`RemovedTime`을 변환합니다.

## 추가 정보 { #further-details }

- [Chromium — 페이지 전환 유형](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — 다운로드 위험 유형](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places 데이터베이스](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — 미리 쓰기 로깅](https://www.sqlite.org/wal.html)
- [Plaso — 브라우저 SQLite 파서](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
