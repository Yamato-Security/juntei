# 부트 섹터 (`$Boot`) { #boot-sector-boot }

## `$Boot`란 { #what-boot-is }

`$Boot`는 모든 NTFS 볼륨의 MFT 엔트리 7이며, 볼륨의 처음 8KiB입니다. 그 첫 번째 섹터인
**볼륨 부트 레코드**(volume boot record)에는 파일 시스템을 기술하는 BIOS 매개변수 블록(BIOS parameter
block)이 들어 있습니다.

- 섹터 및 클러스터 크기, 볼륨의 섹터 수, 그리고 **숨겨진 섹터(hidden sectors)**(디스크에서 볼륨
  앞에 있는 섹터, 즉 파티션이 시작되는 위치)
- `$MFT`와 `$MFTMirr`가 시작되는 위치, FILE 및 INDX 레코드의 크기
- 볼륨을 포맷할 때 설정되는 64비트 **볼륨 일련번호**

첫 번째 섹터의 나머지 부분과 그 뒤의 15개 섹터에는 Windows를 로드하는 **부트 코드**가 들어 있습니다.

일련번호의 하위 32비트는 Windows가 `XXXX-XXXX`(`vol`, `dir`)로 표시하는 값이며,
[LNK 바로 가기](lnk.md), [Jump Lists](jump-lists.md) 및 [Prefetch](prefetch.md)가 파일이 있던
볼륨에 대해 기록하는 값입니다. `$Boot`는 이러한 레코드를 수집된 호스트의 볼륨과 연결해 줍니다.

<span class="evidence-strength">가장 타당한 결론:</span> LNK, Jump List 또는 Prefetch의 볼륨
일련번호가 `$Boot`의 `VolumeSerialNumber`와 같다면 해당 볼륨을 가리키거나, 같은 일련번호가 부여되었거나
같은 일련번호와 함께 복사된 다른 볼륨을 가리킵니다. 일련번호는 변경될 수 있으며 호스트 간에 고유하지 않습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

원시(raw) 접근이 가능한 포렌식 도구로 수집하거나, 볼륨 이미지의 첫 번째 섹터를 읽으십시오. 둘은 같은
바이트입니다. Juntei는 NTFS 부트 섹터로 시작하는 모든 파일을 읽으므로 볼륨 이미지도 입력으로 사용할 수
있습니다. 디스크 이미지는 파티션 테이블로 시작하므로, Juntei에는 파티션의 볼륨을 제공하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 볼륨을 포맷하면 일련번호가 바뀝니다. 일련번호가 현재 어떤 볼륨과도 일치하지 않는 LNK 파일은 이동식
  미디어, 네트워크 공유의 서버, 또는 그 후 포맷된 볼륨을 가리킬 수 있습니다.
- 부트 코드는 같은 Windows로 포맷된 볼륨에서 동일합니다. DFIR Artifact Museum의 Windows 10, 11 및
  Server 2012 R2 `$Boot` 파일에서도 동일합니다. 볼륨 부트 레코드 부트킷은 부트 코드, 또는 부트 코드가
  `$Boot`의 나머지 부분을 찾는 데 사용하는 숨겨진 섹터 필드를 변경합니다. 결론을 내리기 전에
  `BootCodeSHA256`, `JumpInstruction` 및 `HiddenSectors`를 다른 호스트 및 파티션 테이블과 비교하십시오.
  부트 관리자나 암호화 소프트웨어도 부트 코드를 변경할 수 있습니다.
- BitLocker로 암호화된 볼륨의 이미지는 BitLocker 자체의 부트 섹터(OEM ID `-FVE-FS-`)로 시작합니다.
  `-f`로 지정하면 Juntei는 이를 보고하고 `Problems`에 NTFS가 아니라고 표시합니다. NTFS 부트 섹터는
  암호화된 데이터 안에 있습니다. 잠금 해제된 볼륨에서 수집한 `$Boot`는 NTFS 부트 섹터입니다.
- `Problems`는 인식된 구조적 불일치와 잘림을 나열하며, Windows에서의 완전한 마운트 가능성 테스트를
  제공하는 것은 아닙니다. 카빙된 섹터와 복사본을 평가하는 데 사용하고, 그 자체를 공격 소견으로 취급하지
  마십시오. 부트 코드 해시는 초기 점프, BIOS 매개변수 블록 및 종료 시그니처를 제외하므로 이 필드들도
  검사하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

요약에는 각 부트 섹터의 볼륨 일련번호(Windows가 표시하는 형식과 LNK 및 Prefetch 필드가 표시하는 형식),
크기, 클러스터 크기 및 `$MFT` 클러스터가 표시됩니다. 여러 호스트를 파싱하면 부트 코드 해시별로
그룹화하므로, 부트 코드가 다른 볼륨이 눈에 띄게 됩니다.

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 해석 |
|---|---|
| `SourcePath` | 입력 파일입니다. |
| `VolumeSerialNumber` | LNK, Jump List 및 Prefetch 출력에 표시되는 형식의 32비트 일련번호(`180B1E67`)입니다. Windows는 이를 `180B-1E67`로 표시합니다. |
| `VolumeSerialNumber64` | 64비트 일련번호 전체입니다. |
| `OEM-ID` | NTFS 볼륨에서는 `NTFS`입니다. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | 볼륨의 지오메트리입니다. 128을 초과하는 클러스터당 섹터 수는 2의 거듭제곱 형태로 기록되며, 여기서 디코딩됩니다. 유효하지 않으면 비어 있습니다. |
| `TotalSectors`, `VolumeSize` | 볼륨의 섹터 수와 그 바이트 크기이며, 볼륨 끝에 있는 부트 섹터 사본은 포함하지 않습니다. |
| `HiddenSectors` | 디스크에서 볼륨 앞에 있는 섹터 수로, 파티션이 시작되는 위치입니다. |
| `MFT-Cluster`, `MFT-Offset` | `$MFT`가 시작되는 위치로, 클러스터 단위와 볼륨 시작부터의 바이트 단위입니다. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | `$MFTMirr`가 시작되는 위치입니다. |
| `FileRecordSize`, `IndexRecordSize` | FILE 레코드당 바이트 수(보통 1,024)와 INDX 레코드당 바이트 수(보통 4,096)입니다. |
| `MediaDescriptor` | 하드 디스크의 경우 `F8`입니다. |
| `JumpInstruction` | 처음 3바이트(`EB 52 90`)로, BIOS 매개변수 블록을 건너뛰어 부트 코드로 점프합니다. |
| `EndSignature` | 유효한 부트 섹터 끝에 있는 `55 AA`입니다. |
| `BootCodeSHA256` | 부트 코드의 SHA-256으로, 첫 번째 섹터의 0x54부터 0x1FD까지의 바이트와 8KiB의 나머지 부분입니다. |
| `Problems` | Windows가 마운트하지 않을 요인입니다. `NTFS` 이외의 OEM ID, `55 AA` 없음, 유효하지 않은 크기, NTFS가 0으로 유지하는 FAT 필드가 설정됨, 볼륨 끝을 넘어선 `$MFT`, 그리고 8KiB보다 짧은 파일입니다. |

JSON/JSONL은 동일한 필드와 이름을 가지며, 숫자는 숫자로, 알 수 없는 값은 `null`로, `Problems`는
목록으로 표현됩니다.

## 조사 전략 { #investigation-strategy }

1. 각 호스트의 `VolumeSerialNumber`를 해당 호스트의 LNK 파일, Jump Lists 및 Prefetch 파일의 볼륨
   일련번호와 나란히 비교하십시오. 수집된 어떤 볼륨과도 일치하지 않는 일련번호는 다른 볼륨(USB 드라이브,
   공유, 그 후 포맷된 볼륨)입니다.
2. 마운트된 이동식 볼륨의 볼륨 일련번호를 나열하는 [USB 장치](usb.md) 출력에서 나머지 일련번호를
   찾아보십시오.
3. 호스트 간에 `BootCodeSHA256`과 `HiddenSectors`를 비교하십시오. 다른 부트 코드는 디스어셈블러와
   디스크의 파티션 테이블로 조사하고, 부팅 관련 이벤트 로그 항목을 찾으십시오.
4. `MFT-Offset`과 `ClusterSize`를 사용하여 볼륨 이미지에서 `$MFT`를 찾으십시오.

## 추가 정보 { #further-details }

- [Microsoft: NTFS 개요](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS 형식 문서](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (볼륨 헤더)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` 크레이트)
