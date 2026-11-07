# Juntei 소개 { #about-juntei }

Juntei는 메모리 안전 언어인 Rust로 작성된 Windows 포렌식 아티팩트 파서입니다.
[Hayabusa](https://github.com/Yamato-Security/hayabusa)를 보완하는 도구로, Hayabusa가 Windows 이벤트 로그를
분석하는 반면 Juntei는 파일 시스템, 레지스트리, 실행, 사용자 활동, 리소스 사용 및 보안 제품과 관련된
영구적인 아티팩트를 분석합니다.

동일한 바이너리가 Windows, macOS 및 Linux에서 실행됩니다. 이는 분석가가 KAPE, Velociraptor, 디스크 이미지
또는 기타 오프라인 수집본을 받았을 때 아티팩트마다 별도의 프로그램을 사용하는 대신 하나의 일관된
인터페이스를 사용하고자 하는 경우에 유용합니다.

## 설계 목표 { #design-goals }

- **입증 가능한 분석:** 데이터를 몰래 만들어 내지 않고 원시 값을 보존하며 손상을 보고합니다.
- **모두에게 유용함:** 기본 사용법은 초보자도 쉽게 사용할 수 있어야 하며, 숙련자를 위해 사용자 지정이
  가능하고 모든 기능을 갖추어야 합니다.
- **독립 실행형 바이너리:** 실행하는 데 런타임 라이브러리가 필요하지 않습니다. 별도 설정 없이 바로
  작동합니다.
- **매우 빠른 속도:** 많은 메모리를 요구하지 않으면서도 가장 빠른 파서를 목표로 합니다.
- **크로스 플랫폼 사용:** Windows 워크스테이션 없이도 Windows 증거를 분석할 수 있습니다.
- **자동화:** 스프레드시트와 타임라인 도구를 위한 CSV, `jq`, 스크립트 및 데이터 레이크를 위한 JSON/JSONL을 제공합니다.
- **안전한 실패 처리:** 잘못된 형식의 레코드는 보고하고, 가능한 경우 건너뜁니다.
- **투명한 출처:** Juntei는 각 구현의 기반이 된 오픈 소스 파서와 참조 도구를 명시합니다.

## Juntei가 하지 않는 일 { #what-juntei-does-not-do }

Juntei는 라이브 디스크를 수집하거나 증거 보존을 대신하지 않으며, 마운트하는 디스크 이미지는 VHDX 이미지뿐이고
읽기 전용으로 마운트합니다.
또한 모든 아티팩트를 하나의 인시던트 서사로 자동 결합하지 않습니다. Juntei는 제공된 증거를 분석하여
조사와 상관 분석에 적합한 레코드를 생성합니다.

또한 Juntei가 분석을 대신해 주지는 않습니다. 증거에 대한 분석은 DFIR 전문가가 수행해야 합니다.

## 라이선스 { #license }

Juntei는
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5)로 배포되며, 저작권은 Shirofune Security Co., Ltd.에 있습니다. 버전 1.0.0 이전의 소스 코드는
AGPLv3로 공개되었습니다. Juntei를 빌드하는 데 사용된 크레이트 및 기타 서드파티 코드와 데이터의 라이선스는
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES)에
나와 있습니다. Juntei의 기반이 되었거나 검증에 사용된 프로젝트는
[감사의 말](../acknowledgements.md)에 정리되어 있습니다.
