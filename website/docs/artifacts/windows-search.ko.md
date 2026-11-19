# Windows Search 색인 { #windows-search-index }

## 색인에 저장되는 내용 { #what-the-index-records }

Windows Search(`WSearch` 서비스)는 사용자 프로필, 시작 메뉴, Outlook의 사서함, Internet Explorer의 방문 기록, 오프라인 파일 등 범위에 포함된 모든 것을 크롤링하고, 읽은 내용을 ESE 데이터베이스인 `Windows.edb`에 보관합니다. 조사에서 중요한 부분은 두 가지입니다.

- **속성 저장소**(`SystemIndex_PropertyStore`, Windows 7에서는 `SystemIndex_0A`): 색인된 항목마다 한 행이 있으며, 경로, 이름, 유형, 크기, 소유자, 인덱서가 파일에서 복사한 생성·수정·액세스 시간, 인덱서가 읽은 시간, 문서의 제목과 작성자, 텍스트의 시작 부분, 컴퓨터 이름, 그 밖의 수백 가지 속성이 들어 있습니다.
- **수집기**(gatherer) 테이블(`SystemIndex_Gthr`, `SystemIndex_GthrPth`): 크롤러 자체의 항목별 기록으로, URL 경로, 대기열에 넣었을 때의 수정 시간, 크롤 번호와 플래그, 삭제 카운터가 들어 있습니다.

항목은 파일, 메시지 또는 페이지가 사라진 뒤에도 크롤러가 삭제를 알아차릴 때까지 색인에 남습니다. 따라서 색인에는 더 이상 존재하지 않는 파일이 색인 당시의 메타데이터 및 텍스트 발췌와 함께 들어 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 인덱서가 기록된 경로의 항목을 기록된 메타데이터와 함께 `GatherTime`에 지정된 컴퓨터에서 읽었습니다. 색인은 파일이 존재했다는 사실과 그 내용을 보여 줍니다. 사용자가 파일을 열었다는 사실은 보여 주지 않습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7부터 Windows 11 21H2까지, 그리고 같은 시기의 Windows Server 릴리스는 색인을 `Windows.edb`에 보관합니다. Windows 11 22H2 이후는 같은 폴더의 SQLite 데이터베이스(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`)에 보관합니다. Juntei는 이 파일들을 찾아 이름을 표시하고, 지원되지 않는 최신 형식으로 보고합니다.

`Applications\Windows` 폴더 전체를 디스크 이미지나 볼륨 섀도 복사본에서 수집하거나, `WSearch` 서비스를 중지한 뒤 수집하십시오. 서비스가 실행되는 동안 데이터베이스는 열려 있으며, 최신 변경 사항은 Juntei가 재생하지 않는 `MSS*.log` 트랜잭션 로그에만 있습니다. 실행 중인 시스템에서 복사한 색인은 더티 상태로 남습니다. 출력에 그 사실이 표시되며, 디스크에 있는 내용을 읽습니다. 디렉터리 입력과 `auto-parse`는 깊이와 관계없이 모든 `Windows.edb`를 읽습니다. 파일 하나에는 `-f`를 사용하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- `GatherTime`은 인덱서가 항목을 읽은 시간이며, 사용자가 생성, 변경 또는 연 시간이 아닙니다. 색인 작업은 백그라운드에서 실행되어 변경 후 몇 분에서 몇 시간 뒤에 이루어지는 경우가 많고, 컴퓨터가 바쁘거나 배터리로 작동할 때는 일시 중지됩니다.
- `DateCreated`, `DateModified`, `DateAccessed`는 인덱서가 파일에서 복사한 값입니다. 항목이 다시 색인될 때 갱신되므로 `GatherTime` 시점의 파일을 나타냅니다.
- `ItemPath`는 표시 경로입니다. `ItemURL`과 `GatherPath`는 크롤러의 URL로, 파일은 `file:`(Windows 8 이후에서는 볼륨이 `[GUID]`로 표시됨), Internet Explorer의 방문 기록은 `iehistory://{SID}/`, Outlook 항목은 `mapi://`, 오프라인 파일은 `csc://{SID}/`, Windows 앱의 콘텐츠는 `winrt://{SID}/`입니다.
- `User`는 항목의 경로에 지정된 프로필입니다(`C:\Users\<name>\…`). 항목이 있던 위치이지 누가 만들었는지가 아닙니다. `FileOwner`는 인덱서가 본 파일의 소유자입니다.
- `AutoSummary`는 항목 텍스트의 시작 부분입니다. 이후에 삭제되거나 암호화된 문서의 내용을 인용하고 있을 수 있으며, 잘린 발췌이지 문서 자체가 아닙니다.
- 더티 상태의 데이터베이스에는 브랜치 페이지가 아직 가리키지 않는 페이지가 있을 수 있습니다. Juntei는 리프 페이지 자체의 체인을 따라가며, 그렇게 읽은 페이지 수를 요약에 표시합니다. 최신 변경 사항은 여전히 트랜잭션 로그에 있습니다.
- 수집기 테이블에는 있지만 속성 저장소 행이 없는 항목(`InTables`가 `Gthr`)은 대기열에 넣어졌거나 크롤링되었지만 속성이 아직 기록되지 않았거나 제거된 것입니다. 경로, 이름, 수집기 시간은 그대로 있습니다.
- Windows 7은 문자열을 고유의 압축·난독화된 인코딩으로 저장합니다. Juntei는 이를 디코딩하며, 디코딩되지 않는 값은 16진수로 남고 `Problems`에 메모가 붙습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

기본 파일 형식은 CSV입니다. `-t csv`, `-t json` 또는 `-t jsonl`을 지정하고 `-o`를 생략하면 레코드를 stdout으로 출력합니다. 두 옵션 모두 없으면 요약만 표시합니다. 기존 출력을 덮어쓰려면 `-C`를 추가하십시오. `-C`를 지정하더라도 출력이 입력 파일 위에 덮어쓰이는 일은 없습니다.

Juntei는 색인된 항목마다 레코드를 하나씩 기록하며, 속성 저장소와 수집기 테이블을 항목의 식별자로 조인합니다. 요약에는 스키마별 데이터베이스 수, 더티 상태의 데이터베이스 수, 두 테이블 모두에 있는 항목 수, 수집기에만 있는 항목 수, 속성 저장소에만 있는 항목 수, 폴더 수, 삭제됨으로 표시된 항목 수, 저장소별 항목 수, 저장된 시간이 없는 항목 수, 문제 수가 집계됩니다.

손상이 있어도 실행은 중단되지 않습니다. 중간에 잘린 데이터베이스는 남아 있는 페이지에 들어 있는 항목을 유지하며, 읽을 수 없었던 내용은 요약에 표시됩니다. 이름이 `Windows.edb`이지만 Windows Search 색인이 아닌 파일은 보고한 뒤 건너뜁니다. 다른 데이터베이스는 계속 읽습니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `DocumentID` | 색인에서의 항목 식별자로, 속성 저장소의 `WorkID`(Windows 7에서는 `DocID`)와 수집기 행의 `DocumentID`입니다. |
| `InTables` | `Gthr+PropertyStore`, `Gthr`(대기열에 넣어졌거나 크롤링되었지만 속성이 없는 항목) 또는 `PropertyStore`입니다. |
| `ItemPath` | `System_ItemPathDisplay`: Windows가 표시하는 경로입니다. 속성 저장소에 행이 없으면 비어 있습니다. |
| `ItemURL` | `System_ItemUrl`: 항목에 대한 크롤러의 URL입니다(`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | 수집기의 경로로, 범위 트리와 행의 파일 이름을 결합한 것입니다. |
| `Name` | `System_ItemName` 또는 수집기 행의 파일 이름입니다. |
| `ItemType`, `ItemTypeText` | `System_ItemType`(확장명 또는 `Directory`)과 그 표시 이름입니다. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, … 입니다. 여러 값은 `\|`로 연결됩니다. |
| `MimeType` | `System_MIMEType`입니다. |
| `Store` | `System_Search_Store`: `file`, `mapi`(Outlook), `iehistory`, `csc`(오프라인 파일) 또는 `winrt`(Windows 앱)입니다. |
| `IsFolder`, `IsDeleted` | 저장된 그대로의 `System_IsFolder`와 `System_IsDeleted`입니다. |
| `DeletedCount` | 수집기 행의 삭제 카운터입니다. |
| `Size` | 바이트 단위의 `System_Size`입니다. 자리 표시자가 들어 있는 폴더에서는 비어 있습니다. |
| `FileAttributes` | 숫자로 된 `System_FileAttributes`입니다(16 디렉터리, 32 보관, 2 숨김, …). |
| `FileOwner` | 인덱서가 읽은 시점의 `System_FileOwner`(`DOMAIN\user`)입니다. |
| `DateCreated`, `DateModified`, `DateAccessed` | 인덱서가 복사한 시점의 파일 시간(UTC)입니다. |
| `ItemDate` | `System_ItemDate`: 메시지의 보낸 시각이나 사진의 촬영 날짜 같은 항목 자체의 날짜입니다. 파일의 경우 보통 초 단위의 수정 시간입니다. |
| `GatherTime` | `System_Search_GatherTime`: 인덱서가 항목을 읽은 시간(UTC)입니다. |
| `GatherLastModified` | 수집기 행의 `LastModified`: 크롤러가 항목을 대기열에 넣었을 때의 수정 시간(UTC)입니다. |
| `Title`, `Author` | `System_Title`과 `System_Author`입니다. |
| `AutoSummary` | `System_Search_AutoSummary`: 항목 텍스트의 시작 부분입니다. |
| `CompName` | `System_ComputerName`: 색인이 속한 컴퓨터입니다. |
| `TargetURL`, `DateVisited` | Internet Explorer 방문 기록 항목의 URL과 방문 시간입니다(`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Windows 10 활동 기록 항목입니다(`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | 항목의 경로에 지정된 프로필과, 어느 경로에서 읽었는지(`ItemPath`, `ItemURL` 또는 `GatherPath`)입니다. 프로필 밖에 있으면 비어 있습니다. |
| `ScopeID`, `SDID` | 수집기 행의 범위 식별자와 보안 설명자 식별자입니다. |
| `CrawlNumber` | 수집기 행의 `CrawlNumberCrawled`입니다. |
| `GatherFlags` | 수집기 행의 `TransactionFlags`입니다. |
| `Priority` | 아직 대기열에 있는 항목의 크롤 우선순위입니다. 항목이 크롤링되면 ESE가 NULL로 표시하므로 대부분의 항목에서 비어 있습니다. |
| `PropertyCount` | 속성 저장소가 해당 항목에 대해 보유한 속성 수입니다. |
| `OtherProperties` | 그 밖의 모든 속성을 `; `로 구분한 `Name=Value` 형식으로 나타냅니다. |
| `SourceFile` | 읽은 데이터베이스로, 입력 디렉터리 기준 상대 경로이며 `-f`에서는 파일 이름입니다. |
| `Problems` | 디코딩되지 않은 값, 시간이 아닌 시간, 중복 행 및 손상입니다. 숨기지 마십시오. |

JSON과 JSONL은 같은 필드를 타입이 지정된 값으로 유지합니다. 숫자는 숫자 그대로이고 누락된 값은 `null`입니다. `OtherProperties` 대신 `Properties`가 있으며, 여기에는 항목의 모든 속성이 이름, 숫자 식별자, 열 타입, 값과 함께 들어 있고, 시간의 경우 해석된 시간 옆에 원래의 FILETIME 틱 수가 있습니다. `Gather`에는 수집기 행의 열이 들어 있습니다.

## 타임라인 동작 { #timeline-behavior }

`auto-parse`는 레코드를 `FileExistence/WindowsSearch.csv`(또는 `.json`/`.jsonl`)로 저장하고 결합된 `-j` 타임라인에 추가합니다. `super-timeline`은 `DateCreated`, `DateModified`, `DateAccessed`(Windows Search가 기록한 파일 시간), `GatherTime`(항목이 색인된 시간), Internet Explorer 방문 기록 항목의 `DateVisited`, 활동의 `ActivityStartTime`과 `ActivityEndTime`으로 이벤트를 만듭니다. `GatherLastModified`는 속성 저장소에 수정 시간이 없는 항목에 대해서만 이벤트가 됩니다. 이 중 어느 시간도 없는 항목은 파싱된 출력에 남으며 시간이 없는 레코드로 집계됩니다.

## 조사 전략 { #investigation-strategy }

1. `Store`를 `file`로 필터링하고 `ItemPath`, `Name`, `AutoSummary`에서 사건과 관련된 파일, 이름, 단어를 검색하십시오. 색인에는 이후 삭제된 파일도 들어 있습니다.
2. `GatherTime`은 파일이 기록된 메타데이터로 존재했던 마지막 시간으로, `DateModified`는 그 시점의 수정 시간으로 읽으십시오.
3. `FileOwner`와 `User`로 파일을 계정과 연결한 다음 `$MFT`, USN 저널, `$Secure:$SDS`로 확인하십시오.
4. 이메일(`Store`가 `mapi`)과 Internet Explorer 방문 기록(`iehistory`)은 전체 레코드를 보기 위해 Outlook 파일과 WebCache로 피벗하십시오.
5. 호스트 간에 색인을 비교하십시오. `CompName`은 색인이 속한 컴퓨터를 나타내고, 이미지에서 수집한 색인은 그 이미지에 있는 파일을 나타냅니다.

## 테스트한 버전 { #tested-versions }

테스트에는 다른 도구가 독립적으로 읽은 결과가 있는 공개 색인을 사용합니다.

- plaso의 Windows 7 색인(`SystemIndex_0A`): Windows 7의 문자열 인코딩을 포함하여 libyal의 `esedbexport`와 대조했습니다.
- DFIRArtifactMuseum의 Windows 10 색인(더티 상태 그대로): Stroz Friedberg의 SIDR과 대조했습니다. 같은 곳의 Windows 11 색인(ESE 형식 개정 200)은 `esedbexport`와 대조했습니다.
- DFIRArtifactMuseum의 Windows 11 22H2 SQLite 색인: 지원되지 않는 최신 형식으로 보고됩니다.
- 중간에 잘린 데이터베이스, 빈 데이터베이스 또는 다른 종류의 데이터베이스.

## 추가 정보 { #further-details }

- [Microsoft — Windows Search 개요](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE 데이터베이스 파일 형식](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine 소스](https://github.com/microsoft/Extensible-Storage-Engine)
