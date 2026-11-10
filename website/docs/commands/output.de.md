# Gemeinsame Ausgabemuster { #common-output-patterns }

## Ein Artefakt oder eine Sammlung { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` sucht rekursiv nach Dateien, die zum jeweiligen Befehl passen. Spalten mit dem Quellpfad unterscheiden
Hosts und Artefakte in zusammengeführter Ausgabe.

## CSV speichern { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Einige Parser benötigen ein Verzeichnis, weil sie mehrere CSV-Tabellen ausgeben:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## JSONL streamen { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

Meldungen und Fortschritt gehen an die Standardfehlerausgabe; die Standardausgabe bleibt maschinell auswertbar.

## Bewusst überschreiben { #overwrite-intentionally }

Juntei verweigert standardmäßig das Überschreiben vorhandener Ausgaben. Prüfen Sie das Ziel, sichern Sie frühere
Ergebnisse, auf die es ankommt, und fügen Sie dann `-C` / `--clobber` hinzu:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Stiller Modus und Modus ohne Farbe { #quiet-and-no-color-modes }

Verwenden Sie `-q` für die Automatisierung und `-K`, wenn Farbcodes des Terminals unangebracht sind:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Groß-/Kleinschreibung und Pfade { #case-and-paths }

Setzen Sie Windows-Pfade, die Leerzeichen, `$`, Shell-Metazeichen oder Backslashes enthalten, so in Anführungszeichen, wie es Ihre
Shell erfordert. In Bash-ähnlichen Shells verhindern einfache Anführungszeichen, dass `$MFT` und `$J` als Variablen interpretiert werden:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
