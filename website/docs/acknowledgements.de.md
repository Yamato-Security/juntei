# Danksagungen { #acknowledgements }

Juntei existiert, weil die DFIR- und Open-Source-Communitys Parser, Formatforschung,
Referenzausgaben, Spezifikationen und Testbeweismaterial veröffentlicht haben. Die unten aufgeführten Projekte haben die Implementierung,
die Validierung, die Benennung von Feldern oder die erwartete Analystenausgabe beeinflusst. Eine Nennung bedeutet nicht, dass ihr Code
in Juntei eingebettet ist; die jeweilige Lizenz entnehmen Sie bitte dem einzelnen Projekt.

## Von Yamato Security gepflegte Parser-Grundlagen { #parser-foundations-maintained-by-yamato-security }

| Juntei-Bereich | Gepflegter Parser | Upstream/Grundlage |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (Crate `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN-Journal | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (Crate `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (Crate `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Verzeichnisindizes (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (Crate `mft`, Modul `i30`) | Unten genannte Forschung zu Index-Slack |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (Crates `boot` und `secure`) | Microsofts MS-DTYP und die unten genannte NTFS-Forschung |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) und [ese](https://github.com/Sootmark/ese) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Arbeiten von Sootmark shell zu Compound Files und Jump Lists |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| VHDX-Disk-Images (`mount`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) und [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), bereitgestellt mit [fuser](https://github.com/cberner/fuser) | Microsofts MS-VHDX |
| Registry | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Unten genannte Forschung zum Registry-Format und zur Wiederherstellung |

## Referenzwerkzeuge und unabhängige Validierung { #reference-tools-and-independent-validation }

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) von Hexastrike
  lieferte den Katalog der Persistenzprüfungen, die Standard-Allowlist, den LOLBAS-Snapshot und das Referenzverhalten
  für den nativen Rust-Befehl `persistence-hunter`. Seine MIT-Lizenz und sein Copyright-Vermerk
  sind in `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt` erhalten. Das ursprüngliche
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) inspirierte das Python-Projekt;
  die vollständige Herkunft finden Sie in den [Danksagungen zur Persistenz](persistence-hunter/acknowledgements.md).

- Maxim Suhanovs **regf-Spezifikation** und **yarp** prägten die Registry-Strukturen, das Wiedereinspielen von Transaktionsprotokollen
  und die Wiederherstellung gelöschter Datensätze; sein **dfir_ntfs** war die Referenz für INDX-Datensätze und
  das Carving aus Index-Slack.
- Harel Segevs **INDXRipper** ist eine weitere Referenz für das Carving von Dateinamen aus Index-Slack.
- **dfir_ntfs**, Joakim Schichts **LogFileParser** und Stroz Friedbergs **ntfs-linker** prägten die
  Validierung und Interpretation von `$LogFile`; Velocidex' **go-ntfs** lieferte öffentliches NTFS-Testbeweismaterial.
- Die Projekte von Joachim Metz / libyal – **libregf**, **libfsntfs**, **libscca**, **libfwsi** und
  **libesedb** – bieten detaillierte Formatforschung und unabhängige Parsing-Referenzen.
- Fox-ITs **dissect.esedb**, **dissect.ntfs** und **dissect.target** lieferten unabhängige Interpretationen von SRUM, UAL,
  `$Secure:$SDS`, SAM und Defender, die während der Validierung verwendet wurden.
- `fsstat` aus **The Sleuth Kit** war die Referenz für NTFS-Bootsektoren von Volumes, die mit
  `mkntfs` erstellt wurden.
- Einige Werkzeuge von Eric Zimmerman haben Feldnamen und Erwartungen von Untersuchenden etabliert, die für Vergleiche herangezogen wurden.
- **regipy**, **python-registry**, **Notatin** und **artemis** lieferten Registry-Implementierungen,
  Tests oder Beispielinterpretationen.
- **LnkParse3**, AbdulRhmanAlfaifis **lnk_parser** und **jumplist_parser** sowie JLECmd lieferten
  unabhängiges LNK-/Jump-List-Verhalten und öffentliche Testfälle.
- Security Ronins **prefetch-forensic** und das Projekt **forensicnomicon** prägten Junteis
  besonders aussagekräftige Beobachtungen zu Prefetch/Amcache/ShimCache.
- **defender-dump** und dissect.target bieten separate Implementierungen zum Verständnis der Strukturen der Defender-Quarantäne
  und eines sicheren Extraktionsverhaltens.
- **Plaso**, **Chainsaw** und **Velociraptor** lieferten Parservergleiche, Testbeweismaterial oder
  sammlungsorientiertes Referenzverhalten für mehrere Artefakte.

## Spezifikationen und Forschung { #specifications-and-research }

Wir danken Microsoft für die Veröffentlichung der Open Specifications zu Shell Links, Compound Binary Files,
NTFS-/SAM-Protokollen, Registry-, Dienst- und Aufgabenstrukturen sowie der API-Dokumentation. Ebenso danken wir den
Autorinnen und Autoren der forensischen Forschung zu AppCompatCache, Amcache, SRUM, Prefetch, Registry und NTFS, die
diese Artefakte für Praktiker verständlich gemacht hat.

## Testbeweismaterial { #test-evidence }

Öffentliches Beweismaterial aus Andrew Rathbuns
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum), NIST CFReDS, Plaso,
Regipy, Artemis, Testrepositorys von Parsern und weiteren ordnungsgemäß lizenzierten Sammlungen ermöglicht Regressionstests
mit echten Windows-Daten. Juntei hält Hinweise zu Urheberschaft und Lizenz neben den enthaltenen
Testdateien vor.

## Mitwirkende { #contributors }

Wir danken allen, die an Juntei und den Parsern von Yamato Security mitgewirkt, Reviews durchgeführt oder Issues gemeldet haben, sowie allen
Benutzerinnen und Benutzern, die Grenzfälle beigesteuert haben. Die aktuellen Mitwirkenden listet GitHub auf der
[Seite der Juntei-Mitwirkenden](https://github.com/Yamato-Security/juntei/graphs/contributors) auf.

## Lizenz { #license }

Juntei wird unter der
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) veröffentlicht; das Copyright liegt bei Shirofune Security Co., Ltd. Der Quellcode wurde vor Version 1.0.0 unter der
AGPLv3 veröffentlicht. Die Lizenzen der Crates sowie des sonstigen Drittanbietercodes und der Drittanbieterdaten, mit denen es
erstellt wird, finden Sie in
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
