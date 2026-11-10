# 감사 정책 { #audit-policy }

## 개요 { #what-it-is }

감사 정책은 Windows가 Security 이벤트 로그에 어떤 보안 이벤트를 기록할지를 정합니다. Windows Vista
이상에서는 하위 범주별로(Windows 10 및 11에서는 59개), Windows XP 및 Server 2003에서는 범주별로
설정됩니다. 각 항목은 성공, 실패, 둘 다를 기록하거나 아무것도 기록하지 않습니다.

LSA는 현재 적용 중인 정책을 `SECURITY` 하이브에 보관하며, 이는 정책이 `auditpol`, 로컬 보안 정책,
그룹 정책 중 어디에서 왔는지와 관계없습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 하이브가 마지막으로 기록된 시점에 시스템은
이러한 이벤트를 기록하도록(또는 기록하지 않도록) 설정되어 있었습니다. 하위 범주가 감사되지 않는 경우,
이벤트가 없다는 사실은 해당 활동이 발생했는지 여부에 대해 아무것도 말해 주지 않습니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista 이상은 12바이트 헤더, 범주별로 정렬된 하위 범주당 하나의 설정, 그리고 각 범주에 속한
하위 범주 수를 순서대로 보관합니다. 버전 간 차이는 이 개수뿐입니다. 각 버전은 범주 끝에 하위 범주를
추가합니다(읽어 본 Windows 8.x 및 Server 2012 R2 하이브에서는 56개, Windows 10, 11 및 Server
2022에서는 59개). Windows XP 및 Server 2003은 감사 활성화 여부(1바이트, 이어서 Windows가 지우지 않는
패딩)와 9개의 범주 설정을 보관합니다. 이 레이아웃은 아무것도 감사하지 않는 NIST CFReDS Hacking Case의
공개 Windows XP 이미지로 확인했으며, 범주 설정 값은 Microsoft 문서를 따릅니다.

`SECURITY`를 트랜잭션 로그와 함께 수집하고, 호스트 이름이 들어 있는 시스템의 `SYSTEM` 하이브도
수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **알 수 없는 데이터는 알 수 없는 상태로 둡니다.** 지원되지 않는 리비전은 진단 정보와 함께
  보존됩니다. 알 수 없는 설정 플래그는 `Success`와 `Failure`를 비워 두며 감사 설정 수 집계에서
  제외됩니다. 손상된 개수 테이블로 인해 예약된 워드가 다른 감사 설정으로 승격되지는 않습니다.
  캡처에는 이름/GUID 카탈로그 범위를 넘어서는 추가 슬롯이 포함될 수 있습니다. 해당 설정과 숫자
  `SubcategoryId`는 이름/GUID가 빈 상태로 유지되며, 매핑되지 않은 슬롯 자체가 손상의 증거는 아닙니다.


- **순서는 Windows 고유의 순서입니다.** 범주 내에서 설정은 하위 범주 GUID가 아닌 메시지 ID 순서로
  배열됩니다. Special Logon(`%%12548`, Logon/Logoff의 인덱스 4)은 IPsec Quick Mode보다 앞에 옵니다.
  GUID 순서로 읽는 도구는 Windows 10의 기본값을 Special Logon "No Auditing", IPsec Quick Mode
  "Success"로 보고합니다. `SubcategoryId`는 이벤트 4719(System audit policy was changed)가 제공하는
  것과 동일한 ID이므로 둘을 서로 대조할 수 있습니다.
- **출처가 아닌 현재 적용 중인 정책.** 그룹 정책과 로컬 정책은 같은 값을 기록합니다. 그룹 정책의
  `audit.csv`(SYSVOL 및 `C:\Windows\System32\GroupPolicy`에 있음)는 적용하려고 했던 내용을
  보여 줍니다(정책의 `Machine\Microsoft\Windows NT\Audit\audit.csv`).
- **시간에 따른 변경:** `KeyLastWrittenUTC`는 정책이 마지막으로 변경된 대략적인 시간입니다. 이벤트
  4719는 각 변경을 기록합니다(Security 로그에 남아 있는 한). 공격자는 감사를 끈 다음 다시 켤 수 있습니다.
- **하위 범주와 범주.** Windows Vista 이상에서는 Windows가 하위 범주를 사용합니다(단,
  `SCENoApplyLegacyAuditPolicy` 보안 옵션이 꺼져 있고 레거시 범주 정책이 이를 재정의하는 경우는 예외).
- **Windows XP 및 2003:** 감사가 꺼져 있으면(`AuditingEnabled` False) 설정과 관계없이 어떤 범주도
  감사되지 않습니다.
- **이벤트:** `Events`는 Microsoft의 고급 감사 정책 설정 문서에 따른 각 하위 범주의 주요 이벤트를
  나열합니다. 이는 조사에서 일반적으로 찾는 이벤트이며, 해당 하위 범주가 기록하는 모든 이벤트는 아닙니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

요약에는 하이브별로 레이아웃, 감사되는 설정 수, 정책이 마지막으로 기록된 시간이 표시됩니다. 성공과
실패 모두 감사되는 항목, 성공만 감사되는 항목, 실패만 감사되는 항목을 나열한 다음, 감사되지 않는
하위 범주를 주요 이벤트와 함께 나열합니다. 해당 이벤트는 캡처된 정책에서 비활성화되어 있다는 뜻이며,
이것만으로 이전 활동 시점의 정책이 확인되는 것은 아닙니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트(시스템의 `SYSTEM` 하이브에서 얻음)와 `SECURITY` 하이브입니다. |
| `Layout` | `subcategories`(Windows Vista 이상) 또는 `categories`(XP 및 2003)이며, 값이 디코딩되지 않으면 비어 있습니다. |
| `Category`, `Subcategory` | `auditpol`이 사용하는 이름입니다. |
| `SubcategoryGUID`, `SubcategoryID` | 하위 범주의 GUID와, 이벤트 4719가 제공하는 형식의 메시지 ID(`%%12548`)입니다. |
| `Setting` | `none`, `success`, `failure` 또는 `success_and_failure`입니다. |
| `Success`, `Failure` | 성공과 실패가 기록되는지 여부입니다. |
| `AuditingEnabled` | Windows XP 및 2003: 감사 자체가 켜져 있는지 여부입니다. |
| `Events` | 하위 범주의 주요 이벤트 ID입니다. |
| `KeyLastWrittenUTC` | `PolAdtEv`의 마지막 쓰기 시간으로, 정책이 마지막으로 변경된 대략적인 시간입니다. |
| `DataRaw` | 값 전체가 디코딩되지 않는 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 알 수 없는 레이아웃 또는 크기, 합계가 맞지 않는 개수, 3을 초과하는 값입니다. |

## 조사 전략 { #investigation-strategy }

1. 어떤 일이 일어나지 않았다고 결론 내리기 전에 해당 하위 범주가 감사되었는지 확인하십시오.
   Process Creation(4688), Logon(4624, 4625), Special Logon(4672), Other Object Access Events
   (예약된 작업, 4698), Security System Extension(서비스 설치, 4697), User Account 및 Security Group
   Management(4720, 4732), File Share(5140), Kerberos(4768, 4769).
2. 정책을 Security 로그와 비교하십시오. 현재 감사되지 않는 하위 범주의 이벤트가 있다면 정책이
   변경되었다는 뜻입니다. 이벤트 4719와 최근의 `KeyLastWrittenUTC`를 찾으십시오.
3. 그룹 정책의 `audit.csv`와 비교하여 도메인 전체 정책과 로컬 변경을 구별하십시오.
4. `PolAdtEv`의 이전 버전을 확인하려면 `parse-registry read --recover-deleted`와 트랜잭션 로그를
   사용하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 고급 보안 감사 정책 설정](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: 감사 상수(하위 범주 GUID)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
