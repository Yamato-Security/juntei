# 파서 검증 { #parser-validation }

Juntei가 유지 관리하는 파서 크레이트는 공개 증거와 독립적인 구현을 대상으로 테스트됩니다. 이러한 비교는 회귀와
해석 차이를 발견하는 데 도움이 되지만, 어떤 파서도 무오류로 만들지는 않습니다. 원본 증거를 보존하고, 각
레코드의 문제 또는 손상 필드를 검토하며, 중대한 발견 사항은 다른 아티팩트나 구현으로 검증하십시오.

아래 내용은 이전에 Juntei의 메인 README에 있던 검증 정보를 보존한 것입니다. 최신 테스트 픽스처와 구현 세부
사항은 링크된 파서 저장소를 참조하십시오.

## Amcache { #amcache }

Juntei는 Security Ronin의 [amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)을 Apache-2.0
라이선스로 포크한 [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)를 사용합니다. 그
[출력 문서](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)는 모든 실행 파일, 장치,
발견 사항, 요약 필드를 정의합니다.

## 마스터 파일 테이블 { #master-file-table }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)의 mft 크레이트는 Omer Ben-Amram의
[mft](https://github.com/omerbenamram/mft) 크레이트를 유지 관리하는 포크입니다. 크레이트의 샘플, NIST 해킹
사례, DFIR Artifact Museum의 파일 6개, 총 8개의 `$MFT` 파일에 걸쳐 1,048,733개의 CSV 행과 열이 참조 파서와
순서대로 일치했습니다. 비교는 기본 출력, `--all-times`, `--short-names`, `--cutoff`를 대상으로 했으며,
타임스탬프 표시 방식은 의도적인 차이였습니다.

## NTFS 트랜잭션 로그 ($LogFile) { #ntfs-transaction-log-logfile }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)의 logfile 크레이트는
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog)를 기반으로 유지 관리됩니다. 그 검증은 DFIR Artifact
Museum의 완전한 로그 6개, 공개된 버전 1.1 및 2.0 픽스처, 디코딩된 재실행/실행 취소(redo/undo) 메타데이터,
그리고 독립적으로 파싱한 NTFS 데이터와 비교한 저널 쓰기를 다룹니다. 상세한 커버리지와 알려진 차이는 파서
저장소를 참조하십시오.

Juntei는 공개 go-ntfs 테스트 볼륨(2,129개 레코드)에서 모든 CSV 필드를 JSON 및 JSONL과 비교하고, 분리된
재실행/실행 취소 데이터와 원시 페이로드 길이를 검증하며, 손상된 로그를 테스트하고, 증거 덮어쓰기를 거부하며,
현재 MFT 경로가 재사용된 레코드 번호를 따라가지 않도록 하는 CLI 테스트를 추가합니다. 회귀 테스트 케이스는
영향을 받은 MFT 레코드와 그 쓰기에 사용된 열린 속성을 구별하고, 평면 디렉터리 항목/데이터 속성 메타데이터를
확인하며, 볼륨 간 MFT 짝짓기를 방지합니다. 다중 볼륨 `auto-parse` 테스트는 별도의 CSV 파일과 출처 정보를 갖춘
단일 JSONL 파일을 다룹니다.

이 명령어는 비공개로 수집된 Windows 로그 6개에서도 확인되었습니다. 내보낸 CSV 레코드 708,465개 모두가 JSONL과
필드 단위로 일치했으며, 호스트별 `auto-parse` 내보내기도 같은 레코드 수를 유지했습니다. 원본 증거는 저장소와
함께 배포되지 않습니다. 이러한 확인은 출력의 일관성을 검증하는 것이며, 기록된 모든 트랜잭션이 완료된 사용자
동작을 나타낸다는 것을 검증하지는 않습니다.

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)는 ForensicRS의
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)를 유지 관리하는 포크입니다. 디코딩된 출력은
libyal의 독립적인 Prefetch 구현인 [libscca](https://github.com/libyal/libscca)와 비교되었습니다.

## USN 저널 { #usn-journal }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)의 usn 크레이트는
[Sootmark usn](https://github.com/Sootmark/usn)을 유지 관리하는 포크입니다. 실제 저널 6개에서, Juntei가 시간을
고려하여 재구성한 상위 경로는 독립 파서가 경로를 해석한 모든 경우에 그 결과와 일치했습니다. Juntei는 또한 비교
대상이 해석하지 못한 4,321개의 레코드를 해석했습니다. 카버는 880 MB의 `$MFT` 데이터를 대상으로 테스트하는 동안
잘못된 레코드를 하나도 생성하지 않았습니다.

## 디렉터리 인덱스 ($I30) { #directory-indexes-i30 }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) mft 크레이트의 `i30` 모듈은 INDX 레코드와 그
슬랙을 읽습니다. ntfs-3g로 만든 볼륨의 `$I30` 스트림 2개(Juntei의 `test_files/i30/`)와 Windows가 기록한 공개
`$I30` 샘플 3개에서, 사용 중인 항목과 슬랙의 항목은 Maxim Suhanov의
[dfir_ntfs](https://github.com/msuhanov/dfir_ntfs)가 보고한 항목과 같은 이름 및 시간으로 일치했습니다.
dfir_ntfs가 32비트를 넘는 상위 참조(상위 참조 위에 덮어쓰인 바이트)를 보고하는 경우, Juntei는 상위 참조를
비워 둡니다.

## 부트 섹터 ($Boot) { #boot-sector-boot }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)의 boot 크레이트는 Yamato Security가 자체
개발한 것입니다. DFIR Artifact Museum의 공개 `$Boot` 파일 5개(Windows 10, 11 및 Server 2012 R2)에서 모든 필드가
참조 파서와 일치했습니다. 512, 1,024, 4,096바이트 섹터와 4 KiB에서 64 KiB 클러스터로 `mkntfs`를 사용해 만든
볼륨에서, 크기, `$MFT` 및 `$MFTMirr` 클러스터, 일련 번호가 The Sleuth Kit의 `fsstat`와 일치했습니다. Museum의
Windows 11 `$Boot`는 같은 호스트의 LNK 파일이 기록한 볼륨 일련 번호를 가지고 있으며, 5개 파일은 같은 부트 코드를
가지고 있습니다.

## 보안 설명자 ($Secure:$SDS) { #security-descriptors-securesds }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)의 secure 크레이트는 Yamato Security가 자체
개발한 것입니다. Museum의 `$SDS` 스트림 5개(보안 설명자 7,808개, ACE 60,781개)에 걸쳐 보안 ID, 해시, 오프셋,
소유자, 그룹, 제어 플래그, ACE 수가 참조 파서와 일치했으며, 모든 ACE의 유형, 플래그, 마스크, SID, 개체 유형이
`dissect.ntfs`와 일치했습니다. 기록된 모든 해시는 해당 보안 설명자의 해시이고, 모든 미러 사본은 해당 항목과
일치하며, 3,384개의 콜백 ACE 조건을 포함하여 모든 보안 설명자에 SDDL이 있습니다.

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)는 Sootmark의
[ese](https://github.com/Sootmark/ese)와 [srum](https://github.com/Sootmark/srum) 크레이트를 기반으로 합니다.
실제 데이터베이스 11개에 걸쳐, 출력된 45,255개 항목의 모든 열이 `dissect.esedb`가 제공하는 값을 독립적으로
디코딩한 결과와 일치했습니다.

## 사용자 액세스 로깅 { #user-access-logging }

`parse-ese ual`은 같은 ESE 리더로 UAL 데이터베이스를 읽습니다. 실제 데이터베이스 16개(308개 행, 그중 클라이언트
행 145개)에 걸쳐 모든 값이 `dissect.esedb`가 제공하는 값을 독립적으로 디코딩한 결과와 일치했습니다. 모든
클라이언트 행에서 요청 수가 있는 날짜는 `InsertDate`의 UTC 날짜에서 시작하여 `LastAccess`의 UTC 날짜에서
끝났으며, 그 합계는 `TotalAccesses`와 일치했습니다.

## 웹 브라우저 { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser)는 Sootmark의 browser, sqlite, plist
크레이트를 유지 관리하는 포크로, 같은 데이터베이스를 대상으로 다른 Rust 라이브러리 및 Plaso, Velociraptor
(SQLiteHunter), Hindsight와 비교한 끝에 선택되었습니다. `parse-sqlite browser`는 공개된 데이터베이스
38개, 즉 Plaso의 테스트 파일(Chrome 8~68, Firefox 2~118), Chromium 150과 Firefox 140으로 만든 프로필,
ys-browser-parser의 양식 기록 테스트 데이터베이스를 대상으로, 각 데이터베이스의 사본을 `-wal` 파일과 함께
`sqlite3` 라이브러리로 읽은 결과와 비교되었습니다. 방문 293건, 다운로드 21건, 쿠키 692개, 양식 항목 41개가
모두 일치했습니다. 삭제된 항목은 실제로 항목이 삭제된 곳에서만 복구되었습니다. Firefox 140 방문 기록에서
삭제된 페이지 2개와 방문 2건이 그 시각, URL, 제목과 함께 복구되었고, 테스트 데이터베이스의 삭제된 양식 항목도
복구되었습니다.

## Internet Explorer와 레거시 Edge(WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache`는 [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser)로
`WebCacheV01.dat`를 읽습니다. Plaso의 공개 WebCache 데이터베이스 2개, 즉 Internet Explorer 11의 데이터베이스와
레거시 Edge의 방문 기록, 다운로드, 쿠키가 담긴 더티 상태의 Windows 10 데이터베이스를 대상으로
`dissect.database`의 ESE 리더와 비교되었습니다. 4,156개 행 모두가 값 단위로 일치했습니다. 컨테이너 44개,
방문 기록 항목 182개, 다운로드 6건, 쿠키 170개, 기타 항목 3,754개입니다. 비교 과정에서는 다운로드 레코드를
독자적으로 디코딩했습니다. 데이터베이스가 삭제됨으로 표시한 레코드는 양쪽 모두에서 제외했습니다.

## Outlook { #outlook }

`parse-file outlook`은 [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)로 PST, OST,
`.msg`, `.oft` 파일을 읽습니다. 이 파서의 PST 및 OST 리더는 Microsoft의
[outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs)를 포크한 것이고, `.msg` 리더는 Microsoft의 공개
사양을 바탕으로 작성되었습니다.

- **PST와 OST**: 공개된 PST 및 OST 파일 66개(Apache Tika, artemis, Aspose.Email 예제, DFIR Artifact Museum,
  Enron, GroupDocs 예제, java-libpst, libyal의 테스트 데이터, outlook-pst-rs, pst-extractor)를 대상으로 libpff와
  속성 단위로 비교했습니다. 모든 저장소, 폴더, 메시지, 받는 사람, 첨부 파일의 모든 속성과 모든 첨부 파일의
  데이터가 대상입니다. 324,175개의 값이 같았으며, 설명되지 않는 차이는 하나도 없었습니다. 나머지는
  ys-outlook-parser만 읽는 것(libpff가 읽지 못하는 메시지, 받는 사람, 첨부 파일과 libpff가 열지 못하는
  저장소의 값 15,846개, 숨겨진 항목의 값 8,223개), 행 정보상 존재하지 않는데도 libpff가 빈 값이나 0으로
  내놓는 받는 사람 셀(49,077개), MAPI와 같은 방식으로 계산한 폴더 엔트리 ID와 유형(2,004개), libpff가
  원시 데이터로 내놓는 개체와 포함된 메시지(480개)입니다. 그중 2개 파일은 어느 리더도 전혀 읽지 못합니다.
- **`.msg`와 `.oft`**: 메시지, 받는 사람, 첨부 파일 2,404개와 속성 41,002개가 담긴 공개 파일 214개를
  extract-msg와 비교했습니다. 110,114개의 값이 같았고, 다른 791개는 모두 설명됩니다. 601개는 extract-msg의
  시각 오류이고, 35개는 extract-msg가 잘못된 코드 페이지로 디코딩하는 ANSI 파일에서 비롯되며, 나머지는
  extract-msg가 빠뜨리거나 덧붙이는 것입니다.
- **손상된 입력**: 변형된 `.msg` 파일 260,000개 이상과 변형된 PST 160,000개, 그리고 특수 제작된 파일을
  충돌, 멈춤, 무제한 메모리 사용 없이 읽었습니다.

## LNK 바로 가기 { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)는 Sootmark의
[shell](https://github.com/Sootmark/shell) 크레이트를 유지 관리하는 포크입니다. 공개된 바로 가기 1,118개를
대상으로 비교되었습니다. 참조 파서와 공유하는 30개의 CSV 열 중 16개의 개별 값이 달랐습니다. 손상되었거나,
잘렸거나, 자리 표시자이거나, MTP 또는 제어판 항목이 관련된 대상 경로 13개, 알 수 없거나 잘린 블록이 관련된 추가
데이터 블록 목록 2개, 그리고 참조 파서가 다른 바이트에서 읽은 MFT 참조 1개입니다. Juntei는 손상된 구조를 조용히
정규화하지 않고 보고합니다.

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)는 Sootmark의 복합 파일 및 Jump
List 관련 작업을 사용하며, 포함된 링크는 `ys-lnk-parser`로 디코딩합니다. 공개 Jump Lists 77개에 걸쳐, 100 ns
단위의 타임스탬프를 포함한 13,057개의 공유 비교 셀 모두가 참조 파서와 일치했습니다. 참조 파서가 거부한 파일
6개(빈 사용자 지정 목록, 바닥글 누락, 또는 Jump List가 아닌 파일)는 Juntei가 디코딩하거나 명시적으로
보고했습니다.

## 휴지통 { #recycle-bin }

`parse-file recycle-bin`은 Juntei의 자체 파서입니다. [rifiuti2](https://github.com/abelcheung/rifiuti2)의 공개
샘플에서 rifiuti2가 읽는 모든 레코드는 인덱스, 삭제 시각(rifiuti2가 표시하는 대로 초 단위까지), 제거 상태,
크기, 경로가 모두 같았습니다. 총 92개 레코드로, 67개는 Windows 95부터 Server 2003까지의 INFO2 파일 11개에서,
25개는 Windows Vista 및 Windows 10의 `$I` 파일이 들어 있는 폴더 3개에서 나왔습니다. INFO2의 ANSI 경로는
rifiuti2 자체 테스트가 사용하는 코드 페이지에서 일치했습니다. Juntei는 rifiuti2가 거부하는 파일도 읽습니다.
두 가지 `$I` 버전이 섞인 폴더, 경로가 없는 `$I` 헤더, 잘린 레코드가 그것이며, 각각의 손상 내용을 보고합니다.
Plaso 테스트 데이터의 `$I` 및 INFO2 파일에서는 Plaso 테스트가 기대하는 시각, 크기, 경로, 인덱스, 드라이브를
얻습니다. Digital Corpora의 공개 M57-Jean 디스크 이미지(Windows XP)를 `auto-parse --e01`로 읽었을 때 INFO2
레코드는 rifiuti2와 일치했고, 그 옆에 있는 휴지통 파일 `Dc1.jpg`의 이름도 Juntei가 부여한 이름과 같았습니다.

## RDP 비트맵 캐시 { #rdp-bitmap-cache }

`parse-file rdp-cache`는 Juntei의 자체 파서입니다. CODE BLUE 2026 CTF 수집 데이터의 호스트 6대 중 하나인
kofu에서는 `Cache????.bin` 파일 3개에 들어 있는 타일 1,911개가 모두
[bmc-tools](https://github.com/ANSSI-FR/bmc-tools)가 만드는 이미지와 픽셀 단위로 같습니다. ApoorvCTF 2025의
"Phantom Connection" 문제에서 공개된 `Cache0000.bin`의 타일 1,980개도 마찬가지이며, 그중 105개는
64 × 48픽셀입니다. bmc-tools는 모든 타일을 너비 64픽셀로 읽기 때문에 화면 오른쪽 가장자리의 더 좁은 타일이
비뚤어져 나옵니다. Juntei는 각 타일을 그 타일의 너비대로 읽습니다.

타일이 들어 있는 공개 `.bmc` 캐시는 찾지 못했습니다. 위 샘플들도 다른 Windows 10 이상 클라이언트와
마찬가지로 빈 `bcache24.bmc`만 가지고 있습니다. 따라서 `.bmc` 읽기는 형식에 맞춰 만든 파일로
테스트했습니다. 네 가지 색 깊이 모두의 비압축 타일, 작은 타일 뒤에 이전 타일의 끝부분이 이어지는 경우,
손상된 슬롯, 그리고 MS-RDPBCGR 3.1.9의 모든 오더(order)를 사용하는 무작위 RLE 압축 16비트 타일
3,000개이며, 이 타일들에서는 어떤 오더도 타일의 첫 행 끝을 넘어 이어지지 않습니다. Juntei와 bmc-tools는
3,000개 모두를 같은 픽셀로 디코딩합니다. 오더가 첫 행에서 시작해 그 행을 넘어 이어지면 둘의 결과가
달라집니다. Juntei는 사양과 FreeRDP 디코더를 따라 오더 전체에 첫 행의 규칙을 유지하지만, bmc-tools는 오더
중간에 규칙을 바꿉니다.

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)는 Security Ronin의
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)을 유지 관리하는 포크로, 레지스트리
하이브 및 트랜잭션 로그 지원이 추가되었습니다. 공개 `SYSTEM` 하이브 20개(제어 집합 24개, 항목 7,290개)와 단독
AppCompatCache 값 17개에서, 모든 항목이 참조 파서가 출력한 경로, 시간, 실행 플래그, 플래그, 위치와
일치했습니다.

## Microsoft Defender 격리 { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)는 Sootmark의
[defender](https://github.com/Sootmark/defender) 크레이트를 유지 관리하는 포크입니다. 파서의 테스트 도구가
Defender 형식으로 기록한 격리 폴더는 독립적인 `dissect.target` 격리 리더에서도 일관되게 디코딩됩니다.

## 레지스트리 기반 아티팩트 { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)는 할당된 키, 값, 더티 하이브
트랜잭션 재생, 빈 셀 복구, 셀 슬랙, 하이브 빈 이후의 데이터, 재생으로 제거되거나 대체된 레코드에 대해
[yarp](https://github.com/msuhanov/yarp)와 비교됩니다. 아티팩트별 비교는 다음과 같습니다.

- Windows 7 및 더티 상태의 Windows 11 `SYSTEM` 하이브에서 서비스, 값, 실패 시 동작
- 공개 `NTUSER.DAT` 하이브 9개에서 yarp 및 libfwsi로 디코딩한 RecentDocs
- Windows 7–11, Server 2012 R2, Server 2022를 포함하는 공개 `SOFTWARE` 하이브 7개에서 yarp로 독립적으로
  디코딩한 예약된 작업
- 공개 `SAM` 하이브 7개에서 `dissect.target`과 비교한 SAM 계정 구조

참조 도구, 형식 연구, 사양, 공개 증거 출처의 전체 목록은 [감사의 말](../acknowledgements.md)을 참조하십시오.
