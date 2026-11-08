# 아티팩트 가이드 { #artifact-guide }

Windows는 파일 시스템, 레지스트리, 호환성 데이터베이스, 사용자 인터페이스 기록, 리소스 사용량 기록 및
보안 제품에 서로 겹치는 흔적을 보존합니다. Juntei는 이러한 출처를 파싱하지만, 각 출처가 무엇을
의미하는지는 여전히 조사자가 해석해야 합니다.

## 질문별 아티팩트 선택 { #choose-an-artifact-by-question }

| 조사 질문 | 시작점 | 교차 검증 |
|---|---|---|
| 이 실행 파일이 실행되었는가? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), 이벤트 로그 |
| 이 파일이 존재한 적이 있는가? | [Amcache](amcache.md), [$MFT](mft.md) | [USN 저널](usn-journal.md), [$I30](i30.md), ShimCache |
| 파일이 언제 생성, 이름 변경 또는 삭제되었는가? | [USN 저널](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), Defender 격리 데이터 |
| 메타데이터가 변경되기 전에 어떤 이름 또는 타임스탬프가 존재했는가? | [$LogFile](logfile.md) | [$MFT](mft.md), [USN 저널](usn-journal.md), [$I30](i30.md) |
| 파일의 소유자는 누구이며, 누가 변경할 수 있는가? | [$Secure:$SDS](secure-sds.md) 및 [$MFT](mft.md) | [SAM](sam.md), Security 이벤트 로그 |
| 바로 가기 또는 Prefetch 파일의 대상은 어느 볼륨에 있었는가? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [USB 장치](usb.md) |
| 사용자가 무엇을 열었는가? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| 프로그램이 대량의 데이터를 전송했는가? | [SRUM](srum.md) | 방화벽, 프록시, DNS, 패킷 또는 EDR 텔레메트리 |
| 사용자가 어떤 웹사이트를 방문했고, 무엇을 다운로드했는가? | [웹 브라우저](browser.md) | 프록시 및 DNS 로그, `$MFT` 및 USN 저널, Defender, SRUM |
| 어떤 클라이언트와 계정이 어느 날에 이 서버를 사용했는가? | [사용자 액세스 로깅 (UAL)](ual.md) | Security 이벤트 로그, SMB 및 RDP 로그, 방화벽 로그 |
| 지속성이 구성되었는가? | [서비스](services.md), [TaskCache](taskcache.md), [레지스트리](registry.md) | 이벤트 로그, Prefetch, `$MFT` |
| Defender가 무엇을 제거했는가? | [Defender 격리 데이터](defender-quarantine.md) | 이벤트 로그, `$MFT`, USN 저널 |
| 로컬 계정이 추가되거나 변경되었는가? | [SAM](sam.md) | Security 이벤트 로그 및 레지스트리 트랜잭션 기록 |

## 필드보다 경고를 먼저 읽기 { #read-warnings-before-fields }

모든 아티팩트 페이지는 같은 순서를 따릅니다.

1. **그것이 무엇인지**, 그리고 어떤 질문에 답할 수 있는지.
2. **어디에서 찾을 수 있는지**, 그리고 수집이 그것에 어떤 영향을 주는지.
3. **그것이 증명하는 것과 증명하지 않는 것.**
4. Juntei로 **파싱하는 방법.**
5. **모든 출력 필드를 읽는 방법.**
6. 독립적인 증거와 **상관 분석하는 방법.**
7. 저수준 연구를 위한 **추가 정보.**

!!! danger "파서의 탐지 결과를 악의성과 동일시하지 마십시오"
    Juntei의 탐지 결과는 `System32` 밖에 있는 시스템 바이너리 이름처럼 검토할 가치가 있는 패턴을
    식별합니다. 설치 프로그램, 관리자, 호환성 계층 및 소프트웨어 업데이트도 특이한 레코드를 생성할 수
    있습니다. 컨텍스트와 독립적인 증거로 검증하십시오.
