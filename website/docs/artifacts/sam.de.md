# Lokale SAM-Konten { #sam-local-accounts }

## Was der SAM enthält { #what-the-sam-contains }

Der Hive des Security Account Manager (`SAM`) speichert Datensätze lokaler Konten. Juntei dekodiert
Kontonamen, RIDs, Profil-/Kommentarfelder, Kontosteuerungs-Flags, Zeitstempel der letzten Anmeldung/Kennwortänderung, Zähler,
Kennworthinweise, die Verknüpfung mit einem Microsoft-Konto, Profilpfade und ob Windows das Konto
auf der Anmeldeoberfläche ausblendet.

Dies hilft beim Auffinden neu erstellter lokaler Administratoren, aktivierter ruhender Konten, Kennwortänderungen,
ausgeblendeter Konten und von Konten, die für laterale Bewegung vorbereitet wurden.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der lokale SAM bildete das Konto mit
dem aufgezeichneten Zustand ab. Er ist keine vollständige Authentifizierungshistorie und beschreibt keine Domänenkonten.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Sichern Sie den Hive und die Protokolle mit ausreichenden Berechtigungen bzw. per Rohzugriff. Die Hostidentität kann in einer Sammlung anhand
des zugehörigen `SYSTEM`-Hives ergänzt werden.

## Hinweise für die Analyse { #analyst-cautions }

- Dieser Befehl parst **lokale** Konten, keine Domänenbenutzer des Active Directory.
- Juntei extrahiert bewusst keine Kennwort-Hashes.
- RID 500 ist das integrierte Administratorkonto, auch wenn es umbenannt wurde; der Name allein reicht nicht aus.
- `LastLogon` und die Zähler sind Registry-Zustand, kein vollständiges Prüfprotokoll pro Sitzung.
- `HiddenFromLogonUI` blendet ein Konto auf dem Anmeldebildschirm aus; es deaktiviert das Konto nicht.
- Die integrierten Konten `DefaultAccount` und `WDAGUtilityAccount` können legitimerweise ausgeblendet sein.
- Leere Konto-Flags aufgrund eines beschädigten/fehlenden `F`-Werts dürfen nicht als aktiviert interpretiert werden.
- `NamesKeyLastWrittenUTC` gibt näherungsweise den Kontext der Registrierung des Kontonamens wieder, ist aber kein garantierter Erstellungszeitpunkt.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Hostkontext und exakter SAM-Hive. |
| `RID` | Relativer Bezeichner (Relative Identifier). Starker Ankerpunkt für die Kontoidentität; 500 ist der integrierte Administrator. |
| `User` | Aktueller Name des lokalen Kontos. Kann umbenannt sein; über die RID korrelieren. |
| `FullName`, `Comment`, `UserComment` | Beschreibender Text; vom Angreifer steuerbar und nützlich zur Prüfung auf Tarnung. |
| `AccountFlags` | Dekodierte Zustände wie deaktiviert, gesperrt, Kennwort nicht erforderlich, normales Konto, Kennwort läuft nie ab. |
| `AccountFlagsValue` | Numerische Roh-Flags zur Validierung und für nicht unterstützte Bits. |
| `CountryCode`, `CodePage` | Metadaten zu Gebietsschema/Codepage des Kontos; in der Regel Kontext, keine Zuordnung. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | Derselbe Zustand der letzten Anmeldung in voller Genauigkeit und in ganzen Sekunden zur Kompatibilität. Keine vollständige Historie. |
| `PasswordLastSetUTC` | Aufgezeichneter Zeitpunkt der Kennwortänderung. Unerwartet neue/alte Werte untersuchen. |
| `AccountExpirationUTC` | Konfigurierter Ablauf, sofern endlich. Null kann nie/nicht gesetzt bedeuten. |
| `LastFailedLogonUTC` | Letzte im SAM-Zustand aufgezeichnete fehlgeschlagene lokale Anmeldung; mit Security-Ereignissen untermauern. |
| `FailedLogonCount`, `LogonCount` | Kumulative/zustandsbezogene Zähler mit Windows-spezifischem Aktualisierungsverhalten; als Kontext verwenden, nicht als exakte Ereignisanzahl. |
| `PasswordHint` | Sensibler, vom Benutzer angegebener Hinweis; kann Kennwortmuster offenlegen. Angemessen behandeln. |
| `InternetUserName` | Identität des verknüpften Microsoft-Kontos, sofern vorhanden. |
| `HiddenFromLogonUI` | Ausblendungszustand auf der Oberfläche. Hohe RIDs/unerwartete benutzerdefinierte Konten prüfen; bedeutet nicht deaktiviert. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Umgebungs-/Anmeldekonfiguration des lokalen Kontos; ungewöhnliche Skripte/UNC-Pfade können auf Persistenz hindeuten. |
| `KeyLastWrittenUTC` | Änderungszeitpunkt des RID-Kontoschlüssels. Auf Schlüsselebene, nicht feldspezifisch. |
| `NamesKeyLastWrittenUTC` | Änderungszeitpunkt des Schlüssels für die Zuordnung des Benutzernamens; nützliche Näherung für den Lebenszyklus des Kontos. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Rohe 64-Bit-FILETIME-Werte, aufbewahrt zur Validierung und zur Behandlung spezieller Sentinel-Werte. |
| `TransactionLogStatus`, `LogsApplied` | Kontext des Registry-Wiedereinspielens. |
| `Problems` | Abweichungen zwischen F/V/Names, fehlende Einträge, fehlerhafte Offsets oder Dekodierungsschäden. |

## Untersuchungsstrategie { #investigation-strategy }

1. Vergleichen Sie RID, Name, Flags, Profilpfad und `NamesKeyLastWrittenUTC` mit der Baseline des Hosts.
2. Prüfen Sie aktivierte Konten mit RID 1000+, Konten ohne erforderliches Kennwort, mit nie ablaufendem Kennwort, ausgeblendete oder kürzlich geänderte Konten.
3. Korrelieren Sie mit den Security-Ereignissen 4720/4722/4724/4732, Anmeldeereignissen, der Profilerstellung und Remoteaktivitäten.
4. Durchsuchen Sie die Wiederherstellungshistorie der Registry, falls ein Konto vor der Sicherung entfernt wurde.
5. Unterscheiden Sie legitime Bereitstellungs-/Standardkonten von interaktiv durch Angreifer erstellten Konten.

## Weitere Informationen { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [SAM-Plugin von dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
- [Registry Explorer von Eric Zimmerman](https://ericzimmerman.github.io/#!index.md)
