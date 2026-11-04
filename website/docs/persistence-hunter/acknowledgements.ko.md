# Persistence Hunter 감사의 말 { #persistence-hunter-acknowledgements }

Juntei의 `persistence-hunter` 명령어는 두 세대에 걸친 커뮤니티의 지속성 헌팅 성과를 기반으로 합니다. 자신의
기법, 구현, 연구를 조사관들이 사용할 수 있도록 공개해 주신 저자와 기여자 여러분께 감사드립니다.

## PyrsistenceSniper: Python 프로젝트 { #pyrsistencesniper-the-python-project }

[Hexastrike](https://github.com/Hexastrike)의 [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper)는
이 기능의 직접적인 참조 구현입니다. Juntei는 그 130개 검사 카탈로그를 기본 허용 목록, LOLBAS 스냅샷, 분류
동작과 함께 네이티브 Rust로 이식했습니다. 참조 버전은
[커밋 `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f)에
고정되어 있습니다.

이 Python 프로젝트는 MIT 라이선스입니다. Juntei는 그 저작권 및 라이선스 고지를
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt)에
보존합니다. Juntei의 구현은 Python 인터프리터 없이 오프라인으로 실행됩니다. 수집물 처리, 커버리지 보고, 불완전한
증거에 대한 보수적인 해석은 [명령어 가이드](../commands/persistence-hunter.md)에 문서화되어 있습니다.

## PersistenceSniper: 원조 PowerShell 도구 { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper)는 PyrsistenceSniper에 영감을 준 원조
PowerShell 지속성 헌팅 프로젝트입니다. PyrsistenceSniper는 Federico Lagrasta를 출처로 밝히고 있으며, 원래
저장소는 Riccardo Ancarani와 기타 기여자도 함께 밝히고 있습니다. Windows 지속성 메커니즘을 더 쉽게 조사할 수
있게 해 준 원조 프로젝트와 그 기여자들에게 감사드립니다.

Juntei가 직접 이식한 참조 구현은 Python 프로젝트입니다. 원조 PowerShell 도구는 그 계보로서 출처를 밝힙니다.
Juntei는 또한 [hongson11698](https://github.com/hongson11698)이 원조 프로젝트의
[예약된 작업 COM 해석 PR #31](https://github.com/last-byte/PersistenceSniper/pull/31)과
[시작 메뉴 라이브러리/CLSID 폴더 PR #32](https://github.com/last-byte/PersistenceSniper/pull/32)에서 제안한
메커니즘을 독자적으로 구현합니다. 이는 Python 참조 카탈로그에 추가된 것이며, 두 업스트림 프로젝트의 모든 라이브
시스템 기능이 Juntei의 오프라인 명령어에 구현되어 있다는 의미는 아닙니다.

## 지원 연구와 도구 { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/)는 카탈로그에 포함된 기법 식별자를 제공합니다. 기법 매핑은
  메커니즘의 가능한 용도를 설명할 뿐이며, 일치하는 레지스트리 값이나 파일이 그 자체로 ATT&CK에 의해 확인된
  공격은 아닙니다.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)도
  PyrsistenceSniper가 Windows 자동 시작 항목을 조사하는 데 영감을 준 도구로 언급하고 있습니다.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS)는 악용될 수 있는 정상 Windows 바이너리를 문서화합니다.
  이러한 바이너리 중 하나를 인식하면 명령을 해석하는 데 도움이 되지만, 그렇다고 해당 명령이 자동으로 악성이
  되는 것은 아닙니다.

Juntei 전반에서 사용되는 레지스트리, NTFS, Shell Link, 이벤트 로그 및 기타 파서 기반에 대해서는
[프로젝트 전체 감사의 말](../acknowledgements.md)을 참조하십시오.

[지속성 메커니즘 가이드](index.md)로 돌아갑니다.
