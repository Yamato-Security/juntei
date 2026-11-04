---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![디지털 포렌식의 여신 Juntei](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong>는 이벤트 로그가 <em>아닌</em> Windows 포렌식 아티팩트를 위한 Yamato Security의
크로스 플랫폼 파서입니다. 수집된 증거를 일관된 CSV, JSON, JSONL 레코드로 변환하면서, 조사관이 잘못된 결론을
피하는 데 필요한 컨텍스트를 보존합니다.
</p>

<div class="jt-cta" markdown>
[여기에서 시작 :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[아티팩트 알아보기 :material-archive-search:](artifacts/index.md){ .md-button }
[명령어 레퍼런스 :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## 원시 아티팩트에서 근거 있는 결론까지 { #from-raw-artifact-to-defensible-conclusion }

Juntei는 아티팩트가 무엇을 기록하는지 알려 주며, 이 사이트는 그 기록이 **무엇을 증명하고 무엇을 증명하지
않는지**를 설명합니다. Windows DFIR을 한 번도 수행해 본 적이 없더라도 아티팩트 페이지부터 시작하십시오. 각
가이드는 아티팩트의 용도, 디스크상의 위치, 보존 및 수집 시 주의 사항, 파싱 명령, 출력 필드, 해석, 상관 분석,
그리고 심화 자료를 다룹니다.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __파일 시스템 기록__

    ---

    `$MFT`와 USN 저널을 사용하여 파일의 존재, 이름, 이동, 삭제, 메타데이터 변경 및 대체 데이터 스트림을
    재구성합니다.

-   :material-play-circle-outline:{ .lg .middle } __프로그램 증거__

    ---

    Prefetch는 실행을 강력하게 뒷받침할 수 있습니다. Amcache와 ShimCache는 인벤토리, 경로, 해시, 호환성
    및 과거 컨텍스트를 더해 주지만, 신중한 해석이 필요합니다.

-   :material-account-clock:{ .lg .middle } __사용자 활동__

    ---

    LNK 파일, Jump Lists, RecentDocs 및 SRUM은 열린 항목, 애플리케이션 활동, 이동식 또는 네트워크 위치,
    리소스 사용량 및 과거 사용자 컨텍스트를 드러냅니다.

-   :material-shield-bug:{ .lg .middle } __보안 증거__

    ---

    Defender 격리 데이터는 위협 레이블, 원래 경로, 다운로드 출처, 해시, 소유자를 보존할 수 있으며, 격리된
    분석을 위해 격리된 바이트를 안전하게 복구할 수도 있습니다.

-   :material-database-search:{ .lg .middle } __레지스트리 상태__

    ---

    할당된 레지스트리 데이터와 복구 가능한 레지스트리 데이터를 읽고, 트랜잭션 로그를 재생하며, 서비스,
    예약된 작업, 로컬 계정 및 최근 문서 상태를 검사합니다.

-   :material-link-variant:{ .lg .middle } __추측하지 말고 상관 분석하십시오__

    ---

    하나의 아티팩트만으로 전체 이야기를 알 수는 없습니다. 이 가이드는 독립적인 출처들을 결합하여 증거에
    기반한 타임라인을 만드는 방법을 보여 줍니다.

</div>

!!! warning "포렌식 출력은 증거이지 판정이 아닙니다"
    의심스러워 보이는 경로는 단서입니다. 타임스탬프는 실행이 아니라 파일의 메타데이터를 나타낼 수 있습니다.
    레코드가 없다고 해서 그 행위가 일어나지 않았다는 증명이 되지는 않습니다. 원본 증거를 보존하고, 도구
    버전과 명령을 기록하며, 중요한 결론은 교차 검증하십시오.
