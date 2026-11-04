# 부팅 구성 데이터 (BCD) { #boot-configuration-data-bcd }

## BCD 저장소란 { #what-the-bcd-store-is }

Windows Vista 이상은 부팅 구성을 레지스트리 하이브인 BCD 저장소에 보관합니다. BIOS 시스템에서는
`\Boot\BCD`, UEFI 시스템에서는 EFI 시스템 파티션의 `\EFI\Microsoft\Boot\BCD`입니다. `bcdedit`와
Windows 설치 프로그램이 이를 편집하며, 부팅 관리자는 부팅할 때마다 이를 읽습니다.

저장소에는 각각 GUID로 이름이 붙은 개체가 들어 있습니다.

- 기본 항목, 표시 순서, 메뉴 시간 제한을 가진 **Windows 부팅 관리자** (`{bootmgr}`). UEFI 시스템에서는
  펌웨어의 부팅 순서를 가진 **펌웨어 부팅 관리자** (`{fwbootmgr}`)도 있습니다.
- Windows 설치(복구 환경 포함)마다 하나씩 있는 **Windows 부팅 로더**, **재개 애플리케이션**(최대 절전 모드),
  **메모리 테스트**
- 다른 개체가 설정을 상속하는 **상속 설정** 개체 (`{globalsettings}`, `{bootloadersettings}`, …)
- **장치** 개체 (복구 환경의 램디스크 옵션)

각 개체에는 요소가 있습니다. 설명, 장치와 경로, 그리고 `recoveryenabled`, `bootstatuspolicy`,
`testsigning`, `nointegritychecks`, `safeboot` 같은 설정입니다.

두 종류의 공격이 이를 변경합니다.

- **랜섬웨어**는 시스템을 복구할 수 없도록 복구를 끕니다:
  `bcdedit /set {default} recoveryenabled No` 및
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. 일부 계열은 `safeboot`를 설정하여
  보안 제품이 실행되지 않는 안전 모드로 Windows가 재시작되도록 합니다.
- **루트킷과 서명되지 않은 드라이버**는 부팅 시 검사를 약화시키기 위해 `testsigning on` 또는
  `nointegritychecks on`을 사용할 수 있습니다. 이러한 설정만으로는 루트킷이 실행되었다는 것이 증명되지 않습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 수집 시점에 이 부팅 항목에는 이 설정이
있었습니다. 요소 키의 마지막 쓰기 시간은 해당 설정이 마지막으로 기록된 시점입니다.

## 위치 { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

요소의 유형은 16진수 8자리입니다. 클래스(`1` 모든 애플리케이션, `2` 특정 종류의 애플리케이션, `3` 장치),
형식(`1` 장치, `2` 문자열, `3` 개체, `4` 개체 목록, `5` 정수, `6` 부울, `7` 정수 목록), 그리고 번호로
구성됩니다. 명령어는 Microsoft 문서에 따라 요소 이름을 표시합니다 (`BcdLibraryBoolean_AutoRecoveryEnabled`).

수집 도구는 저장소를 복사하지 않는 경우가 많습니다. EFI 시스템 파티션이 마운트되어 있지 않고 `\Boot`는
숨겨져 있기 때문입니다. 명시적으로 수집하거나 파티션을 마운트하십시오. `-d`는 `BCD-Template`도 읽습니다
(`HiveType`이 어느 쪽인지 알려 줍니다). 템플릿은 Windows가 시작점으로 삼는 것이지, 실제로 부팅에 사용하는
저장소가 아닙니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **설정은 상속됩니다.** 부팅 로더는 `BcdLibraryObjectList_InheritedObjects`에 있는 개체(일반적으로
  `{globalsettings}`를 상속하는 `{bootloadersettings}`)로부터 상속합니다. `RecoveryEnabled`, `TestSigning`,
  `NoIntegrityChecks`, `SafeBoot`, `BootStatusPolicy`는 해석된 저장 값(자체 값, 없으면 상속된 값)이며,
  `Inherited`는 어떤 값이 다른 개체에서 왔는지 알려 줍니다. 상속은 최대 8단계까지 추적되며 순환은 제한됩니다.
  참조된 개체가 불완전한 저장소에는 없을 수 있습니다. 빈 필드가 Windows의 실제 기본값을 확정하지는 않습니다.
  펌웨어와 Secure Boot 역시 저장된 옵션이 적용되지 않도록 할 수 있습니다.
- **설정되지 않음은 꺼짐이 아닙니다.** `RecoveryEnabled`가 비어 있다는 것은 해당 요소가 없다는 뜻이며,
  Windows의 기본값이 적용됩니다.
- **시간은 요소별입니다.** 각 요소의 키 시간(`Elements`의 `key_last_written`)은 해당 요소가 마지막으로
  기록된 시점입니다. Windows는 업데이트와 업그레이드 시 일부 요소를 다시 기록하므로, 다른 요소의 시간과
  비교하십시오.
- **장치는 부분적으로만 디코딩됩니다.** GPT 파티션은 해당 파티션과 디스크의 GUID로, 램디스크는 옵션 개체로
  표시됩니다. 그 밖의 장치는 바이트만 보존합니다 (`data_raw`).
- **문서화되지 않은 요소** (`0x15000065`, `0x4…` 클래스)는 이름이 없으며 바이트를 그대로 보존합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

요약에는 각 저장소의 기본 항목과 모든 테스트 서명, 무결성 검사, 복구, 부팅 상태 정책(모든 `Ignore…`) 및
안전 모드 설정이, 설정된 시간 또는 상속 원본 개체와 함께 표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | 호스트, 하이브, 그리고 `BCD` 또는 `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | 개체의 GUID, 유형(`0x10200003`) 및 개체의 종류(`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | 설명, 애플리케이션 경로 및 장치(GPT 파티션과 디스크, 또는 램디스크의 옵션 개체). |
| `IsDefault`, `DisplayOrder` | Windows 부팅 관리자가 기본으로 부팅하는지 여부와 부팅 메뉴에서의 위치. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | 저장소에서 해석한 `recoveryenabled`, `testsigning`, `nointegritychecks`. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) 및 `bootstatuspolicy` (`IgnoreAllFailures`, …), 숫자 값 포함. |
| `Inherited` | 이 값들 중 어떤 것이 다른 개체에서 왔는지, 그리고 어느 개체에서 왔는지. |
| `Elements` | 모든 요소: 유형, 이름, 값, 바이트(장치, 문서화되지 않은 유형, 디코딩되지 않는 데이터의 경우) 및 키 시간. |
| `KeyLastWrittenUTC` | 개체 키의 마지막 쓰기 시간. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트. |
| `Problems` | 유형이 없는 개체, 데이터가 형식에 맞지 않는 요소. |

## 조사 전략 { #investigation-strategy }

1. 기본 항목과 모든 부팅 로더에서 `TestSigning`, `NoIntegrityChecks`, `RecoveryEnabled = false`,
   `Ignore…` `BootStatusPolicy`, `SafeBoot`를 확인합니다.
2. 요소 시간을 사용하여 변경 시점을 특정하고, Prefetch, Amcache, 프로세스 생성 이벤트(4688, Sysmon 1) 및
   System 이벤트 로그의 `bcdedit.exe`와 상관 분석합니다.
3. 있어서는 안 되는 부팅 항목이나 장치(다른 로더 경로, 램디스크), 그리고 변경된 기본 항목이나 표시 순서를
   찾습니다.
4. 저장소를 `BCD-Template` 및 동일 빌드의 다른 시스템과 비교합니다.

## 추가 정보 { #further-details }

- [Microsoft: BCD 부팅 옵션 참조](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (개체 유형)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
