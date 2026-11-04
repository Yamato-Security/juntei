# Microsoft Defender 격리 데이터 { #microsoft-defender-quarantine }

## 격리 데이터에 들어 있는 내용 { #what-the-quarantine-contains }

Microsoft Defender 격리 데이터는 탐지에 관한 메타데이터와 격리된 리소스의 암호화된 콘텐츠를
보존합니다. Juntei는 Entries, ResourceData, Resources 레코드를 서로 연결하여 위협 레이블, 격리 시간,
원래의 논리 경로와 물리 경로, 파일 메타데이터, 소유자, 다운로드 출처 `Zone.Identifier`, 해시 및 무결성
문제를 보고합니다.

이 아티팩트는 원래 탐지/치료를 기록한 이벤트 로그보다 오래 남을 수 있습니다. 또한 격리된 환경에서
리버스 엔지니어링을 수행할 수 있도록 의심스러운 바이트를 그대로 보존할 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> Defender가 기록된 시간에 지정된 위협/리소스에
대한 격리 레코드를 생성했습니다. 제품 레이블은 맬웨어에 대한 독립적인 증명이 아니며, 격리만으로는
이전에 실행되었음을 증명하지 못합니다.

## 위치와 수집 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

`Quarantine` 디렉터리 전체를 수집하십시오. 일치하는 콘텐츠/레코드 파일이 없는 엔트리도 유용한
메타데이터이지만, 파일 바이트나 계산된 해시를 얻을 수는 없습니다.

## 안전 및 분석 시 주의 사항 { #safety-and-analyst-cautions }

- 추출된 콘텐츠는 악성일 가능성이 높습니다. 격리된 분석 시스템을 사용하고 절대 실행하지 마십시오.
- `ThreatName`은 당시 Defender의 시그니처/분류입니다. 오탐이나 이후의 재분류가 있을 수 있습니다.
- 격리 시간은 치료/격리 시간이며, 반드시 다운로드, 생성 또는 실행 시간인 것은 아닙니다.
- 격리된 파일은 실행 전에 탐지되었을 수도 있고 실행된 후에 탐지되었을 수도 있습니다.
- 원래 경로와 Zone.Identifier는 가치가 있지만 존재하지 않거나 공격자가 제어한 값일 수 있습니다.
- 고아(orphan) 레코드는 보존/정리, 불완전한 수집 또는 손상으로 인해 발생할 수 있습니다.
- Juntei는 크기, CRC, ID 및 해시를 검증합니다. 복구된 바이트를 신뢰하기 전에 `Problems`를 검토하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

예를 들어 `tool.exe`는 `tool.exe.quarantined`가 됩니다. 보조 스트림은 `tool.exe.Zone.Identifier.quarantined`처럼
이름에 해당 유형이 포함됩니다. Juntei는 실행 권한도 제거합니다. 확장자와 권한 설정으로 실수로 실행할
위험은 줄어들지만, 복구된 바이트는 여전히 맬웨어로 취급해야 합니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Record` | `resource`, 리소스가 없는 엔트리, `orphan-content` 또는 `orphan-record`입니다. 고아 레코드는 불완전한 연결/수집을 드러냅니다. |
| `QuarantineTime` | UTC 기준 Defender 격리/치료 시간이며, 실행 시간이 아닙니다. |
| `ThreatName`, `ThreatID` | Defender 분류와 숫자 ID입니다. 공급업체의 판정/컨텍스트로 취급하십시오. |
| `ResourceType` | 파일, 레지스트리 키, 서비스, 시작 항목, 예약된 작업 또는 기타 리소스 범주입니다. |
| `Path`, `PhysicalPath` | 원래의 논리/물리 위치입니다. 주요 상관 분석 피벗이며, 철자를 정확히 그대로 보존하십시오. |
| `FileSize` | 리소스 메타데이터에 기록된 크기입니다. `ContentSize` 및 MFT와 비교하십시오. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | 격리된 리소스에 대해 캡처된 메타데이터입니다. 격리 시간과는 별개이며 타임스톰핑(timestomping)의 영향을 받을 수 있습니다. |
| `Attr` | 기록된 파일 속성입니다. 숨김/시스템 상태는 유용한 컨텍스트가 될 수 있습니다. |
| `Owner` | 복구된 보안 설명자의 SID입니다. 소유자가 반드시 실행한 사용자인 것은 아닙니다. |
| `ZoneID`, `ReferrerURL`, `HostURL` | `Zone.Identifier`의 다운로드 영역 및 URL 값입니다. 존재하는 경우 출처를 밝히는 강력한 단서이며, ADS는 없거나 제거되었을 수 있습니다. |
| `ContentSize` | 실제로 복구된 기본 콘텐츠의 크기입니다. 기록된 `FileSize`와 비교하십시오. |
| `MD5`, `SHA1`, `SHA256` | 복구된 바이트로부터 계산된 값입니다. 최신 콘텐츠 식별에는 SHA-256을 사용하고, 상호 운용성을 위해 모두 보존하십시오. |
| `Streams` | 복구된 스트림의 종류/이름/크기입니다. ADS, 보안 데이터 또는 레지스트리 하이브가 포함될 수 있습니다. |
| `ResourceID` | 리소스 메타데이터/콘텐츠 레코드를 연결하는 식별자입니다. |
| `EntryID` | 격리 엔트리 GUID입니다. Defender 기록/이벤트 증거와도 연결됩니다. |
| `ScanID` | 리소스/동작을 그룹화하기 위한 Defender 검사 식별자입니다. |
| `QuarantineFolder` | 원본 루트 폴더/호스트 구분 정보입니다. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | 해당 행을 만드는 데 사용된 정확한 원본 파일입니다. 검증에 필수적입니다. |
| `ExtractedFile` | `--extract`가 기록한 기본 격리 파일입니다. 기본 파일이 추출되지 않은 경우 비어 있습니다. 보조 스트림은 그 옆에 기록됩니다. `auto-parse -x`는 출력 디렉터리 기준 경로(`./HOST1/SecurityProducts/DefenderQuarantined/…`)로 제공합니다. |
| `Problems` | 무결성 불일치, 콘텐츠 누락, ID 불일치, 손상 또는 고아 레코드 컨텍스트입니다. 숨기지 마십시오. |

JSON/JSONL은 CSV의 결합된 텍스트 대신 구조화된 리소스, 스트림 목록, 레코드 엔트리 및 모든 관계 세부
정보를 보존합니다.

## 추출 동작 { #extraction-behavior }

격리 엔트리에 원래 파일 이름이 기록되어 있으면 Juntei는 그 이름 끝에 `.quarantined` 확장자를 붙여
사용합니다. 고아 리소스나 파일이 아닌 리소스에는 콘텐츠 식별자를 사용하며, 파일 이름 충돌을 해결하기
위해 식별자를 추가합니다. Windows에서 유효하지 않은 파일 이름 문자는 대체됩니다. 보안 스트림과 대체
스트림은 기본 파일 옆에 기록됩니다. 각 보안 스트림에 대해서는 읽을 수 있는 텍스트 보고서도 생성됩니다.
Juntei는 실행 권한을 제거하며, 내용이 다른 기존 파일은 `-C` 없이는 덮어쓰지 않습니다. 이러한 제어는
사고를 줄여 주지만 바이트 자체를 안전하게 만들지는 않습니다.

### 복구된 보안 설명자 읽기 { #reading-the-recovered-security-descriptor }

`.security.quarantined` 파일에는 바이너리 형식의 Windows 보안 메타데이터가 들어 있으므로 텍스트
편집기로는 읽을 수 없습니다. 이는 별도의 백업 스트림이며 실행 파일의 또 다른 사본이 아닙니다. Juntei는
`parse-file defender-quarantine --extract` 또는 `auto-parse -x`로 추출할 때 이를 자동으로 디코딩합니다.

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

텍스트 파일 이름은 충돌 식별자를 포함하여 실제 바이너리 파일의 이름을 따릅니다. 생성된 텍스트는 일반
UTF-8이며 다른 추출 파일과 동일한 덮어쓰기 보호가 적용됩니다. 보고서와 원본을 대조할 수 있도록 바이너리
파일 이름, 바이트 수 및 SHA-256이 포함됩니다.

| 보고서 필드 | 해석 방법 |
|---|---|
| 소유자 및 기본 그룹 | 설명자에 기록된 SID입니다. 잘 알려진 SID에는 `BUILTIN\Administrators`나 `NT AUTHORITY\SYSTEM` 같은 이름이 붙습니다. 그 밖의 SID는 해석되지 않은 채로 남으며, 익숙한 RID만으로는 로컬/도메인 계정 이름을 특정하기에 충분하지 않습니다. 소유자가 파일을 생성하거나 실행한 사람을 증명하지는 않습니다. |
| 제어 플래그 | ACL 존재 여부, 상속 및 보호를 포함한 원래의 숫자 플래그와 그 이름입니다. |
| DACL | 순서가 지정된 액세스 제어 항목(ACE)입니다: 허용/거부 유형, 계정 SID, 권한 마스크, 권한 및 상속 플래그. 이는 기록된 항목이며, 사용자 토큰에 대한 유효 액세스를 계산한 결과가 아닙니다. |
| SACL | 기록된 감사 또는 필수 무결성 항목입니다. 감사 성공/실패 플래그는 구성을 나타낼 뿐 액세스 이력이 아닙니다. SACL이 없다고 해서 모든 감사가 비활성화되었음이 입증되지는 않습니다. |
| 개체 GUID | 개체별 ACE에 대한 선택적 GUID입니다. 개체별 권한은 파일 권한으로 잘못 표기되지 않도록 숫자로 유지됩니다. |
| SDDL | 소유자, 그룹 및 지원되는 ACL 항목의 SDDL(Security Descriptor Definition Language) 표현입니다. 제어 플래그는 별도로 확인하십시오. 구성 요소 중 하나라도 손상되었거나 지원되지 않으면 Juntei는 부분적인 SDDL 문자열을 출력하지 않습니다. |
| 디코딩 상태 및 문제 | 이유와 함께 `complete` 또는 `incomplete`로 표시됩니다. 지원되지 않는 ACE는 보고서에 원시 16진수가 유지됩니다. 추출이 성공하면 원본 바이너리는 항상 보존됩니다. 디코딩 문제는 격리 레코드의 `Problems` 필드에도 나타납니다. |

DACL이 없는 경우, NULL DACL, 빈 DACL은 각각 구분하여 보고됩니다. DACL이 없거나 NULL DACL이면 임의
액세스 제한이 없으며, 빈 DACL에는 액세스를 허용하는 ACE가 없습니다. 그 밖의 Windows 액세스 검사와
권한도 여전히 영향을 미칩니다. 상속된 ACE와 상속 전용 ACE는 명시적으로 표시됩니다.

예를 들어 Administrators에 상속된 모든 권한을 부여하는 항목은 다음과 같이 나타날 수 있습니다.

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

디코더는 Windows API, Python 또는 계정 조회 없이 Windows, macOS, Linux에서 오프라인으로 실행됩니다.
기본 허용/거부/감사/경보 ACE, 개체 ACE 및 필수 무결성 레이블을 디코딩합니다. 조건부, 콜백, 리소스 특성
및 기타 지원되지 않는 ACE는 명시적으로 식별되어 원시 형태로 보존되며, 그 액세스 조건은 평가되지
않습니다. 설명자가 손상되어도 다른 스트림의 추출은 중단되지 않습니다.

## 조사 전략 { #investigation-strategy }

1. `ThreatName`, `Path`, `QuarantineTime`, `EntryId` 및 해시에서 시작하십시오.
2. `Problems`를 검증하고 기록된 크기를 복구된 콘텐츠와 비교하십시오.
3. SHA-256을 기준으로 다른 호스트, EDR, 샌드박스 및 위협 인텔리전스 데이터로 피벗하십시오.
4. 원래 경로/시간을 `$MFT`, USN 저널, Prefetch, Amcache 및 이벤트 로그와 상관 분석하십시오.
5. URL/영역 필드를 사용하여 초기 침투를 조사하되, 브라우저/프록시/DNS 증거로 검증하십시오.

## 추가 정보 { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [dissect.target Defender 플러그인](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender 바이러스 백신 치료 및 격리](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: 보안 설명자](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: 보안 설명자 문자열 형식](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE 문자열](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
