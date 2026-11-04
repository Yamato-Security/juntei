# BAM / DAM { #bam-and-dam }

## Was BAM und DAM sind { #what-bam-and-dam-are }

Der Background Activity Moderator (BAM) und der Desktop Activity Moderator (DAM) sind Windows-Dienste
(Windows 10 1709 und später), die steuern, was Anwendungen im Hintergrund tun dürfen.
Sie speichern pro Benutzer den Zeitpunkt der letzten Ausführung jedes Programms im `SYSTEM`-Hive:

- einen Schlüssel pro Benutzer, benannt nach der SID des Benutzers;
- einen Wert pro Programm: Sein Name ist der Pfad des Programms auf seinem Gerät
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) oder die ID einer paketierten App
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- die ersten 8 Bytes der Wertdaten sind der Zeitpunkt der letzten Ausführung (FILETIME).

BAM verknüpft ein Programm mit einem Benutzer und einem Zeitpunkt, ohne dass der Hive des Benutzers benötigt wird, und es listet
Programme auf, die von Wechsel- und Netzwerkgeräten ausgeführt wurden.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Ein Prozess dieses Programms lief in der Sitzung dieses
Benutzers; zuletzt lief er zu diesem Zeitpunkt. BAM zählt keine Ausführungen, und ein Eintrag kann bestehen bleiben, nachdem das
Programm gelöscht wurde.

## Speicherort { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

Der Schlüssel jedes Benutzers hat außerdem die Werte `Version` und `SequenceNumber`; sie sind keine Programme und werden
ausgelassen. Ein von 1709 oder 1803 aktualisiertes System kann beide Layouts aufweisen.

Sichern Sie `SYSTEM` mit seinen Transaktionsprotokollen sowie `SAM` und `SOFTWARE` des Systems: Sie benennen die
Benutzer.

## Hinweise für die Analyse { #analyst-cautions }

- **Ein Zeitpunkt pro Programm und Benutzer.** Nur die jüngste Ausführung wird gespeichert. Prefetch, Amcache und
  Ereignisprotokolle liefern frühere Ausführungen.
- **Jüngste Aktivität.** Einträge von Programmen, die eine Zeit lang nicht gelaufen sind, werden Berichten zufolge
  beim Start von Windows entfernt; BAM zeigt daher überwiegend jüngste Aktivität: Ein Programm, das dort fehlt,
  kann dennoch vor langer Zeit gelaufen sein.
- **Gerätepfade, keine Laufwerksbuchstaben.** `\Device\HarddiskVolume3` ist das dritte Volume, das Windows
  nummeriert hat, nicht `C:`. `DevicePath` und `Path` trennen das Gerät vom Pfad darauf; ordnen Sie das
  Volume mit `MountedDevices`, `$MFT` oder anderen Artefakten zu, bevor Sie einen Laufwerksbuchstaben angeben.
- **Nicht jedes Programm.** BAM und DAM verfolgen Programme in Benutzersitzungen; Dienste erscheinen meist
  nicht. Einträge unter `S-1-5-18` sind typischerweise Prozesse des Setups und des Anmeldebildschirms
  (`FirstLogonAnim.exe`), solche unter `S-1-5-90-0-<n>` der Desktop Window Manager (`DWM-<n>`).
- **Control Sets.** Standardmäßig wird das zuletzt von Windows verwendete Control Set gelesen. `--all-control-sets`
  liest alle: Ein älteres Control Set kann einen Eintrag enthalten, den das aktuelle nicht mehr hat.
- **Benutzernamen.** Der Name eines lokalen Kontos stammt aus dem `SAM` (SID des Computers und RID des
  Kontos); der eines Domänenkontos aus seinem Profilordner in `ProfileList`. `UserSource`
  gibt an, welche Quelle verwendet wurde (`sam`, `profile_list`, `well_known`). Ohne diese wird nur die SID angegeben. Eine RID,
  die der `SAM` nicht mehr enthält (ein gelöschtes Konto), bleibt ohne Namen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Die Zusammenfassung listet jeden Benutzer mit SID, Name und Anzahl der Einträge sowie die zuletzt ausgeführten Programme
(neueste zuerst) auf.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host und der `SYSTEM`-Hive. |
| `ControlSet`, `IsCurrentControlSet` | Das Control Set, in dem der Eintrag steht, und ob es das zuletzt von Windows verwendete ist. |
| `Moderator` | `bam` oder `dam`. |
| `UserSID`, `User`, `UserSource` | Die SID, nach der der Schlüssel benannt ist, der Name des Benutzers und seine Herkunft (`sam`, `profile_list`, `well_known`). |
| `Program` | Der Name des Werts, wie Windows ihn geschrieben hat: ein Gerätepfad oder die ID einer paketierten App. |
| `DevicePath`, `Path` | Bei einem Gerätepfad das Gerät (`\Device\HarddiskVolume3`) und der Pfad darauf (`\Windows\System32\cmd.exe`); leer bei einer paketierten App. |
| `LastRunUTC` | Zeitpunkt der letzten Ausführung des Programms (auf 100 ns genau). |
| `KeyPath`, `KeyLastWrittenUTC` | Der Schlüssel des Benutzers und sein letzter Schreibzugriff: die jüngste Aktualisierung eines seiner Einträge. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn sie keinen Zeitpunkt enthalten: nicht `REG_BINARY` oder weniger als 8 Bytes. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Daten, die keinen Zeitpunkt enthalten, und Werte, die nicht gelesen werden können. |

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie nach `LastRunUTC` und betrachten Sie Programme an für Benutzer beschreibbaren Orten (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), auf anderen Geräten als dem Systemvolume sowie Administrations- und
   Fernzugriffstools.
2. Prüfen Sie, welcher Benutzer sie ausgeführt hat, und achten Sie auf Konten, die keinem aktuellen Profil entsprechen.
3. Korrelieren Sie mit Prefetch (Ausführungsanzahl und frühere Ausführungen), UserAssist (Starts über den Explorer), Amcache
   und ShimCache (Vorhandensein), `$MFT` (die Programmdatei) und Ereignisprotokollen.
4. Verwenden Sie `--all-control-sets` sowie `parse-registry read --recover-deleted` für gelöschte BAM-Werte.

## Weitere Informationen { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Sicherheits-IDs (bekannte SIDs)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Außerhalb des gültigen Bereichs liegende FILETIMEs der letzten Ausführung lassen `LastRunUTC` leer und bewahren den ursprünglichen Wert in `DataRaw`, mit einer Erläuterung in `Problems`. Die Zusammenfassung der jüngsten Aktivität identifiziert jeden Host und hält getrennte Hive-/Benutzerbeobachtungen auseinander.
