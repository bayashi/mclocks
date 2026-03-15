# mclocks

다중 시간대를 위한 데스크톱 시계 애플리케이션🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

시계 관련 기능:

* 🕐 다중 시간대 텍스트 시계
* ⏱️ 타이머
* ⏳ 카운트다운 타이머
* 🔄 Epoch 시간과 날짜-시간 변환기

시간은 누구도 기다려주지 않습니다:

* 📝 스티커 메모

개발자에게 시계는 필수입니다:

* 🔀 간단한 텍스트 변환기
    * SQL `IN` 절을 쉽게 만드는 등
* 🌐 웹 서버
    * 정적 파일 제공
        * Markdown을 풍부하게 렌더링
        * 드래그 앤 드롭 기반 콘텐츠 뷰어
    * 요청 및 응답 덤프 서버
    * 디버깅을 위한 느린 엔드포인트
    * GitHub URL에서 편집기로 파일 열기

🔔 참고: `mclocks`는 인터넷 연결이 필요하지 않습니다 — 모든 것이 100% 로컬에서 실행됩니다.

## 📦 다운로드

https://github.com/bayashi/mclocks/releases 에서 다운로드

### Windows

Windows의 경우 설치 프로그램 `.msi` 파일 또는 실행 파일 `.exe`를 받을 수 있습니다.

### macOS

macOS의 경우 설치를 위한 `.dmg` 파일을 받을 수 있습니다.

(이 문서의 단축키는 Windows OS 기준입니다. macOS를 사용하는 경우 `Ctrl`을 `Ctrl + Command`로, `Alt`를 `Option`으로 적절히 바꿔서 해석해 주세요.)

## ⚙️ config.json

`config.json` 파일을 통해 시계를 원하는 대로 구성할 수 있습니다.

`config.json` 파일은 다음 디렉토리에 위치해야 합니다:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

`mclocks`를 시작한 후 `Ctrl + o`를 눌러 `config.json` 파일을 편집할 수 있습니다.

### config.json 예제

`config.json` 파일은 아래와 같이 JSON 형식으로 작성해야 합니다.

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

## 🔧 config.json의 필드

#### clocks

`clocks` 필드는 `name`과 `timezone` 속성을 포함하는 객체의 배열입니다. 둘 다 문자열이어야 합니다. 기본값은 둘 다 `UTC`입니다.

* `name`은 시계에 표시될 레이블입니다.
* 시간대 선택은 이 [시간대 목록](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones)을 참조하세요.

세 개의 시간대에 대한 `clocks` 배열 예제입니다.

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

문자열: `MM-DD ddd HH:mm`

`format` 필드는 시계를 표시하는 데 사용되는 날짜-시간 형식 문자열입니다. 사용자 정의 날짜-시간 형식을 만들려면 [이 포맷 가이드](https://momentjs.com/docs/#/parsing/string-format/)를 참조하세요.

#### format2

문자열: `""`

`format2` 필드는 `format`과 동일합니다. `Ctrl + f` 키로 서로 전환됩니다. `format2`는 선택적 필드입니다.

#### locale

문자열: `en`

`locale` 필드는 날짜-시간 표시의 언어 설정을 결정합니다. [지원되는 로케일 목록은 여기](https://github.com/kawanet/cdate-locale/blob/main/locales.yml)에서 확인할 수 있습니다.

#### color

문자열: `#fff`

`color` 필드는 날짜-시간 텍스트의 색상을 정의합니다. 이름이 있는 색상, RGB 16진수 값, RGB 값(예: `RGB(255, 0, 0)`) 또는 유효한 CSS 색상 값을 사용할 수 있습니다.

#### font

문자열: `Courier, monospace`

`font`는 날짜-시간을 표시하는 데 사용할 글꼴 이름입니다. 고정폭 글꼴이어야 합니다. 가변폭 글꼴을 설정하면 mclocks가 원치 않는 흔들림 효과를 보일 수 있습니다.

#### size

숫자 | 문자열: 14

`size`는 날짜-시간의 문자 크기로, 픽셀 단위입니다. 단위를 포함하는 문자열로도 지정할 수 있습니다(예: `"125%"`, `"1.5em"`).

#### margin

문자열: `1.65em`

`margin` 필드는 시계 간의 간격을 결정합니다.

#### forefront

불리언: `false`

`forefront` 필드를 `true`로 설정하면 mclocks 애플리케이션이 항상 다른 애플리케이션 창 위에 표시됩니다.

## ⏳ 카운트다운 시계

아래와 같이 `clock` 설정을 구성하면 지정된 `target` 날짜-시간까지의 카운트다운 시계로 표시됩니다.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

위의 카운트다운 `clock`은 다음과 같이 표시됩니다:

    WAC Tokyo D-159 12:34:56

2025년 9월 13일까지 159일 12시간 34분 56초가 남았음을 나타냅니다.

### 카운트다운 형식 변수

`countdown` 필드 텍스트는 다음 템플릿 변수를 사용할 수 있습니다:

* `%TG`: 대상 날짜-시간 문자열
* `%D`: 대상 날짜-시간까지 남은 일수
* `%H`: 대상 날짜-시간까지 남은 시간(시간 단위)
* `%h`: 남은 시간의 "시" 부분(hh:mm:ss의 hh)
* `%M`: 대상 날짜-시간까지 남은 시간(분 단위)
* `%m`: 남은 시간의 "분" 부분(hh:mm:ss의 mm)
* `%S`: 대상 날짜-시간까지 남은 시간(초 단위)
* `%s`: 남은 시간의 "초" 부분(hh:mm:ss의 ss)

## ⏱️ 간단한 타이머

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

`mclocks` 앱 창을 클릭한 후 `Ctrl + 1` 키를 누르면 1분 타이머가 시작됩니다. `Ctrl + Alt + 1` 키를 누르면 10분 타이머가 시작됩니다. 다른 숫자 키도 동일하게 작동합니다. 최대 5개의 타이머를 동시에 시작할 수 있습니다.

`Ctrl + p`로 타이머를 일시 정지/재개합니다.

`Ctrl + 0`으로 가장 오래된 타이머를 삭제합니다. `Ctrl + Alt + 0`으로 가장 최근 타이머를 삭제합니다.

🔔 참고: 카운트다운 시계와 간단한 타이머는 타이머 완료 시 기본적으로 알림을 보냅니다. 알림이 필요하지 않으면 `config.json`에서 `withoutNotification: true`를 설정하세요.

## 🔢 Epoch 시간 표시

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

`mclocks` 앱 창을 클릭한 후 `Ctrl + e` 키를 눌러 Epoch 시간 표시를 전환합니다.

## 🔄 날짜-시간과 Epoch 시간 간 변환

`mclocks` 앱 창을 클릭한 후 날짜-시간 또는 Epoch 시간을 붙여넣으면 변환 결과를 표시하는 대화 상자가 나타납니다. 결과를 클립보드에 복사할 수 있습니다. 복사하지 않으려면 `[No]`를 눌러 대화 상자를 닫으세요.

`Ctrl + v`로 붙여넣을 때 값(Epoch 시간)은 초로 처리됩니다. `Ctrl + Alt + v`는 밀리초, `Ctrl + Alt + Shift + V`는 마이크로초, `Ctrl + Alt + Shift + N + V`는 나노초로 변환됩니다.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

붙여넣은 날짜-시간 값에 시간대 정보가 포함되지 않은 경우 기본적으로 로컬 시간대로 처리됩니다. 특정 시간대로 처리하려면 convtz 옵션에 시간대를 설정하세요.

    "convtz": "UTC"

## 🔀 텍스트 변환 기능

`mclocks` 앱 창을 클릭한 후 다음 키보드 단축키를 사용하여 클립보드 텍스트를 처리하고 편집기에서 엽니다:

* `Ctrl + i`: 클립보드의 각 줄을 큰따옴표로 감싸고 끝에 쉼표를 추가합니다(마지막 줄 제외)
* `Ctrl + Shift + i`: 각 줄 끝에 쉼표를 추가합니다(따옴표 없음, INT 리스트 IN 조건용, 마지막 줄 제외)

빈 줄은 모든 작업에서 그대로 유지됩니다.

(이 텍스트 변환 기능은 시계나 시간과는 관련이 없지만, 소프트웨어 개발자에게는 유용할 수 있습니다! 😊)

## ⌨️ 키보드 단축키

### 도움말 표시

`F1` (Windows) 또는 `Cmd + Shift + /` (macOS)로 브라우저에서 도움말 페이지(이 README)를 엽니다

### 설정, 표시 형식

| 단축키 | 설명 |
|----------|-------------|
| `Ctrl + o` | 편집기에서 `config.json` 파일 열기 |
| `Ctrl + f` | `format`과 `format2` 간 전환 (`config.json`에 `format2`가 정의된 경우) |
| `Ctrl + e` 또는 `Ctrl + u` | Epoch 시간 표시 전환 |

### 타이머

| 단축키 | 설명 |
|----------|-------------|
| `Ctrl + 1` ~ `Ctrl + 9` | 타이머 시작 (1분 × 숫자 키) |
| `Ctrl + Alt + 1` ~ `Ctrl + Alt + 9` | 타이머 시작 (10분 × 숫자 키) |
| `Ctrl + p` | 모든 타이머 일시 정지 / 재개 |
| `Ctrl + 0` | 가장 오래된 타이머 삭제 |
| `Ctrl + Alt + 0` | 가장 최근 타이머 삭제 |

### 스티커 메모

| 단축키 | 설명 |
|----------|-------------|
| `Ctrl + s` | 클립보드 텍스트에서 새 스티커 메모 생성 |

### 클립보드 날짜-시간 작업

| 단축키 | 설명 |
|----------|-------------|
| `Ctrl + c` | 현재 mclocks 텍스트를 클립보드에 복사 |
| `Ctrl + v` | 클립보드 내용 변환 (Epoch 시간을 초로, 또는 날짜-시간) |
| `Ctrl + Alt + v` | 클립보드 내용 변환 (Epoch 시간을 밀리초로) |
| `Ctrl + Alt + Shift + V` | 클립보드 내용 변환 (Epoch 시간을 마이크로초로) |
| `Ctrl + Alt + Shift + N + V` | 클립보드 내용 변환 (Epoch 시간을 나노초로) |

### 텍스트 변환

| 단축키 | 설명 |
|----------|-------------|
| `Ctrl + i` | 클립보드의 각 줄을 큰따옴표로 감싸고 끝에 쉼표를 추가하여 편집기에서 열기 (마지막 줄 제외) |
| `Ctrl + Shift + i` | 각 줄 끝에 쉼표 추가 (따옴표 없음, INT 리스트 IN 조건용)하여 편집기에서 열기 (마지막 줄 제외) |

## 📝 스티커 메모

`mclocks` 앱 창을 클릭한 후 `Ctrl + s`를 눌러 클립보드 텍스트에서 스티커 메모를 생성합니다. 클립보드 내용이 담긴 작은 플로팅 창이 열립니다.

각 스티커 메모에는 다음 기능이 있습니다:

* **토글 버튼** (`▸` / `▾`): 메모를 펼치거나 접습니다. 접힌 상태에서는 한 줄만 표시됩니다.
* **복사 버튼** (`⧉`): 메모 텍스트를 클립보드에 복사합니다.
* **최상위 버튼** (`⊤` / `⊥`): 메모가 다른 창 위에 항상 표시되는지 여부를 전환합니다. 이 설정은 스티커 메모별로 저장됩니다.
* **닫기 버튼** (`✖`): 스티커 메모를 삭제하고 창을 닫습니다.
* **텍스트 영역**: 메모 내용을 자유롭게 편집합니다. 변경 사항은 자동으로 저장됩니다.
* **크기 조절 핸들**: 펼쳐진 상태에서 오른쪽 하단 모서리를 드래그하여 메모 크기를 조절합니다.

스티커 메모는 `config.json`의 `font`, `size`, `color`, `forefront` 설정을 상속합니다. 최상위 설정은 최상위 버튼을 사용하여 스티커 메모별로 재정의할 수 있으며, 재정의하지 않으면 `config.json`의 값이 사용됩니다. 위치, 크기, 열림/닫힘 상태, 최상위 재정의는 유지되며, `mclocks` 재시작 시 모든 메모가 자동으로 복원됩니다.

🔔 참고: macOS에서는 스티커 메모 창 위치가 애플리케이션 종료 시에만 저장됩니다. Windows에서는 창을 이동하거나 크기를 변경할 때 자동으로 저장됩니다.

스티커 메모당 최대 텍스트 크기는 128 KB입니다.

## 🌐 웹 서버

`mclocks`는 내장 웹 서버를 통해 정적 파일을 제공할 수 있습니다. 이 기능을 사용하면 코드 스니펫을 브라우저에서 쉽게 볼 수 있습니다. `config.json`에 `web` 필드를 추가하세요:

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

* `root`: 제공할 파일이 포함된 디렉토리 경로 (필수)
* `port`: 수신할 포트 번호 (기본값: 3030)
* `openBrowserAtStart`: `true`로 설정하면 `mclocks` 시작 시 기본 브라우저에서 웹 서버 URL을 자동으로 엽니다 (기본값: `false`)
* `dump`: `true`로 설정하면 요청 세부 정보를 JSON으로 반환하는 `/dump` 엔드포인트가 활성화됩니다 (기본값: `false`)
* `slow`: `true`로 설정하면 응답을 지연시키는 `/slow` 엔드포인트가 활성화됩니다 (기본값: `false`)
* `status`: `true`로 설정하면 임의의 HTTP 상태 코드를 반환하는 `/status/{code}` 엔드포인트가 활성화됩니다 (기본값: `false`)
* `content.markdown.allowRawHTML`: `true`로 설정하면 Markdown 렌더링에서 raw HTML을 허용하고, `false`이면 Markdown 내 raw HTML을 텍스트로 이스케이프합니다 (기본값: `false`)
* `editor`: 설정되어 있고 `reposDir`을 포함하면 브라우저의 GitHub URL에서 편집기로 로컬 파일을 여는 `/editor` 엔드포인트가 활성화됩니다 (기본값: 미설정)

`config.json`에 `web` 필드가 구성되어 있으면 `mclocks` 시작 시 웹 서버가 자동으로 시작됩니다. `http://127.0.0.1:3030`에서 파일에 접근할 수 있습니다. 웹 서버는 `127.0.0.1` (localhost)에서만 수신하므로 로컬 머신에서만 접근 가능합니다.

### 지원되는 파일 유형

웹 서버는 다음 파일 유형을 지원합니다:

* 텍스트: `html`, `css`, `js`, `json`, `md`, `txt`
* 이미지: `png`, `jpg`, `jpeg`, `gif`, `svg`, `ico`

### 드래그 앤 드롭 기반 콘텐츠 뷰어

정적 파일 호스팅 외에도 웹 서버에는 드래그 앤 드롭 기반 콘텐츠 뷰어 워크플로가 포함되어 있습니다. mclocks 시계 창에 파일 또는 디렉터리를 드래그 앤 드롭하면 임시 로컬 URL을 통해 열어서 볼 수 있습니다.
이 임시 URL은 mclocks가 종료될 때 폐기됩니다.

### /dump 엔드포인트

`web` 설정에서 `dump: true`가 설정되면 웹 서버는 요청 세부 정보를 JSON으로 반환하는 `/dump` 엔드포인트를 제공합니다.

엔드포인트는 다음 내용을 포함하는 JSON 객체로 응답합니다:
* `method`: HTTP 메서드 (예: "GET", "POST")
* `path`: `/dump/` 이후의 요청 경로 (예: `/dump/test`의 경우 "/test")
* `query`: 키-값 객체 배열로 된 쿼리 파라미터 (예: `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: 키-값 객체 배열로 된 요청 헤더 (예: `[{"Content-Type": "application/json"}]`)
* `body`: 문자열로 된 요청 본문 (있는 경우)
* `parsed_body`: Content-Type이 JSON을 나타내면 파싱된 JSON 객체, 파싱 실패 시 오류 메시지 문자열

`http://127.0.0.1:3030/dump` 또는 `/dump/` 하위 경로(예: `/dump/test?key=value`)에서 dump 엔드포인트에 접근합니다.

### /slow 엔드포인트

`web` 설정에서 `slow: true`가 설정되면 웹 서버는 200 OK를 반환하기 전에 응답을 지연시키는 `/slow` 엔드포인트를 제공합니다.

엔드포인트는 모든 HTTP 메서드(GET, POST 등)로 접근 가능하며 다음 경로를 지원합니다:

* `/slow`: 30초(기본값) 대기 후 200 OK 반환
* `/slow/120`: 120초(또는 지정된 초 수) 대기 후 200 OK 반환

최대 허용 값은 901초(15분 + 1초)입니다. 이 제한을 초과하는 요청은 400 Bad Request 오류를 반환합니다.

이 엔드포인트는 타임아웃 동작, 연결 처리 또는 느린 네트워크 상태 시뮬레이션 테스트에 유용합니다.

잘못된 초 파라미터가 제공되면(예: `/slow/abc`) 엔드포인트는 400 Bad Request 오류를 반환합니다.

### /status 엔드포인트

`web` 설정에서 `status: true`가 설정되면 웹 서버는 RFC 표준에 정의된 임의의 HTTP 상태 코드(100-599)를 반환하는 `/status/{code}` 엔드포인트를 제공합니다.

엔드포인트는 상태 코드와 해당 구문이 포함된 일반 텍스트 응답과 HTTP 사양에 따른 적절한 헤더를 반환합니다.

**예시:**
* `http://127.0.0.1:3030/status/200` - 200 OK 반환
* `http://127.0.0.1:3030/status/404` - 404 Not Found 반환
* `http://127.0.0.1:3030/status/500` - 500 Internal Server Error 반환
* `http://127.0.0.1:3030/status/418` - 418 I'm a teapot 반환 (특별 메시지 포함)
* `http://127.0.0.1:3030/status/301` - 301 Moved Permanently 반환 (Location 헤더 포함)

**상태별 헤더:**

엔드포인트는 특정 상태 코드에 대해 자동으로 적절한 헤더를 추가합니다:

* **3xx 리디렉션** (301, 302, 303, 305, 307, 308): `Location` 헤더 추가
* **401 Unauthorized**: `WWW-Authenticate` 헤더 추가
* **405 Method Not Allowed**: `Allow` 헤더 추가
* **407 Proxy Authentication Required**: `Proxy-Authenticate` 헤더 추가
* **416 Range Not Satisfiable**: `Content-Range` 헤더 추가
* **426 Upgrade Required**: `Upgrade` 헤더 추가
* **429 Too Many Requests**: `Retry-After` 헤더 추가 (60초)
* **503 Service Unavailable**: `Retry-After` 헤더 추가 (60초)
* **511 Network Authentication Required**: `WWW-Authenticate` 헤더 추가

**응답 본문 처리:**

* **204 No Content** 및 **304 Not Modified**: 빈 응답 본문 반환 (HTTP 사양에 따름)
* **418 I'm a teapot**: 표준 상태 구문 대신 특별 메시지 "I'm a teapot" 반환
* **기타 모든 상태 코드**: `{code} {phrase}` 형식의 일반 텍스트 반환 (예: "404 Not Found")

이 엔드포인트는 애플리케이션이 다양한 HTTP 상태 코드, 오류 처리, 리디렉션, 인증 요구 사항 및 속도 제한 시나리오를 어떻게 처리하는지 테스트하는 데 유용합니다.

### /editor 엔드포인트

설정 파일에서 `web.editor.reposDir`이 설정되면 웹 서버는 브라우저의 GitHub URL에서 편집기로 로컬 파일을 직접 열 수 있는 `/editor` 엔드포인트를 제공합니다.

**설정:**

`web` 설정에 다음을 추가하세요:

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

* `reposDir`: 로컬 리포지토리 디렉토리 경로. 홈 디렉토리 확장을 위해 `~`를 지원합니다 (예: macOS에서 `"~/repos"` 또는 Windows에서 `"C:/Users/username/repos"`). 이 디렉토리는 존재해야 합니다.
* `includeHost`: `true`이면 로컬 경로 해석에 원래 호스트를 디렉토리로 포함합니다 (예: `{reposDir}/{host}/{owner}/{repo}/...`). `false`이면 `{reposDir}/{owner}/{repo}/...`로 해석됩니다 (기본값: `false`).
* `command`: 편집기 실행 파일의 명령 이름 또는 경로 (기본값: `code`)
* `args`: 인수 템플릿 배열. `{file}` 및 `{line}` 자리표시자를 사용합니다. URL에 `#L...`이 없으면 `{line}`은 1을 사용합니다.

**작동 방식:**

1. `/editor` 엔드포인트를 통해 GitHub 파일 URL에 접근하면 GitHub 경로를 로컬 파일 경로로 변환합니다
2. 로컬 파일 경로는 다음과 같이 구성됩니다: `{reposDir}/{owner}/{repository_name}/{file_path}`
3. 파일이 존재하면 설정된 명령과 인수를 사용하여 지정된 줄 번호에서 편집기로 파일을 엽니다 (기본값: `code -g {local_file_path}:{line_number}`)
4. 파일이 존재하지 않으면 리포지토리 클론 링크가 포함된 오류 페이지가 표시됩니다

**북마클릿:**

GitHub 파일을 로컬 편집기에서 빠르게 열기 위한 북마클릿을 만드세요. `3030`을 설정한 포트 번호로 바꾸세요:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**줄 번호 지원:**

URL의 해시 프래그먼트를 사용하여 줄 번호를 지정할 수 있습니다:
* `https://github.com/username/repo/blob/main/file.rs#L123` → 123번째 줄에서 열기

**오류 처리:**

* 파일이 로컬에 존재하지 않으면 탭이 열린 상태로 유지되며 GitHub에서 리포지토리를 클론하는 링크가 포함된 오류 메시지가 표시됩니다
* 파일이 성공적으로 열리면 탭이 자동으로 닫힙니다
* `web.editor.reposDir`이 설정되지 않았거나 존재하지 않으면 `/editor` 엔드포인트가 활성화되지 않습니다 (404가 반환됩니다)

**예시:**

1. GitHub에서 파일을 보고 있습니다: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. 북마클릿을 클릭하거나 수동으로 이동합니다: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. 로컬에 `~/repos/mclocks/src/app.js`가 존재하면 VS Code가 42번째 줄에서 엽니다
4. 파일이 존재하지 않으면 클론을 위한 `https://github.com/bayashi/mclocks` 링크가 포함된 오류 페이지가 표시됩니다

----------

## 🧠 mclocks MCP 서버

`mclocks`에는 [Cursor](https://www.cursor.com/) 및 [Claude Desktop](https://claude.ai/download) 같은 AI 어시스턴트가 여러 시간대에서 "지금 몇 시야?"에 답하고 날짜-시간 형식과 Epoch 타임스탬프 간에 변환할 수 있게 하는 MCP(Model Context Protocol) 서버가 포함되어 있습니다. MCP 서버는 mclocks의 `config.json`을 자동으로 사용하므로 mclocks에서 설정한 시간대가 AI 응답에 반영됩니다.

### 전제 조건

* [Node.js](https://nodejs.org/) (v18 이상)

Node.js가 설치되어 있지 않으면 공식 웹사이트에서 설치하세요.

### 설정

MCP 설정 파일에 다음 JSON을 추가하세요:

* **Cursor**: 프로젝트 루트의 `.cursor/mcp.json` 또는 글로벌 `~/.cursor/mcp.json`
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

저장 후 애플리케이션을 재시작하세요. MCP 서버가 자동으로 다운로드되어 시작됩니다. 다음 도구들을 사용할 수 있게 됩니다:

* **`current-time`** - 설정된 시간대의 현재 시간 가져오기
* **`local-time`** - 사용자 시간대(`convtz` 설정 또는 시스템 기본값)의 현재 로컬 시간 가져오기
* **`convert-time`** - 날짜-시간 문자열 또는 Epoch 타임스탬프를 여러 시간대로 변환
* **`next-weekday`** - 지정한 요일의 다음 날짜 찾기
* **`date-to-weekday`** - 지정한 날짜의 요일 가져오기
* **`days-until`** - 오늘부터 지정한 날짜까지의 일수 계산
* **`days-between`** - 두 날짜 사이의 일수 계산
* **`date-offset`** - 지정한 날짜의 N일 전 또는 후 날짜 계산

### mclocks config과의 연동

MCP 서버는 mclocks의 `config.json`을 자동으로 읽어 다음을 사용합니다:

* **`clocks`** - 시계에 정의된 시간대가 기본 변환 대상으로 사용됩니다
* **`convtz`** - 시간대 정보가 없는 날짜-시간 문자열을 변환할 때 기본 소스 시간대로 사용됩니다
* **`usetz`** - 역사적으로 정확한 UTC 오프셋을 위한 엄격한 시간대 변환을 활성화합니다 (예: JST는 1888년 이전에 +09:18이었음). 역사적인 날짜-시간을 정확하게 변환해야 할 때 `true`로 설정하세요

`config.json`이 없으면 서버는 내장된 일반적인 시간대 세트로 대체합니다 (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### 환경 변수

`config.json` 설정을 재정의하거나 `config.json`이 없는 경우 MCP 설정에서 환경 변수를 설정할 수 있습니다. 환경 변수는 `config.json`의 값보다 우선합니다.

| 변수 | 설명 | 기본값 |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | `config.json` 경로. 대부분의 경우 서버가 위치를 자동 감지하므로 필요하지 않습니다. | 자동 감지 |
| `MCLOCKS_LOCALE` | 요일 이름 등의 포맷을 위한 로케일 (예: `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | 시간대 정보가 없는 날짜-시간 문자열을 해석하기 위한 기본 소스 시간대 (예: `Asia/Tokyo`) | *(없음)* |
| `MCLOCKS_USETZ` | 엄격한 시간대 변환을 활성화하려면 `true`로 설정 | `false` |

예시:

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

### 사용 예시

설정 후 AI 어시스턴트에게 다음과 같은 질문을 할 수 있습니다:

* "지금 몇 시야?" - mclocks에서 설정한 모든 시간대의 현재 시간을 반환합니다
* "자카르타는 지금 몇 시야?" - 특정 시간대의 현재 시간을 반환합니다
* "Epoch 1705312200을 날짜-시간으로 변환해줘"
* "2024-01-15T10:30:00Z를 Asia/Tokyo로 변환해줘"
* "다음 금요일은 며칠이야?"
* "2026년 12월 25일은 무슨 요일이야?"
* "크리스마스까지 며칠 남았어?"
* "2026년 1월 1일부터 2026년 12월 31일까지 며칠이야?"
* "2026년 4월 1일로부터 90일 후는 며칠이야?"

----------

## 라이선스

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## 저자

Dai Okabayashi: https://github.com/bayashi
