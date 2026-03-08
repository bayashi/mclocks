# mclocks

Birden fazla saat dilimi için masaüstü saat uygulaması🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

Saatle ilgili özellikler:

* 🕐 Birden fazla saat dilimi için metin saati
* ⏱️ Zamanlayıcı
* ⏳ Geri sayım zamanlayıcısı
* 🔄 Epoch zamanı ve tarih-saat dönüştürücü

Zaman kimseyi beklemez:

* 📝 Yapışkan not

Bir geliştirici asla saatsiz kalmaz:

* 🔀 Basit metin dönüştürücü
    * SQL `IN` cümlelerini kolayca oluşturma gibi
* 🌐 Web sunucusu
    * statik dosyalar sunar
    * Markdown'ı zengin şekilde işler
    * sürükle-bırak tabanlı içerik görüntüleyici
    * istek ve yanıt döküm sunucusu
    * hata ayıklama için yavaş uç noktalar
    * GitHub URL'lerinden editörünüzde dosya açma

🔔 NOT: `mclocks` internet bağlantısı gerektirmez — her şey %100 yerel olarak çalışır.

## 📦 İndirme

https://github.com/bayashi/mclocks/releases adresinden

### Windows

Windows için yükleyici `.msi` dosyasını veya çalıştırılabilir `.exe` dosyasını edinebilirsiniz.

### macOS

macOS için yükleme amacıyla `.dmg` dosyasını edinebilirsiniz.

(Bu belgedeki kısayol tuşları Windows işletim sistemi içindir. macOS kullanıyorsanız, lütfen bunları uygun şekilde yorumlayın; `Ctrl` yerine `Ctrl + Command` ve `Alt` yerine `Option` kullanın.)

## ⚙️ config.json

`config.json` dosyası, saatleri tercihlerinize göre yapılandırmanıza olanak tanır.

`config.json` dosyası aşağıdaki dizinlerde bulunmalıdır:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

`mclocks`'u başlattığınızda, `config.json` dosyanızı düzenlemek için `Ctrl + o` tuşlarına basın.

### config.json örneği

`config.json` dosyası aşağıda gösterildiği gibi JSON formatında olmalıdır.

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

## 🔧 config.json alanları

#### clocks

`clocks` alanı, her biri `name` ve `timezone` özelliklerini içeren nesnelerden oluşan bir dizidir. Her ikisi de metin olmalıdır. Varsayılan olarak ikisi de `UTC`'dir.

* `name`, saat için görüntülenecek bir etikettir.
* Saat dilimlerini seçmek için bu [saat dilimleri listesine](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) bakın.

Üç saat dilimi için bir `clocks` dizisi örneği aşağıda verilmiştir.

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

metin: `MM-DD ddd HH:mm`

`format` alanı, saati görüntülemek için kullanılan tarih-saat biçim dizesidir. Özel bir tarih-saat biçimi oluşturmak için [bu biçimlendirme kılavuzuna](https://momentjs.com/docs/#/parsing/string-format/) bakın.

#### format2

metin: `""`

`format2` alanı `format` ile aynıdır. `Ctrl + f` tuşuyla birbirleriyle değiştirilir. `format2` isteğe bağlı bir alandır.

#### locale

metin: `en`

`locale` alanı, tarih-saat görüntüleme için dil ayarlarını belirler. [Desteklenen yerel ayarların listesini burada](https://github.com/kawanet/cdate-locale/blob/main/locales.yml) bulabilirsiniz.

#### color

metin: `#fff`

`color` alanı, tarih-saat metninin rengini tanımlar. Adlandırılmış renkler, RGB onaltılık değerler, RGB değerleri (ör., `RGB(255, 0, 0)`) veya herhangi bir geçerli CSS renk değeri kullanabilirsiniz.

#### font

metin: `Courier, monospace`

`font`, tarih-saati görüntülemek için kullanılan yazı tipi adıdır. Sabit genişlikli bir yazı tipi olmalıdır. Değişken genişlikli bir yazı tipi ayarlarsanız, mclocks'unuzda istenmeyen bir titreme efekti olabilir.

#### size

sayı | metin: 14

`size`, tarih-saat için piksel cinsinden karakter boyutudur. Birim içeren bir metin olarak da belirtilebilir (ör., `"125%"`, `"1.5em"`).

#### margin

metin: `1.65em`

`margin` alanı, saatler arasındaki boşluğu belirler.

#### forefront

mantıksal: `false`

`forefront` alanı `true` olarak ayarlanırsa, mclocks uygulaması her zaman diğer uygulama pencerelerinin üstünde görüntülenir.

## ⏳ Geri sayım saati

Aşağıda gösterildiği gibi `clock` yapılandırması ayarlandığında, belirtilen `target` tarih-saatine kadar geri sayım saati olarak görüntülenir.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

Yukarıdaki geri sayım `clock` aşağıdaki gibi görüntülenir:

    WAC Tokyo D-159 12:34:56

13 Eylül 2025'e kadar 159 gün, 12 saat, 34 dakika ve 56 saniye kaldığını gösterir.

### Geri sayım biçim değişkenleri

`countdown` alanı metni aşağıdaki şablon değişkenlerini kabul eder:

* `%TG`: Hedef tarih-saat dizesi
* `%D`: Hedef tarih-saate kalan gün sayısı
* `%H`: Hedef tarih-saate kalan süre saat olarak
* `%h`: Kalan sürenin saat (hh) kısmı (hh:mm:ss)
* `%M`: Hedef tarih-saate kalan süre dakika olarak
* `%m`: Kalan sürenin dakika (mm) kısmı (hh:mm:ss)
* `%S`: Hedef tarih-saate kalan süre saniye olarak
* `%s`: Kalan sürenin saniye (ss) kısmı (hh:mm:ss)

## ⏱️ Basit zamanlayıcı

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

`mclocks` uygulama penceresine tıklayın, ardından 1 dakikalık zamanlayıcı başlatmak için `Ctrl + 1` tuşuna basın. 10 dakikalık zamanlayıcı başlatmak için `Ctrl + Alt + 1` tuşuna basın. Diğer sayı tuşları da aynı şekilde çalışır. Aynı anda en fazla 5 zamanlayıcı başlatılabilir.

Zamanlayıcıları duraklatmak / devam ettirmek için `Ctrl + p`.

En eski zamanlayıcıyı silmek için `Ctrl + 0`. En yeni zamanlayıcıyı silmek için `Ctrl + Alt + 0`.

🔔 NOT: Geri sayım saati ve basit zamanlayıcı, zamanlayıcı tamamlandığında varsayılan olarak bildirim gönderir. Bildirimlere ihtiyacınız yoksa, `config.json`'da `withoutNotification: true` olarak ayarlayın.

## 🔢 Epoch zamanını görüntüleme

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

`mclocks` uygulama penceresine tıklayın, ardından Epoch zamanı görüntülemesini açıp kapatmak için `Ctrl + e` tuşuna basın.

## 🔄 Tarih-saat ve Epoch zamanı arasında dönüştürme

`mclocks` uygulama penceresine tıklayın, ardından bir tarih-saat veya Epoch zamanı yapıştırın, dönüştürme sonuçlarını gösteren bir iletişim kutusu görünür. Sonuçları panoya kopyalayabilirsiniz. Kopyalamak istemiyorsanız, iletişim kutusunu kapatmak için `[No]` düğmesine basın.

`Ctrl + v` ile yapıştırırken, değer (Epoch zamanı) saniye olarak kabul edilir. `Ctrl + Alt + v` kullanırsanız milisaniye, `Ctrl + Alt + Shift + V` ile mikrosaniye, `Ctrl + Alt + Shift + N + V` ile nanosaniye olarak kabul edilir ve buna göre dönüştürülür.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

Yapıştırılan tarih-saat değerleri saat dilimi bilgisi içermiyorsa, varsayılan olarak yerel saat dilimi olarak kabul edilir. Belirli bir saat dilimi olarak işlemek için convtz seçeneğinde saat dilimini ayarlayın.

    "convtz": "UTC"

## 🔀 Metin dönüştürme özelliği

`mclocks` uygulama penceresine tıklayın, ardından pano metnini işlemek ve editörde açmak için aşağıdaki klavye kısayollarını kullanın:

* `Ctrl + i`: Pano metninin her satırını çift tırnak içine alır ve sonuna virgül ekler (son satır hariç)
* `Ctrl + Shift + i`: Her satırın sonuna virgül ekler (tırnak olmadan) INT listesi IN koşulu için (son satır hariç)

Boş satırlar tüm işlemlerde olduğu gibi korunur.

(Bu metin dönüştürme özelliğinin saatler veya zamanla hiçbir ilgisi yoktur, ancak yazılım geliştiricileri bunu kullanışlı bulabilir! 😊)

## ⌨️ Klavye kısayolları

### Yardımı göster

`F1` (Windows) veya `Cmd + Shift + /` (macOS) ile yardım sayfasını (bu README) tarayıcıda açın

### Yapılandırma, görüntüleme biçimleri

| Kısayol | Açıklama |
|----------|-------------|
| `Ctrl + o` | `config.json` dosyasını editörde aç |
| `Ctrl + f` | `format` ve `format2` arasında geçiş yap (`config.json`'da `format2` tanımlıysa) |
| `Ctrl + e` veya `Ctrl + u` | Epoch zamanı görüntülemesini aç/kapat |

### Zamanlayıcı

| Kısayol | Açıklama |
|----------|-------------|
| `Ctrl + 1` - `Ctrl + 9` | Zamanlayıcı başlat (1 dakika × sayı tuşu) |
| `Ctrl + Alt + 1` - `Ctrl + Alt + 9` | Zamanlayıcı başlat (10 dakika × sayı tuşu) |
| `Ctrl + p` | Tüm zamanlayıcıları duraklat / devam ettir |
| `Ctrl + 0` | En eski zamanlayıcıyı (en soldaki) sil |
| `Ctrl + Alt + 0` | En yeni zamanlayıcıyı (en sağdaki) sil |

### Yapışkan not

| Kısayol | Açıklama |
|----------|-------------|
| `Ctrl + s` | Pano metninden yeni bir yapışkan not oluştur |

### Pano tarih-saat işlemleri

| Kısayol | Açıklama |
|----------|-------------|
| `Ctrl + c` | Geçerli mclocks metnini panoya kopyala |
| `Ctrl + v` | Pano içeriğini dönüştür (Epoch zamanı saniye olarak veya tarih-saat) |
| `Ctrl + Alt + v` | Pano içeriğini dönüştür (Epoch zamanı milisaniye olarak) |
| `Ctrl + Alt + Shift + V` | Pano içeriğini dönüştür (Epoch zamanı mikrosaniye olarak) |
| `Ctrl + Alt + Shift + N + V` | Pano içeriğini dönüştür (Epoch zamanı nanosaniye olarak) |

### Metin dönüştürme

| Kısayol | Açıklama |
|----------|-------------|
| `Ctrl + i` | Panonun her satırını çift tırnak içine al, sonuna virgül ekle ve editörde aç (son satır hariç) |
| `Ctrl + Shift + i` | Her satırın sonuna virgül ekle (tırnak olmadan) INT listesi IN koşulu için ve editörde aç (son satır hariç) |

## 📝 Yapışkan not

`mclocks` uygulama penceresine tıklayın, ardından pano metninden yapışkan not oluşturmak için `Ctrl + s` tuşuna basın. Pano içeriğiyle birlikte küçük bir yüzen pencere açılır.

Her yapışkan not şunlara sahiptir:

* **Geçiş düğmesi** (`▸` / `▾`): Notu genişlet veya daralt. Daraltılmış modda yalnızca tek bir satır gösterilir.
* **Kopyala düğmesi** (`⧉`): Not metnini panoya kopyala.
* **Ön plan düğmesi** (`⊤` / `⊥`): Notun diğer pencerelerin üstünde kalıp kalmayacağını değiştir. Bu ayar yapışkan not başına kaydedilir.
* **Kapat düğmesi** (`✖`): Yapışkan notu sil ve penceresini kapat.
* **Metin alanı**: Not içeriğini serbestçe düzenleyin. Değişiklikler otomatik olarak kaydedilir.
* **Yeniden boyutlandırma tutamacı**: Genişletildiğinde notu yeniden boyutlandırmak için sağ alt köşeyi sürükleyin.

Yapışkan notlar, `config.json`'dan `font`, `size`, `color` ve `forefront` ayarlarını miras alır. Ön plan ayarı, ön plan düğmesi kullanılarak yapışkan not başına geçersiz kılınabilir; geçersiz kılınmazsa `config.json`'daki değer kullanılır. Konumları, boyutları, açık/kapalı durumları ve ön plan geçersiz kılması kalıcı olarak saklanır ve `mclocks` yeniden başlatıldığında tüm notlar otomatik olarak geri yüklenir.

🔔 NOT: macOS'ta yapışkan not pencere konumları yalnızca uygulama kapandığında kaydedilir. Windows'ta konumlar, pencereleri taşıdığınızda veya yeniden boyutlandırdığınızda otomatik olarak kaydedilir.

Yapışkan not başına maksimum metin boyutu 128 KB'dir.

## 🌐 Web sunucusu

`mclocks`, yerleşik bir web sunucusu aracılığıyla statik dosyalar sunabilir. Bu özellik, kod parçacıklarınızı bir tarayıcıda kolayca görüntülemenizi sağlar. `config.json`'unuza bir `web` alanı ekleyin:

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

* `root`: Sunulacak dosyaları içeren dizinin yolu (zorunlu)
* `port`: Dinlenecek port numarası (varsayılan: 3030)
* `open_browser_at_start`: `true` olarak ayarlanırsa, `mclocks` başladığında varsayılan tarayıcıda web sunucusu URL'sini otomatik olarak açar (varsayılan: `false`)
* `dump`: `true` olarak ayarlanırsa, istek ayrıntılarını JSON olarak döndüren `/dump` uç noktasını etkinleştirir (varsayılan: `false`)
* `slow`: `true` olarak ayarlanırsa, yanıtı geciktiren `/slow` uç noktasını etkinleştirir (varsayılan: `false`)
* `status`: `true` olarak ayarlanırsa, rastgele HTTP durum kodları döndüren `/status/{code}` uç noktasını etkinleştirir (varsayılan: `false`)
* `content.markdown.allowRawHTML`: `true` olarak ayarlanırsa Markdown işleme sırasında ham HTML'e izin verir; `false` ise Markdown içindeki ham HTML metin olarak escape edilir (varsayılan: `false`)
* `editor`: Ayarlanmışsa ve `reposDir` içeriyorsa, tarayıcıdaki GitHub URL'lerinden editörünüzde yerel dosyaları açan `/editor` uç noktasını etkinleştirir (varsayılan: ayarlanmamış)

`config.json`'unuzda `web` alanı yapılandırılmışsa, `mclocks` başlatıldığında web sunucusu otomatik olarak başlar. Dosyalara `http://127.0.0.1:3030` adresinden erişin. Web sunucusu yalnızca `127.0.0.1` (localhost) üzerinde dinler, bu nedenle yalnızca yerel makinenizden erişilebilir.

### Desteklenen dosya türleri

Web sunucusu aşağıdaki dosya türlerini destekler:

* Metin: `html`, `css`, `js`, `json`, `md`, `txt`
* Görüntüler: `png`, `jpg`, `jpeg`, `gif`, `svg`, `ico`

### sürükle-bırak tabanlı içerik görüntüleyici

Statik dosya barındırmaya ek olarak, web sunucusu sürükle-bırak tabanlı bir içerik görüntüleyici iş akışını da içerir: mclocks saat penceresine bir dosya veya dizin sürükleyip bıraktığınızda, geçici yerel URL'ler üzerinden açılıp görüntülenebilir.
Bu geçici URL'ler mclocks kapandığında atılır.

### /dump uç noktası

`web` yapılandırmasında `dump: true` ayarlandığında, web sunucusu istek ayrıntılarını JSON olarak döndüren bir `/dump` uç noktası sağlar.

Uç nokta aşağıdakileri içeren bir JSON nesnesiyle yanıt verir:
* `method`: HTTP yöntemi (ör., "GET", "POST")
* `path`: `/dump/` sonrasındaki istek yolu (ör., `/dump/test` için "/test")
* `query`: Anahtar-değer nesneleri dizisi olarak sorgu parametreleri (ör., `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: Anahtar-değer nesneleri dizisi olarak istek başlıkları (ör., `[{"Content-Type": "application/json"}]`)
* `body`: Metin olarak istek gövdesi (varsa)
* `parsed_body`: Content-Type JSON gösteriyorsa ayrıştırılmış JSON nesnesi veya ayrıştırma başarısızsa hata mesajı metni

Dump uç noktasına `http://127.0.0.1:3030/dump` adresinden veya `/dump/` altındaki herhangi bir yoldan (ör., `/dump/test?key=value`) erişin.

### /slow uç noktası

`web` yapılandırmasında `slow: true` ayarlandığında, web sunucusu 200 OK döndürmeden önce yanıtı geciktiren bir `/slow` uç noktası sağlar.

Uç nokta herhangi bir HTTP yöntemiyle (GET, POST, vb.) erişilebilir ve aşağıdaki yolları destekler:

* `/slow`: 30 saniye (varsayılan) bekler ve 200 OK döndürür
* `/slow/120`: 120 saniye (veya belirtilen herhangi bir saniye sayısı) bekler ve 200 OK döndürür

İzin verilen maksimum değer 901 saniyedir (15 dakika + 1 saniye). Bu sınırı aşan istekler 400 Bad Request hatası döndürür.

Bu uç nokta, zaman aşımı davranışını, bağlantı yönetimini veya yavaş ağ koşullarını simüle etmek için test yapmakta kullanışlıdır.

Geçersiz bir saniye parametresi sağlanırsa (ör., `/slow/abc`), uç nokta 400 Bad Request hatası döndürür.

### /status uç noktası

`web` yapılandırmasında `status: true` ayarlandığında, web sunucusu RFC standartlarında tanımlanan rastgele HTTP durum kodlarını (100-599) döndüren bir `/status/{code}` uç noktası sağlar.

Uç nokta, durum kodu ve karşılık gelen ifadeyle birlikte düz metin yanıtı ve HTTP spesifikasyonunun gerektirdiği uygun başlıklarla döndürür.

**Örnekler:**
* `http://127.0.0.1:3030/status/200` - 200 OK döndürür
* `http://127.0.0.1:3030/status/404` - 404 Not Found döndürür
* `http://127.0.0.1:3030/status/500` - 500 Internal Server Error döndürür
* `http://127.0.0.1:3030/status/418` - 418 I'm a teapot döndürür (özel mesajla)
* `http://127.0.0.1:3030/status/301` - 301 Moved Permanently döndürür (Location başlığıyla)

**Duruma özgü başlıklar:**

Uç nokta, belirli durum kodları için otomatik olarak uygun başlıklar ekler:

* **3xx Yönlendirme** (301, 302, 303, 305, 307, 308): `Location` başlığı ekler
* **401 Unauthorized**: `WWW-Authenticate` başlığı ekler
* **405 Method Not Allowed**: `Allow` başlığı ekler
* **407 Proxy Authentication Required**: `Proxy-Authenticate` başlığı ekler
* **416 Range Not Satisfiable**: `Content-Range` başlığı ekler
* **426 Upgrade Required**: `Upgrade` başlığı ekler
* **429 Too Many Requests**: `Retry-After` başlığı ekler (60 saniye)
* **503 Service Unavailable**: `Retry-After` başlığı ekler (60 saniye)
* **511 Network Authentication Required**: `WWW-Authenticate` başlığı ekler

**Yanıt gövdesi işleme:**

* **204 No Content** ve **304 Not Modified**: Boş yanıt gövdesi döndürür (HTTP spesifikasyonuna uygun olarak)
* **418 I'm a teapot**: Standart durum ifadesi yerine özel mesaj "I'm a teapot" döndürür
* **Diğer tüm durum kodları**: `{code} {phrase}` biçiminde düz metin döndürür (ör., "404 Not Found")

Bu uç nokta, uygulamalarınızın farklı HTTP durum kodlarını, hata yönetimini, yönlendirmeleri, kimlik doğrulama gereksinimlerini ve hız sınırlama senaryolarını nasıl ele aldığını test etmek için kullanışlıdır.

### /editor uç noktası

Yapılandırma dosyasında `web.editor.reposDir` ayarlandığında, web sunucusu tarayıcıdaki GitHub URL'lerinden editörünüzde yerel dosyaları doğrudan açmanıza olanak tanıyan bir `/editor` uç noktası sağlar.

**Yapılandırma:**

`web` yapılandırmanıza aşağıdakini ekleyin:

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

* `reposDir`: Yerel depolarınızın dizin yolu. Ana dizin genişletmesi için `~` destekler (ör., macOS'ta `"~/repos"` veya Windows'ta `"C:/Users/username/repos"`). Bu dizin mevcut olmalıdır.
* `includeHost`: `true` ise, yerel yol çözümlemesi orijinal ana bilgisayar adını dizin olarak içerir (ör., `{reposDir}/{host}/{owner}/{repo}/...`). `false` ise `{reposDir}/{owner}/{repo}/...` olarak çözümlenir (varsayılan: `false`).
* `command`: Editör çalıştırılabilir dosyanızın komut adı veya yolu (varsayılan: `code`)
* `args`: Argüman şablonu dizisi. `{file}` ve `{line}` yer tutucularını kullanın. URL'de `#L...` yoksa, `{line}` 1 kullanır.

**Nasıl çalışır:**

1. `/editor` uç noktası aracılığıyla bir GitHub dosya URL'sine eriştiğinizde, GitHub yolunu yerel bir dosya yoluna dönüştürür
2. Yerel dosya yolu şu şekilde oluşturulur: `{reposDir}/{owner}/{repository_name}/{file_path}`
3. Dosya mevcutsa, yapılandırılmış komut ve argümanları kullanarak belirtilen satır numarasında editörünüzde açar (varsayılan: `code -g {local_file_path}:{line_number}`)
4. Dosya mevcut değilse, depoyu klonlamak için bir bağlantı içeren bir hata sayfası görüntülenir

**Yer imi uygulamacığı:**

GitHub dosyalarını yerel editörünüzde hızlıca açmak için bir yer imi uygulamacığı oluşturun. `3030`'u yapılandırdığınız port numarasıyla değiştirin:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**Satır numarası desteği:**

URL'deki hash parçasını kullanarak bir satır numarası belirtebilirsiniz:
* `https://github.com/username/repo/blob/main/file.rs#L123` → 123. satırda açar

**Hata yönetimi:**

* Dosya yerel olarak mevcut değilse, sekme açık kalır ve GitHub'dan depoyu klonlamak için bir bağlantı içeren bir hata mesajı görüntüler
* Dosya başarıyla açılırsa, sekme otomatik olarak kapanır
* `web.editor.reposDir` yapılandırılmamışsa veya mevcut değilse, `/editor` uç noktası etkinleştirilmez (ve 404 alırsınız)

**Örnek:**

1. GitHub'da bir dosya görüntülüyorsunuz: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. Yer imi uygulamacığına tıklayın veya manuel olarak şuraya gidin: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. Yerel olarak `~/repos/mclocks/src/app.js` mevcutsa, VS Code dosyayı 42. satırda açar
4. Dosya mevcut değilse, klonlama için `https://github.com/bayashi/mclocks` bağlantısı içeren bir hata sayfası görüntülenir

----------

## 🧠 mclocks MCP Sunucusu

`mclocks`, [Cursor](https://www.cursor.com/) ve [Claude Desktop](https://claude.ai/download) gibi yapay zeka asistanlarının birden fazla saat diliminde "Saat kaç?" sorusunu yanıtlamasını ve tarih-saat biçimleri ile Epoch zaman damgaları arasında dönüştürme yapmasını sağlayan bir MCP (Model Context Protocol) sunucusu içerir. MCP sunucusu, mclocks `config.json` dosyanızı otomatik olarak kullanır, böylece mclocks'ta yapılandırdığınız saat dilimleri yapay zekanın yanıtlarına yansır.

### Ön koşullar

* [Node.js](https://nodejs.org/) (v18 veya üstü)

Node.js yoksa, resmi web sitesinden yükleyin.

### Kurulum

MCP yapılandırma dosyanıza aşağıdaki JSON'u ekleyin:

* **Cursor**: Proje kökünüzde `.cursor/mcp.json` veya genel `~/.cursor/mcp.json`
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

Kaydettikten sonra uygulamayı yeniden başlatın. MCP sunucusu otomatik olarak indirilip başlatılır. Aşağıdaki araçlar kullanılabilir hale gelir:

* **`current-time`** - Yapılandırılmış saat dilimlerinizde geçerli saati alın
* **`local-time`** - Kullanıcının saat diliminde (`convtz` yapılandırmasından veya sistem varsayılanından) geçerli yerel saati alın
* **`convert-time`** - Bir tarih-saat dizesini veya Epoch zaman damgasını birden fazla saat dilimine dönüştürün
* **`next-weekday`** - Belirli bir haftanın gününün bir sonraki tarihini bulun
* **`date-to-weekday`** - Belirli bir tarih için haftanın gününü alın
* **`days-until`** - Bugünden belirtilen tarihe kadar olan gün sayısını sayın
* **`days-between`** - İki tarih arasındaki gün sayısını sayın
* **`date-offset`** - Belirli bir tarihten N gün önce veya sonraki tarihi hesaplayın

### mclocks config ile nasıl çalışır

MCP sunucusu, mclocks `config.json` dosyanızı otomatik olarak okur ve kullanır:

* **`clocks`** - Saatlerinizde tanımlanan saat dilimleri varsayılan dönüştürme hedefleri olarak kullanılır
* **`convtz`** - Saat dilimi bilgisi içermeyen tarih-saat dizelerini dönüştürürken varsayılan kaynak saat dilimi olarak kullanılır
* **`usetz`** - Tarihsel olarak doğru UTC kaymaları için katı saat dilimi dönüştürmesini etkinleştirir (ör., JST 1888'den önce +09:18 idi). Tarihsel tarih-saatleri doğru şekilde dönüştürmeniz gerektiğinde `true` olarak ayarlayın

`config.json` bulunamazsa, sunucu yerleşik yaygın saat dilimleri kümesine geri döner (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### Ortam değişkenleri

`config.json` ayarlarını geçersiz kılmak istiyorsanız veya `config.json` dosyanız yoksa, MCP yapılandırmanızda ortam değişkenleri ayarlayabilirsiniz. Ortam değişkenleri `config.json`'daki değerlerden önceliklidir.

| Değişken | Açıklama | Varsayılan |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | `config.json` yolu. Çoğu durumda gerekli değildir, çünkü sunucu konumu otomatik olarak algılar. | otomatik algılama |
| `MCLOCKS_LOCALE` | Hafta günü adlarının biçimlendirilmesi vb. için yerel ayar (ör., `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Saat dilimi bilgisi içermeyen tarih-saat dizelerini yorumlamak için varsayılan kaynak saat dilimi (ör., `Asia/Tokyo`) | *(yok)* |
| `MCLOCKS_USETZ` | Katı saat dilimi dönüştürmesini etkinleştirmek için `true` olarak ayarlayın | `false` |

Örnek:

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

### Kullanım örneği

Yapılandırıldıktan sonra, yapay zeka asistanınıza şunlar gibi şeyler sorabilirsiniz:

* "Saat kaç?" - mclocks'ta yapılandırdığınız tüm saat dilimlerinde geçerli saati döndürür
* "Cakarta'da saat kaç?" - Belirli bir saat diliminde geçerli saati döndürür
* "1705312200 epoch'unu tarih-saate dönüştür"
* "2024-01-15T10:30:00Z'yi Asia/Tokyo'ya dönüştür"
* "Gelecek cuma hangi tarih?"
* "25 Aralık 2026 haftanın hangi günü?"
* "Noel'e kaç gün kaldı?"
* "1 Ocak 2026 ile 31 Aralık 2026 arasında kaç gün var?"
* "1 Nisan 2026'dan 90 gün sonrası hangi tarih?"

----------

## Lisans

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Yazar

Dai Okabayashi: https://github.com/bayashi
