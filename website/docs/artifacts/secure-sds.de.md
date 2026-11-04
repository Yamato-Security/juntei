# Sicherheitsbeschreibungen (`$Secure:$SDS`) { #security-descriptors-securesds }

## Was `$SDS` ist { #what-sds-is }

Jede Datei und jedes Verzeichnis auf NTFS hat eine **Sicherheitsbeschreibung** (Security Descriptor): ihren Besitzer, ihre primäre Gruppe, eine
**DACL** (wer was damit tun darf) und eine **SACL** (welche Zugriffe überwacht werden, sowie ihr Integritätslabel).
NTFS speichert jede eindeutige Sicherheitsbeschreibung einmal im Stream `$SDS` der Datei `$Secure` (MFT-Eintrag 9)
und vergibt dafür eine **Sicherheits-ID**. Das `$STANDARD_INFORMATION` einer Datei zeichnet nur diese Sicherheits-ID auf: die
Spalte `SecurityID` von [`parse-ntfs mft`](mft.md). `$SDS` macht daraus einen Besitzer und ACLs.

`$SDS` ist in Blöcke von 256 KiB unterteilt, auf die jeweils eine Kopie folgt (ihre Spiegelung). Jeder Eintrag enthält einen Hash
seiner Sicherheitsbeschreibung, mit dem NTFS eine bereits vorhandene Sicherheitsbeschreibung findet (`$SDH`), sowie die
Sicherheits-ID (die `$SII` indiziert).

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Die Dateien, deren `SecurityID` eine
Sicherheitsbeschreibung benennt, hatten zum Zeitpunkt des letzten Schreibens ihres `$MFT`-Datensatzes diesen Besitzer und diese ACLs. Der Besitzer
einer Datei ist in der Regel das Konto, das sie erstellt hat, doch ein Administrator, ein Installationsprogramm oder ein Programm,
das den Besitz übernimmt, kann ihn ändern.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Sichern Sie ihn mit einem forensischen Werkzeug, das Rohzugriff unterstützt; Sicherungen benennen ihn in der Regel `$Secure_$SDS`, da `:`
nicht in einem Dateinamen vorkommen darf. Sichern Sie zusammen damit die `$MFT` desselben Volumes.

## Hinweise für die Analyse { #analyst-cautions }

- Ein Besitzer ist eine SID. Nur bekannte SIDs (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) und die integrierten Konten eines Computers oder einer Domäne (`-500`
  `Administrator`, …) werden benannt. Lösen Sie die übrigen mit dem `SAM` des Hosts
  ([SAM-Konten](sam.md)), seiner Profilliste ([Systeminformationen](system-info.md)) oder über die Domäne auf.
- Eine Sicherheitsbeschreibung wird von allen Dateien mit demselben Besitzer und denselben ACLs gemeinsam genutzt und verbleibt in der Regel in `$SDS`,
  nachdem diese Dateien gelöscht wurden. Eine Sicherheitsbeschreibung ohne Dateien in der `$MFT` ist für sich genommen keine
  gelöschte Datei.
- Ein ACE ist kein Zugriffsverlauf. Die DACL gibt an, wer auf eine Datei zugreifen *darf*; Ereignisprotokolle und andere
  Artefakte geben an, wer es getan hat. Die Überwachungseinträge der SACL beschreiben die Konfiguration, nicht das, was protokolliert wurde.
- ACEs werden in ihrer Reihenfolge in der ACL aufgeführt. Juntei berechnet keinen effektiven Zugriff: Verweigerungseinträge,
  Vererbung, Gruppenmitgliedschaft, Privilegien und die impliziten Rechte des Besitzers spielen alle eine Rolle.
- `DACL` und `SACL` schreiben jeden ACE so, wie `icacls` es tut. `SDDL` stellt die unterstützten Berechtigungen dar, mit Masken in Hexadezimalschreibweise und Konten als SIDs. Es ist keine bytegenaue Kopie der binären Sicherheitsbeschreibung; nicht darstellbarer bedingter Text lässt es leer und wird als Diagnose gemeldet.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

Die Zusammenfassung zählt die Sicherheitsbeschreibungen und listet ihre Besitzer auf, jeweils mit der Anzahl der Sicherheitsbeschreibungen, die ihnen gehören,
dazu die Sicherheitsbeschreibungen mit einer NULL-DACL (überhaupt keine Einschränkung) sowie etwaige Schäden.

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Interpretation |
|---|---|
| `SourcePath` | Eingabedatei. |
| `SecurityID` | Die Sicherheits-ID: mit der `SecurityID` von [`parse-ntfs mft`](mft.md) für dasselbe Volume verknüpfen. |
| `Hash` | Der mit der Sicherheitsbeschreibung aufgezeichnete Hash. |
| `EntryOffset` | Position des Eintrags im Stream. |
| `FromMirror` | `True` für eine aus dem Spiegelblock gelesene Kopie: eine, die von ihrem Eintrag abweicht (nach diesem aufgeführt), oder die Kopie eines beschädigten Eintrags. |
| `Size` | Die Größe des Eintrags einschließlich seines 20-Byte-Headers. |
| `Owner`, `OwnerName` | Die SID des Besitzers und, falls bekannt, ihr Name. |
| `Group`, `GroupName` | Die SID der primären Gruppe und, falls bekannt, ihr Name. Windows verwendet sie nicht für Zugriffsprüfungen. |
| `Control` | Die Steuerflags der Sicherheitsbeschreibung: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (Vererbung blockiert), `SE_DACL_AUTO_INHERITED`, … verbunden durch `|`. |
| `DACL-ACE-Count`, `DACL` | Die Einträge der DACL, getrennt durch `; `. Leer, wenn keine DACL vorhanden ist; `NULL DACL: no restriction`, wenn die Sicherheitsbeschreibung allen alles gewährt. |
| `SACL-ACE-Count`, `SACL` | Die Einträge der SACL: Überwachungseinträge, das Mandatory-Integrity-Label, Process-Trust-Labels und Ressourcenattribute. |
| `SDDL` | Die gesamte Sicherheitsbeschreibung in SDDL. Nur leer, wenn etwas darin nicht dekodiert wird. |
| `Problems` | Ein Hash, der nicht zur Sicherheitsbeschreibung passt, Teile, die nicht dekodiert werden können, ACEs mit unbekanntem Typ. |

Jeder Eintrag von `DACL` und `SACL` wird so geschrieben, wie `icacls` ihn schreibt: das Konto, der Typ, falls es sich
nicht um einen Zulassungseintrag handelt, die Vererbungsflags und die Rechte.

| Teil | Bedeutung |
|---|---|
| Konto | Der bekannte Name oder die SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | Der ACE-Typ, wenn es sich nicht um einen Zulassungseintrag handelt. Bei einem Mandatory Label wird kein Typ angezeigt. |
| `(I)` | Vom übergeordneten Verzeichnis geerbt. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Vererbt an Dateien (Object Inherit), an Verzeichnisse (Container Inherit), nur an diese und nicht an dieses Objekt (Inherit Only) sowie nur eine Ebene tief (No Propagate). |
| `(SA)`, `(FA)` | Ein Überwachungseintrag für erfolgreichen bzw. fehlgeschlagenen Zugriff. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Vollzugriff, Ändern, Lesen und Ausführen, Lesen, Schreiben. |
| `(RD,WD,AD,…)` | Andernfalls spezifische Rechte: Daten lesen, Daten schreiben, Daten anhängen, erweiterte Attribute lesen und schreiben (`REA`, `WEA`), Ausführen (`X`), Unterelement löschen (`DC`), Attribute lesen und schreiben (`RA`, `WA`), Löschen (`DE`), Lesesteuerung (`RC`), DAC schreiben (`WDAC`), Besitzer schreiben (`WO`), Synchronisieren (`S`) sowie die generischen Rechte `GA`, `GR`, `GW`, `GE`. `N` bedeutet kein Zugriff. |
| `(NW)`, `(NR)`, `(NX)` | Die Richtlinie eines Mandatory Labels: kein Schreiben, kein Lesen, kein Ausführen nach oben (No Write Up, No Read Up, No Execute Up). |
| `if (…)` | Die Bedingung eines Callback-ACE, etwa `(WIN://SYSAPPID Contains "…")`: Der Eintrag gilt nur für Prozesse, für die sie zutrifft. Windows verwendet sie, um App-Container-Paketen Zugriff zu gewähren. |

Process-Trust-Labels und Ressourcenattribute zeigen ihre Maske in Hexadezimalschreibweise, ein Ressourcenattribut
zusätzlich seinen Claim (`("IMAGELOAD",TU,0x0,1)`).

JSON/JSONL haben dieselben Felder und Namen, mit Zahlen als Zahlen, `null` für leere Werte und
`Problems` als Liste.

## Untersuchungsstrategie { #investigation-strategy }

1. Parsen Sie die `$MFT` desselben Volumes und verknüpfen Sie deren `SecurityID` mit der dieser Ausgabe: Jede Datei erhält ihren
   Besitzer und ihre ACLs.
2. Betrachten Sie die Besitzer, die Benutzerkonten sind (`S-1-5-21-…`): welche Dateien einem verdächtigen Konto
   gehören und welche Konten Dateien in Systemverzeichnissen besitzen.
3. Suchen Sie nach freizügigen ACLs auf Verzeichnissen, aus denen Programme ausgeführt oder geladen werden: `Everyone`,
   `Authenticated Users` oder `User` mit `(F)`, `(M)` oder `(W)` sowie NULL-DACLs.
4. Eine Sicherheitsbeschreibung, die nur von einer Datei oder von einer ungewöhnlichen Gruppe von Dateien verwendet wird, kann auf Dateien hinweisen, die
   außerhalb einer normalen Installation erstellt oder geändert wurden.
5. Bestätigen Sie Besitz- und Berechtigungsänderungen mit Einträgen des Security-Ereignisprotokolls (4670, 4663 mit einer
   SACL) und den `SECURITY_CHANGE`-Datensätzen des [USN-Journals](usn-journal.md).

## Weitere Informationen { #further-details }

- [Microsoft: Sicherheitsbeschreibungen](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (Sicherheitsbeschreibungen, ACEs, bedingte Ausdrücke)
- [Microsoft: Zeichenfolgenformat für Sicherheitsbeschreibungen (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS-Formatdokumentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (dessen Crate `secure`)

Bei strukturierter Ausgabe auf stdout (`-t` ohne `-o`) gehen Wiederherstellungswarnungen auf Stream-Ebene an stderr, sodass Pipes lesbares CSV/JSON/JSONL behalten, ohne beschädigte oder wiederhergestellte Bereiche zu verschleiern.
