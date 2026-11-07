# Artefaktleitfaden { #artifact-guide }

Windows bewahrt sich überschneidende Spuren im Dateisystem, in der Registry, in Kompatibilitätsdatenbanken,
im Verlauf der Benutzeroberfläche, in der Ressourcenerfassung und in Sicherheitsprodukten. Juntei parst diese Quellen,
doch die Ermittlerin oder der Ermittler muss weiterhin interpretieren, was jede Quelle bedeutet.

## Artefakt nach Fragestellung wählen { #choose-an-artifact-by-question }

| Fragestellung der Untersuchung | Hier beginnen | Untermauern mit |
|---|---|---|
| Wurde diese ausführbare Datei ausgeführt? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), Ereignisprotokolle |
| War diese Datei jemals vorhanden? | [Amcache](amcache.md), [$MFT](mft.md) | [USN-Journal](usn-journal.md), [$I30](i30.md), ShimCache |
| Wann wurde eine Datei erstellt, umbenannt oder gelöscht? | [USN-Journal](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), Defender-Quarantäne |
| Welcher Name oder Zeitstempel existierte vor einer Metadatenänderung? | [$LogFile](logfile.md) | [$MFT](mft.md), [USN-Journal](usn-journal.md), [$I30](i30.md) |
| Wem gehört eine Datei, und wer darf sie ändern? | [$Secure:$SDS](secure-sds.md) mit [$MFT](mft.md) | [SAM](sam.md), Security-Ereignisprotokolle |
| Auf welchem Volume befand sich das Ziel einer Verknüpfung oder Prefetch-Datei? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [USB-Geräte](usb.md) |
| Was hat ein Benutzer geöffnet? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Hat ein Programm eine große Datenmenge übertragen? | [SRUM](srum.md) | Telemetrie von Firewall, Proxy, DNS, Paketmitschnitten oder EDR |
| Welche Clients und Konten haben diesen Server genutzt, und an welchen Tagen? | [Benutzerzugriffsprotokollierung (UAL)](ual.md) | Security-Ereignisprotokolle, SMB- und RDP-Protokolle, Firewall-Protokolle |
| Wurde Persistenz eingerichtet? | [Dienste](services.md), [TaskCache](taskcache.md), [Registry](registry.md) | Ereignisprotokolle, Prefetch, `$MFT` |
| Was hat Defender entfernt? | [Defender-Quarantäne](defender-quarantine.md) | Ereignisprotokolle, `$MFT`, USN-Journal |
| Wurden lokale Konten hinzugefügt oder geändert? | [SAM](sam.md) | Security-Ereignisprotokolle und Transaktionsverlauf der Registry |

## Warnungen vor den Feldern lesen { #read-warnings-before-fields }

Jede Artefaktseite folgt demselben Ablauf:

1. **Was es ist** und welche Fragen es beantworten kann.
2. **Wo es zu finden ist** und wie sich die Sicherung darauf auswirkt.
3. **Was es beweist – und was nicht.**
4. **Wie es** mit Juntei geparst wird.
5. **Wie jedes Ausgabefeld zu lesen ist.**
6. **Wie es** mit unabhängigen Beweisen korreliert wird.
7. **Weitere Informationen** für tiefergehende Recherche.

!!! danger "Setzen Sie einen Parserbefund nicht mit Bösartigkeit gleich"
    Befunde von Juntei kennzeichnen Muster, die eine Prüfung wert sind, etwa den Namen einer Systembinärdatei außerhalb von
    `System32`. Installationsprogramme, Administratoren, Kompatibilitätsebenen und Softwareupdates können
    ungewöhnliche Datensätze erzeugen. Validieren Sie anhand des Kontexts und unabhängiger Beweise.
