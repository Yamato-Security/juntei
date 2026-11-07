# Подяки { #acknowledgements }

Juntei існує завдяки тому, що спільноти DFIR та відкритого коду публікували парсери, дослідження
форматів, еталонний вивід, специфікації та тестові докази. Перелічені нижче проєкти вплинули на
реалізацію, перевірку, найменування полів або очікуваний аналітиками вивід. Присутність у переліку
не означає, що їхній код вбудовано в Juntei; ліцензію кожного проєкту дивіться в самому проєкті.

## Основи парсерів, що їх підтримує Yamato Security { #parser-foundations-maintained-by-yamato-security }

| Область Juntei | Парсер, що підтримується | Першоджерело/основа |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (крейт `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| Журнал USN | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (крейт `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (крейт `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Індекси каталогів (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (крейт `mft`, модуль `i30`) | Дослідження index slack, згадані нижче |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (крейти `boot` і `secure`) | MS-DTYP від Microsoft і дослідження NTFS, згадані нижче |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) і [ese](https://github.com/Sootmark/ese) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Робота Sootmark shell щодо складених файлів і Jump Lists |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| Реєстр | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Дослідження формату реєстру та відновлення даних, згадані нижче |

## Еталонні інструменти та незалежна перевірка { #reference-tools-and-independent-validation }

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) від Hexastrike
  надав каталог перевірок закріплення, типовий список дозволених елементів, знімок LOLBAS та
  еталонну поведінку для нативної Rust-команди `persistence-hunter`. Його ліцензію MIT і повідомлення
  про авторські права збережено у `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`. Оригінальний
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) надихнув цей Python-проєкт;
  повну історію походження див. у [подяках щодо закріплення](persistence-hunter/acknowledgements.md).

- **Специфікація regf** та **yarp** Максима Суханова вплинули на структури реєстру, відтворення
  журналів транзакцій і відновлення видалених записів; його **dfir_ntfs** був еталоном для записів INDX
  та вирізання (carving) з index slack.
- **INDXRipper** Гареля Сегева — ще один еталон для вирізання імен файлів з index slack.
- **dfir_ntfs**, **LogFileParser** Йоакима Шихта та **ntfs-linker** від Stroz Friedberg вплинули на
  перевірку та інтерпретацію `$LogFile`; **go-ntfs** від Velocidex надав публічні тестові докази NTFS.
- Проєкти Йоахима Метца / libyal — **libregf**, **libfsntfs**, **libscca**, **libfwsi** і
  **libesedb** — містять докладні дослідження форматів і є незалежними еталонами парсингу.
- **dissect.esedb**, **dissect.ntfs** і **dissect.target** від Fox-IT надали незалежні інтерпретації SRUM,
  UAL, `$Secure:$SDS`, SAM і Defender, використані під час перевірки.
- `fsstat` з **The Sleuth Kit** був еталоном для завантажувальних секторів NTFS томів, створених за допомогою
  `mkntfs`.
- Деякі інструменти Еріка Ціммермана встановили назви полів і очікування експертів, використані для порівнянь.
- **regipy**, **python-registry**, **Notatin** та **artemis** надали реалізації реєстру,
  тести або приклади інтерпретацій.
- **LnkParse3**, **lnk_parser** і **jumplist_parser** від AbdulRhmanAlfaifi, а також JLECmd надали
  незалежну поведінку LNK/Jump List і публічні тестові випадки.
- **prefetch-forensic** від Security Ronin і проєкт **forensicnomicon** вплинули на
  найцінніші спостереження Juntei щодо Prefetch/Amcache/ShimCache.
- **defender-dump** і dissect.target надають окремі реалізації для розуміння структур карантину
  Defender і безпечного вилучення даних.
- **Plaso**, **Chainsaw** і **Velociraptor** надали порівняння парсерів, тестові докази або
  еталонну поведінку, орієнтовану на збирання, для кількох артефактів.

## Специфікації та дослідження { #specifications-and-research }

Дякуємо Microsoft за публікацію Open Specifications для Shell Links, Compound Binary Files,
протоколів NTFS/SAM, структур реєстру/служб/завдань і документації API. Також дякуємо
авторам криміналістичних досліджень AppCompatCache, Amcache, SRUM, Prefetch, реєстру та NTFS, завдяки яким
ці артефакти стали зрозумілими для практиків.

## Тестові докази { #test-evidence }

Публічні докази з
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) Ендрю Ратбана, NIST CFReDS, Plaso,
Regipy, Artemis, репозиторіїв тестів парсерів та інших колекцій із належним ліцензуванням роблять можливим
регресійне тестування на реальних даних Windows. Juntei зберігає примітки щодо авторства та ліцензій поруч із
включеними тестовими файлами.

## Учасники { #contributors }

Дякуємо кожному учаснику розробки, рецензенту, автору звітів про проблеми парсерів Juntei і Yamato Security, а також
користувачам, які надали граничні випадки. Поточних учасників перелічено на GitHub на
[сторінці учасників Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors).

## Ліцензія { #license }

Juntei випущено під
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5), авторські права належать Shirofune Security Co., Ltd. До версії 1.0.0 його вихідний код
публікувався під AGPLv3. Ліцензії крейтів та іншого стороннього коду й даних, з якими його
зібрано, наведено в
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
