# mclocks

Aplikasi jam desktop untuk berbagai zona waktu🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

Fitur terkait jam:

* 🕐 Jam teks untuk berbagai zona waktu
* ⏱️ Timer
* ⏳ Timer hitung mundur
* 🔄 Konverter waktu Epoch dan tanggal-waktu

Waktu tidak menunggu siapa pun:

* 📝 Catatan tempel

Seorang pengembang tidak pernah tanpa jam:

* 🔀 Konverter teks sederhana
    * seperti membuat klausa SQL `IN` dengan mudah
* 🌐 Server web
    * menyajikan file statis
    * server dump permintaan dan respons
    * endpoint lambat untuk debugging
    * membuka file di editor Anda dari URL GitHub

🔔 CATATAN: `mclocks` tidak memerlukan koneksi internet — semuanya berjalan 100% secara lokal.

## 📦 Unduh

Dari https://github.com/bayashi/mclocks/releases

### Windows

Untuk Windows, Anda bisa mendapatkan file installer `.msi` atau file eksekusi `.exe`.

### macOS

Untuk macOS, Anda bisa mendapatkan file `.dmg` untuk instalasi.

(Pintasan keyboard dalam dokumen ini untuk Windows OS. Jika Anda menggunakan macOS, silakan sesuaikan, ganti tombol seperti `Ctrl` dengan `Ctrl + Command` dan `Alt` dengan `Option` jika diperlukan.)

## ⚙️ config.json

File `config.json` memungkinkan Anda mengonfigurasi jam sesuai preferensi Anda.

File `config.json` harus berada di direktori berikut:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

Saat Anda memulai `mclocks`, tekan `Ctrl + o` untuk mengedit file `config.json` Anda.

### Contoh config.json

File `config.json` harus diformat sebagai JSON, seperti ditunjukkan di bawah ini.

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

## 🔧 Field-field config.json

#### clocks

Field `clocks` adalah array objek, masing-masing berisi properti `name` dan `timezone`. Keduanya harus berupa string. Secara default, keduanya adalah `UTC`.

* `name` adalah label yang akan ditampilkan untuk jam.
* Untuk memilih zona waktu, silakan lihat [daftar zona waktu](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) ini.

Berikut contoh array `clocks` untuk tiga zona waktu.

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

string: `MM-DD ddd HH:mm`

Field `format` adalah string format tanggal-waktu yang digunakan untuk menampilkan jam. Untuk membuat format tanggal-waktu kustom, silakan lihat [panduan format ini](https://momentjs.com/docs/#/parsing/string-format/).

#### format2

string: `""`

Field `format2` sama dengan `format`. Keduanya bergantian dengan tombol `Ctrl + f`. Field `format2` bersifat opsional.

#### locale

string: `en`

Field `locale` menentukan pengaturan bahasa untuk menampilkan tanggal-waktu. Anda dapat menemukan [daftar locale yang didukung di sini](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

string: `#fff`

Field `color` mendefinisikan warna teks tanggal-waktu. Anda dapat menggunakan nama warna, nilai hex RGB, nilai RGB (mis., `RGB(255, 0, 0)`), atau nilai warna CSS yang valid lainnya.

#### font

string: `Courier, monospace`

`font` adalah nama font untuk menampilkan tanggal-waktu. Harus font monospace. Jika Anda mengatur font dengan lebar variabel, mclocks Anda mungkin mengalami efek goyang yang tidak diinginkan.

#### size

angka | string: 14

`size` adalah ukuran karakter untuk tanggal-waktu, dalam piksel. Juga dapat ditentukan sebagai string yang menyertakan satuan (mis., `"125%"`, `"1.5em"`).

#### margin

string: `1.65em`

Field `margin` menentukan jarak antar jam.

#### forefront

boolean: `false`

Jika field `forefront` diatur ke `true`, aplikasi mclocks akan selalu ditampilkan di atas jendela aplikasi lainnya.

## ⏳ Jam hitung mundur

Dengan mengatur konfigurasi `clock` seperti ditunjukkan di bawah, jam akan ditampilkan sebagai jam hitung mundur menuju tanggal-waktu `target` yang ditentukan.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

Jam hitung mundur `clock` di atas akan ditampilkan seperti berikut:

    WAC Tokyo D-159 12:34:56

Menunjukkan tersisa 159 hari, 12 jam, 34 menit, dan 56 detik hingga 13 September 2025.

### Verb format hitung mundur

Teks field `countdown` menerima verb template berikut:

* `%TG`: String tanggal-waktu target
* `%D`: Jumlah hari tersisa hingga tanggal-waktu target
* `%H`: Waktu tersisa dalam jam hingga tanggal-waktu target
* `%h`: Bagian jam (hh) dari waktu tersisa (hh:mm:ss)
* `%M`: Waktu tersisa dalam menit hingga tanggal-waktu target
* `%m`: Bagian menit (mm) dari waktu tersisa (hh:mm:ss)
* `%S`: Waktu tersisa dalam detik hingga tanggal-waktu target
* `%s`: Bagian detik (ss) dari waktu tersisa (hh:mm:ss)

## ⏱️ Timer sederhana

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

Klik jendela aplikasi `mclocks`, lalu tekan tombol `Ctrl + 1` untuk memulai timer 1 menit. Tekan tombol `Ctrl + Alt + 1` untuk memulai timer 10 menit. Tombol angka lainnya juga berfungsi sama. Timer dapat dimulai hingga 5 sekaligus.

`Ctrl + p` untuk jeda / lanjutkan timer.

`Ctrl + 0` untuk menghapus timer tertua. `Ctrl + Alt + 0` untuk menghapus timer terbaru.

🔔 CATATAN: Jam hitung mundur dan timer sederhana akan mengirim notifikasi secara default saat timer selesai. Jika Anda tidak memerlukan notifikasi, atur `withoutNotification: true` di `config.json`.

## 🔢 Menampilkan waktu Epoch

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

Klik jendela aplikasi `mclocks`, lalu tekan tombol `Ctrl + e` untuk mengaktifkan/menonaktifkan tampilan waktu Epoch.

## 🔄 Konversi antara tanggal-waktu dan waktu Epoch

Klik jendela aplikasi `mclocks`, lalu tempel tanggal-waktu atau waktu Epoch, maka dialog akan muncul menampilkan hasil konversi. Dan Anda dapat menyalin hasilnya ke clipboard. Jika tidak ingin menyalin, tekan `[No]` untuk menutup dialog saja.

Saat menempel dengan `Ctrl + v`, nilai (waktu Epoch) diperlakukan sebagai detik. Jika menggunakan `Ctrl + Alt + v`, diperlakukan sebagai milidetik, dengan `Ctrl + Alt + Shift + V` sebagai mikrodetik, dan dengan `Ctrl + Alt + Shift + N + V` sebagai nanodetik dan dikonversi sesuai.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

Ketika nilai tanggal-waktu yang ditempel tidak menyertakan informasi zona waktu, nilai tersebut diperlakukan sebagai zona waktu lokal secara default. Untuk menanganinya sebagai zona waktu tertentu, atur zona waktu di opsi convtz.

    "convtz": "UTC"

## 🔀 Fitur konversi teks

Klik jendela aplikasi `mclocks`, lalu gunakan pintasan keyboard berikut untuk memproses teks clipboard dan membukanya di editor:

* `Ctrl + i`: Mengapit setiap baris teks clipboard dengan tanda kutip ganda dan menambahkan koma di akhir (kecuali baris terakhir)
* `Ctrl + Shift + i`: Menambahkan koma di akhir setiap baris (tanpa tanda kutip) untuk kondisi IN daftar INT (kecuali baris terakhir)

Baris kosong dipertahankan apa adanya di semua operasi.

(Fitur konversi teks ini tidak ada hubungannya dengan jam atau waktu, tetapi pengembang perangkat lunak mungkin merasa berguna! 😊)

## ⌨️ Pintasan keyboard

### Tampilkan bantuan

`F1` (Windows) atau `Cmd + Shift + /` (macOS) untuk membuka halaman bantuan (README ini) di browser

### Konfigurasi, format tampilan

| Pintasan | Deskripsi |
|----------|-------------|
| `Ctrl + o` | Buka file `config.json` di editor |
| `Ctrl + f` | Beralih antara `format` dan `format2` (jika `format2` didefinisikan di `config.json`) |
| `Ctrl + e` atau `Ctrl + u` | Aktifkan/nonaktifkan tampilan waktu Epoch |

### Timer

| Pintasan | Deskripsi |
|----------|-------------|
| `Ctrl + 1` sampai `Ctrl + 9` | Mulai timer (1 menit × tombol angka) |
| `Ctrl + Alt + 1` sampai `Ctrl + Alt + 9` | Mulai timer (10 menit × tombol angka) |
| `Ctrl + p` | Jeda / lanjutkan semua timer |
| `Ctrl + 0` | Hapus timer tertua (paling kiri) |
| `Ctrl + Alt + 0` | Hapus timer terbaru (paling kanan) |

### Catatan tempel

| Pintasan | Deskripsi |
|----------|-------------|
| `Ctrl + s` | Buat catatan tempel baru dari teks clipboard |

### Operasi tanggal-waktu clipboard

| Pintasan | Deskripsi |
|----------|-------------|
| `Ctrl + c` | Salin teks mclocks saat ini ke clipboard |
| `Ctrl + v` | Konversi konten clipboard (waktu Epoch sebagai detik, atau tanggal-waktu) |
| `Ctrl + Alt + v` | Konversi konten clipboard (waktu Epoch sebagai milidetik) |
| `Ctrl + Alt + Shift + V` | Konversi konten clipboard (waktu Epoch sebagai mikrodetik) |
| `Ctrl + Alt + Shift + N + V` | Konversi konten clipboard (waktu Epoch sebagai nanodetik) |

### Konversi teks

| Pintasan | Deskripsi |
|----------|-------------|
| `Ctrl + i` | Mengapit setiap baris clipboard dengan tanda kutip ganda, menambahkan koma di akhir, dan membuka di editor (kecuali baris terakhir) |
| `Ctrl + Shift + i` | Menambahkan koma di akhir setiap baris (tanpa tanda kutip) untuk kondisi IN daftar INT dan membuka di editor (kecuali baris terakhir) |

## 📝 Catatan tempel

Klik jendela aplikasi `mclocks`, lalu tekan `Ctrl + s` untuk membuat catatan tempel dari teks clipboard. Jendela mengambang kecil akan terbuka dengan konten clipboard.

Setiap catatan tempel memiliki:

* **Tombol toggle** (`▸` / `▾`): Perluas atau lipat catatan. Dalam mode lipat hanya satu baris yang ditampilkan.
* **Tombol salin** (`⧉`): Salin teks catatan ke clipboard.
* **Tombol depan** (`⊤` / `⊥`): Aktifkan/nonaktifkan apakah catatan tetap di atas jendela lain. Pengaturan ini disimpan per catatan tempel.
* **Tombol tutup** (`✖`): Hapus catatan tempel dan tutup jendelanya.
* **Area teks**: Edit konten catatan dengan bebas. Perubahan disimpan otomatis.
* **Pegangan ubah ukuran**: Seret sudut kanan bawah untuk mengubah ukuran catatan saat diperluas.

Catatan tempel mewarisi pengaturan `font`, `size`, `color`, dan `forefront` dari `config.json`. Pengaturan depan dapat ditimpa per catatan tempel menggunakan tombol depan; jika tidak ditimpa, nilai dari `config.json` yang digunakan. Posisi, ukuran, status buka/tutup, dan penimpaan depan disimpan secara persisten, dan semua catatan dipulihkan secara otomatis saat `mclocks` dimulai ulang.

🔔 CATATAN: Di macOS, posisi jendela catatan tempel hanya disimpan saat aplikasi keluar. Di Windows, posisi disimpan secara otomatis saat Anda memindahkan atau mengubah ukuran jendela.

Ukuran teks maksimum per catatan tempel adalah 128 KB.

## 🌐 Server web

`mclocks` dapat menyajikan file statis melalui server web bawaan. Fitur ini memungkinkan Anda dengan mudah melihat cuplikan kode di browser. Tambahkan field `web` ke `config.json` Anda:

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

* `root`: Path ke direktori yang berisi file untuk disajikan (wajib)
* `port`: Nomor port untuk didengarkan (default: 3030)
* `open_browser_at_start`: Jika diatur ke `true`, secara otomatis membuka URL server web di browser default saat `mclocks` dimulai (default: `false`)
* `dump`: Jika diatur ke `true`, mengaktifkan endpoint `/dump` yang mengembalikan detail permintaan sebagai JSON (default: `false`)
* `slow`: Jika diatur ke `true`, mengaktifkan endpoint `/slow` yang menunda respons (default: `false`)
* `status`: Jika diatur ke `true`, mengaktifkan endpoint `/status/{code}` yang mengembalikan kode status HTTP arbitrer (default: `false`)
* `content.markdown.allowRawHTML`: Jika diatur ke `true`, mengizinkan HTML mentah dalam rendering Markdown; jika `false`, HTML mentah dalam Markdown di-escape sebagai teks (default: `false`)
* `editor`: Jika diatur dan berisi `reposDir`, mengaktifkan endpoint `/editor` yang membuka file lokal di editor Anda dari URL GitHub di browser (default: tidak diatur)

Jika field `web` dikonfigurasi di `config.json` Anda, server web dimulai secara otomatis saat `mclocks` diluncurkan. Akses file di `http://127.0.0.1:3030`. Server web hanya mendengarkan di `127.0.0.1` (localhost), jadi hanya dapat diakses dari mesin lokal Anda.

### Tipe file yang didukung

Server web mendukung tipe file berikut:

* Teks: `html`, `css`, `js`, `json`, `md`, `txt`
* Gambar: `png`, `jpg`, `jpeg`, `gif`, `svg`, `ico`

### Endpoint /dump

Ketika `dump: true` diatur dalam konfigurasi `web`, server web menyediakan endpoint `/dump` yang mengembalikan detail permintaan sebagai JSON.

Endpoint merespons dengan objek JSON yang berisi:
* `method`: Metode HTTP (mis., "GET", "POST")
* `path`: Path permintaan setelah `/dump/` (mis., "/test" untuk `/dump/test`)
* `query`: Parameter query sebagai array objek kunci-nilai (mis., `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: Header permintaan sebagai array objek kunci-nilai (mis., `[{"Content-Type": "application/json"}]`)
* `body`: Body permintaan sebagai string (jika ada)
* `parsed_body`: Objek JSON yang diparsing jika Content-Type menunjukkan JSON, atau string pesan error jika parsing gagal

Akses endpoint dump di `http://127.0.0.1:3030/dump` atau path apa pun di bawah `/dump/` (mis., `/dump/test?key=value`).

### Endpoint /slow

Ketika `slow: true` diatur dalam konfigurasi `web`, server web menyediakan endpoint `/slow` yang menunda respons sebelum mengembalikan 200 OK.

Endpoint dapat diakses melalui metode HTTP apa pun (GET, POST, dll.) dan mendukung path berikut:

* `/slow`: Menunggu 30 detik (default) dan mengembalikan 200 OK
* `/slow/120`: Menunggu 120 detik (atau jumlah detik yang ditentukan) dan mengembalikan 200 OK

Nilai maksimum yang diizinkan adalah 901 detik (15 menit + 1 detik). Permintaan yang melebihi batas ini mengembalikan error 400 Bad Request.

Endpoint ini berguna untuk menguji perilaku timeout, penanganan koneksi, atau simulasi kondisi jaringan lambat.

Jika parameter detik yang tidak valid diberikan (mis., `/slow/abc`), endpoint mengembalikan error 400 Bad Request.

### Endpoint /status

Ketika `status: true` diatur dalam konfigurasi `web`, server web menyediakan endpoint `/status/{code}` yang mengembalikan kode status HTTP arbitrer yang didefinisikan dalam standar RFC (100-599).

Endpoint mengembalikan respons teks biasa dengan kode status dan frasa yang sesuai, beserta header yang sesuai sebagaimana disyaratkan oleh spesifikasi HTTP.

**Contoh:**
* `http://127.0.0.1:3030/status/200` - mengembalikan 200 OK
* `http://127.0.0.1:3030/status/404` - mengembalikan 404 Not Found
* `http://127.0.0.1:3030/status/500` - mengembalikan 500 Internal Server Error
* `http://127.0.0.1:3030/status/418` - mengembalikan 418 I'm a teapot (dengan pesan khusus)
* `http://127.0.0.1:3030/status/301` - mengembalikan 301 Moved Permanently (dengan header Location)

**Header khusus status:**

Endpoint secara otomatis menambahkan header yang sesuai untuk kode status tertentu:

* **3xx Pengalihan** (301, 302, 303, 305, 307, 308): Menambahkan header `Location`
* **401 Unauthorized**: Menambahkan header `WWW-Authenticate`
* **405 Method Not Allowed**: Menambahkan header `Allow`
* **407 Proxy Authentication Required**: Menambahkan header `Proxy-Authenticate`
* **416 Range Not Satisfiable**: Menambahkan header `Content-Range`
* **426 Upgrade Required**: Menambahkan header `Upgrade`
* **429 Too Many Requests**: Menambahkan header `Retry-After` (60 detik)
* **503 Service Unavailable**: Menambahkan header `Retry-After` (60 detik)
* **511 Network Authentication Required**: Menambahkan header `WWW-Authenticate`

**Penanganan body respons:**

* **204 No Content** dan **304 Not Modified**: Mengembalikan body respons kosong (sesuai spesifikasi HTTP)
* **418 I'm a teapot**: Mengembalikan pesan khusus "I'm a teapot" alih-alih frasa status standar
* **Semua kode status lainnya**: Mengembalikan teks biasa dalam format `{code} {phrase}` (mis., "404 Not Found")

Endpoint ini berguna untuk menguji bagaimana aplikasi Anda menangani berbagai kode status HTTP, penanganan error, pengalihan, persyaratan autentikasi, dan skenario pembatasan laju.

### Endpoint /editor

Ketika `web.editor.reposDir` diatur dalam file konfigurasi, server web menyediakan endpoint `/editor` yang memungkinkan Anda membuka file lokal di editor Anda langsung dari URL GitHub di browser.

**Konfigurasi:**

Tambahkan yang berikut ke konfigurasi `web` Anda:

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

* `reposDir`: Path ke direktori repositori lokal Anda. Mendukung `~` untuk ekspansi direktori home (mis., `"~/repos"` di macOS atau `"C:/Users/username/repos"` di Windows). Direktori ini harus ada.
* `includeHost`: Jika `true`, resolusi path lokal menyertakan host asli sebagai direktori (mis., `{reposDir}/{host}/{owner}/{repo}/...`). Jika `false`, resolusi ke `{reposDir}/{owner}/{repo}/...` (default: `false`).
* `command`: Nama perintah atau path ke file eksekusi editor Anda (default: `code`)
* `args`: Array template argumen. Gunakan placeholder `{file}` dan `{line}`. Jika `#L...` tidak ada di URL, `{line}` menggunakan 1.

**Cara kerjanya:**

1. Ketika Anda mengakses URL file GitHub melalui endpoint `/editor`, URL tersebut mengonversi path GitHub ke path file lokal
2. Path file lokal dibuat sebagai: `{reposDir}/{owner}/{repository_name}/{file_path}`
3. Jika file ada, file dibuka di editor Anda pada nomor baris yang ditentukan menggunakan perintah dan argumen yang dikonfigurasi (default: `code -g {local_file_path}:{line_number}`)
4. Jika file tidak ada, halaman error ditampilkan dengan tautan untuk mengkloning repositori

**Bookmarklet:**

Buat bookmarklet untuk membuka file GitHub dengan cepat di editor lokal Anda. Ganti `3030` dengan nomor port yang Anda konfigurasi:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**Dukungan nomor baris:**

Anda dapat menentukan nomor baris menggunakan fragmen hash di URL:
* `https://github.com/username/repo/blob/main/file.rs#L123` → Membuka di baris 123

**Penanganan error:**

* Jika file tidak ada secara lokal, tab tetap terbuka dan menampilkan pesan error dengan tautan untuk mengkloning repositori dari GitHub
* Jika file berhasil dibuka, tab ditutup secara otomatis
* Jika `web.editor.reposDir` tidak dikonfigurasi atau tidak ada, endpoint `/editor` tidak diaktifkan (dan Anda akan mendapatkan 404)

**Contoh:**

1. Anda sedang melihat file di GitHub: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. Klik bookmarklet atau navigasi secara manual ke: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. Jika `~/repos/mclocks/src/app.js` ada di lokal Anda, VS Code membukanya di baris 42
4. Jika file tidak ada, halaman error ditampilkan dengan tautan ke `https://github.com/bayashi/mclocks` untuk mengkloning

----------

## 🧠 Server MCP mclocks

`mclocks` menyertakan server MCP (Model Context Protocol) yang memungkinkan asisten AI seperti [Cursor](https://www.cursor.com/) dan [Claude Desktop](https://claude.ai/download) untuk menjawab "Jam berapa sekarang?" di berbagai zona waktu, dan mengonversi antara format tanggal-waktu dan timestamp Epoch. Server MCP secara otomatis menggunakan `config.json` mclocks Anda, sehingga zona waktu yang Anda konfigurasi di mclocks tercermin dalam respons AI.

### Prasyarat

* [Node.js](https://nodejs.org/) (v18 atau lebih baru)

Jika Anda belum memiliki Node.js, instal dari situs web resmi.

### Pengaturan

Tambahkan JSON berikut ke file konfigurasi MCP Anda:

* **Cursor**: `.cursor/mcp.json` di root proyek Anda, atau global `~/.cursor/mcp.json`
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

Setelah menyimpan, mulai ulang aplikasi. Server MCP akan diunduh dan dimulai secara otomatis. Alat-alat berikut menjadi tersedia:

* **`current-time`** - Dapatkan waktu saat ini di zona waktu yang Anda konfigurasi
* **`local-time`** - Dapatkan waktu lokal saat ini di zona waktu pengguna (dari konfigurasi `convtz` atau default sistem)
* **`convert-time`** - Konversi string tanggal-waktu atau timestamp Epoch ke berbagai zona waktu
* **`next-weekday`** - Temukan tanggal kemunculan berikutnya dari hari tertentu dalam seminggu
* **`date-to-weekday`** - Dapatkan hari dalam seminggu untuk tanggal tertentu
* **`days-until`** - Hitung jumlah hari dari hari ini hingga tanggal yang ditentukan
* **`days-between`** - Hitung jumlah hari antara dua tanggal
* **`date-offset`** - Hitung tanggal N hari sebelum atau sesudah tanggal tertentu

### Cara kerjanya dengan konfigurasi mclocks

Server MCP secara otomatis membaca `config.json` mclocks Anda dan menggunakan:

* **`clocks`** - Zona waktu yang didefinisikan di jam Anda digunakan sebagai target konversi default
* **`convtz`** - Digunakan sebagai zona waktu sumber default saat mengonversi string tanggal-waktu tanpa info zona waktu
* **`usetz`** - Mengaktifkan konversi zona waktu ketat untuk offset UTC yang akurat secara historis (mis., JST adalah +09:18 sebelum 1888). Atur ke `true` saat Anda perlu mengonversi tanggal-waktu historis secara akurat

Jika tidak ditemukan `config.json`, server kembali ke set zona waktu umum bawaan (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### Variabel lingkungan

Jika Anda ingin menimpa pengaturan `config.json`, atau jika Anda tidak memiliki `config.json` sama sekali, Anda dapat mengatur variabel lingkungan di konfigurasi MCP Anda. Variabel lingkungan memiliki prioritas di atas nilai di `config.json`.

| Variabel | Deskripsi | Default |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | Path ke `config.json`. Tidak diperlukan dalam kebanyakan kasus, karena server mendeteksi lokasi secara otomatis. | deteksi otomatis |
| `MCLOCKS_LOCALE` | Locale untuk memformat nama hari dalam seminggu, dll. (mis., `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Zona waktu sumber default untuk menginterpretasi string tanggal-waktu tanpa info zona waktu (mis., `Asia/Tokyo`) | *(tidak ada)* |
| `MCLOCKS_USETZ` | Atur ke `true` untuk mengaktifkan konversi zona waktu ketat | `false` |

Contoh:

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

### Contoh penggunaan

Setelah dikonfigurasi, Anda dapat bertanya kepada asisten AI Anda hal-hal seperti:

* "Jam berapa sekarang?" - Mengembalikan waktu saat ini di semua zona waktu yang Anda konfigurasi di mclocks
* "Jam berapa di Jakarta?" - Mengembalikan waktu saat ini di zona waktu tertentu
* "Konversi epoch 1705312200 ke tanggal-waktu"
* "Konversi 2024-01-15T10:30:00Z ke Asia/Tokyo"
* "Jumat depan tanggal berapa?"
* "25 Desember 2026 hari apa?"
* "Berapa hari lagi sampai Natal?"
* "Berapa hari antara 1 Januari 2026 dan 31 Desember 2026?"
* "Tanggal berapa 90 hari setelah 1 April 2026?"

----------

## Lisensi

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Penulis

Dai Okabayashi: https://github.com/bayashi
