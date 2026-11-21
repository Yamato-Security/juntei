# YARA 검사 { #yara-scan }

이식 가능한 Rust 엔진으로 수집한 파일을 검사한다. Windows, macOS, Linux를 지원한다. 파일은 데이터로만 읽으며 실행하지 않는다. 자동 아티팩트 파싱과 별개인 명령이다.

## 규칙 선택 { #choose-rules }

기본 Core는 정확도와 성능을 우선한다. Extended는 헌팅 범위를 넓힌다. Full은 가장 폭넓지만 오탐이 더 많을 수 있다.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: 로컬 UTF-8 규칙 파일이나 디렉터리를 사용하며 다운로드하지 않는다. 디렉터리를 재귀 검색한다. include는 선택한 규칙 디렉터리 안에서만 의존 순서를 유지해 한 번 확장한다. 잘못되거나 지원하지 않는 규칙은 조용히 제외하지 않고 컴파일을 중단한다.

## 제한과 범위 { #limits-and-scope }

복구한 격리 페이로드를 포함한 모든 일반 파일을 대상으로 한다. 심볼릭 링크와 특수 파일은 건너뛴다. 압축 파일이나 디스크 이미지를 풀지 않으며 실시간 메모리를 검사하지 않는다. 안정적인 추출 파일을 사용하고 증거 밖에 새 출력 디렉터리를 지정한다.

- `-T, --threads <N>`: 병렬 검사 수. 기본값은 CPU 수이며 최대 8이다.
- `--timeout <SECONDS>`: 파일별 평가 제한 시간(초). 기본값은 60이다.
- `--max-file-mib <MIB>`: 최대 파일 크기(MiB). 기본값은 64이다. 초과 파일은 건너뛴 것으로 기록한다. 작업자와 버퍼를 늘리면 메모리 사용량도 증가한다.
- `--rules-release YYYYMMDD`: Forge 릴리스 날짜 또는 기본값 latest. 재현할 수 있도록 다운로드한 원본과 해시를 저장한다.

## 결과 { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: 규칙과 파일의 일치 결과. 상대 경로, 증거 호스트 이름, 해시, 메타데이터, 제한된 일치 오프셋을 포함한다. 일치한 바이트 자체는 내보내지 않는다.
- `YaraFiles.csv` / `YaraFiles.jsonl`: 건너뛰기, 읽기 오류, 시간 초과, 부분 결과를 포함한 파일별 검사 범위.
- `Summary.json`: 완료 상태, 설정, 개수, 엔진 버전, 원본 해시.
- `Rules.yar`: 다운로드한 원본 규칙 또는 로컬 규칙을 확장한 스냅샷.
- `CompilationWarnings.txt` / `CompilationError.txt`: 컴파일 경고 또는 컴파일 실패 시 오류.

종료 코드 0은 완료, 3은 불완전한 검사 범위, 2는 설정 또는 출력 실패, 130은 취소를 뜻한다. 일치는 조사 단서이며 침해의 증명이 아니다. 일치하지 않아도 안전하다고 할 수 없다. 규칙 제목과 아티팩트 맥락을 검토한다. 격리 페이로드는 먼저 복호화해야 식별할 수 있다.

## 추가 정보 { #further-details }

- [엔진: Boreal](https://github.com/vthib/boreal)
- [규칙 패키지: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [상세 사용법 및 필드 참조(영어)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [상세 사용법 및 필드 참조(일본어)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
