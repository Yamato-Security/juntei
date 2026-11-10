# 애플리케이션 호환성 { #application-compatibility }

## Windows가 보관하는 정보 { #what-windows-keeps }

ShimCache(`parse-registry shimcache`)와 Amcache(`parse-registry amcache`) 외에도, Windows의
애플리케이션 호환성 기능은 `Microsoft\Windows NT\CurrentVersion\AppCompatFlags` 아래에 다음 정보를
보관합니다.

| 원본 | 하이브와 키 | 표시하는 내용 |
|---|---|---|
| `layers` | `SOFTWARE`(및 `Wow6432Node`)와 `NTUSER.DAT`의 `Layers` | 호환성 레이어로 실행되도록 설정된 프로그램: `RUNASADMIN`(항상 관리자로 실행), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT`의 `Compatibility Assistant\Store`(Windows 8 이상) | 프로그램 호환성 관리자(Program Compatibility Assistant)가 사용자의 실행을 확인한 프로그램 |
| `pca_persisted` | `NTUSER.DAT`의 `Compatibility Assistant\Persisted`(Windows 7) | Windows 7에서의 동일한 정보 |
| `cit_module` | `SOFTWARE`의 `CIT\Module\<module>` | 추적 대상 모듈을 로드한 프로그램과 마지막으로 로드한 시점 |
| `cit_system` | `SOFTWARE`의 `CIT\System`(Windows 7부터 초기 Windows 10까지) | 압축된 CIT 데이터베이스 내의 프로그램 경로와 명령줄 |
| `installed_sdb` | `SOFTWARE`의 `InstalledSDB\{GUID}` | `sdbinst`로 설치된 사용자 지정 shim 데이터베이스: 파일, 설명, 설치 시간 |
| `custom_sdb` | `SOFTWARE`의 `Custom\<program>` | 각 프로그램에 적용된 사용자 지정 shim 데이터베이스와 그 설치 시간 |

<span class="evidence-strength">가장 타당한 결론:</span> 레이어와 shim 데이터베이스는 프로그램의
실행 방식을 바꾸는 구성입니다. 프로그램 호환성 관리자 및 CIT 엔트리는 프로그램이 해당 사용자로 또는 해당
시스템에서 실행되었음(또는 모듈을 로드했음)을 보여 줍니다.

사용자 지정 shim 데이터베이스는 지속성 및 인젝션 기법입니다(MITRE ATT&CK T1546.011). shim은 프로그램이
실행될 때마다 해당 프로그램에 DLL을 로드하거나 다른 프로그램을 시작할 수 있습니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

`SOFTWARE`, 모든 `NTUSER.DAT` 및 각각의 트랜잭션 로그와, shim 데이터베이스의 경로가 가리키는 `.sdb`
파일(보통 `C:\Windows\AppPatch\Custom\`)을 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **CIT 압축 해제에는 상한이 있습니다.** 압축 해제 크기가 64 MiB를 초과한다고 선언된 값은 진단 메시지와
  함께 원시 바이트로 보존됩니다. 잘못된 압축과 크기 불일치는 보고됩니다.


- **프로그램 호환성 관리자 엔트리에는 실행 시간이 없습니다.** `Store` 데이터(`SACP`, `DataRaw`에
  보존됨)에는 시간이 하나 있지만, Windows 8.1, 10, 11에서는 하이브의 모든 프로그램에 대해 동일하며 다른
  시스템에서도 마찬가지로 프로그램이 실행된 시점이 아닙니다. 가장 최근 변경의 시점을 알려 주는 것은
  `Store` 키의 마지막 쓰기 시간뿐입니다.
- **이동식 미디어의 프로그램**은 `SIGN.MEDIA=<id> <path>` 형식으로 이름이 지정됩니다. `Path`는
  미디어상의 경로이며, `Details`에는 미디어 ID가 들어 있습니다.
- **CIT 데이터베이스**는 LZNT1로 압축되어 있고 구조가 공개 문서화되어 있지 않습니다. Juntei는 각
  데이터베이스에서 발견된 프로그램 경로와 명령줄을 데이터베이스당 한 번씩, 횟수나 시간 없이 나열합니다.
  명령줄은 독립된 레코드이며 해당 프로그램과 연결되지 않습니다. Windows 10 이상에서는 이곳에 남는 정보가
  거의 없습니다.
- **shim 데이터베이스.** 모든 `custom_sdb` 및 `installed_sdb` 레코드를 확인하십시오. 정상 소프트웨어가
  이를 설치하는 경우는 드뭅니다. shim이 무엇을 하는지는 `.sdb` 파일 자체에 기록되어 있습니다.
- 사용자가 설정한 **레이어**는 해당 사용자에게만 적용되며, `SOFTWARE`의 레이어는 모든 사용자에게
  적용됩니다. `RUNASADMIN`은 프로그램이 실행될 때마다 권한 상승을 요청하게 합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

요약에는 사용자 지정 shim 데이터베이스, 레이어가 설정된 프로그램, 그리고 CIT 모듈의 마지막 사용 시점이
나열됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | 호스트, 하이브 파일, 그리고 `SOFTWARE` 또는 `NTUSER.DAT`입니다. |
| `User`, `UserSID` | 사용자 하이브의 프로필 폴더 이름과, 시스템의 `ProfileList`에서 얻은 SID입니다. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` 또는 `user`입니다. |
| `Path` | 프로그램(값의 이름), CIT 데이터베이스 내의 경로나 명령줄, 또는 shim 데이터베이스가 적용되는 프로그램입니다. |
| `Layers` | 기록된 그대로의 호환성 레이어입니다(`~ RUNASADMIN`). |
| `LastSeenUTC` | 프로그램이 CIT 모듈을 마지막으로 로드한 시점입니다. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | shim 데이터베이스의 GUID, 파일 및 설치 시간입니다. |
| `Details` | CIT 모듈 이름, CIT 데이터베이스의 값 이름, 프로그램의 이동식 미디어 ID(`SignMedia`), shim 데이터베이스의 `DatabaseDescription` 및 `DatabaseType`입니다. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | 레코드의 출처, 해당 키가 마지막으로 변경된 시점, 그리고 `Wow6432Node` 아래에 있는지 여부입니다. |
| `DataRaw` | `Store` 값의 `SACP` 데이터, 또는 예상과 다른 데이터(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 디코딩하거나 읽을 수 없었던 데이터입니다. |

## 조사 전략 { #investigation-strategy }

1. 모든 사용자 지정 shim 데이터베이스에 대해 `.sdb` 파일, 설치 시간 및 적용 대상 프로그램을
   검토하십시오. Prefetch, Amcache 및 프로세스 생성 이벤트의 `sdbinst.exe`와 상관 분석하십시오.
2. 프로그램 호환성 관리자 엔트리 중 사용자가 쓰기 가능한 폴더에 있는 프로그램과 이동식 미디어에서 실행된
   프로그램을 찾으십시오.
3. 특이한 프로그램에 설정된 `RUNASADMIN` 및 기타 레이어를 확인하십시오.
4. 실행 시간은 ShimCache, Amcache, BAM, Prefetch 및 UserAssist와 상관 분석하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 애플리케이션 호환성 데이터베이스](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
