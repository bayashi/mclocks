# mclocks

L'application d'horloge de bureau pour plusieurs fuseaux horaires🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

Fonctionnalités liées à l'horloge :

* 🕐 Horloge textuelle pour plusieurs fuseaux horaires
* ⏱️ Minuteur
* ⏳ Compte à rebours
* 🔄 Convertisseur entre temps Epoch et date-heure

Le temps n'attend personne :

* 📝 Note adhésive

Un développeur n'est jamais sans horloge :

* 🔀 Convertisseur de texte simple
    * comme la création facile de clauses SQL `IN`
* 🌐 Serveur web
    * sert des fichiers statiques
        * rendu Markdown enrichi
        * visionneuse de contenu basée sur glisser-déposer
    * serveur de vidage des requêtes et réponses
    * endpoints lents pour le débogage
    * ouvrir des fichiers dans votre éditeur depuis des URLs GitHub

🔔 NOTE : `mclocks` n'a pas besoin de connexion internet — tout fonctionne 100% localement.

## 📦 Téléchargement

Depuis https://github.com/bayashi/mclocks/releases

### Windows

Pour Windows, vous pouvez obtenir le fichier d'installation `.msi` ou le fichier exécutable `.exe`.

### macOS

Pour macOS, vous pouvez obtenir le fichier `.dmg` pour l'installation.

(Les raccourcis clavier dans ce document sont pour Windows. Si vous utilisez macOS, veuillez les interpréter en conséquence, en remplaçant `Ctrl` par `Ctrl + Command` et `Alt` par `Option` le cas échéant.)

## ⚙️ config.json

Le fichier `config.json` vous permet de configurer les horloges selon vos préférences.

Le fichier `config.json` doit se trouver dans les répertoires suivants :

* Windows : `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac : `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux : `/home/{USER}/.config/com.bayashi.mclocks/` -->

Lorsque vous démarrez `mclocks`, appuyez sur `Ctrl + o` pour modifier votre fichier `config.json`.

### Exemple de config.json

Le fichier `config.json` doit être formaté en JSON, comme indiqué ci-dessous.

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

Vous pouvez inclure des commentaires et des virgules finales dans `config.json` (compatible JSONC).

## 🔧 Les champs de config.json

#### clocks

Le champ `clocks` est un tableau d'objets, chacun contenant les propriétés `name` et `timezone`. Les deux doivent être des chaînes de caractères. Par défaut, les deux sont `UTC`.

* `name` est une étiquette qui sera affichée pour l'horloge.
* Pour sélectionner les fuseaux horaires, veuillez consulter cette [liste des fuseaux horaires](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).

Voici un exemple d'un tableau `clocks` pour trois fuseaux horaires.

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

chaîne : `MM-DD ddd HH:mm`

Le champ `format` est une chaîne de format date-heure utilisée pour afficher l'horloge. Pour créer un format personnalisé, veuillez consulter [ce guide de formatage](https://momentjs.com/docs/#/parsing/string-format/).

#### format2

chaîne : `""`

Le champ `format2` est identique à `format`. Ils s'alternent avec la touche `Ctrl + f`. Le champ `format2` est optionnel.

#### locale

chaîne : `en`

Le champ `locale` détermine les paramètres de langue pour l'affichage de la date-heure. Vous pouvez trouver [une liste des locales supportées ici](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

chaîne : `#fff`

Le champ `color` définit la couleur du texte de la date-heure. Vous pouvez utiliser des couleurs nommées, des valeurs hexadécimales RGB, des valeurs RGB (ex., `RGB(255, 0, 0)`) ou toute valeur de couleur CSS valide.

#### font

chaîne : `Courier, monospace`

`font` est le nom de la police pour afficher la date-heure. Elle doit être à largeur fixe. Si vous définissez une police à largeur variable, votre mclocks pourrait avoir un effet de tremblement indésirable.

#### size

nombre | chaîne : 14

`size` est la taille des caractères pour la date-heure, en pixels. Elle peut également être spécifiée comme une chaîne incluant une unité (ex., `"125%"`, `"1.5em"`).

#### margin

chaîne : `1.65em`

Le champ `margin` détermine l'espacement entre les horloges.

#### forefront

booléen : `false`

Si le champ `forefront` est défini sur `true`, l'application mclocks sera toujours affichée au-dessus des autres fenêtres d'applications.

## ⏳ Horloge à compte à rebours

En configurant le `clock` comme indiqué ci-dessous, il sera affiché comme une horloge à compte à rebours vers la date-heure `target` spécifiée.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

L'horloge à compte à rebours ci-dessus sera affichée comme suit :

    WAC Tokyo D-159 12:34:56

Indiquant qu'il reste 159 jours, 12 heures, 34 minutes et 56 secondes jusqu'au 13 septembre 2025.

### Verbes de format du compte à rebours

Le texte du champ `countdown` accepte les verbes de modèle suivants :

* `%TG` : Chaîne de date-heure cible
* `%D` : Nombre de jours restants jusqu'à la date-heure cible
* `%H` : Temps restant en heures jusqu'à la date-heure cible
* `%h` : L'heure (hh) du temps restant (hh:mm:ss)
* `%M` : Temps restant en minutes jusqu'à la date-heure cible
* `%m` : La minute (mm) du temps restant (hh:mm:ss)
* `%S` : Temps restant en secondes jusqu'à la date-heure cible
* `%s` : La seconde (ss) du temps restant (hh:mm:ss)

## ⏱️ Minuteur simple

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

Cliquez sur la fenêtre de l'application `mclocks`, puis appuyez sur `Ctrl + 1` pour démarrer un minuteur de 1 minute. Appuyez sur `Ctrl + Alt + 1` pour démarrer un minuteur de 10 minutes. Les autres touches numériques fonctionnent de la même manière. Jusqu'à 5 minuteurs peuvent être démarrés simultanément.

`Ctrl + p` pour mettre en pause / reprendre les minuteurs.

`Ctrl + 0` pour supprimer le minuteur le plus ancien. `Ctrl + Alt + 0` pour supprimer le minuteur le plus récent.

🔔 NOTE : Le compte à rebours et le minuteur simple envoient une notification par défaut lorsqu'ils sont terminés. Si vous n'avez pas besoin de notifications, définissez `withoutNotification: true` dans `config.json`.

## 🔢 Afficher le temps Epoch

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

Cliquez sur la fenêtre de l'application `mclocks`, puis appuyez sur `Ctrl + e` pour basculer l'affichage du temps Epoch.

## 🔄 Convertir entre date-heure et temps Epoch

Cliquez sur la fenêtre de l'application `mclocks`, puis collez une date-heure ou un temps Epoch, et une boîte de dialogue apparaîtra pour afficher les résultats de la conversion. Vous pouvez copier les résultats dans le presse-papiers. Si vous ne souhaitez pas copier, appuyez sur `[No]` pour fermer la boîte de dialogue.

Lors du collage avec `Ctrl + v`, la valeur (temps Epoch) est traitée comme des secondes. Si vous utilisez `Ctrl + Alt + v`, elle est traitée comme des millisecondes, avec `Ctrl + Alt + Shift + V` comme des microsecondes, et avec `Ctrl + Alt + Shift + N + V` comme des nanosecondes et convertie en conséquence.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

Lorsque les valeurs de date-heure collées n'incluent pas d'information de fuseau horaire, elles sont traitées comme fuseau horaire local par défaut. Pour les traiter comme un fuseau horaire spécifique, définissez le fuseau horaire dans l'option convtz.

    "convtz": "UTC"

## 🔀 Fonction de conversion de texte

Cliquez sur la fenêtre de l'application `mclocks`, puis utilisez les raccourcis clavier suivants pour traiter le texte du presse-papiers et l'ouvrir dans un éditeur :

* `Ctrl + i` : Entoure chaque ligne du texte du presse-papiers de guillemets doubles et ajoute une virgule à la fin (sauf la dernière ligne)
* `Ctrl + Shift + i` : Ajoute une virgule à la fin de chaque ligne (sans guillemets) pour les conditions IN de listes INT (sauf la dernière ligne)

Les lignes vides sont préservées telles quelles dans toutes les opérations.

(Cette fonction de conversion de texte n'a rien à voir avec les horloges ou le temps, mais les développeurs pourraient la trouver pratique ! 😊)

## ⌨️ Raccourcis clavier

### Afficher l'aide

`F1` (Windows) ou `Cmd + Shift + /` (macOS) pour ouvrir la page d'aide (ce README) dans le navigateur

### Configuration, formats d'affichage

| Raccourci | Description |
|----------|-------------|
| `Ctrl + o` | Ouvrir le fichier `config.json` dans l'éditeur |
| `Ctrl + f` | Basculer entre `format` et `format2` (si `format2` est défini dans `config.json`) |
| `Ctrl + e` ou `Ctrl + u` | Basculer l'affichage du temps Epoch |

### Minuteur

| Raccourci | Description |
|----------|-------------|
| `Ctrl + 1` à `Ctrl + 9` | Démarrer un minuteur (1 minute × touche numérique) |
| `Ctrl + Alt + 1` à `Ctrl + Alt + 9` | Démarrer un minuteur (10 minutes × touche numérique) |
| `Ctrl + p` | Mettre en pause / reprendre tous les minuteurs |
| `Ctrl + 0` | Supprimer le minuteur le plus ancien |
| `Ctrl + Alt + 0` | Supprimer le minuteur le plus récent |

### Note adhésive

| Raccourci | Description |
|----------|-------------|
| `Ctrl + s` | Créer une nouvelle note adhésive à partir du texte du presse-papiers |

### Opérations de date-heure du presse-papiers

| Raccourci | Description |
|----------|-------------|
| `Ctrl + c` | Copier le texte actuel de mclocks dans le presse-papiers |
| `Ctrl + v` | Convertir le contenu du presse-papiers (temps Epoch en secondes, ou date-heure) |
| `Ctrl + Alt + v` | Convertir le contenu du presse-papiers (temps Epoch en millisecondes) |
| `Ctrl + Alt + Shift + V` | Convertir le contenu du presse-papiers (temps Epoch en microsecondes) |
| `Ctrl + Alt + Shift + N + V` | Convertir le contenu du presse-papiers (temps Epoch en nanosecondes) |

### Conversion de texte

| Raccourci | Description |
|----------|-------------|
| `Ctrl + i` | Entourer chaque ligne du presse-papiers de guillemets doubles, ajouter une virgule à la fin et ouvrir dans l'éditeur (sauf la dernière ligne) |
| `Ctrl + Shift + i` | Ajouter une virgule à la fin de chaque ligne (sans guillemets) pour les conditions IN de listes INT et ouvrir dans l'éditeur (sauf la dernière ligne) |

## 📝 Note adhésive

![sticky-note](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-sticky-note.png)

Cliquez sur la fenêtre de l'application `mclocks`, puis appuyez sur `Ctrl + s` pour créer une note adhésive à partir du texte du presse-papiers. Une petite fenêtre flottante s'ouvrira avec le contenu du presse-papiers.

Chaque note adhésive dispose de :

* **Bouton bascule** (`▸` / `▾`) : Développer ou réduire la note. En mode réduit, seule une ligne est affichée.
* **Bouton copier** (`⧉`) : Copier le texte de la note dans le presse-papiers.
* **Bouton premier plan** (`⊤` / `⊥`) : Basculer si la note reste au-dessus des autres fenêtres. Ce paramètre est sauvegardé par note adhésive.
* **Bouton fermer** (`✖`) : Supprimer la note adhésive et fermer sa fenêtre.
* **Zone de texte** : Modifier librement le contenu de la note. Les modifications sont automatiquement sauvegardées.
* **Poignée de redimensionnement** : Faites glisser le coin inférieur droit pour redimensionner la note lorsqu'elle est développée.

Les notes adhésives héritent des paramètres `font`, `size`, `color` et `forefront` de `config.json`. Le paramètre de premier plan peut être remplacé par note adhésive en utilisant le bouton de premier plan ; s'il n'est pas remplacé, la valeur de `config.json` est utilisée. Leur position, taille, état ouvert/fermé et le remplacement du premier plan sont persistés, et toutes les notes sont automatiquement restaurées lorsque `mclocks` redémarre.

La taille maximale de texte par note adhésive est de 128 Ko.

## 🌐 Serveur web

`mclocks` démarre toujours un serveur web local intégré au lancement. Si vous configurez un champ `web` dans `config.json`, il peut aussi servir des fichiers statiques depuis votre répertoire :

    {
      "web": {
        "root": "/path/to/your/webroot",
        "dump": true,
        "slow": true,
        "status": true,
        "content": {
          "markdown": {
            "allowRawHTML": false,
            "openExternalLinkInNewTab": true
          }
        },
        "editor": {
          "reposDir": "/path/to/your/repos"
        }
      }
    }

* `root` : Chemin vers le répertoire contenant les fichiers à servir (obligatoire uniquement pour l'hébergement de fichiers statiques)
* `port` : Numéro de port préféré du serveur web principal (`>=2000`, par défaut : `3030`). S'il est occupé, mclocks teste des ports décroissants (`-1`) jusqu'à en trouver un libre.
* `openBrowserAtStart` : Si `true`, ouvre automatiquement l'URL du serveur web dans le navigateur par défaut au démarrage de `mclocks` (par défaut : `false`)
* `dump` : Si `true`, active l'endpoint `/dump` qui retourne les détails de la requête en JSON (par défaut : `false`)
* `slow` : Si `true`, active l'endpoint `/slow` qui retarde la réponse (par défaut : `false`)
* `status` : Si `true`, active l'endpoint `/status/{code}` qui retourne des codes de statut HTTP arbitraires (par défaut : `false`)
* `content.markdown.allowRawHTML` : Si `true`, autorise le HTML brut dans le rendu Markdown ; si `false`, le HTML brut est échappé comme texte (par défaut : `false`)
* `content.markdown.openExternalLinkInNewTab` : Les liens Markdown externes s'ouvrent dans un nouvel onglet et les liens internes dans le même ; si `false`, tous les liens Markdown s'ouvrent dans le même onglet (par défaut : `true`)
* `editor` : Si défini et contient `reposDir`, active l'endpoint `/editor` qui ouvre des fichiers locaux dans votre éditeur depuis des URLs GitHub du navigateur (par défaut : non défini)

### Visionneuse de contenu par glisser-déposer

En plus de l'hébergement de fichiers statiques, mclocks prend en charge un flux de visionneuse par glisser-déposer :

* Déposez un répertoire sur la fenêtre de l'horloge pour l'ouvrir dans le visionneur web via une URL locale temporaire.
* Déposez un fichier unique pour l'ouvrir dans le visionneur web lorsque le type est pris en charge par le visionneur de fichiers temporaires.
* Les URLs temporaires générées sont locales uniquement et sont supprimées à la fermeture de mclocks.

### Mode de contenu

Le visionneur web prend en charge les options de requête `mode` telles que `content`, `raw` et `source`.

* `content` (par défaut) : Sert le fichier avec son type de contenu détecté pour un rendu navigateur normal lorsque c'est possible.
* `raw` : Renvoie les fichiers non binaires en `text/plain` pour afficher le texte brut sans rendu navigateur.
* `source` : Ouvre la mise en page du visionneur source avec résumé/barre latérale pour les formats pris en charge, et permet l'inspection en texte brut pour les fichiers texte non pris en charge.

### Endpoint /dump

Lorsque `dump: true` est défini dans la configuration `web`, le serveur web fournit un endpoint `/dump` qui retourne les détails de la requête en JSON.

Le endpoint répond avec un objet JSON contenant :
* `method` : Méthode HTTP (ex., "GET", "POST")
* `path` : Chemin de la requête après `/dump/` (ex., "/test" pour `/dump/test`)
* `query` : Paramètres de requête comme un tableau d'objets clé-valeur (ex., `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers` : En-têtes de la requête comme un tableau d'objets clé-valeur (ex., `[{"Content-Type": "application/json"}]`)
* `body` : Corps de la requête comme chaîne de caractères (si présent)
* `parsed_body` : Objet JSON parsé si le Content-Type indique du JSON, ou chaîne de message d'erreur si le parsing échoue

Accédez au endpoint dump à `http://127.0.0.1:3030/dump` ou tout chemin sous `/dump/` (ex., `/dump/test?key=value`).

### Endpoint /slow

Lorsque `slow: true` est défini dans la configuration `web`, le serveur web fournit un endpoint `/slow` qui retarde la réponse avant de retourner 200 OK.

Le endpoint est accessible via n'importe quelle méthode HTTP (GET, POST, etc.) et supporte les chemins suivants :

* `/slow` : Attend 30 secondes (par défaut) et retourne 200 OK
* `/slow/120` : Attend 120 secondes (ou tout nombre de secondes spécifié) et retourne 200 OK

La valeur maximale autorisée est de 901 secondes (15 minutes + 1 seconde). Les requêtes dépassant cette limite retournent une erreur 400 Bad Request.

Ce endpoint est utile pour tester le comportement des timeouts, la gestion des connexions ou simuler des conditions réseau lentes.

Si un paramètre de secondes invalide est fourni (ex., `/slow/abc`), le endpoint retourne une erreur 400 Bad Request.

### Endpoint /status

Lorsque `status: true` est défini dans la configuration `web`, le serveur web fournit un endpoint `/status/{code}` qui retourne des codes de statut HTTP arbitraires définis dans les standards RFC (100-599).

Le endpoint retourne une réponse en texte brut avec le code de statut et sa phrase correspondante, ainsi que les en-têtes appropriés requis par la spécification HTTP.

**Exemples :**
* `http://127.0.0.1:3030/status/200` - retourne 200 OK
* `http://127.0.0.1:3030/status/404` - retourne 404 Not Found
* `http://127.0.0.1:3030/status/500` - retourne 500 Internal Server Error
* `http://127.0.0.1:3030/status/418` - retourne 418 I'm a teapot (avec message spécial)
* `http://127.0.0.1:3030/status/301` - retourne 301 Moved Permanently (avec en-tête Location)

**En-têtes spécifiques par statut :**

Le endpoint ajoute automatiquement les en-têtes appropriés pour des codes de statut spécifiques :

* **3xx Redirection** (301, 302, 303, 305, 307, 308) : Ajoute l'en-tête `Location`
* **401 Unauthorized** : Ajoute l'en-tête `WWW-Authenticate`
* **405 Method Not Allowed** : Ajoute l'en-tête `Allow`
* **407 Proxy Authentication Required** : Ajoute l'en-tête `Proxy-Authenticate`
* **416 Range Not Satisfiable** : Ajoute l'en-tête `Content-Range`
* **426 Upgrade Required** : Ajoute l'en-tête `Upgrade`
* **429 Too Many Requests** : Ajoute l'en-tête `Retry-After` (60 secondes)
* **503 Service Unavailable** : Ajoute l'en-tête `Retry-After` (60 secondes)
* **511 Network Authentication Required** : Ajoute l'en-tête `WWW-Authenticate`

**Gestion du corps de réponse :**

* **204 No Content** et **304 Not Modified** : Retourne un corps de réponse vide (conformément à la spécification HTTP)
* **418 I'm a teapot** : Retourne le message spécial "I'm a teapot" au lieu de la phrase de statut standard
* **Tous les autres codes de statut** : Retourne du texte brut au format `{code} {phrase}` (ex., "404 Not Found")

Ce endpoint est utile pour tester comment vos applications gèrent les différents codes de statut HTTP, la gestion des erreurs, les redirections, les exigences d'authentification et les scénarios de limitation de débit.

### Endpoint /editor

Lorsque `web.editor.reposDir` est défini dans le fichier de configuration, le serveur web fournit un endpoint `/editor` qui vous permet d'ouvrir des fichiers locaux dans votre éditeur directement depuis des URLs GitHub du navigateur.

**Configuration :**

Ajoutez ce qui suit à votre configuration `web` :

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

* `reposDir` : Chemin vers votre répertoire de dépôts locaux. Supporte `~` pour l'expansion du répertoire personnel (ex., `"~/repos"` sur macOS ou `"C:/Users/username/repos"` sur Windows). Ce répertoire doit exister.
* `includeHost` : Si `true`, la résolution du chemin local inclut l'hôte original comme répertoire (ex., `{reposDir}/{host}/{owner}/{repo}/...`). Si `false`, il résout en `{reposDir}/{owner}/{repo}/...` (par défaut : `false`).
* `command` : Nom de la commande ou chemin vers l'exécutable de votre éditeur (par défaut : `code`)
* `args` : Tableau de modèle d'arguments. Utilisez les marqueurs `{file}` et `{line}`. Si `#L...` n'est pas présent dans l'URL, `{line}` utilise 1.

**Comment ça marche :**

1. Lorsque vous accédez à une URL de fichier GitHub via le endpoint `/editor`, il convertit le chemin GitHub en un chemin de fichier local
2. Le chemin du fichier local est construit comme : `{reposDir}/{owner}/{repository_name}/{file_path}`
3. Si le fichier existe, il l'ouvre dans votre éditeur au numéro de ligne spécifié en utilisant la commande et les arguments configurés (par défaut : `code -g {local_file_path}:{line_number}`)
4. Si le fichier n'existe pas, une page d'erreur est affichée avec un lien pour cloner le dépôt

**Bookmarklet :**

Créez un bookmarklet pour ouvrir rapidement des fichiers GitHub dans votre éditeur local. Remplacez `3030` par votre numéro de port configuré :

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**Support des numéros de ligne :**

Vous pouvez spécifier un numéro de ligne en utilisant le fragment hash dans l'URL :
* `https://github.com/username/repo/blob/main/file.rs#L123` → Ouvre à la ligne 123

**Gestion des erreurs :**

* Si le fichier n'existe pas localement, l'onglet reste ouvert et affiche un message d'erreur avec un lien pour cloner le dépôt depuis GitHub
* Si le fichier est ouvert avec succès, l'onglet se ferme automatiquement
* Si `web.editor.reposDir` n'est pas configuré ou n'existe pas, le endpoint `/editor` n'est pas activé (et vous obtiendrez une 404)

**Exemple :**

1. Vous consultez un fichier sur GitHub : `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. Cliquez sur le bookmarklet ou naviguez manuellement vers : `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. Si `~/repos/mclocks/src/app.js` existe en local, VS Code l'ouvre à la ligne 42
4. Si le fichier n'existe pas, une page d'erreur s'affiche avec un lien vers `https://github.com/bayashi/mclocks` pour le cloner

----------

## 🧠 Serveur MCP mclocks

`mclocks` inclut un serveur MCP (Model Context Protocol) qui permet aux assistants IA tels que [Cursor](https://www.cursor.com/) et [Claude Desktop](https://claude.ai/download) de répondre à « Quelle heure est-il ? » dans plusieurs fuseaux horaires, et de convertir entre les formats date-heure et les timestamps Epoch. Le serveur MCP utilise automatiquement votre `config.json` de mclocks, de sorte que les fuseaux horaires configurés dans mclocks sont reflétés dans les réponses de l'IA.

### Prérequis

* [Node.js](https://nodejs.org/) (v18 ou ultérieur)

Si vous n'avez pas Node.js, installez-le depuis le site officiel.

### Configuration

Ajoutez le JSON suivant à votre fichier de configuration MCP :

* **Cursor** : `.cursor/mcp.json` à la racine de votre projet, ou global `~/.cursor/mcp.json`
* **Claude Desktop** (`claude_desktop_config.json`) :
  * Windows : `%APPDATA%\Claude\claude_desktop_config.json`
  * macOS : `~/Library/Application Support/Claude/claude_desktop_config.json`
  * Linux : `~/.config/Claude/claude_desktop_config.json`

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

Après avoir sauvegardé, redémarrez l'application. Le serveur MCP sera automatiquement téléchargé et démarré. Les outils suivants deviennent disponibles :

* **`current-time`** - Obtenir l'heure actuelle dans vos fuseaux horaires configurés
* **`local-time`** - Obtenir l'heure locale actuelle dans le fuseau horaire de l'utilisateur (depuis la configuration `convtz` ou la valeur par défaut du système)
* **`convert-time`** - Convertir une chaîne date-heure ou un timestamp Epoch vers plusieurs fuseaux horaires
* **`next-weekday`** - Trouver la date de la prochaine occurrence d'un jour de la semaine donné
* **`date-to-weekday`** - Obtenir le jour de la semaine pour une date donnée
* **`days-until`** - Compter le nombre de jours entre aujourd'hui et une date spécifiée
* **`days-between`** - Compter le nombre de jours entre deux dates
* **`date-offset`** - Calculer la date N jours avant ou après une date donnée

### Comment ça fonctionne avec la configuration mclocks

Le serveur MCP lit automatiquement votre `config.json` de mclocks et utilise :

* **`clocks`** - Les fuseaux horaires définis dans vos horloges sont utilisés comme cibles de conversion par défaut
* **`convtz`** - Utilisé comme fuseau horaire source par défaut lors de la conversion de chaînes date-heure sans information de fuseau horaire
* **`usetz`** - Active la conversion stricte des fuseaux horaires pour des décalages UTC historiquement précis (ex., JST était +09:18 avant 1888). Définissez-le sur `true` lorsque vous avez besoin de convertir des dates-heures historiques avec précision

Si aucun `config.json` n'est trouvé, le serveur utilise un ensemble intégré de fuseaux horaires courants (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### Variables d'environnement

Si vous souhaitez remplacer les paramètres de `config.json`, ou si vous n'avez pas de `config.json`, vous pouvez définir des variables d'environnement dans votre configuration MCP. Les variables d'environnement ont priorité sur les valeurs de `config.json`.

| Variable | Description | Par défaut |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | Chemin vers `config.json`. Non requis dans la plupart des cas, car le serveur détecte automatiquement l'emplacement. | détection automatique |
| `MCLOCKS_LOCALE` | Locale pour le formatage des noms de jours de la semaine, etc. (ex., `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Fuseau horaire source par défaut pour interpréter les chaînes date-heure sans information de fuseau horaire (ex., `Asia/Tokyo`) | *(aucun)* |
| `MCLOCKS_USETZ` | Définir sur `true` pour activer la conversion stricte des fuseaux horaires | `false` |

Exemple :

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

### Exemple d'utilisation

Une fois configuré, vous pouvez demander à votre assistant IA des choses comme :

* « Quelle heure est-il ? » - Retourne l'heure actuelle dans tous vos fuseaux horaires configurés dans mclocks
* « Quelle heure est-il à Jakarta ? » - Retourne l'heure actuelle dans un fuseau horaire spécifique
* « Convertis l'epoch 1705312200 en date-heure »
* « Convertis 2024-01-15T10:30:00Z en Asia/Tokyo »
* « Quel jour est le prochain vendredi ? »
* « Quel jour de la semaine est le 25 décembre 2026 ? »
* « Combien de jours avant Noël ? »
* « Combien de jours entre le 1er janvier 2026 et le 31 décembre 2026 ? »
* « Quelle date est 90 jours après le 1er avril 2026 ? »

----------

## Licence

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Auteur

Dai Okabayashi: https://github.com/bayashi
