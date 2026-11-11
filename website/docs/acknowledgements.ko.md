# 감사의 말 { #acknowledgements }

Juntei는 DFIR 및 오픈 소스 커뮤니티가 공개한 파서, 형식 연구, 참조 출력, 사양, 테스트 증거 덕분에
존재합니다. 아래 프로젝트들은 구현, 검증, 필드 명명 또는 분석가가 기대하는 출력에 참고가 되었습니다.
목록에 포함되어 있다고 해서 해당 코드가 Juntei에 포함되어 있다는 의미는 아닙니다. 라이선스는 각 프로젝트를
확인하십시오.

## Yamato Security가 유지 관리하는 파서 기반 { #parser-foundations-maintained-by-yamato-security }

| Juntei 영역 | 유지 관리 파서 | 업스트림/기반 |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` 크레이트) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN 저널 | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`usn` 크레이트) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`logfile` 크레이트) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| 디렉터리 인덱스 (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` 크레이트, `i30` 모듈) | 아래에 기재된 인덱스 슬랙 연구 |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` 및 `secure` 크레이트) | Microsoft의 MS-DTYP 및 아래에 기재된 NTFS 연구 |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) 및 [ese](https://github.com/Sootmark/ese) |
| 웹 브라우저 / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite) 및 [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark shell의 복합 파일 및 Jump List 관련 작업 |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| VHDX 디스크 이미지 (`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) 및 [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), [fuser](https://github.com/cberner/fuser)로 제공 | Microsoft의 MS-VHDX |
| E01 / Ex01 (`mount`, `auto-parse --e01`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) | [ewf-image](https://github.com/ebrig/ewf-image) (Apache-2.0); [libewf](https://github.com/libyal/libewf) (EWF1/EWF2) |
| 레지스트리 | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | 아래에 기재된 레지스트리 형식 및 복구 연구 |

## 참조 도구와 독립 검증 { #reference-tools-and-independent-validation }

- Hexastrike의 [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper)는 네이티브 Rust
  `persistence-hunter` 명령어의 지속성 검사 카탈로그, 기본 허용 목록, LOLBAS 스냅샷 및 참조 동작을
  제공했습니다. 해당 MIT 라이선스와 저작권 고지는 `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`에
  보존되어 있습니다. 원본인 [PersistenceSniper](https://github.com/last-byte/PersistenceSniper)가 이 Python
  프로젝트에 영감을 주었습니다. 전체 계보는 [지속성 관련 감사의 말](persistence-hunter/acknowledgements.md)을
  참조하십시오.

- Maxim Suhanov의 **regf specification**과 **yarp**는 레지스트리 구조, 트랜잭션 로그 재생 및 삭제된 레코드
  복구에 참고가 되었습니다. 그의 **dfir_ntfs**는 INDX 레코드와 인덱스 슬랙 카빙의 참조 구현이었습니다.
- Harel Segev의 **INDXRipper**는 인덱스 슬랙에서 파일 이름을 카빙하기 위한 또 다른 참조 구현입니다.
- **dfir_ntfs**, Joakim Schicht의 **LogFileParser**, Stroz Friedberg의 **ntfs-linker**는 `$LogFile` 검증과
  해석에 참고가 되었습니다. Velocidex의 **go-ntfs**는 공개 NTFS 테스트 증거를 제공했습니다.
- Joachim Metz / libyal 프로젝트인 **libregf**, **libfsntfs**, **libscca**, **libfwsi**, **libesedb**는
  상세한 형식 연구와 독립적인 파싱 참조 구현을 제공합니다.
- Fox-IT의 **dissect.esedb**, **dissect.ntfs**, **dissect.target**은 검증 과정에서 사용된 SRUM, UAL,
  `$Secure:$SDS`, SAM 및 Defender에 대한 독립적인 해석을 제공했습니다.
- **The Sleuth Kit**의 `fsstat`는 `mkntfs`로 만든 볼륨의 NTFS 부트 섹터에 대한 참조 구현이었습니다.
- Eric Zimmerman의 일부 도구는 비교에 사용된 필드 이름과 조사관의 기대 사항을 확립했습니다.
- **regipy**, **python-registry**, **Notatin**, **artemis**는 레지스트리 구현, 테스트 또는 샘플 해석을
  제공했습니다.
- **LnkParse3**, AbdulRhmanAlfaifi의 **lnk_parser** 및 **jumplist_parser**, 그리고 JLECmd는 LNK/Jump List에
  대한 독립적인 동작과 공개 테스트 케이스를 제공했습니다.
- Security Ronin의 **prefetch-forensic**과 **forensicnomicon** 프로젝트는 Juntei의 고가치
  Prefetch/Amcache/ShimCache 관찰 항목에 참고가 되었습니다.
- **defender-dump**와 dissect.target은 Defender 격리 구조와 안전한 추출 동작을 이해하기 위한 별도의 구현을
  제공합니다.
- **Plaso**, **Chainsaw**, **Velociraptor**는 여러 아티팩트에 걸쳐 파서 비교, 테스트 증거 또는 수집 중심의
  참조 동작을 제공했습니다.
- Obsidian Forensics의 **Hindsight**와 Velocidex의 **SQLiteHunter** 정의는 브라우저 방문 기록, 다운로드, 쿠키 및
  양식 기록에 대한 참고 자료가 되었습니다.
- Abel Cheung의 **rifiuti2**는 휴지통 `$I` 파일과 INFO2의 참고 자료가 되었으며, libyal의 **dtformats**
  문서가 두 형식을 설명합니다.

## 사양과 연구 { #specifications-and-research }

Shell Links, Compound Binary Files, NTFS/SAM 프로토콜, 레지스트리/서비스/작업 구조에 대한 Open
Specifications와 API 문서를 공개한 Microsoft에 감사드립니다. 또한 이러한 아티팩트를 실무자가 이해할 수 있도록
해 준 AppCompatCache, Amcache, SRUM, Prefetch, 레지스트리 및 NTFS 포렌식 연구의 저자들에게도 감사드립니다.

## 테스트 증거 { #test-evidence }

Andrew Rathbun의 [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum), NIST CFReDS,
Plaso, rifiuti2, Regipy, Artemis, 파서 테스트 저장소 및 기타 적절한 라이선스를 갖춘 컬렉션의 공개 증거 덕분에 실제
Windows 데이터를 대상으로 한 회귀 테스트가 가능합니다. Juntei는 포함된 테스트 파일 옆에 출처/라이선스 고지를
함께 보관합니다.

## 기여자 { #contributors }

경계 사례를 제공해 주신 모든 Juntei 및 Yamato Security 파서 기여자, 리뷰어, 이슈 보고자, 사용자 여러분께
감사드립니다. 현재 기여자 목록은 GitHub의 [Juntei 기여자 페이지](https://github.com/Yamato-Security/juntei/graphs/contributors)에
있습니다.

## 라이선스 { #license }

Juntei는 [Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5)로 배포되며, 저작권은 Shirofune Security Co., Ltd.에 있습니다. 버전 1.0.0 이전에는 소스 코드가
AGPLv3로 공개되었습니다. Juntei를 빌드하는 데 사용된 크레이트와 기타 서드 파티 코드 및 데이터의 라이선스는
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES)에 있습니다.
