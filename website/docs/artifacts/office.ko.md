# Microsoft Office (최근 파일, 신뢰할 수 있는 문서, 매크로 설정) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Office가 기록하는 내용 { #what-office-records }

Microsoft Office는 사용자별로 `NTUSER.DAT`에, 버전 및 애플리케이션별로 다음을 보관합니다.

- **최근 파일과 위치:** 애플리케이션이 연 문서인 `File MRU`와, 그 문서를 연 폴더인 `Place MRU`가 있으며,
  각 항목에는 마지막으로 연 시간이 있습니다. Microsoft 365는 로그인한 계정별 목록(`User MRU\<account>`)도
  보관합니다.
- **신뢰할 수 있는 문서:** 사용자가 문서에서 *편집 사용* 또는 *콘텐츠 사용*을 클릭하면, Office는 이를
  `TrustRecords`에 기억합니다. 문서의 경로, 신뢰된 시점, 매크로가 사용 설정되었는지 여부입니다.
- **매크로 설정:** `VBAWarnings`(어떤 매크로가 실행되는지), `BlockContentExecutionFromInternet`(인터넷에서
  받은 파일의 매크로가 차단되는지), `AccessVBOM`(프로그램이 VBA 프로젝트를 변경할 수 있는지), 그리고 문서의
  매크로가 확인 없이 실행되는 폴더인 **신뢰할 수 있는 위치**입니다. 그룹 정책은 `Software\Policies` 아래에서
  같은 설정을 지정할 수 있습니다.

악성 문서는 보통 사용자가 매크로를 사용 설정해야 합니다. 매크로가 사용 설정된 신뢰 레코드는 문서, 사용자,
시간을 하나로 연결합니다.

<span class="evidence-strength">가장 타당한 결론:</span> 사용자가 이 애플리케이션에서 이 문서를 열었습니다
(마지막으로는 이 시간에). 사용자가 이 시간에 이 문서의 편집 또는 매크로를 사용 설정했습니다. Office는 이러한
방식으로 매크로를 실행하도록 구성되어 있었습니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

버전: `16.0`은 Office 2016 이상(2019, 2021, 2024, Microsoft 365), `15.0`은 Office 2013, `14.0`은 Office
2010, `12.0`은 Office 2007입니다.

항목은 `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`와 같은 형태입니다. `T`는
마지막으로 연 시간(16진수 FILETIME)이며, 경로는 `*` 뒤에 옵니다. 신뢰 레코드는 24바이트입니다. 문서가 신뢰된
FILETIME 뒤에 마지막 4바이트에 플래그가 있으며, 매크로가 사용 설정된 경우 `0x7FFFFFFF`, 편집만 사용 설정된
경우 `0x00000001`입니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **시간.** `LastOpenedUTC`는 각 항목의 고유한 시간입니다. `TrustedAtUTC`는 사용자가 문서를 신뢰한 시점,
  즉 *편집 사용* 또는 *콘텐츠 사용*을 클릭한 순간이며, 가장 최근에 연 시점보다 이전일 수 있습니다.
- **매크로 사용 설정이지 실행이 아닙니다.** `MacrosEnabled`는 사용자가 문서의 매크로를 사용 설정했다는
  뜻이며, 매크로가 실행되었다는 것이나 무엇을 했는지를 증명하지 않습니다. 프로세스 생성, Prefetch,
  PowerShell 로그 및 문서 자체와 상관 분석하십시오.
- **경로는 Office가 저장한 그대로입니다.** 신뢰 레코드는 `%USERPROFILE%`, 슬래시, `file:///` URL, URL
  인코딩(`%20`)을 사용할 수 있는 경로로 문서를 지칭합니다. 클라우드 문서의 MRU 항목은 URL입니다.
- **기본값과 실제 적용 정책.** Office는 기본 신뢰할 수 있는 위치(`Templates`, `XLSTART`, `STARTUP`, …)를
  생성합니다. 위치의 `Date`는 유용한 단서이지만 누가 만들었는지에 대한 증거는 아닙니다. 관리자, 스크립트,
  정책, 복사된 레지스트리 설정도 이를 채울 수 있습니다. `VBAWarnings`가 없다는 것만으로는 실제 적용되는 매크로
  정책을 확정할 수 없습니다. Office 버전, 인터넷 파일 표시 및 기타 관리 설정도 영향을 줍니다.
  [Microsoft의 인터넷 매크로 차단 설명](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **삭제된 문서.** 목록은 문서보다 오래 남으므로, 존재하지 않는 파일도 여기에 표시될 수 있습니다. 목록은
  짧으며 Office의 *고정되지 않은 항목 지우기*와 정리 도구에 의해 지워집니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

요약에는 매크로가 사용 설정된 문서, 매크로를 더 자유롭게 실행하게 하는 설정(`VBAWarnings` 1,
`BlockContentExecutionFromInternet` 0, `AccessVBOM` 1, `AllowNetworkLocations` 1), 사용자가 추가했거나 정책이
설정한 신뢰할 수 있는 위치, 그리고 마지막으로 연 문서가 표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 소유자(경로 또는 셸 폴더에서), 시스템 `ProfileList`의 SID, 그리고 하이브. |
| `OfficeVersion`, `App` | 버전 키(`16.0`)와 애플리케이션(`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` 또는 `Trusted Locations`. |
| `Account` | `User MRU` 목록의 Microsoft 365 계정. |
| `Policy` | 설정 또는 위치가 정책으로 지정되었는지 여부. |
| `MRU-Position`, `IsMostRecent` | 목록 내 항목의 위치(`Item 1`이 0이며 가장 최근). |
| `Path` | 문서 또는 폴더. 신뢰할 수 있는 문서의 저장된 경로. 신뢰할 수 있는 위치의 `Path`. |
| `LastOpenedUTC` | 항목을 마지막으로 연 시간. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | 문서가 신뢰된 시간, 매크로가 사용 설정되었는지 여부, 그리고 레코드의 플래그. 알 수 없는 플래그는 `MacrosEnabled`를 비워 두고 진단 메시지를 생성합니다. |
| `Setting`, `Value` | 매크로 설정과 그 값 및 의미(`1 (all macros enabled)`). 신뢰할 수 있는 위치의 키(`Location3`)와 그 밖의 값(`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | 키와 키의 마지막 쓰기 시간. |
| `DataRaw` | 디코딩되지 않을 때 값의 바이트(16진수). |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트. |
| `Problems` | `*`가 없거나 시간이 올바르지 않은 항목, 24바이트가 아닌 신뢰 레코드, `Path`가 없는 위치, 잘못된 유형. |

## 조사 전략 { #investigation-strategy }

1. `MacrosEnabled`가 true인 것부터 시작합니다. 어떤 문서인지, 언제, 누가 했는지 확인합니다. 파일(`$MFT`,
   USN 저널, `Zone.Identifier`), 그 출처(브라우저 및 메일 기록), 그리고 그다음에 무엇이 실행되었는지
   찾습니다.
2. 매크로 설정과 신뢰할 수 있는 위치를 확인합니다. 모든 매크로 사용 설정, 인터넷 매크로 미차단, VBA 프로젝트
   액세스, 또는 사용자가 추가한 위치(`Downloads`, `Temp`, 네트워크 공유 내)는 기본 설정을 약화시킵니다.
3. 이동식 및 네트워크 위치를 포함한 문서 경로에 대해 `LastOpenedUTC`를 LNK 파일, Jump Lists, RecentDocs와
   함께 타임라인에 배치합니다.

## 추가 정보 { #further-details }

- [Microsoft: Microsoft 365 파일에서 매크로 사용 또는 사용 안 함](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: 신뢰할 수 있는 문서](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: 신뢰할 수 있는 위치 추가, 제거 또는 변경](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
