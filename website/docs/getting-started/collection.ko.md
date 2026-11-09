# 증거 수집 { #collecting-evidence }

Juntei는 Windows에서 이미 수집한 파일을 파싱합니다. 신뢰할 수 있는 수집 도구를 사용하고 가능한 한 원본 메타데이터를 보존하십시오. VHDX 및 E01 디스크 이미지는 추출 없이 파싱할 수 있습니다. [디스크 이미지 마운트](../commands/mount.md)를 참조하십시오.

## 일반적인 위치 { #common-locations }

| 아티팩트 | 일반적인 위치 |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS 볼륨 메타데이터 파일 `C:\$MFT` |
| `$LogFile` | NTFS 볼륨 메타데이터 파일 `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN 저널 | NTFS 메타데이터 스트림 `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| 사용자 액세스 로깅(Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer와 레거시 Edge(WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| 웹 브라우저 | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` 및 기타 브라우저의 프로필 폴더 |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` 및 기타 사용자/시스템 위치 |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` 및 `CustomDestinations` |
| ShimCache | `SYSTEM` 하이브: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender 격리 | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| 머신 레지스트리 | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| 사용자 레지스트리 | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## 함께 수집해야 하는 파일 { #files-that-must-travel-together }

- 레지스트리 하이브는 인접한 `.LOG`, `.LOG1`, `.LOG2` 파일과 함께 수집하십시오. 하이브가 더티 상태이면
  Juntei는 레지스트리 명령어와 ShimCache에 대해 로그를 재생할 수 있습니다.
- SRUM의 `SRUDB.dat`는 일관된 상태로 수집하십시오. Juntei는 현재 더티 상태의 ESE 데이터베이스를 보고하지만
  `SRU*.log`를 재생하지는 않으므로, 해당 로그에만 존재하는 레코드는 사용할 수 없습니다.
- 가능하면 `$MFT`를 `$Extend/$J`와 함께 수집하십시오. USN 레코드는 전체 경로가 아니라 상위 파일 참조를
  저장하므로, MFT가 있으면 경로 해석이 향상됩니다.
- `$LogFile`은 같은 볼륨의 `$MFT`와 최대한 가까운 시점에 수집하십시오. `$LogFile`은 빠르게 순환하므로, 나중에
  수집하면 이전의 재실행/실행 취소(redo/undo) 레코드를 완전히 잃을 수 있습니다.
- 계정 및 Wi-Fi 프로필 이름을 해석하려면 SRUM을 일치하는 `SOFTWARE` 하이브와 함께 수집하십시오.
- 사용자 액세스 로깅의 `Sum` 디렉터리 전체를 수집하십시오. `SystemIdentity.mdb`가 역할의 이름과 연도별
  `{GUID}.mdb` 데이터베이스를 알려 줍니다. Juntei는 UAL의 `Svc*.log` 트랜잭션 로그도 재생하지 않습니다.
- 브라우저 데이터베이스는 옆에 있는 `-wal` 파일(`places.sqlite-wal`,
  `cookies.sqlite-wal`)과 함께 수집하십시오. 최신 변경 사항과, 삭제된 항목을 복구하는 데 쓰이는 이전 페이지
  버전은 브라우저가 체크포인트를 수행할 때까지만 그곳에 있습니다.
- `WebCache` 폴더 전체를 수집하십시오. Windows는 사용자가 로그온해 있는 동안 `WebCacheV01.dat`를 열어 두며,
  최신 변경 사항은 Juntei가 재생하지 않는 `V01*.log` 트랜잭션 로그에만 있습니다.
- Entries, ResourceData, Resources를 대응시킬 수 있도록 Defender `Quarantine` 트리 전체를 수집하십시오.

## 라이브 시스템에 관한 주의 사항 { #live-system-cautions }

열려 있는 파일은 복사 중에 변경될 수 있습니다. 레지스트리 하이브와 ESE 데이터베이스는 내부적으로 더티 상태일
수 있습니다. ShimCache는 종료 시까지 메모리 내 최신 항목을 기록하지 않을 수 있습니다. 일반 파일 복사보다는
볼륨 스냅샷이나 원시 접근을 사용하는 포렌식 수집 도구가 바람직합니다.

## 안전 { #safety }

증거에는 악성코드, 익스플로잇 샘플, 악성 LNK 파일, 스프레드시트 수식 텍스트가 포함될 수 있습니다. 격리된
시스템에서 분석하고, 복구된 스트림을 실행하지 말며, 위험한 콘텐츠를 다룰 의도가 있을 때만 `--extract`를
사용하십시오. 손대지 않은 원본은 별도로 보존하십시오.
