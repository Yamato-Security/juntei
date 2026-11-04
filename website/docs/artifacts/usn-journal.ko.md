# USN 저널 (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## USN 저널이란 { #what-the-usn-journal-is }

NTFS는 파일 및 디렉터리 변경 사항을 USN(Update Sequence Number) 변경 저널에 기록합니다. 레코드는 생성,
데이터 확장/덮어쓰기, 속성 또는 보안 변경, 이름 변경, 닫기, 삭제를 설명할 수 있습니다. 레코드에는 파일 참조,
상위 참조, 이름, 이유, 속성, 타임스탬프, 그리고 단조 증가하는 USN이 저장됩니다.

저널은 특히 파일이 삭제된 경우에 공격자의 일련의 파일 작업을 재구성하는 데 매우 유용합니다. 저널은 변경된
바이트가 아니라 **메타데이터/내용이 변경되었다는 사실**을 기록합니다.

<span class="evidence-strength">가장 타당한 결론:</span> NTFS는 해당 타임스탬프에 참조된 파일에 대해 명시된
변경 이유를 기록했습니다. 이유는 `CLOSE` 레코드가 나올 때까지 누적되는 경우가 많습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J`는 NTFS 메타데이터의 명명된 스트림입니다. 원시(raw) 접근을 지원하는 포렌식 도구로 수집하십시오. USN
레코드는 전체 경로가 아니라 상위 파일 참조를 저장하므로, 같은 볼륨의 `$MFT`도 함께 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 저널은 순환 구조이며 크기가 제한되어 있습니다. 오래된 레코드는 덮어쓰입니다.
- `FILE_DELETE`는 NTFS가 삭제를 처리했다는 뜻이며, 내용은 다른 곳에서 복구 가능할 수 있습니다.
- `DATA_OVERWRITE` 같은 이유는 바이트, 책임 있는 프로세스 또는 사용자를 드러내지 않습니다.
- 하나의 레코드에 여러 이유가 결합될 수 있으며, `CLOSE`는 흔히 누적된 변경의 끝을 나타냅니다.
- 경로는 재구성된 것입니다. 상위 디렉터리가 이름이 변경되었거나, 삭제되었거나, 재사용되었을 수 있습니다.
- 파일 참조의 시퀀스 번호가 중요합니다. 엔트리 번호만으로는 잘못된 세대와 연결될 수 있습니다.
- 카빙은 원래 저널 컨텍스트 없이 유효한 레코드를 복구할 수 있으므로, 그 차이를 보고하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 해석 |
|---|---|
| `Name` | 이 USN 레코드에 저장된 파일/디렉터리 이름. 전체 경로가 아닙니다. |
| `Extension` | `Name`에서 파싱한 편의용 확장자. 신뢰할 수 있는 콘텐츠 유형으로 사용하지 마십시오. |
| `EntryNumber`, `SequenceNumber` | 파일의 NTFS 참조. 레코드 세대를 식별하려면 둘 다 사용하십시오. |
| `ParentEntryNumber`, `ParentSequenceNumber` | 변경 시점의 상위 디렉터리 NTFS 참조. |
| `ParentPath` | Juntei가 시간을 고려하여 재구성한 상위 경로. 알 수 없음 표시는 상위 경로를 해석할 수 없었다는 뜻입니다. |
| `UpdateSequenceNumber` | USN 순서 값. 안정적인 정렬과 Amcache/MFT 연계 조사에 유용합니다. |
| `UpdateTimestamp` | V2/V3 레코드에 저장된 타임스탬프. 카빙된 컨텍스트의 타임스탬프도 다른 레코드와 같은 수준으로 검증하십시오. |
| `UpdateReasons` | `|`로 연결된 하나 이상의 명명된 변경 플래그. 조합과 `CLOSE`를 함께 해석하십시오. |
| `FileAttributes` | 변경 시점에 기록된 NTFS 속성. 알 수 없는 비트는 16진수로 남습니다. |
| `OffsetToData` | 입력 내 바이트 오프셋. 검증과 카빙된 레코드에 필수적입니다. |
| `SourceFile` | 입력 저널/이미지. 호스트와 볼륨을 구별합니다. |
| `SourceInfo` | 변경이 저널에 어떻게 들어왔는지 설명하는 소스 플래그. 대개 행위자 특정이 아니라 컨텍스트입니다. |
| `SecurityID` | 레코드의 NTFS 보안 식별자 인덱스. 사용자 SID가 아닙니다. |
| `Version` | USN 레코드 버전. Juntei는 V2/V3를 CSV에 기록하며, JSON은 V4 범위 추적 익스텐트를 보존합니다. |

JSON/JSONL은 128비트 V3 참조를 올바르게 표현하고 V4 범위 추적 레코드와 그 익스텐트를 보존합니다. 버전별
저수준 필드가 중요한 경우 이를 사용하십시오.

## 조사 전략 { #investigation-strategy }

1. 시간, 이름, 확장자, 상위 경로 또는 파일 참조로 필터링합니다.
2. 정확한 저널 순서를 보려면 시간뿐만 아니라 `UpdateSequenceNumber`로도 정렬합니다.
3. 이동/이름 변경은 `RENAME_OLD_NAME`과 `RENAME_NEW_NAME`을 짝지어 확인합니다.
4. `$MFT`를 사용하여 레코드 상태, 경로, 크기, 타임스탬프를 검증합니다.
5. 이벤트 로그, EDR, Prefetch, 서비스/작업 또는 Defender로부터 프로세스 귀속 정보를 상관 분석합니다.

## 추가 정보 { #further-details }

- [Microsoft: 변경 저널](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: 변경 저널 이유 플래그](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-ntfs-parser의 usn 크레이트](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
