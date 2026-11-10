# 디스크 이미지 마운트 { #mounting-disk-images }

`juntei mount`는 실행 중 VHDX / E01 / Ex01 디스크 이미지 내부 파일을 디렉터리에서 읽기 전용으로 제공합니다. Hayabusa와 Juntei 같은 도구로 파일 추출이나 root 권한 없이 스캔할 수 있습니다. FUSE를 사용하며 Linux에서 작동합니다. 리더와 마운트 라이브러리는 [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser)에서 관리합니다.

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

다른 터미널에서 다음을 실행합니다.

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

마운트를 해제하려면 첫 번째 터미널에서 Ctrl+C를 누르거나 `fusermount3 -u /mnt/host1`를 실행하십시오.

## 마운트에 표시되는 내용 { #what-the-mount-shows }

- 마운트 지점에 NTFS 볼륨 파일이 표시됩니다. 수집된 이미지에는 `C`, `LongFileNames` 같은 디렉터리와 `$MFT` 같은 볼륨 메타데이터가 포함될 수 있습니다. 여러 NTFS 볼륨은 `Volume1`, `Volume2` 등으로 표시됩니다.
- 모든 파일은 읽기 전용이며 마운트 지점의 소유자에게 속합니다. 파일 시간은 NTFS
  `$STANDARD_INFORMATION` 시간입니다.
- 이름은 NTFS에 저장된 그대로 나열되며, 정확히 일치하는 이름으로 조회됩니다. DOS 8.3 이름과 대체 데이터
  스트림은 표시되지 않습니다.
- 압축된 파일은 압축이 해제됩니다(파일당 최대 1 GiB). EFS로 암호화된 파일은 읽을 수 없습니다.

## 증거는 변경되지 않음 { #the-evidence-is-not-changed }

이미지는 읽기 전용으로 열리며 마운트는 쓰기를 거부합니다. 정상적으로 닫히지 않은 VHDX의 보류 중인 로그는 메모리에서 적용되고 이미지에 기록되지 않습니다. 일부 DiscUtils 기록기는 NTFS 부트 섹터의 점프 명령과 `55 AA` 서명을 0으로 남깁니다. Juntei는 이러한 볼륨을 그대로 읽습니다.

## 사용법 { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 이미지의 네이티브 분석 { #parsing-every-image-with-auto-parse }

`--vhdx`를 지정하면 `auto-parse`는 [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser)로 **Linux, macOS, Windows**에서 VHDX를 네이티브로 읽습니다. OS 마운트, FUSE, 관리자 권한이 필요하지 않습니다.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

파서는 파일 경로가 필요하므로 NTFS 리더가 제공하는 파일을 비공개 임시 디렉터리에 스트리밍 복사합니다. **결과가 생성되는 아티팩트뿐 아니라 제공되는 모든 파일을 저장할 공간이 필요합니다.** 버퍼 크기는 제한되며 지원되는 파일시스템에서는 0 영역을 희소 영역으로 기록합니다. 이름, 디렉터리, 하드 링크, 수정 시각을 보존합니다. LNK와 Jump List의 소스 생성·수정·접근 시각은 임시 파일이 아닌 원래 NTFS 메타데이터 스냅샷에서 가져옵니다. 파서 오류를 포함한 정상 종료 시 임시 복사본을 삭제하지만 강제 종료 시 남을 수 있습니다. 원본은 읽기 전용이며 미적용 VHDX 로그는 메모리에서만 반영합니다.

호스트 검색, CSV/JSON/JSONL, `-j`는 일반 증거 디렉터리와 같은 파이프라인을 사용합니다. `manifest.json`의 `Image`는 원본 이미지를, `EvidenceRoot`는 이미지 내부 상대 경로를 식별합니다. 읽기 오류와 안전하지 않은 이름은 `ImageErrors` 및 `manifest.csv`의 실패 행에 기록됩니다. 읽을 수 있는 결과는 보존하고 불완전한 입력은 종료 코드 1을 반환합니다. 위험하거나 모호한 이름을 임의로 바꾸지 않으며 명명된 NTFS 스트림을 추가 파일로 추출하지 않습니다. 트리아지 이미지에 일반 파일로 수집된 아티팩트는 사용할 수 있습니다.

다른 프로그램에 파일시스템을 제공하는 별도 [`mount` 명령](mount.md)은 Linux FUSE가 필요합니다. `auto-parse --vhdx`는 이를 사용하지 않습니다.

스테이징은 기본적으로 출력 파일시스템의 전용 디렉터리를 사용합니다. `--temp-dir`로 다른 디스크의 기존 디렉터리를 지정할 수 있습니다. 원본 메타데이터는 출력 파일시스템에 별도로 보관하므로 스테이징 디스크가 가득 차도 읽을 수 있는 파일은 파싱할 수 있습니다. 복사본과 결과를 위한 공간을 모두 확보하세요. 출력 파일시스템 자체가 가득 차면 결과를 저장할 수 없습니다. Ctrl+C는 파서 프로세스를 중지하고 전용 작업 공간을 정리한 뒤 코드 130으로 종료합니다. SIGKILL이나 전원 차단으로는 복사본이 남을 수 있습니다. 이미지 검색 중 접근할 수 없는 하위 디렉터리는 건너뛰지만 입력 루트를 읽지 못하면 오류입니다.

스테이징 디렉터리는 출력 디렉터리 밖에 있어야 합니다. Linux와 macOS에서는 SIGTERM과 SIGHUP도 파서 프로세스를 중지하고 작업 공간을 정리합니다.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 및 Ex01 디스크 이미지 { #e01-and-ex01-disk-images }

`--e01`은 **Linux, macOS, Windows**에서 물리 EWF1(`.E01`) 및 EWF2(`.Ex01`) 이미지를 네이티브로 읽습니다. FUSE, OS 마운트, 관리자 권한이 필요하지 않습니다. [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser)와 위에서 설명한 동일한 NTFS 파싱 경로를 사용합니다.

분할 이미지는 첫 번째 세그먼트를 지정하세요. `.E02`를 포함한 모든 관련 파일을 원래 이름으로 같은 디렉터리에 보관하세요. 전체 디스크(MBR/GPT)와 단일 NTFS 볼륨을 지원합니다. 두 형식이 섞인 디렉터리에는 `--vhdx --e01`을 사용하세요.

임시 공간과 원본 메타데이터 요구 사항은 동일합니다. 이미지는 읽기 전용입니다. 수집 누락과 열기 또는 읽기 중 발견된 손상은 `ImageErrors`와 `manifest.csv`에 기록됩니다. 읽을 수 있는 결과는 보존하고 종료 코드 1을 반환합니다. 접근한 데이터만 검사하며 모든 청크나 전체 미디어 해시를 검증하지는 않습니다. 논리 파일 `.L01`, `.Lx01`은 거부하며 암호 옵션은 없습니다.

별도의 `mount` 명령은 여전히 Linux FUSE가 필요하지만 네이티브 자동 파싱에는 필요하지 않습니다.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## OS에 노출되는 마운트의 요구 사항 { #os-visible-mount-requirements }

`juntei mount`는 Linux FUSE 3(`fusermount3`)와 `/dev/fuse` 접근 권한이 필요합니다. Windows와 macOS에서는 위의 네이티브 분석을 사용하세요.
