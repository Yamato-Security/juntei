# 휴지통 { #recycle-bin }

## 휴지통이 기록하는 내용 { #what-the-recycle-bin-records }

사용자가 파일 탐색기에서 파일이나 폴더를 삭제하면 Windows는 이를 해당 볼륨의 휴지통으로 옮기고, 그 옆에
작은 인덱스 레코드를 기록합니다. 이 레코드에는 원래 경로, 크기, 삭제된 시각이 담깁니다. 각 사용자는
볼륨마다 자신의 SID로 이름 붙은 휴지통 폴더를 가지므로, 이 폴더를 보면 어느 계정이 항목을 삭제했는지도
알 수 있습니다.

| Windows 버전 | 인덱스 | 삭제된 콘텐츠 |
|---|---|---|
| Vista 이상 | 항목마다 `$I…` 파일 하나(Vista~8.1은 형식 버전 1, Windows 10 이상은 버전 2) | `$R…`(`$R` 뒤의 이름은 인덱스 파일과 같음): 파일 자체, 또는 내용을 포함한 폴더 |
| NT 4.0, 2000, XP, Server 2003 | 휴지통마다 `INFO2` 파일 하나: ANSI 경로와 Unicode 경로를 담은 800바이트 레코드 | `D<drive letter><index><extension>`(예: `Dc1.txt`) |
| 95, 98, ME | `INFO2` 파일 하나(Windows 95는 `INFO`): ANSI 경로만 담은 280바이트 레코드 | 같은 형식의 `D…` 이름 |

<span class="evidence-strength">가장 타당한 결론:</span> 레코드가 들어 있는 휴지통의 계정이 기록된 시각에
원래 경로에 있던 항목을 휴지통으로 옮겼습니다. Shift+Delete로 삭제하거나 명령줄 또는 대부분의
프로그램에서 삭제하면 휴지통을 거치지 않으므로 레코드가 남지 않습니다.

## 위치 및 수집 { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

모든 볼륨에 있는 휴지통의 `$I` 파일 또는 INFO2를 수집하십시오. `$R` 및 `D…` 파일도 함께 수집하면 삭제된
항목 중 어느 것이 아직 남아 있는지 알 수 있습니다. 이 파일들이 없으면 모든 항목의 `RecycledFileFound`가
`false`가 됩니다. 삭제된 콘텐츠는 그 자체로 증거이며 악성코드일 수도 있으므로 그에 맞게 취급하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- `DeletedTime`은 항목이 휴지통으로 옮겨진 시각으로, 시스템 시계를 기준으로 한 UTC FILETIME입니다.
  휴지통을 비운 시각이 아닙니다. 항목 자체의 생성 및 수정 시간은 `$MFT`와 `$R` 파일에 있습니다.
- SID는 항목을 삭제한 계정이며, 반드시 파일의 소유자인 것은 아닙니다. SOFTWARE 하이브의 `ProfileList`에
  프로필이 없는 SID(삭제된 계정, 또는 한 번도 로그온하지 않은 도메인 계정)에는 `User`가 없습니다.
- 휴지통을 비우거나 휴지통에서 항목을 복원 또는 삭제하면 Windows Vista 이상은 해당 `$I` 및 `$R` 파일을
  삭제합니다. 그 이름은 USN 저널, `$MFT`, `$I30` 슬랙에 남아 있을 수 있지만, Juntei는 이들로부터 삭제된
  `$I` 레코드를 복구하지 않습니다. INFO2는 이러한 레코드를 남겨 두고 ANSI 경로의 첫 바이트를 지웁니다.
  이때 `RemovedFromBin`은 `true`이며, Juntei는 드라이브 번호로부터 그 문자를 되살립니다.
- `$I` 파일의 `FileSize`는 항목의 크기이며, 폴더라면 그 내용의 총 크기입니다. INFO2에서는 항목이
  디스크에서 차지한 공간을 클러스터 단위로 나타냅니다(작은 파일은 4,096바이트).
- INFO2의 ANSI 경로는 흔히 8.3 형식이며 시스템의 코드 페이지가 필요합니다. 일본어 Windows에는
  `--code-page 932`를, 중국어에는 936 또는 950을 사용하십시오. Unicode 경로가 있으면 그것이
  `OriginalPath`입니다.
- 일부 Windows Vista 시스템은 크기 필드가 1바이트 짧은 543바이트 `$I` 파일을 기록했습니다. Juntei는 그
  시간과 경로를 읽고 `FileSize`는 비워 두며, 이를 문제로 보고합니다.
- `RecycledFileFound`는 엔드포인트가 아니라 수집본을 설명합니다. `false`는 수집 도구가 `$R` 파일을
  가져오지 않았거나, 항목이 더 이상 휴지통에 없다는 뜻일 수 있습니다.
- 비시스템 볼륨의 휴지통은 수집 메타데이터나 알려진 드라이브/볼륨 폴더 구조로 같은 컴퓨터임을 확인할 수 있을 때만 프로필을 사용합니다. 대응 관계가 모호하면 사용자는 알 수 없는 상태로 남으며 다른 호스트의 프로필을 가져오지 않습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

`-d`를 사용하면 Juntei는 `$I` 뒤에 영문자 또는 숫자 여섯 개가 오는 이름의 파일(확장자는 무관), `INFO2`,
그리고 `RECYCLED` 폴더 안의 `INFO`를 읽습니다. `-f`를 사용하면 이름과 관계없이 해당 파일을 읽습니다.

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 의미 |
|---|---|
| `SourceFile` | `$I` 또는 INFO2 파일입니다. |
| `DeletedTime` | 항목이 휴지통으로 옮겨진 시각(UTC)입니다. |
| `OriginalPath` | 항목이 있던 위치입니다. INFO2의 경우 레코드에 Unicode 경로가 있으면 그것을, 없으면 ANSI 경로를 사용합니다. |
| `FileSize` | `$I`: 항목의 크기(바이트, 폴더는 총합)입니다. INFO2: 디스크에서 차지한 공간(클러스터 단위)입니다. 읽을 수 없으면 비어 있습니다. |
| `User`, `SID` | 휴지통 폴더의 SID와, 그 SID의 `ProfileList` 프로필 이름이 가리키는 사용자입니다. |
| `RecycledFile`, `RecycledFileFound` | 인덱스 옆에 있는 삭제된 콘텐츠의 경로(`$R…`, `D<drive><index><extension>`)와 그것이 수집본에 있는지 여부입니다. |
| `Format`, `FormatVersion` | `$I` 1(Vista~8.1) 또는 2(Windows 10 이상), `INFO2` 0(95), 2(NT 4.0), 4(98) 또는 5(ME, 2000~Server 2003)입니다. |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: 레코드 번호(`D…` 파일 이름에 들어가는 번호), 드라이브, INFO2 안에서 레코드가 시작하는 위치입니다. UNC 경로에는 드라이브가 없습니다. |
| `RemovedFromBin` | INFO2: 항목이 휴지통에서 복원되거나 삭제되었으면 `true`입니다. |
| `ShortPath` | INFO2: `OriginalPath`와 다를 때의 ANSI 경로입니다(흔히 8.3 이름). |
| `Problems` | 손상 내용: 잘리거나 지나치게 긴 파일, 잘못된 드라이브 번호, 유효한 UTF-16이 아닌 경로, 시각으로 해석할 수 없는 삭제 시간입니다. |

JSON과 JSONL에는 같은 필드가 있으며, 숫자와 부울 값은 타입이 지정되고, 레코드에 없는 값은 `null`,
`Problems`는 목록입니다.

휴지통 디렉터리를 나열할 수 없거나 콘텐츠 이름이 모호하면 `RecycledFileFound`는 알 수 없음(CSV 빈 셀 또는 JSON `null`)이며 원인은 `Problems`에 기록됩니다. 하드 링크를 통한 경우를 포함해 휴지통 콘텐츠를 덮어쓰거나 삭제된 폴더 안에 출력할 수 없습니다. 어떤 인덱스도 읽지 못하면 기존 결과를 보존합니다.

## 자동 분석 { #automatic-parsing }

`auto-parse`는 `FileExistence/RecycleBin.csv`(`-t json` 또는 `-t jsonl`을 사용하면 `RecycleBin.json` 또는
`RecycleBin.jsonl`)와 `FileExistence/RecycleBin-logs.txt`를 생성합니다. 이 명령의 `--code-page`는 INFO2에
적용됩니다. 디스크 이미지에서는 `$I` 파일과 INFO2를 복사하고 `$R` 및 `D…` 파일은 빈 자리 표시자로
만들므로, 이 경우에도 `RecycledFileFound`가 채워집니다. 결합된 `-j` 타임라인에는 모든 행이 `Artifact`
식별자 `recycle-bin`과 함께 포함되고, `-Z`는 `DeletedTime`을 변환하며,
[`super-timeline`](../commands/super-timeline.md)은 항목마다 `Recycle Bin deletion (moved to the bin)`
이벤트를 하나씩 만듭니다.

## 조사 전략 { #investigation-strategy }

1. 사용자별로 `DeletedTime` 순으로 정렬하고, 조사 중인 활동의 시각과 비교하십시오.
2. LNK, Jump Lists, ShellBags, RecentDocs에서 `OriginalPath`를 찾아 항목이 삭제되기 전에 사용되었는지
   확인하십시오.
3. USN 저널에서 삭제는 `DeletedTime` 무렵에 `$Recycle.Bin` 안으로의 이름 변경으로 나타나며, `$MFT`에서는
   항목 자체의 시간을 알 수 있습니다.
4. `RecycledFileFound`가 `true`이면 `$R` 또는 `D…` 콘텐츠의 해시를 계산하고 분석하십시오.
5. 휴지통이 비워졌다면 USN 저널과 `$MFT`에서 `$I` 및 `$R` 이름을 검색하십시오.

## 추가 정보 { #further-details }

- [libyal dtformats — Windows Recycle.Bin 파일 형식](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler 파일 형식](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — 휴지통 및 INFO2 파서](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
