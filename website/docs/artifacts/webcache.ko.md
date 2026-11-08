# Internet Explorer와 레거시 Edge { #internet-explorer-and-legacy-edge }

## WebCache 데이터베이스란 { #what-the-webcache-database-is }

`WebCacheV01.dat`는 각 사용자 프로필에 있는 Windows 인터넷 기능(WinINet)의 데이터베이스로, ESE
데이터베이스입니다. Internet Explorer 10과 11은 방문 기록, 다운로드, 캐시, 쿠키를 여기에 저장하며, 레거시
Edge, 즉 Edge가 Chromium 기반으로 바뀌기 전의 Windows 10용 Edge도 마찬가지입니다. 레코드의 종류마다 하나의
컨테이너, 즉 별도의 테이블이 있습니다.

- **방문 기록**(`History`, 그리고 하루 또는 일주일 단위의 `MSHist01…`): 방문한 URL. 각 URL에는 페이지 제목,
  그 기록의 대상 계정, 마지막 방문 시각, 방문 횟수가 함께 기록됩니다. 파일 탐색기가 연 로컬 파일도 `file:///` URL로
  여기에 기록됩니다.
- **다운로드**(`iedownload`): 파일을 가져온 URL, 저장된 위치와 다운로드 중에 기록되던 위치, 크기, 다운로드
  시작 시각, 참조 페이지, MIME 유형.
- **쿠키**: Windows 10 이상에서는 쿠키 자체(`CookieEntryEx` 테이블)로, 호스트, 이름, 값, 시각이 포함됩니다.
  그 이전에는 쿠키를 담고 있던 쿠키 파일입니다.
- **캐시**(`Content`): 페이지를 표시하기 위해 다운로드한 파일. 각 파일에는 URL, 디스크상의 사본 위치, HTTP
  응답 헤더가 기록됩니다. WinINet을 사용하는 다른 프로그램도 다운로드한 내용을 여기에 캐시합니다.
- 웹 스토리지(`DOMStore`), 호환성 목록(`iecompat`) 등.

이 데이터베이스는 브라우저보다 오래 남습니다. Internet Explorer는 사용 중지되었고 Chromium 기반 Edge는 자체
SQLite 데이터베이스를 사용하지만([웹 브라우저](browser.md) 참조), 이 데이터베이스는 여전히 모든 프로필에
있으며, 이전에 기록된 내용과 파일 탐색기 및 다른 프로그램이 지금도 기록하는 내용을 담고 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 사용자의 프로필이 기록된 시각에 해당 URL, 다운로드
또는 캐시된 파일을 기록했습니다. 데이터베이스는 누가 키보드 앞에 있었는지도, 캐시된 파일이 열렸는지도 알려
주지 않습니다.

## 위치와 수집 { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

컨테이너가 가리키는 파일(캐시된 파일, 쿠키 파일)은 각각의 폴더에 있습니다.
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, 그리고 앱의 경우
`%LOCALAPPDATA%\Packages\` 아래입니다.

`WebCache` 폴더 전체를 수집하십시오. Windows는 사용자가 로그온해 있는 동안 데이터베이스를 열어 두므로, 실행
중인 시스템에서 얻은 사본은 보통 더티 상태입니다. 최신 변경 사항은 `V01*.log` 트랜잭션 로그에만 있습니다.

!!! warning "트랜잭션 로그"
    Juntei는 ESE 트랜잭션 로그를 재생하지 않습니다. 정상적으로 종료되지 않은 데이터베이스는 요약에서 더티
    상태로 남은 것으로 집계되고 문제로 보고됩니다. 아직 로그에만 있는 변경 사항은 누락됩니다. 이를 포함하려면
    먼저 폴더의 사본을 만들고, 그 사본 안에서 Windows의 `esentutl /r V01 /d`를 실행하여 로그를 재생하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 시각은 UTC이며 100나노초 단위입니다. 단, 하루 또는 일주일 단위 방문 기록 컨테이너의 `ModifiedTime`은
  예외로, 오프셋 없이 기록된 시스템의 로컬 시각이며 `-Z`도 그대로 둡니다. 원본 시스템의 당시 표준 시간대로
  변환하십시오. `AccessedTime`과의 차이는 그 표준 시간대의 오프셋인 경우가 많지만 항상 그렇지는 않습니다. 두
  시각이 서로 다른 시점일 수도 있습니다.
- 방문 기록 항목은 방문이 아니라 URL입니다. `AccessedTime`은 마지막 방문이고 `AccessCount`는 방문 횟수입니다.
  `History` 컨테이너는 URL마다 하나의 항목을 보관하고, 하루 또는 일주일 단위의 컨테이너는 그 기간의 항목을
  보관하며, 그 이름은 기간의 첫날과 마지막 날의 다음 날을 나타냅니다(`MSHist012026110120261102`: 2026년
  11월 1일).
- 방문 기록에 있는 `file:///` URL은 파일 탐색기와 다른 프로그램에서 연 로컬 파일이며, 웹 페이지가 아닙니다.
- `Account`는 항목이 기록된 대상 계정(`Visited: alice@…`)이고, `User`는 데이터베이스 경로가 속한 프로필
  폴더입니다. 둘은 다를 수 있습니다. 프로필 폴더의 이름이 항상 계정 이름과 같지는 않으며, 다른 곳으로 복사된
  데이터베이스의 경로에는 프로필 폴더가 없습니다.
- `Package`는 항목이 누구의 것인지를 나타냅니다. 레거시 Edge(`microsoft.microsoftedge_8wekyb3d8bbwe`),
  향상된 보호 모드의 Internet Explorer(`windows_ie_ac_001`) 또는 다른 앱의 것입니다. 비어 있으면 사용자 자신의
  것, 즉 Internet Explorer, 파일 탐색기 및 기타 프로그램의 것입니다.
- Microsoft는 다운로드 레코드를 문서화하지 않았습니다. Juntei는 시작 시각과 크기는 헤더가 레코드 안에
  들어맞을 때만 읽고, 경로와 기타 문자열은 마지막 두 문자열이 URL과 Windows 경로일 때만 읽습니다. 그렇지 않으면
  행에는 항목 자체의 URL이 들어가고, 시작 시각을 읽을 수 없을 때는 항목의 액세스 시각이 들어가며, 문제가
  보고됩니다. `Publisher`는 브라우저가 기록한 이름일 뿐, 파일의 서명을 확인한 결과가 아닙니다.
- 캐시 항목은 파일이 다운로드되어 캐시되었음을 나타냅니다. 디스크의 사본은 사라졌거나, 교체되었거나, 한 번도
  열리지 않았을 수 있습니다.
- 삭제된 항목은 아직 복구되지 않습니다. ESE는 삭제된 레코드를 페이지를 정리할 때까지 페이지에 남겨 둘 수
  있지만, Juntei는 활성 레코드만 읽습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d`는 `WebCacheV01.dat`를 이름으로 찾고, `-f`는 이름과 관계없이 데이터베이스를 읽습니다.

## 출력 테이블 { #output-tables }

CSV는 다섯 개의 파일을 생성하며, 행이 없으면 헤더만 있는 파일이 됩니다. JSON 및 JSONL 행에는 `SourceFile` 다음에
`Table`(`History`, `Downloads`, `Cookies`, `Entries` 또는 `Containers`)이 포함됩니다.

| CSV 파일 | 내용 |
|---|---|
| `WebCache-History.csv` | 방문 기록 컨테이너의 항목. |
| `WebCache-Downloads.csv` | 다운로드. |
| `WebCache-Cookies.csv` | 쿠키(Windows 10 이상). |
| `WebCache-Entries.csv` | 그 밖의 모든 컨테이너의 항목: 캐시된 파일, 쿠키 파일, 웹 스토리지, 호환성 목록 등. |
| `WebCache-Containers.csv` | 컨테이너. |

## 필드 { #fields }

모든 테이블은 다음 필드로 시작합니다.

| 필드 | 해석 |
|---|---|
| `SourceFile` | 행을 읽어 온 데이터베이스. |
| `User` | 경로상의 프로필 소유자인 Windows 사용자(`Users` 다음 폴더). |
| `Package` | 행의 데이터를 소유한 앱: 컨테이너 디렉터리(또는 쿠키 테이블의 디렉터리)의 `AppData\Local\Packages` 아래 폴더. 사용자 자신의 데이터이면 비어 있습니다. |

### 방문 기록 { #history }

| 필드 | 해석 |
|---|---|
| `Container` | 컨테이너: `History`, 또는 `MSHist01` 뒤에 기간의 첫날과 마지막 날의 다음 날이 이어지는 이름. |
| `AccessedTime` | URL의 마지막 방문(UTC). 하루 또는 일주일 단위의 컨테이너에서는 그 기간의 마지막 방문. |
| `ModifiedTime` | 항목이 마지막으로 변경된 시각(UTC). 하루 또는 일주일 단위의 컨테이너에서는 오프셋 없는 시스템의 로컬 시각. |
| `CreationTime` | 항목이 생성된 시각(UTC). |
| `ExpiryTime`, `SyncTime` | 항목이 방문 기록에서 빠질 예정인 시각과, 마지막으로 동기화된 시각(UTC). |
| `Account` | 항목이 기록된 대상 계정. |
| `URL` | 페이지 또는 파일. `Visited: account@` 접두사는 제외됩니다. |
| `Title` | 항목에 기록된 그대로의 페이지 제목. 하루 또는 일주일 단위의 컨테이너에서는 비어 있습니다. |
| `AccessCount`, `SyncCount` | URL을 방문한 횟수와 항목이 동기화된 횟수. |
| `EntryType`, `Flags` | 저장된 그대로의 항목 유형과 플래그. |
| `EntryID`, `ContainerID` | 컨테이너 내 항목의 ID와 컨테이너의 ID. |

### 다운로드 { #downloads }

| 필드 | 해석 |
|---|---|
| `StartTime` | 다운로드가 시작된 시각(UTC). |
| `URL` | 파일을 가져온 곳. |
| `TargetPath`, `CurrentPath` | 파일이 저장된 위치와, 다운로드 중에 기록되던 위치. 레거시 Edge는 `.partial` 파일, Internet Explorer는 캐시의 사본입니다. |
| `TotalBytes` | 파일 크기. |
| `Referrer` | 참조 페이지. |
| `MimeType` | 서버가 제공한 파일 유형(Internet Explorer). |
| `Publisher` | 레거시 Edge가 파일 게시자로 기록한 이름. |
| `AccessedTime` | 항목 자체의 액세스 시각(UTC). 대부분 다운로드 시작 시각입니다. |
| `DownloadID`, `ContainerID` | 컨테이너 내 항목의 ID와 컨테이너의 ID. |

### 쿠키 { #cookies }

| 필드 | 해석 |
|---|---|
| `LastModified`, `Expires` | 쿠키가 설정된 시각과 만료 시각(UTC). |
| `Host`, `Name`, `Path` | 쿠키의 대상 사이트, 쿠키의 이름과 경로. |
| `Value` | 저장된 그대로의 값. |
| `Flags` | 저장된 그대로의 쿠키 플래그. |
| `Directory` | 데이터베이스가 쿠키 테이블에 지정한 디렉터리로, 누구의 쿠키인지를 나타냅니다(`…\INetCookies\ESE\`, 레거시 Edge의 `…\MicrosoftEdge\Cookies\ESE\`, 앱의 디렉터리). |
| `CookieTable`, `EntryID` | 테이블(`CookieEntryEx_<n>`)과 그 안에서의 쿠키 ID. |

### 항목 { #entries }

| 필드 | 해석 |
|---|---|
| `Container` | 컨테이너: `Content`(캐시된 파일), `Cookies`(쿠키 파일), `DOMStore`(웹 스토리지), `iecompat` 등. |
| `AccessedTime` | 항목이 마지막으로 사용된 시각(UTC). |
| `ModifiedTime` | 서버가 알려 준 리소스의 마지막 수정 시각(UTC). |
| `CreationTime` | 항목이 생성된 시각(UTC). |
| `ExpiryTime`, `SyncTime` | 항목의 만료 시각과, 마지막으로 서버에 확인한 시각(UTC). |
| `URL` | 저장된 그대로의 URL: 캐시된 파일은 URL, 쿠키 파일은 `Cookie:alice@example.com/`, 그 밖에 `DOMStore:…`, `iecompat:…`. |
| `Account` | 쿠키 파일의 계정. |
| `Filename`, `FilePath` | 항목의 파일과 그 위치: 컨테이너의 디렉터리와 항목이 지정한 하위 폴더. |
| `FileSize` | 파일 크기. |
| `AccessCount`, `SyncCount` | 항목이 사용된 횟수와 서버에 확인한 횟수. |
| `EntryType`, `Flags` | 저장된 그대로의 항목 유형과 플래그. |
| `RedirectURL` | URL이 리디렉션된 곳. |
| `ResponseHeaders`, `RequestHeaders` | HTTP 응답 헤더(`HTTP/1.1 200 OK`, `Content-Type`, `Server` 등)와 요청 헤더. 줄 바꿈까지 저장된 그대로입니다. |
| `EntryID`, `ContainerID` | 컨테이너 내 항목의 ID와 컨테이너의 ID. |

### 컨테이너 { #containers }

| 필드 | 해석 |
|---|---|
| `ContainerID`, `Name` | 컨테이너의 ID와 이름. |
| `Directory` | 파일이 디스크상에 있는 위치. |
| `SecureDirectories` | 파일이 들어 있는 `Directory`의 하위 폴더(CSV에서는 쉼표로 구분). |
| `PartitionID` | 사용자 자신의 컨테이너는 `M`, Internet Explorer 보호 모드의 컨테이너(`Low` 폴더)는 `L`, 앱의 컨테이너는 해당 앱 컨테이너의 SID. |
| `LastAccessTime`, `LastScavengeTime` | 컨테이너가 마지막으로 사용된 시각과 마지막으로 정리된 시각(UTC). |

## 자동 분석 { #automatic-parsing }

`auto-parse`는 테이블을 `BrowserActivity`에 `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv`, `WebCache-Containers.csv`로 기록하며(`-t json` 또는
`-t jsonl`을 사용하면 하나의 `WebCache.json` 또는 `WebCache.jsonl` 파일), `BrowserActivity/WebCache-logs.txt`도
함께 생성합니다. 결합된 `-j` 타임라인에는 모든 행이 `Comp`와 `Artifact` 식별자 `webcache`와 함께 포함됩니다.
`-Z`는 하루 또는 일주일 단위 방문 기록 컨테이너의 로컬 시각인 `ModifiedTime`을 제외한 모든 시각, 즉 `Time`으로
끝나는 필드, `LastModified`, `Expires`를 변환합니다.

## 추가 정보 { #further-details }

- [Microsoft — WinINet 정보](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE 데이터베이스 파일 형식](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache 파서](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
