# Beweise sichern { #collecting-evidence }

Juntei parst bereits von Windows gesicherte Dateien. Verwenden Sie ein vertrauenswürdiges Erfassungswerkzeug und bewahren Sie nach Möglichkeit die ursprünglichen Metadaten. VHDX- und E01-Disk-Images lassen sich ohne Extraktion parsen: siehe [Disk-Images einbinden](../commands/mount.md).

## Gängige Speicherorte { #common-locations }

| Artefakt | Typischer Speicherort |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS-Metadatendatei des Volumes `C:\$MFT` |
| `$LogFile` | NTFS-Metadatendatei des Volumes `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN-Journal | NTFS-Metadaten-Stream `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Benutzerzugriffsprotokollierung (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer und Edge Legacy (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`, `%USERPROFILE%\Documents\Outlook Files\*.pst` (`%LOCALAPPDATA%\Microsoft\Outlook\*.pst` vor Outlook 2010), `%APPDATA%\Microsoft\Templates\*.oft` sowie `.msg`-Dateien, wo immer sie gespeichert wurden |
| Webbrowser | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` und die Profilordner der anderen Browser |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` und weitere Benutzer-/Systemspeicherorte |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` und `CustomDestinations` |
| ShimCache | `SYSTEM`-Hive: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender-Quarantäne | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Maschinen-Registry | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Benutzer-Registry | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Dateien, die zusammen gesichert werden müssen { #files-that-must-travel-together }

- Sichern Sie Registry-Hives zusammen mit den benachbarten Dateien `.LOG`, `.LOG1` und `.LOG2`. Juntei kann bei „dirty“ Hives
  die Protokolle für Registry-Befehle und ShimCache einspielen.
- Sichern Sie `SRUDB.dat` von SRUM konsistent. Juntei meldet derzeit eine „dirty“ ESE-Datenbank, spielt aber
  `SRU*.log` nicht ein; Datensätze, die nur in diesen Protokollen existieren, sind nicht verfügbar.
- Sichern Sie `$MFT` nach Möglichkeit zusammen mit `$Extend/$J`. Der USN-Datensatz speichert Dateireferenzen auf das übergeordnete Verzeichnis,
  keinen vollständigen Pfad; die MFT verbessert die Pfadauflösung.
- Sichern Sie `$LogFile` zeitlich möglichst nah zusammen mit der `$MFT` desselben Volumes. `$LogFile` wird
  schnell überschrieben; bei einer späteren Sicherung können die früheren Redo/Undo-Datensätze vollständig verloren gehen.
- Sichern Sie SRUM zusammen mit dem passenden `SOFTWARE`-Hive, um Konto- und WLAN-Profilnamen aufzulösen.
- Sichern Sie das gesamte Verzeichnis `Sum` der Benutzerzugriffsprotokollierung: `SystemIdentity.mdb` benennt die Rollen
  und die jährlichen `{GUID}.mdb`-Datenbanken. Auch deren Transaktionsprotokolle `Svc*.log` spielt Juntei nicht ein.
- Sichern Sie Browserdatenbanken zusammen mit den `-wal`-Dateien daneben (`places.sqlite-wal`,
  `cookies.sqlite-wal`): Die neuesten Änderungen und die älteren Seitenversionen, aus denen gelöschte Einträge
  wiederhergestellt werden, sind nur dort vorhanden, bis der Browser einen Checkpoint ausführt.
- Sichern Sie den gesamten Ordner `WebCache`: Windows hält `WebCacheV01.dat` geöffnet, solange der Benutzer
  angemeldet ist, und die neuesten Änderungen stehen nur in den Transaktionsprotokollen `V01*.log`, die Juntei nicht
  einspielt.
- Sichern Sie die OST- und PST-Dateien von Outlook bei geschlossenem Outlook oder aus einem Datenträgerabbild bzw. einer
  Volumeschattenkopie: Outlook hält sie geöffnet, solange es läuft. Suchen Sie PSTs auch außerhalb der Standardordner:
  Benutzer verschieben Archive in andere Ordner, auf andere Laufwerke und auf Dateifreigaben.
- Sichern Sie den gesamten Defender-Verzeichnisbaum `Quarantine`, damit Entries, ResourceData und Resources einander zugeordnet werden können.

## Hinweise zu Live-Systemen { #live-system-cautions }

Geöffnete Dateien können sich während eines Kopiervorgangs ändern. Registry-Hives und ESE-Datenbanken können intern „dirty“ sein.
ShimCache schreibt seine neuesten Einträge aus dem Arbeitsspeicher möglicherweise erst beim Herunterfahren. Ein forensisches Sicherungswerkzeug,
das Volume-Snapshots oder Rohzugriff verwendet, ist einer gewöhnlichen Dateikopie vorzuziehen.

## Sicherheit { #safety }

Beweise können Malware, Exploit-Samples, bösartige LNK-Dateien und Tabellenkalkulationsformeln als Text enthalten.
Analysieren Sie auf einem isolierten System, führen Sie wiederhergestellte Streams nicht aus und verwenden Sie `--extract` nur, wenn Sie
bewusst mit gefährlichen Inhalten umgehen wollen. Bewahren Sie die unveränderte Quelle separat auf.
