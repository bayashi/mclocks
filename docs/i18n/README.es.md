# mclocks

La aplicación de reloj de escritorio para múltiples zonas horarias🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

Funciones relacionadas con el reloj:

* 🕐 Reloj de texto para múltiples zonas horarias
* ⏱️ Temporizador
* ⏳ Temporizador de cuenta regresiva
* 🔄 Conversor entre tiempo Epoch y fecha-hora

El tiempo no espera a nadie:

* 📝 Nota adhesiva

Un desarrollador nunca está sin reloj:

* 🔀 Conversor de texto simple
    * como crear fácilmente cláusulas SQL `IN`
* 🌐 Servidor web
    * sirve archivos estáticos
    * servidor de volcado de solicitudes y respuestas
    * endpoints lentos para depuración
    * abrir archivos en tu editor desde URLs de GitHub

🔔 NOTA: `mclocks` no necesita conexión a internet — todo funciona 100% localmente.

## 📦 Descarga

Desde https://github.com/bayashi/mclocks/releases

### Windows

Para Windows, puedes obtener el instalador `.msi` o el archivo ejecutable `.exe`.

### macOS

Para macOS, puedes obtener el archivo `.dmg` para instalar.

(Los atajos de teclado en este documento son para Windows. Si usas macOS, por favor interprétalos en consecuencia, reemplazando teclas como `Ctrl` por `Ctrl + Command` y `Alt` por `Option` donde corresponda.)

## ⚙️ config.json

El archivo `config.json` te permite configurar los relojes según tus preferencias.

El archivo `config.json` debe ubicarse en los siguientes directorios:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

Cuando inicies `mclocks`, presiona `Ctrl + o` para editar tu archivo `config.json`.

### Ejemplo de config.json

El archivo `config.json` debe estar formateado como JSON, como se muestra a continuación.

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

## 🔧 Los campos de config.json

#### clocks

El campo `clocks` es un array de objetos, cada uno con las propiedades `name` y `timezone`. Ambos deben ser cadenas de texto. Por defecto, ambos son `UTC`.

* `name` es una etiqueta que se mostrará para el reloj.
* Para seleccionar zonas horarias, consulta esta [lista de zonas horarias](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).

Aquí hay un ejemplo de un array `clocks` con tres zonas horarias.

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

cadena: `MM-DD ddd HH:mm`

El campo `format` es una cadena de formato de fecha-hora utilizada para mostrar el reloj. Para crear un formato personalizado, consulta [esta guía de formato](https://momentjs.com/docs/#/parsing/string-format/).

#### format2

cadena: `""`

El campo `format2` es igual que `format`. Se alternan entre sí con la tecla `Ctrl + f`. El campo `format2` es opcional.

#### locale

cadena: `en`

El campo `locale` determina la configuración de idioma para mostrar la fecha-hora. Puedes encontrar [una lista de locales soportados aquí](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

cadena: `#fff`

El campo `color` define el color del texto de fecha-hora. Puedes usar colores con nombre, valores hexadecimales RGB, valores RGB (ej., `RGB(255, 0, 0)`) o cualquier valor de color CSS válido.

#### font

cadena: `Courier, monospace`

`font` es el nombre de la fuente para mostrar la fecha-hora. Debe ser una fuente monoespaciada. Si configuras una fuente de ancho variable, tu mclocks podría tener un efecto de oscilación indeseable.

#### size

número | cadena: 14

`size` es el tamaño del carácter para la fecha-hora, en píxeles. También se puede especificar como una cadena que incluya una unidad (ej., `"125%"`, `"1.5em"`).

#### margin

cadena: `1.65em`

El campo `margin` determina el espacio entre los relojes.

#### forefront

booleano: `false`

Si el campo `forefront` se establece en `true`, la aplicación mclocks siempre se mostrará encima de las demás ventanas de aplicaciones.

## ⏳ Reloj de cuenta regresiva

Al configurar el `clock` como se muestra a continuación, se mostrará como un reloj de cuenta regresiva hacia la fecha-hora `target` especificada.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

El `clock` de cuenta regresiva anterior se mostrará así:

    WAC Tokyo D-159 12:34:56

Indicando que faltan 159 días, 12 horas, 34 minutos y 56 segundos hasta el 13 de septiembre de 2025.

### Verbos de formato de cuenta regresiva

El texto del campo `countdown` acepta los siguientes verbos de plantilla:

* `%TG`: Cadena de fecha-hora objetivo
* `%D`: Días restantes hasta la fecha-hora objetivo
* `%H`: Tiempo restante en horas hasta la fecha-hora objetivo
* `%h`: La hora (hh) del tiempo restante (hh:mm:ss)
* `%M`: Tiempo restante en minutos hasta la fecha-hora objetivo
* `%m`: El minuto (mm) del tiempo restante (hh:mm:ss)
* `%S`: Tiempo restante en segundos hasta la fecha-hora objetivo
* `%s`: El segundo (ss) del tiempo restante (hh:mm:ss)

## ⏱️ Temporizador simple

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

Haz clic en la ventana de la aplicación `mclocks`, luego presiona `Ctrl + 1` para iniciar un temporizador de 1 minuto. Presiona `Ctrl + Alt + 1` para iniciar un temporizador de 10 minutos. Las demás teclas numéricas funcionan de la misma manera. Se pueden iniciar hasta 5 temporizadores simultáneamente.

`Ctrl + p` para pausar / reanudar los temporizadores.

`Ctrl + 0` para eliminar el temporizador más antiguo. `Ctrl + Alt + 0` para eliminar el temporizador más reciente.

🔔 NOTA: El reloj de cuenta regresiva y el temporizador simple envían una notificación por defecto cuando se completan. Si no necesitas notificaciones, configura `withoutNotification: true` en `config.json`.

## 🔢 Mostrar tiempo Epoch

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

Haz clic en la ventana de la aplicación `mclocks`, luego presiona `Ctrl + e` para alternar la visualización del tiempo Epoch.

## 🔄 Convertir entre fecha-hora y tiempo Epoch

Haz clic en la ventana de la aplicación `mclocks`, luego pega una fecha-hora o un tiempo Epoch, y aparecerá un diálogo con los resultados de la conversión. También puedes copiar los resultados al portapapeles. Si no deseas copiar, presiona `[No]` para cerrar el diálogo.

Al pegar con `Ctrl + v`, el valor (tiempo Epoch) se trata como segundos. Si usas `Ctrl + Alt + v`, se trata como milisegundos, con `Ctrl + Alt + Shift + V` como microsegundos, y con `Ctrl + Alt + Shift + N + V` como nanosegundos y se convierte en consecuencia.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

Cuando los valores de fecha-hora pegados no incluyen información de zona horaria, se tratan como zona horaria local por defecto. Para tratarlos como una zona horaria específica, configura la zona horaria en la opción convtz.

    "convtz": "UTC"

## 🔀 Función de conversión de texto

Haz clic en la ventana de la aplicación `mclocks`, luego usa los siguientes atajos de teclado para procesar el texto del portapapeles y abrirlo en un editor:

* `Ctrl + i`: Encierra cada línea del texto del portapapeles entre comillas dobles y añade una coma al final (excepto la última línea)
* `Ctrl + Shift + i`: Añade una coma al final de cada línea (sin comillas) para condiciones IN de listas INT (excepto la última línea)

Las líneas vacías se preservan tal cual en todas las operaciones.

(Esta función de conversión de texto no tiene nada que ver con relojes o tiempo, ¡pero los desarrolladores de software podrían encontrarla útil! 😊)

## ⌨️ Atajos de teclado

### Mostrar ayuda

`F1` (Windows) o `Cmd + Shift + /` (macOS) para abrir la página de ayuda (este README) en el navegador

### Configuración, formatos de visualización

| Atajo | Descripción |
|----------|-------------|
| `Ctrl + o` | Abrir el archivo `config.json` en el editor |
| `Ctrl + f` | Alternar entre `format` y `format2` (si `format2` está definido en `config.json`) |
| `Ctrl + e` o `Ctrl + u` | Alternar la visualización del tiempo Epoch |

### Temporizador

| Atajo | Descripción |
|----------|-------------|
| `Ctrl + 1` a `Ctrl + 9` | Iniciar temporizador (1 minuto × tecla numérica) |
| `Ctrl + Alt + 1` a `Ctrl + Alt + 9` | Iniciar temporizador (10 minutos × tecla numérica) |
| `Ctrl + p` | Pausar / reanudar todos los temporizadores |
| `Ctrl + 0` | Eliminar el temporizador más antiguo (el de más a la izquierda) |
| `Ctrl + Alt + 0` | Eliminar el temporizador más reciente (el de más a la derecha) |

### Nota adhesiva

| Atajo | Descripción |
|----------|-------------|
| `Ctrl + s` | Crear una nueva nota adhesiva a partir del texto del portapapeles |

### Operaciones de fecha-hora del portapapeles

| Atajo | Descripción |
|----------|-------------|
| `Ctrl + c` | Copiar el texto actual de mclocks al portapapeles |
| `Ctrl + v` | Convertir el contenido del portapapeles (tiempo Epoch como segundos, o fecha-hora) |
| `Ctrl + Alt + v` | Convertir el contenido del portapapeles (tiempo Epoch como milisegundos) |
| `Ctrl + Alt + Shift + V` | Convertir el contenido del portapapeles (tiempo Epoch como microsegundos) |
| `Ctrl + Alt + Shift + N + V` | Convertir el contenido del portapapeles (tiempo Epoch como nanosegundos) |

### Conversión de texto

| Atajo | Descripción |
|----------|-------------|
| `Ctrl + i` | Encerrar cada línea del portapapeles entre comillas dobles, añadir coma al final y abrir en el editor (excepto la última línea) |
| `Ctrl + Shift + i` | Añadir coma al final de cada línea (sin comillas) para condiciones IN de listas INT y abrir en el editor (excepto la última línea) |

## 📝 Nota adhesiva

Haz clic en la ventana de la aplicación `mclocks`, luego presiona `Ctrl + s` para crear una nota adhesiva a partir del texto del portapapeles. Se abrirá una pequeña ventana flotante con el contenido del portapapeles.

Cada nota adhesiva tiene:

* **Botón de alternar** (`▸` / `▾`): Expandir o colapsar la nota. En modo colapsado solo se muestra una línea.
* **Botón de copiar** (`⧉`): Copiar el texto de la nota al portapapeles.
* **Botón de primer plano** (`⊤` / `⊥`): Alternar si la nota permanece encima de otras ventanas. Esta configuración se guarda por nota adhesiva.
* **Botón de cerrar** (`✖`): Eliminar la nota adhesiva y cerrar su ventana.
* **Área de texto**: Editar libremente el contenido de la nota. Los cambios se guardan automáticamente.
* **Control de redimensionamiento**: Arrastrar la esquina inferior derecha para cambiar el tamaño de la nota cuando está expandida.

Las notas adhesivas heredan las configuraciones de `font`, `size`, `color` y `forefront` de `config.json`. La configuración de primer plano se puede anular por nota adhesiva usando el botón de primer plano; si no se anula, se usa el valor de `config.json`. Su posición, tamaño, estado de apertura/cierre y la anulación de primer plano se persisten, y todas las notas se restauran automáticamente cuando `mclocks` se reinicia.

🔔 NOTA: En macOS, las posiciones de las ventanas de notas adhesivas solo se guardan cuando la aplicación se cierra. En Windows, las posiciones se guardan automáticamente al mover o redimensionar las ventanas.

El tamaño máximo de texto por nota adhesiva es de 128 KB.

## 🌐 Servidor web

`mclocks` puede servir archivos estáticos a través de un servidor web integrado. Esta función te permite ver fácilmente tus fragmentos de código en un navegador. Añade un campo `web` a tu `config.json`:

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

* `root`: Ruta al directorio que contiene los archivos a servir (obligatorio)
* `port`: Número de puerto para escuchar (por defecto: 3030)
* `open_browser_at_start`: Si se establece en `true`, abre automáticamente la URL del servidor web en el navegador predeterminado cuando `mclocks` se inicia (por defecto: `false`)
* `dump`: Si se establece en `true`, habilita el endpoint `/dump` que devuelve los detalles de la solicitud como JSON (por defecto: `false`)
* `slow`: Si se establece en `true`, habilita el endpoint `/slow` que retrasa la respuesta (por defecto: `false`)
* `status`: Si se establece en `true`, habilita el endpoint `/status/{code}` que devuelve códigos de estado HTTP arbitrarios (por defecto: `false`)
* `content.markdown.allowRawHTML`: Si se establece en `true`, permite HTML sin procesar dentro del renderizado de Markdown; si es `false`, el HTML sin procesar en Markdown se escapa como texto (por defecto: `false`)
* `editor`: Si se establece y contiene `reposDir`, habilita el endpoint `/editor` que abre archivos locales en tu editor desde URLs de GitHub del navegador (por defecto: no establecido)

Si el campo `web` está configurado en tu `config.json`, el servidor web se inicia automáticamente cuando `mclocks` se lanza. Accede a los archivos en `http://127.0.0.1:3030`. El servidor web solo escucha en `127.0.0.1` (localhost), por lo que solo es accesible desde tu máquina local.

### Tipos de archivo soportados

El servidor web soporta los siguientes tipos de archivo:

* Texto: `html`, `css`, `js`, `json`, `md`, `txt`
* Imágenes: `png`, `jpg`, `jpeg`, `gif`, `svg`, `ico`

### Endpoint /dump

Cuando se establece `dump: true` en la configuración `web`, el servidor web proporciona un endpoint `/dump` que devuelve los detalles de la solicitud como JSON.

El endpoint responde con un objeto JSON que contiene:
* `method`: Método HTTP (ej., "GET", "POST")
* `path`: Ruta de la solicitud después de `/dump/` (ej., "/test" para `/dump/test`)
* `query`: Parámetros de consulta como un array de objetos clave-valor (ej., `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: Cabeceras de la solicitud como un array de objetos clave-valor (ej., `[{"Content-Type": "application/json"}]`)
* `body`: Cuerpo de la solicitud como cadena de texto (si está presente)
* `parsed_body`: Objeto JSON parseado si el Content-Type indica JSON, o cadena de mensaje de error si el parseo falla

Accede al endpoint dump en `http://127.0.0.1:3030/dump` o cualquier ruta bajo `/dump/` (ej., `/dump/test?key=value`).

### Endpoint /slow

Cuando se establece `slow: true` en la configuración `web`, el servidor web proporciona un endpoint `/slow` que retrasa la respuesta antes de devolver 200 OK.

El endpoint es accesible a través de cualquier método HTTP (GET, POST, etc.) y soporta las siguientes rutas:

* `/slow`: Espera 30 segundos (por defecto) y devuelve 200 OK
* `/slow/120`: Espera 120 segundos (o cualquier número de segundos especificado) y devuelve 200 OK

El valor máximo permitido es 901 segundos (15 minutos + 1 segundo). Las solicitudes que excedan este límite devuelven un error 400 Bad Request.

Este endpoint es útil para probar el comportamiento de timeout, manejo de conexiones o simular condiciones de red lentas.

Si se proporciona un parámetro de segundos inválido (ej., `/slow/abc`), el endpoint devuelve un error 400 Bad Request.

### Endpoint /status

Cuando se establece `status: true` en la configuración `web`, el servidor web proporciona un endpoint `/status/{code}` que devuelve códigos de estado HTTP arbitrarios definidos en los estándares RFC (100-599).

El endpoint devuelve una respuesta en texto plano con el código de estado y su frase correspondiente, junto con las cabeceras apropiadas según la especificación HTTP.

**Ejemplos:**
* `http://127.0.0.1:3030/status/200` - devuelve 200 OK
* `http://127.0.0.1:3030/status/404` - devuelve 404 Not Found
* `http://127.0.0.1:3030/status/500` - devuelve 500 Internal Server Error
* `http://127.0.0.1:3030/status/418` - devuelve 418 I'm a teapot (con mensaje especial)
* `http://127.0.0.1:3030/status/301` - devuelve 301 Moved Permanently (con cabecera Location)

**Cabeceras específicas por estado:**

El endpoint añade automáticamente cabeceras apropiadas para códigos de estado específicos:

* **3xx Redirección** (301, 302, 303, 305, 307, 308): Añade cabecera `Location`
* **401 Unauthorized**: Añade cabecera `WWW-Authenticate`
* **405 Method Not Allowed**: Añade cabecera `Allow`
* **407 Proxy Authentication Required**: Añade cabecera `Proxy-Authenticate`
* **416 Range Not Satisfiable**: Añade cabecera `Content-Range`
* **426 Upgrade Required**: Añade cabecera `Upgrade`
* **429 Too Many Requests**: Añade cabecera `Retry-After` (60 segundos)
* **503 Service Unavailable**: Añade cabecera `Retry-After` (60 segundos)
* **511 Network Authentication Required**: Añade cabecera `WWW-Authenticate`

**Manejo del cuerpo de respuesta:**

* **204 No Content** y **304 Not Modified**: Devuelve cuerpo de respuesta vacío (según la especificación HTTP)
* **418 I'm a teapot**: Devuelve el mensaje especial "I'm a teapot" en lugar de la frase de estado estándar
* **Todos los demás códigos de estado**: Devuelve texto plano en formato `{code} {phrase}` (ej., "404 Not Found")

Este endpoint es útil para probar cómo tus aplicaciones manejan diferentes códigos de estado HTTP, manejo de errores, redirecciones, requisitos de autenticación y escenarios de limitación de velocidad.

### Endpoint /editor

Cuando se establece `web.editor.reposDir` en el archivo de configuración, el servidor web proporciona un endpoint `/editor` que te permite abrir archivos locales en tu editor directamente desde URLs de GitHub del navegador.

**Configuración:**

Añade lo siguiente a tu configuración `web`:

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

* `reposDir`: Ruta al directorio de tus repositorios locales. Soporta `~` para la expansión del directorio home (ej., `"~/repos"` en macOS o `"C:/Users/username/repos"` en Windows). Este directorio debe existir.
* `includeHost`: Si es `true`, la resolución de la ruta local incluye el host original como directorio (ej., `{reposDir}/{host}/{owner}/{repo}/...`). Si es `false`, se resuelve a `{reposDir}/{owner}/{repo}/...` (por defecto: `false`).
* `command`: Nombre del comando o ruta al ejecutable de tu editor (por defecto: `code`)
* `args`: Array de plantilla de argumentos. Usa los marcadores `{file}` y `{line}`. Si `#L...` no está presente en la URL, `{line}` usa 1.

**Cómo funciona:**

1. Cuando accedes a una URL de archivo de GitHub a través del endpoint `/editor`, convierte la ruta de GitHub a una ruta de archivo local
2. La ruta del archivo local se construye como: `{reposDir}/{owner}/{repository_name}/{file_path}`
3. Si el archivo existe, lo abre en tu editor en el número de línea especificado usando el comando y los argumentos configurados (por defecto: `code -g {local_file_path}:{line_number}`)
4. Si el archivo no existe, se muestra una página de error con un enlace para clonar el repositorio

**Bookmarklet:**

Crea un bookmarklet para abrir rápidamente archivos de GitHub en tu editor local. Reemplaza `3030` con tu número de puerto configurado:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**Soporte de número de línea:**

Puedes especificar un número de línea usando el fragmento hash en la URL:
* `https://github.com/username/repo/blob/main/file.rs#L123` → Abre en la línea 123

**Manejo de errores:**

* Si el archivo no existe localmente, la pestaña permanece abierta y muestra un mensaje de error con un enlace para clonar el repositorio desde GitHub
* Si el archivo se abre correctamente, la pestaña se cierra automáticamente
* Si `web.editor.reposDir` no está configurado o no existe, el endpoint `/editor` no se habilita (y obtendrás un 404)

**Ejemplo:**

1. Estás viendo un archivo en GitHub: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. Haz clic en el bookmarklet o navega manualmente a: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. Si `~/repos/mclocks/src/app.js` existe en tu local, VS Code lo abre en la línea 42
4. Si el archivo no existe, se muestra una página de error con un enlace a `https://github.com/bayashi/mclocks` para clonarlo

----------

## 🧠 Servidor MCP de mclocks

`mclocks` incluye un servidor MCP (Model Context Protocol) que permite a asistentes de IA como [Cursor](https://www.cursor.com/) y [Claude Desktop](https://claude.ai/download) responder "¿Qué hora es?" en múltiples zonas horarias, y convertir entre formatos de fecha-hora y timestamps Epoch. El servidor MCP usa automáticamente tu `config.json` de mclocks, por lo que las zonas horarias configuradas en mclocks se reflejan en las respuestas de la IA.

### Requisitos previos

* [Node.js](https://nodejs.org/) (v18 o posterior)

Si no tienes Node.js, instálalo desde el sitio web oficial.

### Configuración

Añade el siguiente JSON a tu archivo de configuración MCP:

* **Cursor**: `.cursor/mcp.json` en la raíz de tu proyecto, o global `~/.cursor/mcp.json`
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

Después de guardar, reinicia la aplicación. El servidor MCP se descargará e iniciará automáticamente. Las siguientes herramientas estarán disponibles:

* **`current-time`** - Obtener la hora actual en tus zonas horarias configuradas
* **`local-time`** - Obtener la hora local actual en la zona horaria del usuario (desde la configuración `convtz` o el valor por defecto del sistema)
* **`convert-time`** - Convertir una cadena de fecha-hora o timestamp Epoch a múltiples zonas horarias
* **`next-weekday`** - Encontrar la fecha de la próxima ocurrencia de un día de la semana dado
* **`date-to-weekday`** - Obtener el día de la semana para una fecha dada
* **`days-until`** - Contar el número de días desde hoy hasta una fecha especificada
* **`days-between`** - Contar el número de días entre dos fechas
* **`date-offset`** - Calcular la fecha N días antes o después de una fecha dada

### Cómo funciona con la configuración de mclocks

El servidor MCP lee automáticamente tu `config.json` de mclocks y usa:

* **`clocks`** - Las zonas horarias definidas en tus relojes se usan como destinos de conversión por defecto
* **`convtz`** - Se usa como la zona horaria de origen por defecto al convertir cadenas de fecha-hora sin información de zona horaria
* **`usetz`** - Habilita la conversión estricta de zonas horarias para offsets UTC históricamente precisos (ej., JST era +09:18 antes de 1888). Establécelo en `true` cuando necesites convertir fechas-hora históricas con precisión

Si no se encuentra `config.json`, el servidor recurre a un conjunto integrado de zonas horarias comunes (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### Variables de entorno

Si deseas anular la configuración de `config.json`, o si no tienes un `config.json`, puedes establecer variables de entorno en tu configuración MCP. Las variables de entorno tienen prioridad sobre los valores de `config.json`.

| Variable | Descripción | Por defecto |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | Ruta a `config.json`. No es necesario en la mayoría de los casos, ya que el servidor detecta la ubicación automáticamente. | detección automática |
| `MCLOCKS_LOCALE` | Locale para formatear nombres de días de la semana, etc. (ej., `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Zona horaria de origen por defecto para interpretar cadenas de fecha-hora sin información de zona horaria (ej., `Asia/Tokyo`) | *(ninguno)* |
| `MCLOCKS_USETZ` | Establecer en `true` para habilitar la conversión estricta de zonas horarias | `false` |

Ejemplo:

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

### Ejemplo de uso

Una vez configurado, puedes preguntarle a tu asistente de IA cosas como:

* "¿Qué hora es?" - Devuelve la hora actual en todas tus zonas horarias configuradas en mclocks
* "¿Qué hora es en Yakarta?" - Devuelve la hora actual en una zona horaria específica
* "Convierte el epoch 1705312200 a fecha-hora"
* "Convierte 2024-01-15T10:30:00Z a Asia/Tokyo"
* "¿Qué fecha es el próximo viernes?"
* "¿Qué día de la semana es el 25 de diciembre de 2026?"
* "¿Cuántos días faltan para Navidad?"
* "¿Cuántos días hay entre el 1 de enero de 2026 y el 31 de diciembre de 2026?"
* "¿Qué fecha es 90 días después del 1 de abril de 2026?"

----------

## Licencia

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Autor

Dai Okabayashi: https://github.com/bayashi
