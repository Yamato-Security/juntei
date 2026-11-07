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

## auto-parse로 모든 이미지 파싱 { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx`는 별도로 마운트하지 않고도 증거 디렉터리에 있는 모든 VHDX 디스크 이미지(또는 `-d`로
지정한 단일 이미지) 안의 Windows 시스템을 파싱합니다.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- 각 이미지는 새 임시 디렉터리 안에 이미지 이름을 딴 폴더(`host1-triage`)로 읽기 전용 마운트되며, 그곳에서
  발견된 시스템은 디스크에 있는 수집물과 마찬가지로 파싱됩니다. 이미지는 실행이 끝날 때와 Ctrl+C를 눌렀을 때
  마운트 해제됩니다.
- 터미널에는 이미지 목록과 각 시스템의 이미지가 표시됩니다:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- 매니페스트에서 `EvidenceDirectory`는 지정된 디렉터리이며, 각 시스템의 `Image`는 해당 시스템의 이미지를
  나타냅니다. `EvidenceRoot`와 파서 명령어는 마운트 폴더(`./host1-triage/C`)를 사용합니다.
- 이미지에 Volume Shadow Copies의 파일이 `VSS32` 같은 폴더에 들어 있으면 각 폴더를 별도 시스템 (`HOST1_2`, …)으로 찾아 파싱합니다.
- 마운트할 수 없는 이미지는 보고됩니다. 나머지 이미지는 계속 파싱되며, 실행은 종료 상태 1로 끝납니다.

NTFS 볼륨을 열 수 없으면 Juntei는 문제를 보고하고 읽을 수 있는 볼륨은 유지합니다. 이미지의 일부만 읽을 수 있으면 `auto-parse --vhdx`는 종료 코드 1을 반환하며, `mount`도 마운트 해제 후 1을 반환합니다.

## 요구 사항 { #requirements }

- Linux: FUSE 3(`fusermount3`, `fuse3` 패키지)과 `/dev/fuse`에 대한 액세스 권한. 일반 사용자는 보통 이
  권한을 가지고 있습니다. root 권한은 필요하지 않습니다.
- Windows와 macOS는 지원되지 않습니다. Windows에서는 대신 관리자 권한으로 실행한 PowerShell에서 이미지를 읽기
  전용으로 연결하고, 해당 드라이브 문자를 도구에 지정하십시오.

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
