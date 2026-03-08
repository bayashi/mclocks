# mclocks

Die Desktop-Uhr-Anwendung für mehrere Zeitzonen🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

Uhrenbezogene Funktionen:

* 🕐 Textuhr für mehrere Zeitzonen
* ⏱️ Timer
* ⏳ Countdown-Timer
* 🔄 Epoch-Zeit- und Datum-/Uhrzeit-Konverter

Die Zeit wartet auf niemanden:

* 📝 Haftnotiz

Ein Entwickler ist nie ohne Uhr:

* 🔀 Einfacher Textkonverter
    * wie das einfache Erstellen von SQL `IN`-Klauseln
* 🌐 Webserver
    * stellt statische Dateien bereit
        * rendert Markdown im Rich-Format
        * Drag-and-drop-basierter Content-Viewer
    * Request- und Response-Dump-Server
    * langsame Endpunkte zum Debuggen
    * Dateien im Editor über GitHub-URLs öffnen

🔔 HINWEIS: `mclocks` benötigt keine Internetverbindung — alles läuft zu 100% lokal.

## 📦 Download

Von https://github.com/bayashi/mclocks/releases

### Windows

Für Windows können Sie die Installationsdatei `.msi` oder die ausführbare Datei `.exe` herunterladen.

### macOS

Für macOS können Sie die `.dmg`-Datei zur Installation herunterladen.

(Die Tastenkombinationen in diesem Dokument gelten für Windows. Wenn Sie macOS verwenden, interpretieren Sie diese bitte entsprechend und ersetzen Sie Tasten wie `Ctrl` durch `Ctrl + Command` und `Alt` durch `Option`.)

## ⚙️ config.json

Die Datei `config.json` ermöglicht es Ihnen, die Uhren nach Ihren Vorlieben zu konfigurieren.

Die Datei `config.json` sollte sich in den folgenden Verzeichnissen befinden:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

Wenn Sie `mclocks` starten, drücken Sie `Ctrl + o`, um Ihre `config.json`-Datei zu bearbeiten.

### Beispiel für config.json

Die Datei `config.json` sollte wie unten gezeigt im JSON-Format formatiert sein.

    {
      "clocks": [
        { "name": "UTC", "timezone": "UTC" }
      ],
      "format": "MM-DD ddd HH:mm",
      "locale": "en",
      "color": "#fff",
      "font": "Courier, monospace",
      "size": 14,
      "margin": "1.65em",
      "forefront": false
    }

## 🔧 Die Felder von config.json

#### clocks

Das Feld `clocks` ist ein Array von Objekten, die jeweils die Eigenschaften `name` und `timezone` enthalten. Beide sollten Zeichenketten sein. Standardmäßig sind beide `UTC`.

* `name` ist eine Bezeichnung, die für die Uhr angezeigt wird.
* Zur Auswahl der Zeitzonen siehe diese [Liste der Zeitzonen](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).

Hier ist ein Beispiel eines `clocks`-Arrays für drei Zeitzonen.

    {
      "clocks": [
        { "name": "Tokyo", "timezone": "Asia/Tokyo" },
        { "name": "UTC", "timezone": "UTC" },
        { "name": "SF", "timezone": "America/Los_Angeles" }
      ],
      "format": "MM-DD ddd HH:mm",
      ...snip...
    }

#### format

Zeichenkette: `MM-DD ddd HH:mm`

Das Feld `format` ist eine Datum-/Uhrzeit-Formatzeichenkette zur Anzeige der Uhr. Um ein benutzerdefiniertes Format zu erstellen, siehe [diese Formatierungsanleitung](https://momentjs.com/docs/#/parsing/string-format/).

#### format2

Zeichenkette: `""`

Das Feld `format2` ist identisch mit `format`. Sie werden mit der Taste `Ctrl + f` untereinander gewechselt. Das Feld `format2` ist optional.

#### locale

Zeichenkette: `en`

Das Feld `locale` bestimmt die Spracheinstellungen für die Anzeige von Datum und Uhrzeit. Eine [Liste der unterstützten Locales finden Sie hier](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

Zeichenkette: `#fff`

Das Feld `color` definiert die Farbe des Datum-/Uhrzeittextes. Sie können benannte Farben, RGB-Hexadezimalwerte, RGB-Werte (z.B. `RGB(255, 0, 0)`) oder jeden gültigen CSS-Farbwert verwenden.

#### font

Zeichenkette: `Courier, monospace`

`font` ist der Schriftname zur Anzeige von Datum und Uhrzeit. Es sollte eine Festbreitenschrift sein. Wenn Sie eine Proportionalschrift einstellen, kann Ihr mclocks einen unerwünschten Wackeleffekt haben.

#### size

Zahl | Zeichenkette: 14

`size` ist die Zeichengröße für Datum und Uhrzeit in Pixeln. Es kann auch als Zeichenkette mit Einheit angegeben werden (z.B. `"125%"`, `"1.5em"`).

#### margin

Zeichenkette: `1.65em`

Das Feld `margin` bestimmt den Abstand zwischen den Uhren.

#### forefront

Boolesch: `false`

Wenn das Feld `forefront` auf `true` gesetzt wird, wird die mclocks-Anwendung immer über anderen Anwendungsfenstern angezeigt.

## ⏳ Countdown-Uhr

Durch die Konfiguration der `clock` wie unten gezeigt wird sie als Countdown-Uhr bis zum angegebenen `target`-Datum angezeigt.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

Die obige Countdown-`clock` wird wie folgt angezeigt:

    WAC Tokyo D-159 12:34:56

Dies zeigt an, dass noch 159 Tage, 12 Stunden, 34 Minuten und 56 Sekunden bis zum 13. September 2025 verbleiben.

### Countdown-Format-Variablen

Der Text des `countdown`-Feldes akzeptiert folgende Template-Variablen:

* `%TG`: Zieldatum-/-uhrzeit-Zeichenkette
* `%D`: Verbleibende Tage bis zum Zieldatum
* `%H`: Verbleibende Zeit in Stunden bis zum Zieldatum
* `%h`: Die Stunde (hh) der verbleibenden Zeit (hh:mm:ss)
* `%M`: Verbleibende Zeit in Minuten bis zum Zieldatum
* `%m`: Die Minute (mm) der verbleibenden Zeit (hh:mm:ss)
* `%S`: Verbleibende Zeit in Sekunden bis zum Zieldatum
* `%s`: Die Sekunde (ss) der verbleibenden Zeit (hh:mm:ss)

## ⏱️ Einfacher Timer

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

Klicken Sie auf das `mclocks`-Anwendungsfenster und drücken Sie dann `Ctrl + 1`, um einen 1-Minuten-Timer zu starten. Drücken Sie `Ctrl + Alt + 1`, um einen 10-Minuten-Timer zu starten. Andere Zifferntasten funktionieren ebenso. Es können bis zu 5 Timer gleichzeitig gestartet werden.

`Ctrl + p` zum Pausieren / Fortsetzen der Timer.

`Ctrl + 0` zum Löschen des ältesten Timers. `Ctrl + Alt + 0` zum Löschen des neuesten Timers.

🔔 HINWEIS: Countdown-Uhr und einfacher Timer senden standardmäßig eine Benachrichtigung, wenn der Timer abgelaufen ist. Wenn Sie keine Benachrichtigungen benötigen, setzen Sie `withoutNotification: true` in `config.json`.

## 🔢 Epoch-Zeit anzeigen

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

Klicken Sie auf das `mclocks`-Anwendungsfenster und drücken Sie dann `Ctrl + e`, um die Anzeige der Epoch-Zeit umzuschalten.

## 🔄 Zwischen Datum/Uhrzeit und Epoch-Zeit konvertieren

Klicken Sie auf das `mclocks`-Anwendungsfenster und fügen Sie dann ein Datum/eine Uhrzeit oder eine Epoch-Zeit ein. Ein Dialog erscheint mit den Konvertierungsergebnissen. Sie können die Ergebnisse in die Zwischenablage kopieren. Wenn Sie nicht kopieren möchten, drücken Sie `[No]`, um den Dialog zu schließen.

Beim Einfügen mit `Ctrl + v` wird der Wert (Epoch-Zeit) als Sekunden behandelt. Bei `Ctrl + Alt + v` als Millisekunden, bei `Ctrl + Alt + Shift + V` als Mikrosekunden und bei `Ctrl + Alt + Shift + N + V` als Nanosekunden und entsprechend konvertiert.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

Wenn eingefügte Datum-/Uhrzeitwerte keine Zeitzoneninformation enthalten, werden sie standardmäßig als lokale Zeitzone behandelt. Um sie als bestimmte Zeitzone zu behandeln, setzen Sie die Zeitzone in der convtz-Option.

    "convtz": "UTC"

## 🔀 Textkonvertierungsfunktion

Klicken Sie auf das `mclocks`-Anwendungsfenster und verwenden Sie dann die folgenden Tastenkombinationen, um den Zwischenablagetext zu verarbeiten und im Editor zu öffnen:

* `Ctrl + i`: Umschließt jede Zeile des Zwischenablagetextes mit Anführungszeichen und fügt ein Komma am Ende hinzu (außer der letzten Zeile)
* `Ctrl + Shift + i`: Fügt ein Komma am Ende jeder Zeile hinzu (ohne Anführungszeichen) für INT-Listen-IN-Bedingungen (außer der letzten Zeile)

Leerzeilen werden bei allen Operationen beibehalten.

(Diese Textkonvertierungsfunktion hat nichts mit Uhren oder Zeit zu tun, aber Softwareentwickler könnten sie nützlich finden! 😊)

## ⌨️ Tastenkombinationen

### Hilfe anzeigen

`F1` (Windows) oder `Cmd + Shift + /` (macOS) zum Öffnen der Hilfeseite (dieses README) im Browser

### Konfiguration, Anzeigeformate

| Tastenkombination | Beschreibung |
|----------|-------------|
| `Ctrl + o` | `config.json`-Datei im Editor öffnen |
| `Ctrl + f` | Zwischen `format` und `format2` wechseln (wenn `format2` in `config.json` definiert ist) |
| `Ctrl + e` oder `Ctrl + u` | Epoch-Zeit-Anzeige umschalten |

### Timer

| Tastenkombination | Beschreibung |
|----------|-------------|
| `Ctrl + 1` bis `Ctrl + 9` | Timer starten (1 Minute × Zifferntaste) |
| `Ctrl + Alt + 1` bis `Ctrl + Alt + 9` | Timer starten (10 Minuten × Zifferntaste) |
| `Ctrl + p` | Alle Timer pausieren / fortsetzen |
| `Ctrl + 0` | Ältesten Timer (ganz links) löschen |
| `Ctrl + Alt + 0` | Neuesten Timer (ganz rechts) löschen |

### Haftnotiz

| Tastenkombination | Beschreibung |
|----------|-------------|
| `Ctrl + s` | Neue Haftnotiz aus Zwischenablagetext erstellen |

### Zwischenablage-Datum/Uhrzeit-Operationen

| Tastenkombination | Beschreibung |
|----------|-------------|
| `Ctrl + c` | Aktuellen mclocks-Text in die Zwischenablage kopieren |
| `Ctrl + v` | Zwischenablageinhalt konvertieren (Epoch-Zeit als Sekunden oder Datum/Uhrzeit) |
| `Ctrl + Alt + v` | Zwischenablageinhalt konvertieren (Epoch-Zeit als Millisekunden) |
| `Ctrl + Alt + Shift + V` | Zwischenablageinhalt konvertieren (Epoch-Zeit als Mikrosekunden) |
| `Ctrl + Alt + Shift + N + V` | Zwischenablageinhalt konvertieren (Epoch-Zeit als Nanosekunden) |

### Textkonvertierung

| Tastenkombination | Beschreibung |
|----------|-------------|
| `Ctrl + i` | Jede Zeile der Zwischenablage mit Anführungszeichen umschließen, Komma am Ende hinzufügen und im Editor öffnen (außer der letzten Zeile) |
| `Ctrl + Shift + i` | Komma am Ende jeder Zeile hinzufügen (ohne Anführungszeichen) für INT-Listen-IN-Bedingungen und im Editor öffnen (außer der letzten Zeile) |

## 📝 Haftnotiz

Klicken Sie auf das `mclocks`-Anwendungsfenster und drücken Sie dann `Ctrl + s`, um eine Haftnotiz aus dem Zwischenablagetext zu erstellen. Ein kleines schwebendes Fenster öffnet sich mit dem Inhalt der Zwischenablage.

Jede Haftnotiz hat:

* **Umschalttaste** (`▸` / `▾`): Notiz erweitern oder zusammenklappen. Im zusammengeklappten Modus wird nur eine Zeile angezeigt.
* **Kopiertaste** (`⧉`): Notiztext in die Zwischenablage kopieren.
* **Vordergrundtaste** (`⊤` / `⊥`): Umschalten, ob die Notiz über anderen Fenstern bleibt. Diese Einstellung wird pro Haftnotiz gespeichert.
* **Schließtaste** (`✖`): Haftnotiz löschen und ihr Fenster schließen.
* **Textbereich**: Notizinhalt frei bearbeiten. Änderungen werden automatisch gespeichert.
* **Größenänderungsgriff**: Ziehen Sie die untere rechte Ecke, um die Notiz im erweiterten Zustand zu vergrößern/verkleinern.

Haftnotizen erben die Einstellungen `font`, `size`, `color` und `forefront` aus `config.json`. Die Vordergrund-Einstellung kann pro Haftnotiz über die Vordergrundtaste überschrieben werden; wenn nicht überschrieben, wird der Wert aus `config.json` verwendet. Position, Größe, Öffnungs-/Schließungszustand und Vordergrund-Überschreibung werden persistent gespeichert, und alle Notizen werden beim Neustart von `mclocks` automatisch wiederhergestellt.

🔔 HINWEIS: Unter macOS werden die Fensterpositionen von Haftnotizen nur beim Beenden der Anwendung gespeichert. Unter Windows werden Positionen automatisch beim Verschieben oder Größenändern der Fenster gespeichert.

Die maximale Textgröße pro Haftnotiz beträgt 128 KB.

## 🌐 Webserver

`mclocks` kann statische Dateien über einen integrierten Webserver bereitstellen. Diese Funktion ermöglicht es Ihnen, Ihre Code-Snippets einfach in einem Browser anzuzeigen. Fügen Sie ein `web`-Feld zu Ihrer `config.json` hinzu:

    {
      "web": {
        "root": "/path/to/your/webroot",
        "dump": true,
        "slow": true,
        "status": true,
        "content": {
          "markdown": {
            "allowRawHTML": false
          }
        },
        "editor": {
          "reposDir": "/path/to/your/repos"
        }
      }
    }

* `root`: Pfad zum Verzeichnis mit den bereitzustellenden Dateien (erforderlich)
* `port`: Portnummer zum Lauschen (Standard: 3030)
* `open_browser_at_start`: Wenn auf `true` gesetzt, wird beim Start von `mclocks` automatisch die Webserver-URL im Standardbrowser geöffnet (Standard: `false`)
* `dump`: Wenn auf `true` gesetzt, aktiviert den `/dump`-Endpunkt, der Anforderungsdetails als JSON zurückgibt (Standard: `false`)
* `slow`: Wenn auf `true` gesetzt, aktiviert den `/slow`-Endpunkt, der die Antwort verzögert (Standard: `false`)
* `status`: Wenn auf `true` gesetzt, aktiviert den `/status/{code}`-Endpunkt, der beliebige HTTP-Statuscodes zurückgibt (Standard: `false`)
* `content.markdown.allowRawHTML`: Wenn auf `true` gesetzt, wird rohes HTML beim Markdown-Rendering zugelassen; bei `false` wird rohes HTML in Markdown als Text escaped (Standard: `false`)
* `editor`: Wenn gesetzt und `reposDir` enthält, aktiviert den `/editor`-Endpunkt, der lokale Dateien in Ihrem Editor über GitHub-URLs im Browser öffnet (Standard: nicht gesetzt)

Wenn das `web`-Feld in Ihrer `config.json` konfiguriert ist, startet der Webserver automatisch beim Start von `mclocks`. Greifen Sie auf Dateien unter `http://127.0.0.1:3030` zu. Der Webserver lauscht nur auf `127.0.0.1` (localhost) und ist daher nur von Ihrem lokalen Rechner aus erreichbar.

### Unterstützte Dateitypen

Der Webserver unterstützt die folgenden Dateitypen:

* Text: `html`, `css`, `js`, `json`, `md`, `txt`
* Bilder: `png`, `jpg`, `jpeg`, `gif`, `svg`, `ico`

### Drag-and-drop-basierter Content-Viewer

Zusätzlich zur statischen Dateibereitstellung enthält der Webserver auch einen Drag-and-drop-basierten Content-Viewer-Workflow: Wenn Sie eine Datei oder ein Verzeichnis auf das mclocks-Uhrfenster ziehen und ablegen, kann es über temporäre lokale URLs geöffnet und angezeigt werden.
Diese temporären URLs werden verworfen, wenn mclocks beendet wird.

### /dump-Endpunkt

Wenn `dump: true` in der `web`-Konfiguration gesetzt ist, stellt der Webserver einen `/dump`-Endpunkt bereit, der Anforderungsdetails als JSON zurückgibt.

Der Endpunkt antwortet mit einem JSON-Objekt, das Folgendes enthält:
* `method`: HTTP-Methode (z.B. "GET", "POST")
* `path`: Anforderungspfad nach `/dump/` (z.B. "/test" für `/dump/test`)
* `query`: Abfrageparameter als Array von Schlüssel-Wert-Objekten (z.B. `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: Anforderungsheader als Array von Schlüssel-Wert-Objekten (z.B. `[{"Content-Type": "application/json"}]`)
* `body`: Anforderungskörper als Zeichenkette (falls vorhanden)
* `parsed_body`: Geparster JSON-Objekt, wenn der Content-Type JSON angibt, oder Fehlermeldungs-Zeichenkette, wenn das Parsen fehlschlägt

Greifen Sie auf den Dump-Endpunkt unter `http://127.0.0.1:3030/dump` oder einem beliebigen Pfad unter `/dump/` zu (z.B. `/dump/test?key=value`).

### /slow-Endpunkt

Wenn `slow: true` in der `web`-Konfiguration gesetzt ist, stellt der Webserver einen `/slow`-Endpunkt bereit, der die Antwort verzögert, bevor er 200 OK zurückgibt.

Der Endpunkt ist über jede HTTP-Methode (GET, POST usw.) zugänglich und unterstützt die folgenden Pfade:

* `/slow`: Wartet 30 Sekunden (Standard) und gibt 200 OK zurück
* `/slow/120`: Wartet 120 Sekunden (oder eine beliebige angegebene Sekundenzahl) und gibt 200 OK zurück

Der maximal zulässige Wert beträgt 901 Sekunden (15 Minuten + 1 Sekunde). Anfragen, die dieses Limit überschreiten, geben einen 400 Bad Request-Fehler zurück.

Dieser Endpunkt ist nützlich zum Testen von Timeout-Verhalten, Verbindungsbehandlung oder zur Simulation langsamer Netzwerkbedingungen.

Wenn ein ungültiger Sekundenparameter angegeben wird (z.B. `/slow/abc`), gibt der Endpunkt einen 400 Bad Request-Fehler zurück.

### /status-Endpunkt

Wenn `status: true` in der `web`-Konfiguration gesetzt ist, stellt der Webserver einen `/status/{code}`-Endpunkt bereit, der beliebige in RFC-Standards definierte HTTP-Statuscodes (100-599) zurückgibt.

Der Endpunkt gibt eine Klartext-Antwort mit dem Statuscode und der entsprechenden Phrase zurück, zusammen mit entsprechenden Headern gemäß der HTTP-Spezifikation.

**Beispiele:**
* `http://127.0.0.1:3030/status/200` - gibt 200 OK zurück
* `http://127.0.0.1:3030/status/404` - gibt 404 Not Found zurück
* `http://127.0.0.1:3030/status/500` - gibt 500 Internal Server Error zurück
* `http://127.0.0.1:3030/status/418` - gibt 418 I'm a teapot zurück (mit spezieller Nachricht)
* `http://127.0.0.1:3030/status/301` - gibt 301 Moved Permanently zurück (mit Location-Header)

**Statusspezifische Header:**

Der Endpunkt fügt automatisch entsprechende Header für bestimmte Statuscodes hinzu:

* **3xx Weiterleitung** (301, 302, 303, 305, 307, 308): Fügt `Location`-Header hinzu
* **401 Unauthorized**: Fügt `WWW-Authenticate`-Header hinzu
* **405 Method Not Allowed**: Fügt `Allow`-Header hinzu
* **407 Proxy Authentication Required**: Fügt `Proxy-Authenticate`-Header hinzu
* **416 Range Not Satisfiable**: Fügt `Content-Range`-Header hinzu
* **426 Upgrade Required**: Fügt `Upgrade`-Header hinzu
* **429 Too Many Requests**: Fügt `Retry-After`-Header hinzu (60 Sekunden)
* **503 Service Unavailable**: Fügt `Retry-After`-Header hinzu (60 Sekunden)
* **511 Network Authentication Required**: Fügt `WWW-Authenticate`-Header hinzu

**Behandlung des Antwortkörpers:**

* **204 No Content** und **304 Not Modified**: Gibt leeren Antwortkörper zurück (gemäß HTTP-Spezifikation)
* **418 I'm a teapot**: Gibt spezielle Nachricht "I'm a teapot" statt der Standard-Statusphrase zurück
* **Alle anderen Statuscodes**: Gibt Klartext im Format `{code} {phrase}` zurück (z.B. "404 Not Found")

Dieser Endpunkt ist nützlich zum Testen, wie Ihre Anwendungen verschiedene HTTP-Statuscodes, Fehlerbehandlung, Weiterleitungen, Authentifizierungsanforderungen und Rate-Limiting-Szenarien behandeln.

### /editor-Endpunkt

Wenn `web.editor.reposDir` in der Konfigurationsdatei gesetzt ist, stellt der Webserver einen `/editor`-Endpunkt bereit, der es Ihnen ermöglicht, lokale Dateien direkt über GitHub-URLs im Browser in Ihrem Editor zu öffnen.

**Konfiguration:**

Fügen Sie Folgendes zu Ihrer `web`-Konfiguration hinzu:

    {
      "web": {
        "root": "/path/to/your/webroot",
        "editor": {
          "reposDir": "~/repos",
          "includeHost": false,
          "command": "code",
          "args": ["-g", "{file}:{line}"]
        }
      }
    }

* `reposDir`: Pfad zu Ihrem lokalen Repository-Verzeichnis. Unterstützt `~` für Home-Verzeichnis-Erweiterung (z.B. `"~/repos"` unter macOS oder `"C:/Users/username/repos"` unter Windows). Dieses Verzeichnis muss existieren.
* `includeHost`: Wenn `true`, enthält die lokale Pfadauflösung den ursprünglichen Host als Verzeichnis (z.B. `{reposDir}/{host}/{owner}/{repo}/...`). Wenn `false`, wird zu `{reposDir}/{owner}/{repo}/...` aufgelöst (Standard: `false`).
* `command`: Befehlsname oder Pfad zur ausführbaren Datei Ihres Editors (Standard: `code`)
* `args`: Argumentvorlagen-Array. Verwenden Sie die Platzhalter `{file}` und `{line}`. Wenn `#L...` in der URL nicht vorhanden ist, verwendet `{line}` den Wert 1.

**Funktionsweise:**

1. Wenn Sie über den `/editor`-Endpunkt auf eine GitHub-Datei-URL zugreifen, wird der GitHub-Pfad in einen lokalen Dateipfad konvertiert
2. Der lokale Dateipfad wird wie folgt erstellt: `{reposDir}/{owner}/{repository_name}/{file_path}`
3. Wenn die Datei existiert, wird sie in Ihrem Editor an der angegebenen Zeilennummer mit dem konfigurierten Befehl und den Argumenten geöffnet (Standard: `code -g {local_file_path}:{line_number}`)
4. Wenn die Datei nicht existiert, wird eine Fehlerseite mit einem Link zum Klonen des Repositorys angezeigt

**Bookmarklet:**

Erstellen Sie ein Bookmarklet, um GitHub-Dateien schnell in Ihrem lokalen Editor zu öffnen. Ersetzen Sie `3030` durch Ihre konfigurierte Portnummer:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**Zeilennummern-Unterstützung:**

Sie können eine Zeilennummer über das Hash-Fragment in der URL angeben:
* `https://github.com/username/repo/blob/main/file.rs#L123` → Öffnet an Zeile 123

**Fehlerbehandlung:**

* Wenn die Datei lokal nicht existiert, bleibt der Tab geöffnet und zeigt eine Fehlermeldung mit einem Link zum Klonen des Repositorys von GitHub
* Wenn die Datei erfolgreich geöffnet wird, schließt sich der Tab automatisch
* Wenn `web.editor.reposDir` nicht konfiguriert ist oder nicht existiert, ist der `/editor`-Endpunkt nicht aktiviert (und Sie erhalten einen 404-Fehler)

**Beispiel:**

1. Sie betrachten eine Datei auf GitHub: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. Klicken Sie auf das Bookmarklet oder navigieren Sie manuell zu: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. Wenn `~/repos/mclocks/src/app.js` lokal existiert, öffnet VS Code die Datei an Zeile 42
4. Wenn die Datei nicht existiert, wird eine Fehlerseite mit einem Link zu `https://github.com/bayashi/mclocks` zum Klonen angezeigt

----------

## 🧠 mclocks MCP-Server

`mclocks` enthält einen MCP-Server (Model Context Protocol), der KI-Assistenten wie [Cursor](https://www.cursor.com/) und [Claude Desktop](https://claude.ai/download) ermöglicht, die Frage „Wie spät ist es?" über mehrere Zeitzonen hinweg zu beantworten und zwischen Datum-/Uhrzeitformaten und Epoch-Zeitstempeln zu konvertieren. Der MCP-Server verwendet automatisch Ihre mclocks `config.json`, sodass die in mclocks konfigurierten Zeitzonen in den KI-Antworten berücksichtigt werden.

### Voraussetzungen

* [Node.js](https://nodejs.org/) (v18 oder höher)

Wenn Sie Node.js nicht haben, installieren Sie es von der offiziellen Website.

### Einrichtung

Fügen Sie das folgende JSON zu Ihrer MCP-Konfigurationsdatei hinzu:

* **Cursor**: `.cursor/mcp.json` im Projektstamm oder global `~/.cursor/mcp.json`
* **Claude Desktop** (`claude_desktop_config.json`):
  * Windows: `%APPDATA%\Claude\claude_desktop_config.json`
  * macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
  * Linux: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "mclocks-datetime-util": {
      "command": "npx",
      "args": ["-y", "mclocks-datetime-util"]
    }
  }
}
```

Nach dem Speichern starten Sie die Anwendung neu. Der MCP-Server wird automatisch heruntergeladen und gestartet. Die folgenden Werkzeuge werden verfügbar:

* **`current-time`** - Aktuelle Zeit in Ihren konfigurierten Zeitzonen abrufen
* **`local-time`** - Aktuelle Ortszeit in der Zeitzone des Benutzers abrufen (aus der `convtz`-Konfiguration oder dem Systemstandard)
* **`convert-time`** - Eine Datum-/Uhrzeitzeichenkette oder einen Epoch-Zeitstempel in mehrere Zeitzonen konvertieren
* **`next-weekday`** - Das Datum des nächsten Vorkommens eines bestimmten Wochentags finden
* **`date-to-weekday`** - Den Wochentag für ein bestimmtes Datum ermitteln
* **`days-until`** - Die Anzahl der Tage von heute bis zu einem bestimmten Datum zählen
* **`days-between`** - Die Anzahl der Tage zwischen zwei Daten zählen
* **`date-offset`** - Das Datum N Tage vor oder nach einem bestimmten Datum berechnen

### Wie es mit der mclocks-Konfiguration funktioniert

Der MCP-Server liest automatisch Ihre mclocks `config.json` und verwendet:

* **`clocks`** - In Ihren Uhren definierte Zeitzonen werden als Standard-Konvertierungsziele verwendet
* **`convtz`** - Wird als Standard-Quellzeitzone beim Konvertieren von Datum-/Uhrzeitzeichenketten ohne Zeitzoneninformation verwendet
* **`usetz`** - Aktiviert strikte Zeitzonenkonvertierung für historisch genaue UTC-Offsets (z.B. war JST vor 1888 +09:18). Setzen Sie es auf `true`, wenn Sie historische Datum-/Uhrzeitwerte genau konvertieren müssen

Wenn keine `config.json` gefunden wird, greift der Server auf einen integrierten Satz gängiger Zeitzonen zurück (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### Umgebungsvariablen

Wenn Sie die `config.json`-Einstellungen überschreiben möchten oder überhaupt keine `config.json` haben, können Sie Umgebungsvariablen in Ihrer MCP-Konfiguration setzen. Umgebungsvariablen haben Vorrang vor Werten in `config.json`.

| Variable | Beschreibung | Standard |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | Pfad zur `config.json`. In den meisten Fällen nicht erforderlich, da der Server den Speicherort automatisch erkennt. | automatische Erkennung |
| `MCLOCKS_LOCALE` | Locale für die Formatierung von Wochentagsnamen usw. (z.B. `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Standard-Quellzeitzone für die Interpretation von Datum-/Uhrzeitzeichenketten ohne Zeitzoneninformation (z.B. `Asia/Tokyo`) | *(keine)* |
| `MCLOCKS_USETZ` | Auf `true` setzen, um strikte Zeitzonenkonvertierung zu aktivieren | `false` |

Beispiel:

```json
{
  "mcpServers": {
    "mclocks-datetime-util": {
      "command": "npx",
      "args": ["-y", "mclocks-datetime-util"],
      "env": {
        "MCLOCKS_LOCALE": "ja",
        "MCLOCKS_CONVTZ": "Asia/Tokyo"
      }
    }
  }
}
```

### Verwendungsbeispiel

Nach der Konfiguration können Sie Ihren KI-Assistenten Dinge fragen wie:

* „Wie spät ist es?" - Gibt die aktuelle Zeit in allen Ihren in mclocks konfigurierten Zeitzonen zurück
* „Wie spät ist es in Jakarta?" - Gibt die aktuelle Zeit in einer bestimmten Zeitzone zurück
* „Konvertiere Epoch 1705312200 in Datum/Uhrzeit"
* „Konvertiere 2024-01-15T10:30:00Z nach Asia/Tokyo"
* „Welches Datum hat der nächste Freitag?"
* „Welcher Wochentag ist der 25. Dezember 2026?"
* „Wie viele Tage bis Weihnachten?"
* „Wie viele Tage zwischen dem 1. Januar 2026 und dem 31. Dezember 2026?"
* „Welches Datum ist 90 Tage nach dem 1. April 2026?"

----------

## Lizenz

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Autor

Dai Okabayashi: https://github.com/bayashi
