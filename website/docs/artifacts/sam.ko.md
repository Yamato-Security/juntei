# SAM 로컬 계정 { #sam-local-accounts }

## SAM에 들어 있는 내용 { #what-the-sam-contains }

보안 계정 관리자(`SAM`) 하이브는 로컬 계정 레코드를 저장합니다. Juntei는 계정 이름, RID, 프로필/설명 필드,
계정 제어 플래그, 마지막 로그온/암호 타임스탬프, 카운터, 암호 힌트, Microsoft 계정 연결, 프로필 경로, 그리고
Windows가 로그인 UI에서 계정을 숨기는지 여부를 디코딩합니다.

이는 새로 생성된 로컬 관리자, 사용 설정된 휴면 계정, 암호 변경, 숨겨진 계정, 측면 이동을 위한 준비 계정을
찾는 데 유용합니다.

<span class="evidence-strength">가장 타당한 결론:</span> 로컬 SAM은 기록된 상태로 해당 계정을 나타내고
있었습니다. 이는 완전한 인증 기록이 아니며 도메인 계정을 설명하지 않습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

충분한 권한/원시 접근으로 하이브와 로그를 수집하십시오. 호스트 식별 정보는 수집물에 포함된 해당 `SYSTEM`
하이브로 보강할 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- 이 명령어는 Active Directory 도메인 사용자가 아니라 **로컬** 계정을 파싱합니다.
- Juntei는 의도적으로 암호 해시를 추출하지 않습니다.
- RID 500은 이름이 변경되었더라도 기본 제공 Administrator 계정입니다. 이름만으로는 충분하지 않습니다.
- `LastLogon`과 카운터는 레지스트리 상태이며, 세션별 완전한 감사 기록이 아닙니다.
- `HiddenFromLogonUI`는 로그인 화면에서 계정을 숨길 뿐이며, 계정을 사용 안 함으로 설정하지 않습니다.
- 기본 제공 `DefaultAccount`와 `WDAGUtilityAccount`는 정상적으로 숨겨져 있을 수 있습니다.
- 손상되었거나 없는 `F` 값으로 인한 null 계정 플래그를 사용 설정 상태로 해석해서는 안 됩니다.
- `NamesKeyLastWrittenUTC`는 계정 이름 등록 시점을 근사하지만, 생성 시간을 보장하지는 않습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트 컨텍스트와 정확한 SAM 하이브. |
| `RID` | 상대 식별자. 강력한 계정 식별 기준점이며, 500은 기본 제공 Administrator입니다. |
| `User` | 현재 로컬 계정 이름. 이름이 변경될 수 있으므로 RID로 상관 분석하십시오. |
| `FullName`, `Comment`, `UserComment` | 설명 텍스트. 공격자가 제어할 수 있으며 위장 여부 검토에 유용합니다. |
| `AccountFlags` | 사용 안 함, 잠김, 암호 필요 없음, 일반 계정, 암호 사용 기간 제한 없음 등 디코딩된 상태. |
| `AccountFlagsValue` | 검증 및 지원되지 않는 비트를 위한 원시 숫자 플래그. |
| `CountryCode`, `CodePage` | 계정의 로캘/코드 페이지 메타데이터. 대개 행위자 특정이 아니라 컨텍스트입니다. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | 같은 마지막 로그온 상태를 전체 정밀도와 초 단위 호환 형식으로 표시. 완전한 기록이 아닙니다. |
| `PasswordLastSetUTC` | 기록된 암호 변경 시간. 예상치 못하게 최근이거나 오래된 값을 조사하십시오. |
| `AccountExpirationUTC` | 유한한 경우 구성된 만료 시간. null은 만료 없음/설정 안 됨을 의미할 수 있습니다. |
| `LastFailedLogonUTC` | SAM 상태에 기록된 마지막 로컬 로그온 실패. Security 이벤트로 확인하십시오. |
| `FailedLogonCount`, `LogonCount` | Windows 고유의 갱신 동작을 따르는 누적/상태 카운터. 정확한 이벤트 수가 아니라 컨텍스트로 사용하십시오. |
| `PasswordHint` | 사용자가 제공한 민감한 힌트. 암호 패턴을 드러낼 수 있으므로 적절히 취급하십시오. |
| `InternetUserName` | 존재하는 경우 연결된 Microsoft 계정 ID. |
| `HiddenFromLogonUI` | UI 숨김 상태. 높은 RID/예상치 못한 사용자 지정 계정을 검토하십시오. 사용 안 함을 의미하지 않습니다. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | 로컬 계정 환경/로그온 구성. 비정상적인 스크립트/UNC 경로는 지속성을 나타낼 수 있습니다. |
| `KeyLastWrittenUTC` | RID 계정 키의 변경 시간. 필드별이 아닌 키 수준입니다. |
| `NamesKeyLastWrittenUTC` | 사용자 이름 매핑 키의 변경 시간. 계정 수명 주기를 근사하는 데 유용합니다. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | 검증 및 특수 센티널 값 처리를 위해 보존된 원시 64비트 FILETIME 값. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 재생 컨텍스트. |
| `Problems` | F/V/Names 간 불일치, 누락된 항목, 잘못된 오프셋 또는 디코딩 손상. |

## 조사 전략 { #investigation-strategy }

1. RID, 이름, 플래그, 프로필 경로, `NamesKeyLastWrittenUTC`를 호스트 기준선과 비교합니다.
2. 사용 설정된 RID 1000 이상 계정, 암호 필요 없음, 만료 없음, 숨김 또는 최근 변경된 계정을 검토합니다.
3. Security 4720/4722/4724/4732 이벤트, 로그온 이벤트, 프로필 생성 및 원격 활동과 상관 분석합니다.
4. 수집 전에 계정이 제거되었다면 레지스트리 복구 기록을 검색합니다.
5. 정상적인 프로비저닝/기본 계정과 공격자가 대화형으로 생성한 계정을 구별합니다.

## 추가 정보 { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM 플러그인](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
