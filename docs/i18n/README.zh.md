# mclocks

多时区桌面时钟应用程序🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

时钟相关功能：

* 🕐 多时区文本时钟
* ⏱️ 计时器
* ⏳ 倒计时器
* 🔄 Epoch 时间与日期时间转换器

时间不等人：

* 📝 便签

开发者离不开时钟：

* 🔀 简单文本转换
    * 例如轻松创建 SQL `IN` 子句
* 🌐 Web 服务器
    * 提供静态文件服务
        * 丰富渲染 Markdown
        * 基于拖放的内容查看器
    * 请求和响应转储服务器
    * 用于调试的慢速端点
    * 从 GitHub URL 在编辑器中打开文件

🔔 注意：`mclocks` 不需要互联网连接——一切都在本地 100% 运行。

## 📦 下载

从 https://github.com/bayashi/mclocks/releases 下载

### Windows

Windows 用户可以获取安装程序 `.msi` 文件或可执行文件 `.exe`。

### macOS

macOS 用户可以获取 `.dmg` 文件进行安装。

（本文档中的快捷键适用于 Windows 操作系统。如果您使用 macOS，请相应替换按键，例如将 `Ctrl` 替换为 `Ctrl + Command`，将 `Alt` 替换为 `Option`。）

## ⚙️ config.json

`config.json` 文件允许您根据偏好配置时钟。

`config.json` 文件应位于以下目录：

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

启动 `mclocks` 后，按 `Ctrl + o` 编辑您的 `config.json` 文件。

### config.json 示例

`config.json` 文件应按如下所示的 JSON 格式编写。

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

## 🔧 config.json 的各字段

#### clocks

`clocks` 字段是一个对象数组，每个对象包含 `name` 和 `timezone` 属性，两者均为字符串。默认值均为 `UTC`。

* `name` 是时钟显示的标签。
* 选择时区请参考此[时区列表](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones)。

以下是包含三个时区的 `clocks` 数组示例。

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

字符串：`MM-DD ddd HH:mm`

`format` 字段是用于显示时钟的日期时间格式字符串。要创建自定义日期时间格式，请参考[此格式指南](https://momentjs.com/docs/#/parsing/string-format/)。

#### format2

字符串：`""`

`format2` 字段与 `format` 相同。可以通过 `Ctrl + f` 键相互切换。`format2` 是可选字段。

#### locale

字符串：`en`

`locale` 字段决定日期时间显示的语言设置。您可以在[此处找到支持的语言环境列表](https://github.com/kawanet/cdate-locale/blob/main/locales.yml)。

#### color

字符串：`#fff`

`color` 字段定义日期时间文本的颜色。您可以使用颜色名称、RGB 十六进制值、RGB 值（例如 `RGB(255, 0, 0)`）或任何有效的 CSS 颜色值。

#### font

字符串：`Courier, monospace`

`font` 是用于显示日期时间的字体名称。应使用等宽字体。如果设置非等宽字体，mclocks 可能会出现不理想的抖动效果。

#### size

数字 | 字符串：14

`size` 是日期时间的字符大小，以像素为单位。也可以指定为包含单位的字符串（例如 `"125%"`、`"1.5em"`）。

#### margin

字符串：`1.65em`

`margin` 字段决定时钟之间的间距。

#### forefront

布尔值：`false`

如果将 `forefront` 字段设置为 `true`，mclocks 应用程序将始终显示在其他应用程序窗口的最前面。

## ⏳ 倒计时时钟

通过如下配置 `clock`，它将显示为到指定 `target` 日期时间的倒计时时钟。

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

上述倒计时 `clock` 将显示如下：

    WAC Tokyo D-159 12:34:56

表示距离 2025 年 9 月 13 日还剩 159 天 12 小时 34 分 56 秒。

### 倒计时格式占位符

`countdown` 字段文本接受以下模板占位符：

* `%TG`：目标日期时间字符串
* `%D`：到目标日期时间的剩余天数
* `%H`：到目标日期时间的剩余小时数
* `%h`：剩余时间的"时"部分（hh:mm:ss 中的 hh）
* `%M`：到目标日期时间的剩余分钟数
* `%m`：剩余时间的"分"部分（hh:mm:ss 中的 mm）
* `%S`：到目标日期时间的剩余秒数
* `%s`：剩余时间的"秒"部分（hh:mm:ss 中的 ss）

## ⏱️ 简单计时器

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

点击 `mclocks` 应用窗口，然后按 `Ctrl + 1` 键启动 1 分钟计时器。按 `Ctrl + Alt + 1` 键启动 10 分钟计时器。其他数字键同样适用。最多可同时启动 5 个计时器。

`Ctrl + p` 暂停/恢复计时器。

`Ctrl + 0` 删除最早的计时器。`Ctrl + Alt + 0` 删除最新的计时器。

🔔 注意：倒计时时钟和简单计时器在完成时默认会发送通知。如果不需要通知，请在 `config.json` 中设置 `withoutNotification: true`。

## 🔢 显示 Epoch 时间

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

点击 `mclocks` 应用窗口，然后按 `Ctrl + e` 键切换 Epoch 时间的显示。

## 🔄 日期时间与 Epoch 时间的转换

点击 `mclocks` 应用窗口，然后粘贴日期时间或 Epoch 时间，将弹出显示转换结果的对话框。您可以将结果复制到剪贴板。如果不想复制，按 `[No]` 关闭对话框即可。

使用 `Ctrl + v` 粘贴时，值（Epoch 时间）被视为秒。使用 `Ctrl + Alt + v` 时被视为毫秒，`Ctrl + Alt + Shift + V` 时被视为微秒，`Ctrl + Alt + Shift + N + V` 时被视为纳秒并进行相应转换。

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

当粘贴的日期时间值不包含时区信息时，默认按本地时区处理。要按特定时区处理，请在 convtz 选项中设置时区。

    "convtz": "UTC"

## 🔀 文本转换功能

点击 `mclocks` 应用窗口，然后使用以下键盘快捷键处理剪贴板文本并在编辑器中打开：

* `Ctrl + i`：将剪贴板的每行用双引号括起来，并在末尾添加逗号（最后一行除外）
* `Ctrl + Shift + i`：在每行末尾添加逗号（不加引号），用于 INT 列表的 IN 条件（最后一行除外）

空行在所有操作中保持原样。

（此文本转换功能与时钟或时间无关，但软件开发者可能会觉得很方便！😊）

## ⌨️ 键盘快捷键

### 显示帮助

`F1`（Windows）或 `Cmd + Shift + /`（macOS）在浏览器中打开帮助页面（此 README）

### 配置、显示格式

| 快捷键 | 说明 |
|----------|-------------|
| `Ctrl + o` | 在编辑器中打开 `config.json` 文件 |
| `Ctrl + f` | 在 `format` 和 `format2` 之间切换（如果在 `config.json` 中定义了 `format2`） |
| `Ctrl + e` 或 `Ctrl + u` | 切换 Epoch 时间的显示 |

### 计时器

| 快捷键 | 说明 |
|----------|-------------|
| `Ctrl + 1` 到 `Ctrl + 9` | 启动计时器（1 分钟 × 数字键） |
| `Ctrl + Alt + 1` 到 `Ctrl + Alt + 9` | 启动计时器（10 分钟 × 数字键） |
| `Ctrl + p` | 暂停/恢复所有计时器 |
| `Ctrl + 0` | 删除最早的计时器 |
| `Ctrl + Alt + 0` | 删除最新的计时器 |

### 便签

| 快捷键 | 说明 |
|----------|-------------|
| `Ctrl + s` | 从剪贴板文本创建新便签 |

### 剪贴板日期时间操作

| 快捷键 | 说明 |
|----------|-------------|
| `Ctrl + c` | 将当前 mclocks 文本复制到剪贴板 |
| `Ctrl + v` | 转换剪贴板内容（Epoch 时间按秒处理，或日期时间） |
| `Ctrl + Alt + v` | 转换剪贴板内容（Epoch 时间按毫秒处理） |
| `Ctrl + Alt + Shift + V` | 转换剪贴板内容（Epoch 时间按微秒处理） |
| `Ctrl + Alt + Shift + N + V` | 转换剪贴板内容（Epoch 时间按纳秒处理） |

### 文本转换

| 快捷键 | 说明 |
|----------|-------------|
| `Ctrl + i` | 将剪贴板的每行用双引号括起来，末尾添加逗号，在编辑器中打开（最后一行除外） |
| `Ctrl + Shift + i` | 在每行末尾添加逗号（不加引号，用于 INT 列表的 IN 条件），在编辑器中打开（最后一行除外） |

## 📝 便签

点击 `mclocks` 应用窗口，然后按 `Ctrl + s` 从剪贴板文本创建便签。将打开一个显示剪贴板内容的小型浮动窗口。

每个便签具有以下功能：

* **切换按钮** (`▸` / `▾`)：展开或折叠便签。折叠模式下仅显示一行。
* **复制按钮** (`⧉`)：将便签文本复制到剪贴板。
* **置顶按钮** (`⊤` / `⊥`)：切换便签是否始终显示在其他窗口之上。此设置按便签单独保存。
* **关闭按钮** (`✖`)：删除便签并关闭其窗口。
* **文本区域**：自由编辑便签内容。更改会自动保存。
* **调整大小手柄**：展开时拖动右下角可调整便签大小。

便签继承 `config.json` 中的 `font`、`size`、`color` 和 `forefront` 设置。置顶设置可通过置顶按钮按便签单独覆盖；如果未覆盖，则使用 `config.json` 中的值。位置、大小、展开/折叠状态和置顶覆盖设置会被持久化，`mclocks` 重启时所有便签会自动恢复。

🔔 注意：在 macOS 上，便签窗口位置仅在应用程序退出时保存。在 Windows 上，移动或调整窗口大小时会自动保存位置。

每个便签的最大文本大小为 128 KB。

## 🌐 Web 服务器

`mclocks` 可以通过内置 Web 服务器提供静态文件服务。此功能使您可以轻松在浏览器中查看代码片段。在 `config.json` 中添加 `web` 字段：

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

* `root`：包含要提供的文件的目录路径（必需）
* `port`：监听的端口号（默认：3030）
* `openBrowserAtStart`：如果设置为 `true`，`mclocks` 启动时自动在默认浏览器中打开 Web 服务器 URL（默认：`false`）
* `dump`：如果设置为 `true`，启用以 JSON 返回请求详情的 `/dump` 端点（默认：`false`）
* `slow`：如果设置为 `true`，启用延迟响应的 `/slow` 端点（默认：`false`）
* `status`：如果设置为 `true`，启用返回任意 HTTP 状态码的 `/status/{code}` 端点（默认：`false`）
* `content.markdown.allowRawHTML`：如果设置为 `true`，允许在 Markdown 渲染中使用原始 HTML；如果为 `false`，Markdown 中的原始 HTML 会被转义为文本（默认：`false`）
* `editor`：如果设置并包含 `reposDir`，启用从浏览器的 GitHub URL 在编辑器中打开本地文件的 `/editor` 端点（默认：未设置）

如果 `config.json` 中配置了 `web` 字段，Web 服务器将在 `mclocks` 启动时自动开始。通过 `http://127.0.0.1:3030` 访问文件。Web 服务器仅在 `127.0.0.1`（localhost）上监听，因此只能从本地机器访问。

### 支持的文件类型

Web 服务器支持以下文件类型：

* 文本：`html`、`css`、`js`、`json`、`md`、`txt`
* 图片：`png`、`jpg`、`jpeg`、`gif`、`svg`、`ico`

### 基于拖放的内容查看器

除了静态文件托管之外，Web 服务器还包含基于拖放的内容查看流程：当您将文件或目录拖放到 mclocks 时钟窗口上时，可以通过临时本地 URL 打开并查看。
这些临时 URL 会在 mclocks 退出时被丢弃。

### /dump 端点

当 `web` 配置中设置了 `dump: true` 时，Web 服务器提供以 JSON 返回请求详情的 `/dump` 端点。

端点返回包含以下内容的 JSON 对象：
* `method`：HTTP 方法（例如："GET"、"POST"）
* `path`：`/dump/` 之后的请求路径（例如：`/dump/test` 的路径为 "/test"）
* `query`：查询参数的键值对象数组（例如：`[{"key1": "value1"}, {"key2": "value2"}]`）
* `headers`：请求头的键值对象数组（例如：`[{"Content-Type": "application/json"}]`）
* `body`：请求体字符串（如果存在）
* `parsed_body`：如果 Content-Type 为 JSON 则为解析后的 JSON 对象，解析失败则为错误消息字符串

通过 `http://127.0.0.1:3030/dump` 或 `/dump/` 下的任意路径（例如 `/dump/test?key=value`）访问。

### /slow 端点

当 `web` 配置中设置了 `slow: true` 时，Web 服务器提供在返回 200 OK 之前延迟响应的 `/slow` 端点。

此端点可通过任何 HTTP 方法（GET、POST 等）访问，支持以下路径：

* `/slow`：等待 30 秒（默认）后返回 200 OK
* `/slow/120`：等待 120 秒（或任何指定的秒数）后返回 200 OK

最大允许值为 901 秒（15 分钟 + 1 秒）。超过此限制的请求将返回 400 Bad Request 错误。

此端点适用于测试超时行为、连接处理或模拟慢速网络状况。

如果提供了无效的秒数参数（例如 `/slow/abc`），端点将返回 400 Bad Request 错误。

### /status 端点

当 `web` 配置中设置了 `status: true` 时，Web 服务器提供返回 RFC 标准中定义的任意 HTTP 状态码（100-599）的 `/status/{code}` 端点。

端点返回包含状态码及其对应短语的纯文本响应，并根据 HTTP 规范包含适当的头部。

**示例：**
* `http://127.0.0.1:3030/status/200` - 返回 200 OK
* `http://127.0.0.1:3030/status/404` - 返回 404 Not Found
* `http://127.0.0.1:3030/status/500` - 返回 500 Internal Server Error
* `http://127.0.0.1:3030/status/418` - 返回 418 I'm a teapot（带有特殊消息）
* `http://127.0.0.1:3030/status/301` - 返回 301 Moved Permanently（带有 Location 头部）

**特定状态码的头部：**

端点会自动为特定状态码添加适当的头部：

* **3xx 重定向** (301, 302, 303, 305, 307, 308)：添加 `Location` 头部
* **401 Unauthorized**：添加 `WWW-Authenticate` 头部
* **405 Method Not Allowed**：添加 `Allow` 头部
* **407 Proxy Authentication Required**：添加 `Proxy-Authenticate` 头部
* **416 Range Not Satisfiable**：添加 `Content-Range` 头部
* **426 Upgrade Required**：添加 `Upgrade` 头部
* **429 Too Many Requests**：添加 `Retry-After` 头部（60 秒）
* **503 Service Unavailable**：添加 `Retry-After` 头部（60 秒）
* **511 Network Authentication Required**：添加 `WWW-Authenticate` 头部

**响应体处理：**

* **204 No Content** 和 **304 Not Modified**：返回空响应体（符合 HTTP 规范）
* **418 I'm a teapot**：返回特殊消息 "I'm a teapot" 而非标准状态短语
* **所有其他状态码**：返回 `{code} {phrase}` 格式的纯文本（例如："404 Not Found"）

此端点适用于测试应用程序如何处理不同的 HTTP 状态码、错误处理、重定向、身份验证要求和速率限制场景。

### /editor 端点

当配置文件中设置了 `web.editor.reposDir` 时，Web 服务器提供 `/editor` 端点，允许您从浏览器的 GitHub URL 直接在编辑器中打开本地文件。

**配置：**

在 `web` 配置中添加以下内容：

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

* `reposDir`：本地仓库目录路径。支持使用 `~` 展开主目录（例如：macOS 上的 `"~/repos"` 或 Windows 上的 `"C:/Users/username/repos"`）。此目录必须存在。
* `includeHost`：如果为 `true`，本地路径解析时将原始主机作为目录包含（例如 `{reposDir}/{host}/{owner}/{repo}/...`）。如果为 `false`，则解析为 `{reposDir}/{owner}/{repo}/...`（默认：`false`）。
* `command`：编辑器可执行文件的命令名或路径（默认：`code`）
* `args`：参数模板数组。使用 `{file}` 和 `{line}` 占位符。如果 URL 中没有 `#L...`，`{line}` 使用 1。

**工作原理：**

1. 通过 `/editor` 端点访问 GitHub 文件 URL 时，它会将 GitHub 路径转换为本地文件路径
2. 本地文件路径构建为：`{reposDir}/{owner}/{repository_name}/{file_path}`
3. 如果文件存在，使用配置的命令和参数在指定行号处用编辑器打开文件（默认：`code -g {local_file_path}:{line_number}`）
4. 如果文件不存在，将显示带有克隆仓库链接的错误页面

**书签小工具：**

创建书签小工具以快速在本地编辑器中打开 GitHub 文件。将 `3030` 替换为您配置的端口号：

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**行号支持：**

您可以使用 URL 的哈希片段指定行号：
* `https://github.com/username/repo/blob/main/file.rs#L123` → 在第 123 行打开

**错误处理：**

* 如果文件在本地不存在，标签页保持打开并显示带有从 GitHub 克隆仓库链接的错误消息
* 如果文件成功打开，标签页自动关闭
* 如果 `web.editor.reposDir` 未配置或不存在，`/editor` 端点不会启用（将返回 404）

**示例：**

1. 您正在 GitHub 上浏览文件：`https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. 点击书签小工具或手动导航到：`http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. 如果本地存在 `~/repos/mclocks/src/app.js`，VS Code 将在第 42 行打开它
4. 如果文件不存在，将显示带有 `https://github.com/bayashi/mclocks` 链接的错误页面以供克隆

----------

## 🧠 mclocks MCP 服务器

`mclocks` 包含一个 MCP（Model Context Protocol）服务器，使 [Cursor](https://www.cursor.com/) 和 [Claude Desktop](https://claude.ai/download) 等 AI 助手能够跨多个时区回答"现在几点？"，并在日期时间格式和 Epoch 时间戳之间进行转换。MCP 服务器会自动使用您的 mclocks `config.json`，因此您在 mclocks 中配置的时区会反映在 AI 的响应中。

### 前提条件

* [Node.js](https://nodejs.org/)（v18 或更高版本）

如果您没有安装 Node.js，请从官方网站安装。

### 设置

将以下 JSON 添加到您的 MCP 配置文件中：

* **Cursor**：项目根目录的 `.cursor/mcp.json`，或全局的 `~/.cursor/mcp.json`
* **Claude Desktop** (`claude_desktop_config.json`)：
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

保存后，重启应用程序。MCP 服务器将自动下载并启动。以下工具将可用：

* **`current-time`** - 获取您配置的时区中的当前时间
* **`local-time`** - 获取用户时区（来自 `convtz` 配置或系统默认值）的当前本地时间
* **`convert-time`** - 将日期时间字符串或 Epoch 时间戳转换为多个时区
* **`next-weekday`** - 查找指定星期几的下一个日期
* **`date-to-weekday`** - 获取指定日期是星期几
* **`days-until`** - 计算从今天到指定日期的天数
* **`days-between`** - 计算两个日期之间的天数
* **`date-offset`** - 计算指定日期前后 N 天的日期

### 与 mclocks config 的配合

MCP 服务器自动读取您的 mclocks `config.json` 并使用：

* **`clocks`** - 时钟中定义的时区作为默认转换目标
* **`convtz`** - 转换不含时区信息的日期时间字符串时用作默认源时区
* **`usetz`** - 启用历史精确的 UTC 偏移量的严格时区转换（例如：JST 在 1888 年之前为 +09:18）。当您需要准确转换历史日期时间时，设置为 `true`

如果未找到 `config.json`，服务器将回退到内置的常用时区集（UTC、America/New_York、America/Los_Angeles、Europe/London、Europe/Berlin、Asia/Tokyo、Asia/Shanghai、Asia/Kolkata、Australia/Sydney）。

### 环境变量

如果您想覆盖 `config.json` 设置，或者根本没有 `config.json`，可以在 MCP 配置中设置环境变量。环境变量优先于 `config.json` 中的值。

| 变量 | 说明 | 默认值 |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | `config.json` 的路径。大多数情况下不需要，因为服务器会自动检测位置。 | 自动检测 |
| `MCLOCKS_LOCALE` | 用于格式化星期名称等的语言环境（例如 `ja`、`pt`、`de`） | `en` |
| `MCLOCKS_CONVTZ` | 解释不含时区信息的日期时间字符串时的默认源时区（例如 `Asia/Tokyo`） | *（无）* |
| `MCLOCKS_USETZ` | 设置为 `true` 以启用严格时区转换 | `false` |

示例：

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

### 使用示例

配置完成后，您可以向 AI 助手提问：

* "现在几点？"- 返回您在 mclocks 中配置的所有时区的当前时间
* "雅加达现在几点？"- 返回特定时区的当前时间
* "将 Epoch 1705312200 转换为日期时间"
* "将 2024-01-15T10:30:00Z 转换为 Asia/Tokyo"
* "下个星期五是几号？"
* "2026 年 12 月 25 日是星期几？"
* "距离圣诞节还有几天？"
* "2026 年 1 月 1 日到 2026 年 12 月 31 日之间有多少天？"
* "2026 年 4 月 1 日之后 90 天是几号？"

----------

## 许可证

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## 作者

Dai Okabayashi: https://github.com/bayashi
