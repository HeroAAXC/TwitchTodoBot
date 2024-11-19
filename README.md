# Twitch ToDo Bot

![Admin Panel](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/f31658cc7d2fd32618851a7582806333ae2a0287/AdminPanel.png "Twitch Todo Bot Admin Panel")

If you have questions or issues, feel free to contact me (via GitHub issue or Discord).

![OBS Gui](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/f31658cc7d2fd32618851a7582806333ae2a0287/todo_chat.png "OBS Gui")

## Installation

Download the program (`TodoBot.exe`) from the releases and copy it to the desired location.

Important: The files `credentials.json` and `channels.csv` MUST be present.

To start: Double-click the file. You can stop the bot via the admin panel; see **Easy Configuration**.

## Easy Configuration
Only the credentials file needs to be edited manually.  
The rest is accessible via the WebUI at [localhost on port 300](http://localhost:3000/) (if the bot is running, just click the link).

## Integration with OBS
To display the ToDo panel in OBS, follow these illustrated step-by-step instructions:

![Create Source](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source1.png "Create a new source")
Create a new source.

![Add Browser Source](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source2.png "Add a browser source")
Add a browser source.

![Create Source](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source3.png "Create and name the new source")
Create and name the new source. Make sure the "Visible" checkbox is checked.

![Configure in OBS](https://raw.githubusercontent.com/HeroAAXC/TwitchTodoBotImages/refs/heads/main/browser_source4.png "Configure within OBS")
Enter the URL `http://localhost:3000/todos`. Adjust width and height as needed.  
If the text inside the window is too large, increase the width and height here and scale the entire panel down in the main window (drag and drop).  
Make sure _NOT_ to check the "Local file" box.  
Click OK. If your bot is already running, you'll now see the ToDo panel.

## Configuration / Config Files

### channels.csv

This file lists all channels to be monitored (separated by _line breaks_). Make sure to use lowercase channel logins (UTF-8 encoded).  
It can be edited with any text editor.

#### Example

```csv
thebiggreekschach
vanimio
lotnisko7
```

### credentials.json
This file stores the static login credentials unencrypted in JSON format.
You need to obtain these credentials yourself from Twitch.

#### Example
```json
{
  "login": "mynicetodobot",
  "token": "SomeChars"
}
```

### mods.json
This file specifies all mods (in lowercase and UTF-8 encoded) who can reset the ToDo bot or save data to the disk, in JSON format.
It can be edited with any text editor.

#### Example
```json
["vanimio", "broncosorestore"]
```

### todos.json
This file stores all ToDos. This file should not be edited manually.
