# mclocks

複数タイムゾーン対応のデスクトップ時計アプリケーション🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

時計関連の機能:

* 🕐 複数タイムゾーン対応のテキスト時計
* ⏱️ タイマー
* ⏳ カウントダウンタイマー
* 🔄 エポック時間と日時の変換

時間は待ってくれない:

* 📝 付箋メモ

開発者に時計は欠かせない:

* 🔀 シンプルなテキスト変換
    * SQL の `IN` 句の簡単な作成など
* 🌐 Web サーバー
    * 静的ファイルの配信
        * Markdownをリッチにレンダリング
        * ドラッグ＆ドロップベースのコンテンツビューアー
    * リクエスト・レスポンスのダンプサーバー
    * デバッグ用の遅延エンドポイント
    * GitHub URL からエディタでファイルを開く

🔔 注意: `mclocks` はインターネット接続を必要としません。すべて100%ローカルで動作します。

## 📦 ダウンロード

https://github.com/bayashi/mclocks/releases からダウンロードできます。

### Windows

Windows 向けには、インストーラーの `.msi` ファイル、または実行ファイル `.exe` を入手できます。

### macOS

macOS 向けには、インストール用の `.dmg` ファイルを入手できます。

（このドキュメントのショートカットキーは Windows OS 向けです。macOS をお使いの場合は、`Ctrl` を `Command` に、`Alt` を `Option` に読み替えてください。）

## ⚙️ config.json

`config.json` ファイルで時計の設定をカスタマイズできます。

`config.json` ファイルは以下のディレクトリに配置します:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

`mclocks` を起動し、`Ctrl + o` を押すと `config.json` ファイルを編集できます。

### config.json の例

`config.json` ファイルは以下のように JSON 形式で記述します。

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

## 🔧 config.json の各フィールド

#### clocks

`clocks` フィールドは、`name` と `timezone` プロパティを含むオブジェクトの配列です。どちらも文字列で指定します。デフォルトはどちらも `UTC` です。

* `name` は時計に表示されるラベルです。
* タイムゾーンの選択については、こちらの[タイムゾーン一覧](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones)を参照してください。

以下は3つのタイムゾーンを設定した `clocks` 配列の例です。

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

文字列: `MM-DD ddd HH:mm`

`format` フィールドは時計の表示に使用する日時フォーマット文字列です。カスタムフォーマットの作成については、[こちらのフォーマットガイド](https://momentjs.com/docs/#/parsing/string-format/)を参照してください。

#### format2

文字列: `""`

`format2` フィールドは `format` と同じ書式です。`Ctrl + f` キーで相互に切り替えできます。`format2` はオプションのフィールドです。

#### locale

文字列: `en`

`locale` フィールドは日時表示の言語設定を決定します。[サポートされているロケールの一覧はこちら](https://github.com/kawanet/cdate-locale/blob/main/locales.yml)です。

#### color

文字列: `#fff`

`color` フィールドは日時テキストの色を定義します。名前付きの色、RGB 16進値、RGB 値（例: `RGB(255, 0, 0)`）、または有効な CSS カラー値を使用できます。

#### font

文字列: `Courier, monospace`

`font` は日時表示に使用するフォント名です。等幅フォントを使用してください。可変幅フォントを設定すると、mclocks が不自然にちらつく可能性があります。

#### size

数値 | 文字列: 14

`size` は日時の文字サイズで、ピクセル単位で指定します。単位を含む文字列でも指定可能です（例: `"125%"`、`"1.5em"`）。

#### margin

文字列: `1.65em`

`margin` フィールドは時計間のスペースを決定します。

#### forefront

真偽値: `false`

`forefront` フィールドを `true` に設定すると、mclocks アプリケーションは常に他のウィンドウの最前面に表示されます。

## ⏳ カウントダウン時計

以下のように `clock` の設定を行うと、指定した `target` 日時へのカウントダウン時計として表示されます。

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

上記のカウントダウン `clock` は以下のように表示されます:

    WAC Tokyo D-159 12:34:56

2025年9月13日まで残り159日12時間34分56秒であることを示しています。

### カウントダウンフォーマットの書式

`countdown` フィールドのテキストでは以下のテンプレート書式が使用できます:

* `%TG`: ターゲットの日時文字列
* `%D`: ターゲット日時までの残り日数
* `%H`: ターゲット日時までの残り時間（時間単位）
* `%h`: 残り時間の「時」部分（hh:mm:ss の hh）
* `%M`: ターゲット日時までの残り時間（分単位）
* `%m`: 残り時間の「分」部分（hh:mm:ss の mm）
* `%S`: ターゲット日時までの残り時間（秒単位）
* `%s`: 残り時間の「秒」部分（hh:mm:ss の ss）

## ⏱️ シンプルタイマー

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

`mclocks` のウィンドウをクリックし、`Ctrl + 1` キーを押すと1分タイマーが開始します。`Ctrl + Alt + 1` キーで10分タイマーが開始します。他の数字キーも同様に動作します。タイマーは最大5つまで同時に起動できます。

`Ctrl + p` でタイマーの一時停止・再開ができます。

`Ctrl + 0` で最も古いタイマーを削除します。`Ctrl + Alt + 0` で最も新しいタイマーを削除します。

🔔 注意: カウントダウン時計とシンプルタイマーは、タイマー完了時にデフォルトで通知を送信します。通知が不要な場合は、`config.json` で `withoutNotification: true` を設定してください。

## 🔢 エポック時間の表示

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

`mclocks` のウィンドウをクリックし、`Ctrl + e` キーを押すとエポック時間の表示を切り替えられます。

## 🔄 日時とエポック時間の変換

`mclocks` のウィンドウをクリックし、日時またはエポック時間を貼り付けると、変換結果を表示するダイアログが表示されます。結果はクリップボードにコピーできます。コピーしない場合は `[No]` を押してダイアログを閉じてください。

`Ctrl + v` で貼り付けた場合、値（エポック時間）は秒として扱われます。`Ctrl + Alt + v` ではミリ秒、`Ctrl + Alt + Shift + V` ではマイクロ秒、`Ctrl + Alt + Shift + N + V` ではナノ秒として変換されます。

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

貼り付けた日時値にタイムゾーン情報が含まれていない場合、デフォルトではローカルタイムゾーンとして扱われます。特定のタイムゾーンとして扱いたい場合は、convtz オプションにタイムゾーンを設定してください。

    "convtz": "UTC"

## 🔀 テキスト変換機能

`mclocks` のウィンドウをクリックし、以下のキーボードショートカットでクリップボードのテキストを加工してエディタで開きます:

* `Ctrl + i`: クリップボードの各行をダブルクォートで囲み、末尾にカンマを追加します（最終行を除く）
* `Ctrl + Shift + i`: 各行の末尾にカンマを追加します（クォートなし、INT リストの IN 条件用、最終行を除く）

空行はすべての操作でそのまま保持されます。

（このテキスト変換機能は時計や時刻とは関係ありませんが、ソフトウェア開発者には便利かもしれません！😊）

## ⌨️ キーボードショートカット

### ヘルプの表示

`F1`（Windows）または `Cmd + Shift + /`（macOS）でブラウザにヘルプページ（この README）を開きます。

### 設定・表示フォーマット

| ショートカット | 説明 |
|----------|-------------|
| `Ctrl + o` | `config.json` ファイルをエディタで開く |
| `Ctrl + f` | `format` と `format2` を切り替え（`config.json` で `format2` が定義されている場合） |
| `Ctrl + e` または `Ctrl + u` | エポック時間の表示を切り替え |

### タイマー

| ショートカット | 説明 |
|----------|-------------|
| `Ctrl + 1` ～ `Ctrl + 9` | タイマー開始（1分 × 数字キー） |
| `Ctrl + Alt + 1` ～ `Ctrl + Alt + 9` | タイマー開始（10分 × 数字キー） |
| `Ctrl + p` | すべてのタイマーを一時停止 / 再開 |
| `Ctrl + 0` | 最も古いタイマー（左端）を削除 |
| `Ctrl + Alt + 0` | 最も新しいタイマー（右端）を削除 |

### 付箋メモ

| ショートカット | 説明 |
|----------|-------------|
| `Ctrl + s` | クリップボードのテキストから新しい付箋メモを作成 |

### クリップボードの日時操作

| ショートカット | 説明 |
|----------|-------------|
| `Ctrl + c` | 現在の mclocks テキストをクリップボードにコピー |
| `Ctrl + v` | クリップボードの内容を変換（エポック時間を秒として、または日時） |
| `Ctrl + Alt + v` | クリップボードの内容を変換（エポック時間をミリ秒として） |
| `Ctrl + Alt + Shift + V` | クリップボードの内容を変換（エポック時間をマイクロ秒として） |
| `Ctrl + Alt + Shift + N + V` | クリップボードの内容を変換（エポック時間をナノ秒として） |

### テキスト変換

| ショートカット | 説明 |
|----------|-------------|
| `Ctrl + i` | クリップボードの各行をダブルクォートで囲み、末尾にカンマを追加してエディタで開く（最終行を除く） |
| `Ctrl + Shift + i` | 各行の末尾にカンマを追加（クォートなし、INT リストの IN 条件用）してエディタで開く（最終行を除く） |

## 📝 付箋メモ

`mclocks` のウィンドウをクリックし、`Ctrl + s` を押すとクリップボードのテキストから付箋メモを作成します。クリップボードの内容を表示する小さなフローティングウィンドウが開きます。

各付箋メモには以下の機能があります:

* **トグルボタン** (`▸` / `▾`): メモの展開・折りたたみ。折りたたみ時は1行のみ表示されます。
* **コピーボタン** (`⧉`): メモのテキストをクリップボードにコピーします。
* **最前面ボタン** (`⊤` / `⊥`): メモを他のウィンドウの上に常に表示するかどうかを切り替えます。この設定は付箋メモごとに保存されます。
* **閉じるボタン** (`✖`): 付箋メモを削除し、ウィンドウを閉じます。
* **テキストエリア**: メモの内容を自由に編集できます。変更は自動保存されます。
* **リサイズハンドル**: 展開時に右下隅をドラッグしてメモのサイズを変更できます。

付箋メモは `config.json` の `font`、`size`、`color`、`forefront` の設定を継承します。最前面の設定は付箋メモごとに最前面ボタンで上書きでき、上書きしない場合は `config.json` の値が使用されます。位置、サイズ、開閉状態、最前面の上書き設定は保持され、`mclocks` の再起動時にすべてのメモが自動的に復元されます。

🔔 注意: macOS では、付箋メモのウィンドウ位置はアプリケーション終了時にのみ保存されます。Windows では、ウィンドウの移動やリサイズ時に自動的に保存されます。

付箋メモ1つあたりの最大テキストサイズは 128 KB です。

## 🌐 Web サーバー

`mclocks` は内蔵の Web サーバーで静的ファイルを配信できます。この機能を使えば、コードスニペットをブラウザで簡単に確認できます。`config.json` に `web` フィールドを追加してください:

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

* `root`: 配信するファイルが格納されたディレクトリのパス（必須）
* `port`: リッスンするポート番号（デフォルト: 3030）
* `open_browser_at_start`: `true` に設定すると、`mclocks` 起動時にデフォルトブラウザで Web サーバーの URL を自動的に開きます（デフォルト: `false`）
* `dump`: `true` に設定すると、リクエストの詳細を JSON で返す `/dump` エンドポイントが有効になります（デフォルト: `false`）
* `slow`: `true` に設定すると、レスポンスを遅延させる `/slow` エンドポイントが有効になります（デフォルト: `false`）
* `status`: `true` に設定すると、任意の HTTP ステータスコードを返す `/status/{code}` エンドポイントが有効になります（デフォルト: `false`）
* `content.markdown.allowRawHTML`: `true` に設定すると Markdown レンダリング時に生の HTML を許可します。`false` の場合は Markdown 内の生 HTML はテキストとしてエスケープ表示されます（デフォルト: `false`）
* `editor`: `reposDir` を含む場合、ブラウザの GitHub URL からローカルファイルをエディタで開く `/editor` エンドポイントが有効になります（デフォルト: 未設定）

`config.json` に `web` フィールドが設定されている場合、`mclocks` の起動時に Web サーバーが自動的に開始されます。`http://127.0.0.1:3030` でファイルにアクセスできます。Web サーバーは `127.0.0.1`（localhost）のみでリッスンするため、ローカルマシンからのみアクセス可能です。

### サポートされるファイルタイプ

Web サーバーは以下のファイルタイプをサポートしています:

* テキスト: `html`、`css`、`js`、`json`、`md`、`txt`
* 画像: `png`、`jpg`、`jpeg`、`gif`、`svg`、`ico`

### ドラッグ＆ドロップベースのコンテンツビューアー

静的ファイル配信に加えて、Web サーバーにはドラッグ＆ドロップベースのコンテンツビューアー機能があります。mclocks の時計ウィンドウにファイルまたはディレクトリをドラッグ＆ドロップすると、一時的なローカルURLで開いて閲覧できます。
これらの一時URLは mclocks の終了時に破棄されます。

### /dump エンドポイント

`web` 設定で `dump: true` が設定されている場合、リクエストの詳細を JSON で返す `/dump` エンドポイントが利用可能になります。

エンドポイントは以下の内容を含む JSON オブジェクトを返します:
* `method`: HTTP メソッド（例: "GET"、"POST"）
* `path`: `/dump/` 以降のリクエストパス（例: `/dump/test` の場合は "/test"）
* `query`: クエリパラメータのキーバリューオブジェクトの配列（例: `[{"key1": "value1"}, {"key2": "value2"}]`）
* `headers`: リクエストヘッダーのキーバリューオブジェクトの配列（例: `[{"Content-Type": "application/json"}]`）
* `body`: リクエストボディの文字列（存在する場合）
* `parsed_body`: Content-Type が JSON の場合はパースされた JSON オブジェクト、パースに失敗した場合はエラーメッセージの文字列

`http://127.0.0.1:3030/dump` または `/dump/` 配下の任意のパス（例: `/dump/test?key=value`）でアクセスできます。

### /slow エンドポイント

`web` 設定で `slow: true` が設定されている場合、レスポンスを遅延させてから 200 OK を返す `/slow` エンドポイントが利用可能になります。

このエンドポイントは任意の HTTP メソッド（GET、POST など）でアクセスでき、以下のパスをサポートします:

* `/slow`: 30秒（デフォルト）待機後、200 OK を返します
* `/slow/120`: 120秒（または指定した秒数）待機後、200 OK を返します

最大値は 901秒（15分+1秒）です。この制限を超えるリクエストは 400 Bad Request エラーを返します。

このエンドポイントは、タイムアウト動作、接続処理、または低速なネットワーク状況のシミュレーションのテストに便利です。

無効な秒数パラメータが指定された場合（例: `/slow/abc`）、エンドポイントは 400 Bad Request エラーを返します。

### /status エンドポイント

`web` 設定で `status: true` が設定されている場合、RFC 標準で定義された任意の HTTP ステータスコード（100-599）を返す `/status/{code}` エンドポイントが利用可能になります。

エンドポイントは、ステータスコードとそれに対応するフレーズをプレーンテキストで返し、HTTP 仕様に従った適切なヘッダーを含めます。

**例:**
* `http://127.0.0.1:3030/status/200` - 200 OK を返します
* `http://127.0.0.1:3030/status/404` - 404 Not Found を返します
* `http://127.0.0.1:3030/status/500` - 500 Internal Server Error を返します
* `http://127.0.0.1:3030/status/418` - 418 I'm a teapot を返します（特別なメッセージ付き）
* `http://127.0.0.1:3030/status/301` - 301 Moved Permanently を返します（Location ヘッダー付き）

**ステータス固有のヘッダー:**

エンドポイントは特定のステータスコードに対して適切なヘッダーを自動的に追加します:

* **3xx リダイレクト** (301, 302, 303, 305, 307, 308): `Location` ヘッダーを追加
* **401 Unauthorized**: `WWW-Authenticate` ヘッダーを追加
* **405 Method Not Allowed**: `Allow` ヘッダーを追加
* **407 Proxy Authentication Required**: `Proxy-Authenticate` ヘッダーを追加
* **416 Range Not Satisfiable**: `Content-Range` ヘッダーを追加
* **426 Upgrade Required**: `Upgrade` ヘッダーを追加
* **429 Too Many Requests**: `Retry-After` ヘッダーを追加（60秒）
* **503 Service Unavailable**: `Retry-After` ヘッダーを追加（60秒）
* **511 Network Authentication Required**: `WWW-Authenticate` ヘッダーを追加

**レスポンスボディの処理:**

* **204 No Content** と **304 Not Modified**: 空のレスポンスボディを返します（HTTP 仕様に準拠）
* **418 I'm a teapot**: 標準のステータスフレーズの代わりに特別なメッセージ "I'm a teapot" を返します
* **その他のステータスコード**: `{code} {phrase}` 形式のプレーンテキストを返します（例: "404 Not Found"）

このエンドポイントは、アプリケーションがさまざまな HTTP ステータスコード、エラー処理、リダイレクト、認証要件、レート制限シナリオをどのように処理するかをテストするのに便利です。

### /editor エンドポイント

設定ファイルで `web.editor.reposDir` が設定されている場合、ブラウザの GitHub URL からローカルファイルをエディタで直接開ける `/editor` エンドポイントが利用可能になります。

**設定:**

`web` 設定に以下を追加します:

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

* `reposDir`: ローカルリポジトリのディレクトリパス。ホームディレクトリの展開に `~` をサポートします（例: macOS では `"~/repos"`、Windows では `"C:/Users/username/repos"`）。このディレクトリは存在する必要があります。
* `includeHost`: `true` の場合、ローカルパスの解決に元のホストをディレクトリとして含めます（例: `{reposDir}/{host}/{owner}/{repo}/...`）。`false` の場合は `{reposDir}/{owner}/{repo}/...` に解決されます（デフォルト: `false`）。
* `command`: エディタの実行ファイルのコマンド名またはパス（デフォルト: `code`）
* `args`: 引数テンプレートの配列。`{file}` と `{line}` のプレースホルダーを使用します。URL に `#L...` がない場合、`{line}` は 1 になります。

**動作の仕組み:**

1. `/editor` エンドポイント経由で GitHub ファイル URL にアクセスすると、GitHub パスをローカルファイルパスに変換します
2. ローカルファイルパスは `{reposDir}/{owner}/{repository_name}/{file_path}` として構築されます
3. ファイルが存在する場合、設定されたコマンドと引数を使用して指定した行番号でエディタでファイルを開きます（デフォルト: `code -g {local_file_path}:{line_number}`）
4. ファイルが存在しない場合、リポジトリのクローン用リンク付きのエラーページが表示されます

**ブックマークレット:**

GitHub ファイルをローカルエディタで素早く開くためのブックマークレットを作成できます。`3030` を設定したポート番号に置き換えてください:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**行番号のサポート:**

URL のハッシュフラグメントで行番号を指定できます:
* `https://github.com/username/repo/blob/main/file.rs#L123` → 123行目で開きます

**エラー処理:**

* ファイルがローカルに存在しない場合、タブは開いたままで、GitHub からリポジトリをクローンするためのリンク付きエラーメッセージが表示されます
* ファイルが正常に開かれた場合、タブは自動的に閉じます
* `web.editor.reposDir` が設定されていないか存在しない場合、`/editor` エンドポイントは有効になりません（404 が返されます）

**例:**

1. GitHub でファイルを閲覧中: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. ブックマークレットをクリックするか、手動で以下にアクセス: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. ローカルに `~/repos/mclocks/src/app.js` が存在する場合、VS Code で42行目を開きます
4. ファイルが存在しない場合、`https://github.com/bayashi/mclocks` へのリンク付きエラーページが表示されます

----------

## 🧠 mclocks MCP サーバー

`mclocks` には、[Cursor](https://www.cursor.com/) や [Claude Desktop](https://claude.ai/download) などの AI アシスタントが複数のタイムゾーンで「今何時？」に答えたり、日時形式とエポックタイムスタンプの間で変換したりできる MCP（Model Context Protocol）サーバーが含まれています。MCP サーバーは mclocks の `config.json` を自動的に使用するため、mclocks で設定したタイムゾーンが AI の応答に反映されます。

### 前提条件

* [Node.js](https://nodejs.org/)（v18 以降）

Node.js がインストールされていない場合は、公式サイトからインストールしてください。

### セットアップ

MCP 設定ファイルに以下の JSON を追加してください:

* **Cursor**: プロジェクトルートの `.cursor/mcp.json`、またはグローバルの `~/.cursor/mcp.json`
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

保存後、アプリケーションを再起動してください。MCP サーバーが自動的にダウンロードされ、起動します。以下のツールが利用可能になります:

* **`current-time`** - 設定したタイムゾーンでの現在時刻を取得
* **`local-time`** - ユーザーのタイムゾーン（`convtz` 設定またはシステムのデフォルト）での現在のローカル時刻を取得
* **`convert-time`** - 日時文字列またはエポックタイムスタンプを複数のタイムゾーンに変換
* **`next-weekday`** - 指定した曜日の次の日付を検索
* **`date-to-weekday`** - 指定した日付の曜日を取得
* **`days-until`** - 今日から指定した日付までの日数を計算
* **`days-between`** - 2つの日付間の日数を計算
* **`date-offset`** - 指定した日付の N 日前または後の日付を計算

### mclocks config との連携

MCP サーバーは mclocks の `config.json` を自動的に読み取り、以下を使用します:

* **`clocks`** - 時計で定義されたタイムゾーンがデフォルトの変換先として使用されます
* **`convtz`** - タイムゾーン情報のない日時文字列を変換する際のデフォルトのソースタイムゾーンとして使用されます
* **`usetz`** - 歴史的に正確な UTC オフセットでの厳密なタイムゾーン変換を有効にします（例: JST は1888年以前は +09:18 でした）。歴史的な日時を正確に変換する必要がある場合に `true` に設定してください

`config.json` が見つからない場合、サーバーは組み込みの一般的なタイムゾーンセット（UTC、America/New_York、America/Los_Angeles、Europe/London、Europe/Berlin、Asia/Tokyo、Asia/Shanghai、Asia/Kolkata、Australia/Sydney）にフォールバックします。

### 環境変数

`config.json` の設定を上書きしたい場合、または `config.json` がない場合は、MCP 設定で環境変数を設定できます。環境変数は `config.json` の値よりも優先されます。

| 変数 | 説明 | デフォルト |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | `config.json` のパス。ほとんどの場合、サーバーが自動検出するため不要です。 | 自動検出 |
| `MCLOCKS_LOCALE` | 曜日名などのフォーマット用ロケール（例: `ja`、`pt`、`de`） | `en` |
| `MCLOCKS_CONVTZ` | タイムゾーン情報のない日時文字列を解釈するためのデフォルトのソースタイムゾーン（例: `Asia/Tokyo`） | *（なし）* |
| `MCLOCKS_USETZ` | 厳密なタイムゾーン変換を有効にするには `true` に設定 | `false` |

例:

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

### 使用例

設定後、AI アシスタントに以下のような質問ができます:

* 「今何時？」- mclocks で設定したすべてのタイムゾーンの現在時刻を返します
* 「ジャカルタは今何時？」- 特定のタイムゾーンの現在時刻を返します
* 「エポック 1705312200 を日時に変換して」
* 「2024-01-15T10:30:00Z を Asia/Tokyo に変換して」
* 「次の金曜日は何日？」
* 「2026年12月25日は何曜日？」
* 「クリスマスまであと何日？」
* 「2026年1月1日から2026年12月31日までは何日間？」
* 「2026年4月1日から90日後は何日？」

----------

## ライセンス

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## 作者

Dai Okabayashi: https://github.com/bayashi
