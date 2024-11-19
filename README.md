# Twitch ToDo Bot

![Admin Panel](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/f31658cc7d2fd32618851a7582806333ae2a0287/AdminPanel.png "Twitch Todo Bot Admin Panel")

Bei Fragen oder Problemen einfach an mich wenden (per github issue oder Discord).

![OBS Gui](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/f31658cc7d2fd32618851a7582806333ae2a0287/todo_chat.png "OBS Gui")

## Installation

Lade das Programm (TodoBot.exe) unter releases herunter und kopiere as an den gewünschten Ort.

Wichtig: die Dateien credentials.json und channels.csv MÜSSEN vorhanden sein.

Zum starten: Doppelklicke die Datei. Stoppen kannst du den Bot im Admin Panel, siehe Einfache Konfiguration.


## Einfache Konfiguration
Lediglich die credentials file muss über Datein bearbeitet werden.
Der Rest ist über das WebUI erreichbar unter [dem localhost auf Port 300](http://localhost:3000/) erreichbar (wenn der Bot aktiv ist, einfach klicken).

## Einbindung in OBS
Um das Todo Panel in OBS anzuzeigen, musst du folgende bebilderte Schritt-für-Schritt-Anleitung beachten:

![Quelle öffnen](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source1.png "erstelle eine neue Quelle")
Erstelle eine neue Quelle

![Browser Quelle hinzufügen](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source2.png "füge eine Browser Quelle hinzu")
Füge eine Browser Quelle hinzu.

![Erselle die Quelle](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source3.png "erstelle und benenne die neue Quelle")
Erstelle und benenne die neue Quelle. Achte darauf, dass ein Haken bei "Quelle sichtbar" ist.

![Konfiguration innerhalb OBS](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source4.png "Konfiguration innerhalb OBS")
Gib bei Url http://localhost:3000/todos ein. Breite und Höhe kannst du nach belieben anpassen. Wenn die Schrift innerhalb des Fensters zu groß ist, musst du die Höhe und Breite in diesem Fenster vergrößern und das gesamte Panel im Hauptfenster kleiner skalieren (drag and drop).
Achte darauf, dass _KEIN_ Haken bei "Aus Datei" ist.
Klicke anschließend auf Okay. Wenn dein Bot bereits läuft siehst du nun das todo Panel.


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
