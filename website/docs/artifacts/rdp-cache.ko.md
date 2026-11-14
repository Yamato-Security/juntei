# RDP 비트맵 캐시 { #rdp-bitmap-cache }

## RDP 비트맵 캐시가 기록하는 내용 { #what-the-rdp-bitmap-cache-records }

원격 데스크톱 클라이언트(`mstsc.exe`)는 영구 비트맵 캐싱이 켜진 상태(기본값)로 연결하면, 이후 세션에서 같은
부분을 다시 받지 않아도 되도록 원격 화면의 조각을 디스크에 저장합니다. 각 조각은 최대 64 × 64픽셀의
타일입니다. 이 타일들을 모으면 사용자가 원격 컴퓨터에서 본 것, 즉 창, 텍스트, 파일 이름, 명령, 바탕 화면의
작은 조각이 됩니다.

| 클라이언트 | 캐시 파일 | 타일 |
|---|---|---|
| Windows 7 이상 | `Cache0000.bin`, `Cache0001.bin`, …, 각각 최대 100MB | 32비트, 연속으로 저장 |
| Windows XP, Server 2003 이하 | `bcache2.bmc`(픽셀당 8비트), `bcache22.bmc`(16), `bcache23.bmc`(24), `bcache24.bmc`(32), 각각 최대 10MB | 64 × 64 슬롯마다 하나, 일부는 압축됨 |

Windows 10 이상에서는 `.bin` 파일 옆에 빈 `bcache24.bmc`도 남습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 캐시가 들어 있는 프로필의 사용자가 원격 데스크톱
클라이언트로, 캐시된 내용을 표시한 컴퓨터에 연결했습니다. 캐시로는 그 컴퓨터가 어느 컴퓨터였는지,
언제 연결했는지 알 수 없습니다.

## 위치 및 수집 { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```
`HKCU\Software\Microsoft\Terminal Server Client`의 `BitmapPersistCacheLocation` 값으로 캐시 위치를 옮길 수
있습니다. 모든 프로필의 `Cache` 폴더 전체를 볼륨 섀도 복사본에서도 수집하십시오. 캐시 파일의 이전 사본에는
현재 파일에 더 이상 없는 타일이 남아 있을 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- 타일에는 시간이 없으며, 캐시는 타일이 어느 컴퓨터에서 왔는지도 기록하지 않습니다. 다른 증거를 통해 타일을
  RDP 연결과 연관 지으십시오. 클라이언트의 `Terminal Server Client` 레지스트리 키
  (`parse-registry remote-access`), `Default.rdp` 및 기타 `.rdp` 파일, `mstsc.exe`의 Jump Lists, `$MFT`에
  있는 캐시 파일의 시간, 원격 컴퓨터의 로그온 이벤트 등이 있습니다.
- 하나의 캐시 파일에는 여러 세션과 여러 원격 컴퓨터의 타일이 섞여 있습니다. 파일이 가득 차면 클라이언트가
  오래된 타일을 교체하므로, 타일이 며칠 또는 몇 달 전의 것일 수 있습니다.
- 타일은 화면 전체가 아니라 화면의 조각입니다. 화면에서 서로 붙어 있던 타일은 파일에서도 붙어 있는 경우가
  많지만 항상 그렇지는 않습니다. [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)를
  사용하거나 콜라주를 눈으로 보며 화면을 재구성하십시오.
- 대부분의 타일은 64 × 64픽셀입니다. 화면의 오른쪽과 아래쪽 가장자리에 있는 타일은 해상도에 따라 너비가
  좁거나 높이가 낮습니다(예: 너비 58 또는 62픽셀, 높이 28 또는 35픽셀). Juntei는 모든 타일을 각자의 크기로
  읽습니다. 모든 타일을 너비 64픽셀로 읽는 도구에서는 좁은 타일이 비뚤어져 보입니다.
- 단색 타일(`SingleColor`)은 대부분 배경입니다. 내용을 찾으려면 이를 걸러 내십시오.
- `.bmc` 파일에서 8비트 타일은 세션 팔레트의 색으로 표현되지만, 캐시는 그 팔레트를 보관하지 않습니다.
  Juntei는 Windows 기본 팔레트를 사용하므로 색이 틀릴 수 있습니다. 압축된 타일은 원격 데스크톱 프로토콜의
  인터리브 RLE(MS-RDPBCGR 3.1.9)를 사용하며, Juntei는 사양에 정의된 대로 압축을 해제합니다.
- 64 × 64 슬롯보다 작은 `.bmc` 타일 뒤에는 이전 타일의 끝부분이 남습니다. Juntei는 이렇게 남은 행을
  별도의 이미지(`RemnantImage`)로 저장합니다. 이 행들은 슬롯에 있는 타일이 아니라 더 오래된 타일에
  속합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```
```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```
`-d`를 사용하면 Juntei는 `Cache`, 숫자, `.bin`으로 이루어진 이름의 파일과 `bcache`, 숫자, `.bmc`로 이루어진
이름의 파일을 읽습니다. `RDP8bmp` 시그니처로 시작하지 않는 `Cache????.bin` 파일은 다른 프로그램의
파일이므로, 요약에 개수를 집계하고 건너뜁니다. `-f`를 사용하면 이름과 관계없이 해당 파일을 읽습니다.

### 이미지 { #images }

`--extract`는 캐시 파일마다 `<DIR>/<user>/<cache file>/` 폴더를 만듭니다. 캐시 파일과 볼륨 섀도 복사본에
있는 그 사본처럼 두 캐시 파일이 같은 폴더를 쓰게 되는 경우, 두 번째 폴더에는 번호가 붙습니다
(`Cache0001.bin_2`). 폴더에는 다음 파일이 들어 있습니다.

- `<cache file>_<index>.bmp`: 각 타일(원래 크기 그대로). 이름은 bmc-tools가 사용하는 이름과 같으므로
  RdpCacheStitcher로 이 폴더를 열 수 있습니다.
- `<cache file>_old_<index>.bmp`: `.bmc` 전용. 작은 타일 뒤에 남은 이전 타일의 끝부분.
- `<cache file>_collage.png`: 모든 타일을 파일 순서대로 한 행에 64개씩(`--collage-width`) 배치한 이미지로,
  각 타일은 64 × 64 셀의 왼쪽 위부터 놓입니다. 인덱스가 *i*인 타일은 0부터 세어 *i* / 64번째 행,
  *i* % 64번째 열에 있습니다. 셀의 빈 부분, 즉 좁은 타일의 옆, 낮은 타일의 아래, 읽을 수 없었던 타일의
  자리는 마젠타로 채워집니다.

`--collage-only`를 사용하면 콜라주만 저장합니다. 이미 있는 이미지는 `-C` 없이는 교체되지 않으며, 해당 타일의
`Problems`에 그 사실이 기록됩니다. 캐시 파일 자체는 `-C`를 지정해도 절대 교체되지 않습니다.

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 의미 |
|---|---|
| `SourceFile` | 캐시 파일입니다. |
| `User` | 캐시 파일이 들어 있는 프로필 폴더의 사용자로, 경로에서 판단합니다. |
| `TileIndex` | 파일 안에서 타일의 번호입니다. 0부터 세며, 타일 이미지 이름에 들어가는 번호와 같습니다. |
| `Width`, `Height` | 타일의 크기(픽셀)입니다. |
| `SingleColor` | 타일의 모든 픽셀이 같은 색이면 `true`입니다. |
| `TileImage` | `--extract` 사용 시: 타일의 BMP 파일입니다. |
| `Collage` | `--extract` 사용 시: 캐시 파일의 콜라주입니다. |
| `RemnantImage` | `.bmc`에서 `--extract` 사용 시: 슬롯에 남아 있던 이전 타일의 끝부분입니다. |
| `Key` | 원격 데스크톱 프로토콜에서 타일의 64비트 키(16진수)입니다. |
| `Format`, `FormatVersion` | `bin`과 시그니처 뒤의 버전(Windows 10 이상은 6), 또는 `bmc`입니다. |
| `BitsPerPixel` | `.bin`은 32, `.bmc`는 8, 16, 24 또는 32입니다. |
| `Compressed`, `Flags` | `.bmc`: 타일의 압축 여부와 그 헤더의 플래그입니다. |
| `Offset` | 파일에서 타일의 헤더가 시작하는 위치입니다. |
| `Problems` | 손상 내용: 타일 도중에 끝나는 파일, 어떤 타일에도 있을 수 없는 타일 헤더, 타일 크기에 맞지 않는 데이터, 압축을 해제할 수 없는 압축 데이터, 저장할 수 없었던 이미지입니다. |

JSON과 JSONL에는 같은 필드가 있으며, 숫자와 부울 값은 타입이 지정되고, 타일에 없는 값은 `null`,
`Problems`는 목록입니다. 마지막 타일 뒤의 데이터처럼 어느 한 타일에 속하지 않는 손상은 실행 메시지로
보고됩니다.

## 자동 분석 { #automatic-parsing }

`auto-parse`는 `LateralMovement/RdpCache.csv`(`-t json` 또는 `-t jsonl`을 사용하면 `RdpCache.json` 또는
`RdpCache.jsonl`)와 `LateralMovement/RdpCache-logs.txt`를 생성하고, 각 캐시 파일의 콜라주를
`LateralMovement/RdpCache/` 폴더에 저장합니다. `--extract-rdp-tiles`를 추가하면 모든 타일도 저장하는데,
사용자 한 명만으로도 수만 개의 파일이 될 수 있습니다. 디스크 이미지에서는 캐시 파일을 복사합니다. 결합된
`-j` 타임라인에는 모든 행이 `Artifact` 식별자 `rdp-cache`와 함께 포함됩니다. 타일에는 시간이 없으므로
[`super-timeline`](../commands/super-timeline.md)은 타일로부터 이벤트를 만들지 않습니다.

## 조사 전략 { #investigation-strategy }

1. 콜라주를 열고 텍스트를 찾으십시오. 컴퓨터 및 사용자 이름, 파일 경로, 명령, 콘솔 창, 브라우저 주소 표시줄,
   대화 상자 등입니다.
2. `--extract`로 저장한 타일 폴더를 사용해 RdpCacheStitcher로 화면을 재구성하십시오.
3. 클라이언트의 레지스트리(`parse-registry remote-access`), `.rdp` 파일, `mstsc.exe`의 Jump Lists에서
   사용자가 연결한 컴퓨터를 찾으십시오. 그 컴퓨터들에서 일치하는 RDP 로그온과 사용자 액세스 로깅(UAL)
   레코드를 확인하십시오.
4. 캐시 파일과 볼륨 섀도 복사본에 있는 그 사본의 `$MFT` 시간을 사용해 타일이 기록된 시기를 좁히십시오.

## 추가 정보 { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: RDP 비트맵 캐시(프랑스어)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP 캐시 플러그인](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — 영구 비트맵 캐시](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
