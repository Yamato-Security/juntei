# 첫 번째 조사 { #first-investigation }

이 절차는 `evidence/`에 오프라인 트리아지 수집 데이터를 받았다고 가정합니다.

## 1. 원본 보존 { #1-preserve-the-source }

검증된 사본으로 작업하십시오. 해시, 수집 시간, 시간대, 호스트, 볼륨 및 수집 도구를 기록하십시오. Juntei는
증거를 읽기만 하지만, 상주 스트림이나 격리 데이터 추출 같은 옵션은 새 출력 파일을 생성하므로 그 출력은 별도의
분석 디렉터리로 지정하십시오.

## 2. 보유한 증거 목록 작성 { #2-inventory-what-you-have }

[증거 수집](collection.md)에 나열된 경로를 찾아보십시오. 일부 아티팩트가 없는 것은 정상입니다. 수집 데이터에
레지스트리 하이브가 있다고 해서 트랜잭션 로그도 있다고 가정하지 마십시오.

## 3. 먼저 요약 실행 { #3-run-summaries-first }

많은 명령어는 출력 파일을 지정하지 않으면 간결한 요약을 출력합니다.

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

레코드를 신뢰하기 전에 손상, 더티 데이터베이스 및 적용되지 않은 로그에 대한 경고를 읽으십시오.

여러 컴퓨터의 수집 데이터에서 지원되는 모든 아티팩트를 한 번에 파싱하려면 다음을 사용하십시오.

```bash
juntei auto-parse -d evidence/ -o analysis/
```

호스트 탐색, 병렬 처리, 출력 레이아웃 및 SIEM 수집용 통합 JSONL에 대해서는
[수집 데이터 자동 분석](../commands/auto-parse.md)을 참조하십시오.

## 4. 구조화된 출력 저장 { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

스크립트 작성과 중첩된 값에는 JSONL을, 스프레드시트나 타임라인 뷰어에는 CSV를 사용하는 것이 좋습니다.

## 5. 신뢰할 수 있는 식별자에서 피벗 { #5-pivot-from-a-reliable-identifier }

파일 이름, 전체 경로, SHA-1/SHA-256, 사용자 SID, MFT 엔트리/시퀀스, 작업 GUID, 서비스 이름 또는 타임스탬프
범위로 시작하십시오. 대소문자 정규화는 Windows가 대소문자를 구분하지 않고 처리하는 필드에만 적용하십시오.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. 상관 분석 { #6-correlate }

[아티팩트 간 조사 절차](../resources/workflows.md)를 사용하십시오. 실용적인 순서는 다음과 같습니다.

1. 실행 여부와 최근 실행 시간은 Prefetch로 확인합니다.
2. 생성, 경로, 이름 변경 및 삭제는 `$MFT`와 USN 저널로 확인합니다.
3. 과거의 존재와 메타데이터는 Amcache와 ShimCache로 확인합니다.
4. 사용자/애플리케이션 컨텍스트는 LNK, Jump Lists, RecentDocs 및 SRUM으로 확인합니다.
5. 지속성은 레지스트리 서비스와 TaskCache로 확인합니다.
6. 탐지, 제거, 출처 및 해시는 Defender 격리 데이터로 확인합니다.

## 7. 재현성 기록 { #7-document-reproducibility }

원본 경로, 원본 해시, Juntei 버전, 정확한 명령, 출력 해시, 시스템 시간대, 그리고 이후의 모든 필터링을
기록하십시오. 생성된 증거 출력을 기록 없이 편집하지 마십시오.
