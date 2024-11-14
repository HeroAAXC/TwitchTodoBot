# Twitch ToDo Bot

Bei Fragen einfach an mich wenden (per github issue oder Discord)

## Config Files

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

Hier werden alle mods (in kleinbuchstaben und UTF-8 codiert), die den todo-bot resetten können oder die Daten auf der Festplatte abspeichern können im json Format angegeben.
Diese können einfach mit dem editor deines Vertrauens verändert werden.

#### Beispiel

```json
["vanimio", "broncosorestore"]
```

### todos.json

hier werden alle todos hinterlegt, die Datei sollte nicht verändert werden.
