# RecentDocs { #recentdocs }

## RecentDocs란 { #what-recentdocs-is }

Windows 탐색기는 파일과 폴더의 최근 사용(MRU) 목록을 각 사용자의 `NTUSER.DAT`에 저장합니다. 루트 목록은
유형에 관계없이 최근 항목을 다루며, 하위 키는 `.docx` 같은 확장자별로 그룹화되고 `Folder` 목록도 있습니다.
`MRUListEx`는 번호가 매겨진 값의 순서를 지정합니다.

RecentDocs는 나중에 삭제되거나 이동된 파일을 포함하여 사용자가 상호 작용한 문서를 식별하는 데 유용합니다.
값에는 Windows가 사용자의 Recent 폴더에서 사용한 바로 가기 이름도 들어 있을 수 있으며, 이를 LNK 증거와
상관 분석할 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 탐색기가 해당 사용자의 RecentDocs 목록에 항목을
기록했습니다. 키 시간은 해당 특정 목록에서 가장 최근 항목의 시점만 알려 줍니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

관련된 모든 프로필의 `NTUSER.DAT`와 그 트랜잭션 로그, 그리고 사용자의 Recent LNK 디렉터리를
수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- `MRUListEx`는 순서를 알려 줄 뿐 모든 항목의 타임스탬프를 제공하지 않습니다.
- 키의 마지막 쓰기 시간은 가장 최근의 업데이트에 해당합니다. 이로부터 근거 있는 `LastOpenedUTC`를 도출할
  수 있는 것은 MRU 위치 0뿐입니다.
- 항목이 루트 목록과 확장자 목록에 모두 나타날 수 있습니다. 두 번 열린 것으로 중복 집계하지 마십시오.
- 애플리케이션과 탐색기의 동작은 다를 수 있으며, 일부 접근 방법은 RecentDocs를 업데이트하지 않을 수
  있습니다.
- 정리/개인 정보 보호 도구와 목록 크기 제한으로 인해 오래된 증거가 제거됩니다.
- 파일 이름은 공격자가 제어할 수 있으며 콘텐츠 유형의 증명이 아닙니다.
- 사용자 추론은 프로필 경로나 셸 폴더 데이터에서 얻어집니다. 비어 있거나 모호한 사용자 이름은
  검토하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host` | 가능한 경우 수집/시스템 컨텍스트에서 도출한 호스트입니다. |
| `User` | 경로 또는 셸 폴더 경로에서 추론한 프로필 소유자입니다. 비어 있으면 불확실성을 그대로 유지하십시오. |
| `SourcePath` | 입력 `NTUSER.DAT`입니다. 여러 사용자를 결합할 때 필수적입니다. |
| `Extension` | 하위 목록(`.docx`, `Folder` 등)입니다. 루트 RecentDocs 목록에서는 null입니다. |
| `MRU-Position` | 0이 이 목록 내에서 가장 최근입니다. 상대적인 순서이며 경과 시간이 아닙니다. |
| `ValueIndex` | `MRUListEx`가 참조하는 번호가 매겨진 레지스트리 값입니다. 구조적 피벗 용도로만 사용하십시오. |
| `Filename` | 디코딩된 항목 이름입니다. 전체 경로는 LNK, Jump List, MFT 또는 애플리케이션 데이터를 통해 상관 분석하십시오. |
| `LinkName` | 셸 항목에 인코딩된 바로 가기 이름으로, 사용자의 Recent 디렉터리에서 발견되는 경우가 많습니다. |
| `KeyLastWrittenUTC` | 목록 키가 마지막으로 변경된 시간입니다. 모든 행이 아니라 목록에 적용됩니다. |
| `LastOpenedUTC` | 키 시간이 그러한 해석을 뒷받침하는 MRU 위치 0에서만 채워집니다. |
| `ValueDataRaw` | 디코딩에 실패했거나 증거 검증이 필요한 경우의 원시 바이트입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 로그가 재생되었는지 여부와 어떤 로그가 재생되었는지입니다. |
| `Problems` | 누락된 MRU 값, 목록에 없는 값, 잘못된 유형 또는 디코딩 손상입니다. |

## 조사 전략 { #investigation-strategy }

1. 각 사용자의 루트 목록과 확장자 목록을 구분하고, 동일한 항목의 중복을 신중하게 제거하십시오.
2. 순서에는 MRU 위치를 사용하고, 타임스탬프가 있는 주장에는 `LastOpenedUTC`만 사용하십시오.
3. `LinkName`/`Filename`을 Recent LNK, Jump List, `$MFT` 및 USN 데이터와 매칭하십시오.
4. 미끼 이름, 이중 확장자, 네트워크/이동식 대상, 그리고 초기 침투 시점 전후의 문서를 검토하십시오.
5. 실행 여부가 중요한 경우 애플리케이션 Prefetch/SRUM 및 이벤트 로그와 상관 분석하십시오.

## 추가 정보 { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [Eric Zimmerman의 RECmd 및 Registry Explorer](https://github.com/EricZimmerman/RECmd)
- [SANS Windows Forensic Analysis 포스터](https://www.sans.org/posters/windows-forensic-analysis/)
