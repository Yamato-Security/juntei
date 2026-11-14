# RDP-Bitmap-Cache { #rdp-bitmap-cache }

## Was der RDP-Bitmap-Cache aufzeichnet { #what-the-rdp-bitmap-cache-records }

Wenn der Remotedesktopclient (`mstsc.exe`) mit aktiviertem persistentem Bitmap-Caching eine Verbindung herstellt (das
ist die Standardeinstellung), speichert er Teile des entfernten Bildschirms auf dem Datenträger, damit spätere Sitzungen
sie nicht erneut empfangen müssen. Jeder Teil ist eine Kachel von höchstens 64 × 64 Pixeln. Zusammen ergeben die Kacheln
kleine Ausschnitte dessen, was der Benutzer auf den entfernten Computern gesehen hat: Fenster, Text, Dateinamen, Befehle
und Desktops.

| Client | Cachedateien | Kacheln |
|---|---|---|
| Windows 7 und neuer | `Cache0000.bin`, `Cache0001.bin`, …, jeweils bis zu 100 MB | 32 Bit, eine nach der anderen |
| Windows XP, Server 2003 und älter | `bcache2.bmc` (8 Bit pro Pixel), `bcache22.bmc` (16), `bcache23.bmc` (24) und `bcache24.bmc` (32), jeweils bis zu 10 MB | Eine pro Slot von 64 × 64; einige komprimiert |

Windows 10 und neuer legen außerdem eine leere `bcache24.bmc` neben den `.bin`-Dateien an.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Benutzer, in dessen Profil der Cache liegt,
hat sich mit dem Remotedesktopclient mit einem Computer verbunden, der die zwischengespeicherten Inhalte angezeigt hat.
Der Cache gibt nicht an, welcher Computer das war und wann.

## Speicherorte und Sicherung { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

Der Wert `BitmapPersistCacheLocation` unter `HKCU\Software\Microsoft\Terminal Server Client` kann den Cache an einen
anderen Ort verlegen. Sichern Sie den gesamten Ordner `Cache` jedes Profils, auch aus Volumeschattenkopien: Eine ältere
Kopie einer Cachedatei kann Kacheln enthalten, die die aktuelle nicht mehr hat.

## Hinweise für die Analyse { #analyst-cautions }

- Kacheln haben keine Zeitangaben, und der Cache nennt nicht die Computer, von denen sie stammen. Ordnen Sie sie
  RDP-Verbindungen anhand anderer Beweismittel zu: den Registry-Schlüsseln `Terminal Server Client` des Clients
  (`parse-registry remote-access`), `Default.rdp` und anderen `.rdp`-Dateien, den Jump Lists von `mstsc.exe`, den
  Zeiten der Cachedateien in `$MFT` und den Anmeldeereignissen der entfernten Computer.
- Eine Cachedatei mischt Kacheln vieler Sitzungen und mehrerer entfernter Computer. Der Client ersetzt alte Kacheln,
  wenn sich eine Datei füllt; eine Kachel kann also Tage oder Monate alt sein.
- Kacheln sind Teile von Bildschirmen, keine ganzen Bildschirme. Kacheln, die auf dem Bildschirm nebeneinander lagen,
  liegen oft auch in der Datei nebeneinander, aber nicht immer. Setzen Sie Bildschirme mit
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) oder nach Augenmaß anhand der Collagen zusammen.
- Die meisten Kacheln sind 64 × 64 Pixel groß. Kacheln am rechten und unteren Rand eines Bildschirms sind je nach
  Auflösung schmaler oder niedriger: zum Beispiel 58 oder 62 Pixel breit oder 28 oder 35 Pixel hoch. Juntei liest jede
  Kachel in ihrer eigenen Größe. Werkzeuge, die jede Kachel als 64 Pixel breit lesen, zeigen die schmalen verzerrt an.
- Die meisten einfarbigen Kacheln (`SingleColor`) sind Hintergrund. Filtern Sie sie heraus, um Inhalte zu finden.
- In `.bmc`-Dateien bestehen 8-Bit-Kacheln aus Farben der Palette der Sitzung, die der Cache nicht speichert. Juntei
  verwendet die Standardpalette von Windows, daher können ihre Farben falsch sein. Komprimierte Kacheln verwenden das
  Interleaved RLE des Remotedesktopprotokolls (MS-RDPBCGR 3.1.9), das Juntei so dekomprimiert, wie die Spezifikation
  es definiert.
- Eine `.bmc`-Kachel, die kleiner als ihr Slot von 64 × 64 ist, lässt dahinter das Ende einer früheren Kachel
  stehen. Juntei speichert diese übrig gebliebenen Zeilen als separates Bild (`RemnantImage`). Sie gehören zu einer
  älteren Kachel, nicht zu der Kachel im Slot.

## Mit Juntei parsen { #parse-with-juntei }

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

Mit `-d` liest Juntei Dateien, deren Name aus `Cache`, Ziffern und `.bin` besteht, sowie Dateien aus `bcache`, Ziffern
und `.bmc`. Eine `Cache????.bin`-Datei, die nicht mit der Signatur `RDP8bmp` beginnt, gehört zu einem anderen Programm:
Sie wird in der Zusammenfassung gezählt und übersprungen. Mit `-f` liest Juntei die Datei unabhängig von ihrem Namen.

### Bilder { #images }

`--extract` legt für jede Cachedatei einen Ordner an, `<DIR>/<user>/<cache file>/`. Würden sich zwei Cachedateien einen
Ordner teilen, etwa eine Cachedatei und ihre Kopie aus einer Volumeschattenkopie, wird der zweite Ordner nummeriert
(`Cache0001.bin_2`). Der Ordner enthält:

- `<cache file>_<index>.bmp`: jede Kachel in ihrer eigenen Größe. Die Namen sind dieselben, die bmc-tools verwendet,
  sodass RdpCacheStitcher den Ordner öffnen kann.
- `<cache file>_old_<index>.bmp`: nur bei `.bmc`, das Ende einer früheren Kachel, das hinter einer kleineren übrig
  geblieben ist.
- `<cache file>_collage.png`: alle Kacheln in der Reihenfolge der Datei, 64 pro Zeile (`--collage-width`), jede in einer
  Zelle von 64 × 64, beginnend an deren linker oberer Ecke. Die Kachel mit dem Index *i* steht in Zeile *i* / 64 und
  Spalte *i* % 64, jeweils ab 0 gezählt. Magenta füllt die leeren Teile der Zellen: neben einer schmalen Kachel, unter
  einer niedrigen und dort, wo eine Kachel nicht gelesen werden konnte.

`--collage-only` speichert nur die Collagen. Ein bereits vorhandenes Bild wird ohne `-C` nicht ersetzt, und `Problems`
der zugehörigen Kachel vermerkt dies. Die Cachedateien selbst werden nie ersetzt, auch nicht mit `-C`.

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Bedeutung |
|---|---|
| `SourceFile` | Die Cachedatei. |
| `User` | Der Benutzer, in dessen Profilordner die Cachedatei liegt, ermittelt aus ihrem Pfad. |
| `TileIndex` | Die Nummer der Kachel in der Datei, ab 0 gezählt, wie im Namen ihres Bildes. |
| `Width`, `Height` | Die Größe der Kachel in Pixeln. |
| `SingleColor` | `true`, wenn alle Pixel der Kachel dieselbe Farbe haben. |
| `TileImage` | Mit `--extract`: die BMP-Datei der Kachel. |
| `Collage` | Mit `--extract`: die Collage der Cachedatei. |
| `RemnantImage` | `.bmc` mit `--extract`: das Ende einer früheren Kachel, das im Slot übrig geblieben ist. |
| `Key` | Der 64-Bit-Schlüssel der Kachel im Remotedesktopprotokoll, hexadezimal. |
| `Format`, `FormatVersion` | `bin` mit der Version nach der Signatur (6 ab Windows 10) oder `bmc`. |
| `BitsPerPixel` | 32 bei `.bin`; 8, 16, 24 oder 32 bei `.bmc`. |
| `Compressed`, `Flags` | `.bmc`: ob die Kachel komprimiert war, und die Flags ihres Headers. |
| `Offset` | Wo der Header der Kachel in der Datei beginnt. |
| `Problems` | Beschädigungen: eine Datei, die innerhalb einer Kachel endet, ein Kachelheader, den keine Kachel haben könnte, Daten, die nicht zur Größe der Kachel passen, komprimierte Daten, die sich nicht dekomprimieren lassen, ein Bild, das nicht gespeichert werden konnte. |

JSON und JSONL haben dieselben Felder, mit typisierten Zahlen und booleschen Werten, `null` für Werte, die eine Kachel
nicht hat, und `Problems` als Liste. Beschädigungen, die keine einzelne Kachel betreffen, etwa Daten nach der letzten
Kachel, stehen in den Meldungen des Laufs.

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt `LateralMovement/RdpCache.csv` (`RdpCache.json` bzw. `RdpCache.jsonl` mit
`-t json` bzw. `-t jsonl`), zusammen mit `LateralMovement/RdpCache-logs.txt`, und speichert die Collage jeder
Cachedatei in einem Ordner `LateralMovement/RdpCache/`. Mit `--extract-rdp-tiles` wird zusätzlich jede Kachel
gespeichert; das können für einen einzigen Benutzer Zehntausende Dateien sein. Aus Datenträgerabbildern werden die
Cachedateien kopiert. Die kombinierte Zeitleiste `-j` enthält jede Zeile mit dem `Artifact`-Bezeichner `rdp-cache`.
Die Kacheln haben keine Zeitangaben, daher erzeugt [`super-timeline`](../commands/super-timeline.md) aus ihnen keine
Ereignisse.

## Untersuchungsstrategie { #investigation-strategy }

1. Öffnen Sie die Collagen, und suchen Sie nach Text: Computer- und Benutzernamen, Dateipfaden, Befehlen,
   Konsolenfenstern, Adressleisten von Browsern und Dialogfeldern.
2. Setzen Sie Bildschirme mit RdpCacheStitcher aus einem Ordner von Kacheln zusammen, die mit `--extract` gespeichert
   wurden.
3. Ermitteln Sie die Computer, mit denen sich der Benutzer verbunden hat, aus der Registry des Clients
   (`parse-registry remote-access`), aus `.rdp`-Dateien und aus den Jump Lists von `mstsc.exe`. Suchen Sie auf diesen
   Computern nach den passenden RDP-Anmeldungen und nach Datensätzen der Benutzerzugriffsprotokollierung.
4. Grenzen Sie anhand der `$MFT`-Zeiten der Cachedateien und ihrer Kopien in Volumeschattenkopien ein, wann Kacheln
   geschrieben wurden.

## Weitere Informationen { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: der RDP-Bitmap-Cache (auf Französisch)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — Plugin für den RDP-Cache](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistenter Bitmap-Cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
