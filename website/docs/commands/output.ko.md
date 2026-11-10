# 일반적인 출력 방식 { #common-output-patterns }

## 단일 아티팩트 또는 수집 데이터 { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d`는 해당 명령어에 맞는 파일을 재귀적으로 검색합니다. 원본 경로 열을 통해 결합된 출력에서 호스트와
아티팩트를 구별할 수 있습니다.

## CSV 저장 { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

일부 파서는 여러 CSV 테이블을 출력하므로 디렉터리가 필요합니다.

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## JSONL 스트리밍 { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

메시지와 진행 상황은 표준 오류로 출력되므로, 표준 출력은 파싱 가능한 상태로 유지됩니다.

## 의도적인 덮어쓰기 { #overwrite-intentionally }

Juntei는 기본적으로 기존 출력을 덮어쓰지 않습니다. 대상을 확인하고 중요한 이전 결과를 보존한 다음
`-C` / `--clobber`를 추가하십시오.

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## 출력 억제 및 색상 없음 모드 { #quiet-and-no-color-modes }

자동화에는 `-q`를, 터미널 색상 코드가 적절하지 않을 때는 `-K`를 사용하십시오.

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## 대소문자와 경로 { #case-and-paths }

공백, `$`, 셸 메타 문자 또는 백슬래시가 포함된 Windows 경로는 사용하는 셸에 맞게 따옴표로 묶으십시오.
Bash 계열 셸에서는 작은따옴표를 사용하면 `$MFT`와 `$J`가 변수로 해석되지 않습니다.

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
