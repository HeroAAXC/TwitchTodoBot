# Twitch ToDo Bot

Bei Fragen einfach an mich wenden (per github issue oder Discord)

## Installation

Lade das Programm (TodoBot.exe) unter releases herunter und kopiere as an den gewünschten Ort.

Wichtig: die Dateien credentials.json und channels.csv MÜSSEN vorhanden sein.

Zum starten: Erstelle im selben Verzeichnis, in das du die .exe kopiert hast eine Datei namens "Start.bash".

In diese kommt folgender Inhalt:
```bash
./TodoBot.exe
```

## Konfiguration / config files

### channels.csv

Alle Channels, denen zugehört werden soll, werden hier aufgelistet (durch _Absätze_ getrennt). (wichtig die channel logins, also alles kleingeschrieben (und UTF-8))
Diese können einfach mit dem editor deines Vertrauens verändert werden.

#### Beispiel

```csv
thebiggreekschach
vanimio
lotnisko7
```

### credentials.json

Hier werden die statischen Anmeldedaten _unverschlüsselt_ gespeichert im json Format.
Diese musst du selbst bei Twitch holen.

#### Beispiel

```json
{
  "login": "meintollertodobot",
  "token": "3247z89cefjnkernf44gmnk5ozt590"
}
```

### mods.json

Hier werden alle mods (in Kleinbuchstaben und UTF-8 codiert), die den todo-bot resetten können oder die Daten auf der Festplatte abspeichern können im json Format angegeben.
Diese können einfach mit dem editor deines Vertrauens verändert werden.

#### Beispiel

```json
["vanimio", "broncosorestore"]
```

### todos.json

hier werden alle todos hinterlegt, die Datei sollte nicht verändert werden.
