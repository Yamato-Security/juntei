# 디스크 이미지 마운트 { #mounting-disk-images }

`juntei mount`는 실행 중 VHDX 디스크 이미지 내부 파일을 디렉터리에서 읽기 전용으로 제공합니다. Hayabusa와 Juntei 같은 도구로 파일 추출이나 root 권한 없이 스캔할 수 있습니다. FUSE를 사용하며 Linux에서 작동합니다. 리더와 마운트 라이브러리는 [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser)에서 관리합니다.

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
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 이미지의 네이티브 분석 { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx`는 **Linux, macOS, Windows**에서 VHDX를 네이티브로 읽습니다. OS 마운트, FUSE, 관리자 권한이 필요하지 않습니다.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

파서는 파일 경로를 요구하므로 NTFS 리더가 제공하는 파일을 비공개 임시 디렉터리에 복사합니다. **해당 파일을 저장할 충분한 임시 디스크 공간을 확보하세요.** 원본 이미지는 읽기 전용이며 정상 종료 시 임시 복사본을 삭제합니다.

읽기 오류와 안전하지 않은 이름은 `ImageErrors` 및 `manifest.csv`의 실패 행에 기록합니다. 읽을 수 있는 증거의 결과는 유지하며, 불완전한 입력은 종료 코드 1을 반환합니다.

다른 프로그램에 파일을 제공하는 별도 `mount` 명령은 여전히 Linux, FUSE 3 및 `/dev/fuse` 접근 권한을 요구합니다.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
