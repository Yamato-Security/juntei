# 보안 설명자 (`$Secure:$SDS`) { #security-descriptors-securesds }

## `$SDS`란 { #what-sds-is }

NTFS의 모든 파일과 디렉터리에는 **보안 설명자**가 있습니다. 보안 설명자는 소유자, 기본 그룹, **DACL**(누가
무엇을 할 수 있는지), **SACL**(어떤 액세스를 감사하는지와 무결성 레이블)로 구성됩니다. NTFS는 서로 다른
설명자를 `$Secure` 파일(MFT 엔트리 9)의 `$SDS` 스트림에 한 번씩만 보관하고, 각 설명자에 **보안 ID**를
부여합니다. 파일의 `$STANDARD_INFORMATION`에는 이 보안 ID만 기록되며, 이는
[`parse-ntfs mft`](mft.md)의 `SecurityID` 열입니다. `$SDS`는 이를 소유자와 ACL로 변환해 줍니다.

`$SDS`는 256 KiB 블록으로 구성되며, 각 블록 뒤에는 그 사본(미러)이 이어집니다. 각 엔트리에는 설명자의
해시가 있으며, NTFS는 이를 사용하여 이미 존재하는 설명자를 찾습니다(`$SDH`). 또한 보안 ID도 있습니다
(`$SII`가 이를 인덱싱합니다).

<span class="evidence-strength">가장 타당한 결론:</span> `SecurityID`가 해당 설명자를 가리키는 파일들은
`$MFT` 레코드가 마지막으로 기록되었을 때 그 소유자와 ACL을 가지고 있었습니다. 파일의 소유자는 일반적으로
파일을 생성한 계정이지만, 관리자, 설치 프로그램 또는 소유권을 가져오는 프로그램이 이를 변경할 수 있습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

원시 접근을 지원하는 포렌식 도구로 수집하십시오. `:`는 파일 이름에 사용할 수 없으므로 수집 데이터에서는
보통 `$Secure_$SDS`라는 이름이 붙습니다. 같은 볼륨의 `$MFT`도 함께 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 소유자는 SID입니다. 잘 알려진 SID(`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …)와 머신 또는 도메인의 기본 제공 계정(`-500`
  `Administrator`, …)만 이름이 표시됩니다. 나머지는 호스트의 `SAM`([SAM 계정](sam.md)), 프로필 목록
  ([시스템 정보](system-info.md)) 또는 도메인을 사용하여 해석하십시오.
- 설명자는 소유자와 ACL이 같은 모든 파일이 공유하며, 해당 파일이 삭제된 뒤에도 보통 `$SDS`에 남아
  있습니다. `$MFT`에 파일이 없는 설명자가 그 자체로 삭제된 파일을 의미하지는 않습니다.
- ACE는 액세스 기록이 아닙니다. DACL은 누가 파일에 액세스*할 수 있는지*를 나타내며, 실제로 누가 액세스했는지는
  이벤트 로그와 다른 아티팩트가 알려 줍니다. SACL의 감사 항목은 구성을 나타낼 뿐 무엇이 기록되었는지를
  나타내지 않습니다.
- ACE는 ACL 내 순서대로 나열됩니다. Juntei는 유효 액세스를 계산하지 않습니다. 거부 항목, 상속, 그룹
  구성원 자격, 권한(privilege), 소유자의 암묵적 권한이 모두 영향을 미칩니다.
- `DACL`과 `SACL`은 각 ACE를 `icacls`와 같은 방식으로 표기합니다. `SDDL`은 지원되는 사용 권한을
  표현하며, 마스크는 16진수로, 계정은 SID로 나타냅니다. 바이너리 설명자를 바이트 단위로 그대로 복사한 것이
  아니며, 표현할 수 없는 조건식 텍스트가 있으면 비워 두고 진단 메시지를 출력합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

요약에는 설명자 수가 집계되고, 소유자별로 소유한 설명자 수와 함께 소유자가 나열되며, NULL DACL(제한이
전혀 없음)을 가진 설명자와 손상 사항이 표시됩니다.

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 해석 |
|---|---|
| `SourcePath` | 입력 파일입니다. |
| `SecurityID` | 보안 ID입니다. 같은 볼륨에 대한 [`parse-ntfs mft`](mft.md)의 `SecurityID`와 조인하십시오. |
| `Hash` | 설명자와 함께 기록된 해시입니다. |
| `EntryOffset` | 스트림 내에서 엔트리의 위치입니다. |
| `FromMirror` | 미러 블록에서 읽은 사본이면 `True`입니다. 해당 엔트리와 내용이 다른 사본(엔트리 다음에 나열됨) 또는 손상된 엔트리의 사본입니다. |
| `Size` | 20바이트 헤더를 포함한 엔트리의 크기입니다. |
| `Owner`, `OwnerName` | 소유자의 SID와, 잘 알려진 경우 그 이름입니다. |
| `Group`, `GroupName` | 기본 그룹의 SID와, 잘 알려진 경우 그 이름입니다. Windows는 이를 액세스 검사에 사용하지 않습니다. |
| `Control` | 설명자의 제어 플래그입니다: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED`(상속 차단), `SE_DACL_AUTO_INHERITED`, …가 `|`로 연결됩니다. |
| `DACL-ACE-Count`, `DACL` | `; `로 구분된 DACL 항목입니다. DACL이 없으면 비어 있고, 설명자가 모든 사용자에게 모든 권한을 부여하면 `NULL DACL: no restriction`입니다. |
| `SACL-ACE-Count`, `SACL` | SACL 항목입니다: 감사 항목, 필수 무결성 레이블, 프로세스 신뢰 레이블 및 리소스 특성. |
| `SDDL` | SDDL로 표현한 설명자 전체입니다. 디코딩되지 않은 부분이 있을 때만 비어 있습니다. |
| `Problems` | 설명자와 일치하지 않는 해시, 디코딩할 수 없는 부분, 유형을 알 수 없는 ACE입니다. |

`DACL`과 `SACL`의 각 항목은 `icacls`가 표기하는 방식대로 기록됩니다: 계정, 허용 항목이 아닌 경우 그 유형,
상속 플래그, 그리고 권한입니다.

| 부분 | 의미 |
|---|---|
| 계정 | 잘 알려진 이름 또는 SID입니다. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | 허용 항목이 아닌 경우의 ACE 유형입니다. 필수 레이블에는 유형이 표시되지 않습니다. |
| `(I)` | 상위 디렉터리에서 상속되었습니다. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | 파일에 상속(개체 상속), 디렉터리에 상속(컨테이너 상속), 이 개체가 아닌 하위 항목에만 상속(상속 전용), 한 단계 아래까지만 상속(전파 안 함)입니다. |
| `(SA)`, `(FA)` | 성공 또는 실패한 액세스에 대한 감사 항목입니다. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | 모든 권한, 수정, 읽기 및 실행, 읽기, 쓰기입니다. |
| `(RD,WD,AD,…)` | 그 밖의 특정 권한입니다: 데이터 읽기, 데이터 쓰기, 데이터 추가, 확장 특성 읽기 및 쓰기(`REA`, `WEA`), 실행(`X`), 하위 항목 삭제(`DC`), 특성 읽기 및 쓰기(`RA`, `WA`), 삭제(`DE`), 읽기 제어(`RC`), DAC 쓰기(`WDAC`), 소유자 쓰기(`WO`), 동기화(`S`), 그리고 일반 권한 `GA`, `GR`, `GW`, `GE`. `N`은 액세스 없음입니다. |
| `(NW)`, `(NR)`, `(NX)` | 필수 레이블의 정책입니다: 상위 쓰기 금지, 상위 읽기 금지, 상위 실행 금지. |
| `if (…)` | `(WIN://SYSAPPID Contains "…")` 같은 콜백 ACE의 조건입니다. 해당 조건이 참인 프로세스에만 항목이 적용됩니다. Windows는 이를 사용하여 앱 컨테이너 패키지에 액세스 권한을 부여합니다. |

프로세스 신뢰 레이블과 리소스 특성은 마스크를 16진수로 표시하며, 리소스 특성은 클레임
(`("IMAGELOAD",TU,0x0,1)`)도 표시합니다.

JSON/JSONL은 같은 필드와 이름을 사용하며, 숫자는 숫자로, 빈 값은 `null`로, `Problems`는 목록으로
표현됩니다.

## 조사 전략 { #investigation-strategy }

1. 같은 볼륨의 `$MFT`를 파싱하고 그 `SecurityID`를 이 출력의 보안 ID와 조인하십시오. 그러면 각
   파일의 소유자와 ACL을 얻을 수 있습니다.
2. 사용자 계정(`S-1-5-21-…`)인 소유자를 살펴보십시오. 의심스러운 계정이 어떤 파일을 소유하는지, 어떤 계정이
   시스템 디렉터리의 파일을 소유하는지 확인하십시오.
3. 프로그램이 실행되거나 로드되는 디렉터리에서 허용 범위가 넓은 ACL을 찾으십시오: `(F)`, `(M)` 또는
   `(W)`가 부여된 `Everyone`, `Authenticated Users` 또는 `User`, 그리고 NULL DACL.
4. 파일 하나만 사용하거나 특이한 파일 집합이 사용하는 설명자는 정상적인 설치 과정 밖에서 생성되거나 변경된
   파일을 드러낼 수 있습니다.
5. 소유권 및 사용 권한 변경은 Security 이벤트 로그 항목(4670, SACL이 있는 경우 4663)과
   [USN 저널](usn-journal.md)의 `SECURITY_CHANGE` 레코드로 뒷받침하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 보안 설명자](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (보안 설명자, ACE, 조건식)
- [Microsoft: 보안 설명자 문자열 형식(SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS 형식 문서](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`secure` 크레이트)

구조화된 출력을 stdout으로 보내는 경우(`-o` 없이 `-t` 사용) 스트림 수준의 복구 경고는 stderr로 출력되므로,
파이프에서는 손상되거나 복구된 영역을 숨기지 않으면서도 읽을 수 있는 CSV/JSON/JSONL이 유지됩니다.
