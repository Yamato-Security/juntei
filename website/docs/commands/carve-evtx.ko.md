# 이벤트 로그 카빙 { #carve-evtx }

로그가 지워지거나 삭제된 뒤에도 남아 있는 이벤트 로그 청크를 디스크 이미지, 비할당 영역, 페이지 파일 또는 메모리 이미지에서 복구합니다. 64 KiB 크기의 EVTX 청크는 각각 자체 문자열 테이블과 템플릿 테이블을 가지고 있으므로, 로그의 나머지 부분이 없어도 청크 안의 레코드를 읽을 수 있습니다. Juntei는 복구한 청크를 채널별로 하나씩 일반 `.evtx` 파일로 저장하므로 Hayabusa나 이벤트 뷰어에서 바로 열 수 있습니다.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

`-o`를 지정하지 않으면 요약만 표시합니다.

## 청크 찾기 { #finding-chunks }

- 모든 바이트 오프셋에서 청크 시그니처를 검색하며, 각 헤더의 값(헤더 크기, 레코드 번호, 오프셋)이 타당해야 합니다. 체크섬은 보고만 하고 필수 조건으로 삼지 않습니다. 일부가 덮어써진 청크에도 읽을 수 있는 레코드가 남아 있을 수 있기 때문입니다.
- NTFS 압축된 로그의 청크는 압축을 해제합니다(LZNT1, 16 클러스터 단위). 디스크 이미지에서는 볼륨의 클러스터 크기를 사용하고, 그 밖의 파일에서는 `--cluster-size <BYTES>`(기본값 4096)를 사용합니다.
- 레코드는 헤더에 기록된 마지막 레코드 오프셋까지 크기를 따라 차례로 읽으며, 레코드 ID가 반복되어도 멈추지 않습니다. 손상된 레코드를 만나면 다음 유효한 레코드부터 계속 읽습니다. 청크 슬랙에 남은 이전 레코드도 함께 나열합니다.

## 디스크 이미지와 NTFS 볼륨 { #disk-images }

E01 또는 VHDX 이미지, 또는 NTFS가 있는 원시 디스크나 볼륨에서는 모든 청크를 해당 클러스터에 따라 분류합니다.

| 위치 | 의미 |
|---|---|
| `FreeSpace` | 어떤 파일도 사용하지 않는 클러스터입니다. |
| `DeletedLog` | 삭제된 로그의 `$MFT` 레코드가 아직 가리키는 비할당 클러스터, 또는 이 클러스터들로 재구성한 로그입니다. |
| `LiveLog` | 사용 중인 이벤트 로그의 클러스터입니다. `--keep-live`를 지정하지 않으면 제외됩니다. |
| `AllocatedFile` | `pagefile.sys`나 볼륨 섀도 복사본의 저장소 파일 같은 다른 파일의 클러스터입니다. |
| `AllocatedUnknown` | 어떤 `$MFT` 레코드도 가리키지 않는 할당된 클러스터입니다. |
| `OutsideVolumes` | 디스크의 어느 NTFS 볼륨에도 속하지 않는 영역입니다. |
| `Unclassified` | 입력에 NTFS 볼륨이 없습니다(페이지 파일 또는 메모리 이미지). |

`$MFT` 레코드가 비할당 클러스터만 가리키는 삭제된 로그는 압축된 로그를 포함해 통째로 재구성되어 `Recovered/` 아래에 저장됩니다. `--free-space-only`는 비할당 클러스터만 검색합니다. 더 빠르지만 페이지 파일, 볼륨 섀도 복사본 및 기타 파일은 건너뜁니다.

## 중복 제거 { #deduplication }

각 청크는 한 번만 기록됩니다. 사용 중인 로그에 속하는 청크, 다른 청크와 동일한 청크, 또는 모든 레코드가 이미 사용 중인 로그나 앞서 기록한 청크에 있는 청크는 제외됩니다. 레코드가 더 많은 청크를 먼저 검토하므로, 같은 청크의 더 오래되고 짧은 사본은 아무것도 더하지 않습니다. NTFS가 없는 입력에서는 `--live-logs <DIR>`로 사용 중인 로그를 지정하십시오.

## 결과 { #results }

- `Carved/<CHANNEL>.evtx`: 채널별 로그 하나로, Windows의 로그 이름 규칙을 따릅니다(`Microsoft-Windows-Sysmon%4Operational.evtx`). 청크 헤더의 레코드 번호는 다시 매기고 체크섬은 다시 계산합니다. 손상된 레코드와 그 뒤의 모든 내용은 슬랙이 됩니다. 그 밖에는 각 청크를 슬랙까지 포함해 그대로 유지하므로, Hayabusa의 `-x` 같은 레코드 복구를 여전히 적용할 수 있습니다.
- `Recovered/Volume<N>/...`: `$MFT` 레코드로부터 재구성한 삭제된 로그입니다.
- `chunks.csv` / `chunks.jsonl`: 발견된 모든 청크입니다. 위치, 처리 결과(`Status`), 채널, 레코드 ID, 시간, 체크섬 상태를 기록합니다.
- `records.csv` / `records.jsonl`: 기록된 청크의 모든 레코드입니다. `OnlyInCarvedData`는 사용 중인 로그에 없는 레코드를 표시합니다.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

카빙은 연속된 청크만 찾을 수 있습니다. 연속되지 않은 클러스터나 메모리의 연속되지 않은 물리 페이지에 걸쳐 나뉜 청크는 불완전하며, 뒷부분의 레코드가 손상되었거나 누락되었을 수 있습니다. `hiberfil.sys`는 먼저 압축을 해제하십시오. 종료 코드는 검색이 완료되면 0, 오류가 발생하면 1입니다.

## 추가 정보 { #further-details }

- [상세 사용법 및 필드 참조(영어)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [상세 사용법 및 필드 참조(일본어)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX 형식(libevtx 문서)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
