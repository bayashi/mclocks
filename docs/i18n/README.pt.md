# mclocks

O aplicativo de relógio de desktop para múltiplos fusos horários🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

Recursos relacionados ao relógio:

* 🕐 Relógio de texto para múltiplos fusos horários
* ⏱️ Temporizador
* ⏳ Temporizador de contagem regressiva
* 🔄 Conversor entre tempo Epoch e data-hora

O tempo não espera por ninguém:

* 📝 Nota adesiva

Um desenvolvedor nunca fica sem relógio:

* 🔀 Conversor de texto simples
    * como criar facilmente cláusulas SQL `IN`
* 🌐 Servidor web
    * serve arquivos estáticos
        * renderiza Markdown de forma avançada
        * visualizador de conteúdo baseado em arrastar e soltar
    * servidor de dump de requisições e respostas
    * endpoints lentos para depuração
    * abrir arquivos no seu editor a partir de URLs do GitHub

🔔 NOTA: `mclocks` não precisa de conexão com a internet — tudo funciona 100% localmente.

## 📦 Download

Em https://github.com/bayashi/mclocks/releases

### Windows

Para Windows, você pode obter o instalador `.msi` ou o arquivo executável `.exe`.

### macOS

Para macOS, você pode obter o arquivo `.dmg` para instalar.

(Os atalhos de teclado neste documento são para Windows. Se você usa macOS, interprete-os adequadamente, substituindo teclas como `Ctrl` por `Ctrl + Command` e `Alt` por `Option` quando apropriado.)

## ⚙️ config.json

O arquivo `config.json` permite configurar os relógios de acordo com suas preferências.

O arquivo `config.json` deve estar localizado nos seguintes diretórios:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`

<!-- * Linux: `/home/{USER}/.config/com.bayashi.mclocks/` -->

Ao iniciar o `mclocks`, pressione `Ctrl + o` para editar seu arquivo `config.json`.

### Exemplo de config.json

O arquivo `config.json` deve ser formatado como JSON, conforme mostrado abaixo.

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

Você pode incluir comentários e vírgulas finais em `config.json` (compatível com JSONC).

### 🔧 Os campos do config.json

#### clocks

O campo `clocks` é um array de objetos, cada um contendo as propriedades `name` e `timezone`. Ambos devem ser strings. Por padrão, ambos são `UTC`.

* `name` é um rótulo que será exibido para o relógio.
* Para selecionar fusos horários, consulte esta [lista de fusos horários](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).

Aqui está um exemplo de um array `clocks` para três fusos horários.

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

O campo `format` é uma string de formato de data-hora usada para exibir o relógio. Para criar um formato personalizado, consulte [este guia de formatação](https://momentjs.com/docs/#/parsing/string-format/).

#### format2

string: `""`

O campo `format2` é igual ao `format`. Eles alternam entre si com a tecla `Ctrl + f`. O campo `format2` é opcional.

#### locale

string: `en`

O campo `locale` determina as configurações de idioma para exibir a data-hora. Você pode encontrar [uma lista de locales suportados aqui](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

string: `#fff`

O campo `color` define a cor do texto de data-hora. Você pode usar cores nomeadas, valores hexadecimais RGB, valores RGB (ex., `RGB(255, 0, 0)`) ou qualquer valor de cor CSS válido.

#### font

string: `Courier, monospace`

`font` é o nome da fonte para exibir a data-hora. Deve ser uma fonte monoespaçada. Se você definir uma fonte de largura variável, seu mclocks poderá ter um efeito de oscilação indesejável.

#### size

número | string: 14

`size` é o tamanho do caractere para a data-hora, em pixels. Também pode ser especificado como uma string que inclui uma unidade (ex., `"125%"`, `"1.5em"`).

#### margin

string: `1.65em`

O campo `margin` determina o espaço entre os relógios.

#### forefront

booleano: `false`

Se o campo `forefront` for definido como `true`, o aplicativo mclocks sempre será exibido acima das outras janelas de aplicativos.

## ⏳ Relógio de contagem regressiva

Ao configurar o `clock` conforme mostrado abaixo, ele será exibido como um relógio de contagem regressiva até a data-hora `target` especificada.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

O `clock` de contagem regressiva acima será exibido assim:

    WAC Tokyo D-159 12:34:56

Indicando que faltam 159 dias, 12 horas, 34 minutos e 56 segundos até 13 de setembro de 2025.

### Verbos de formato da contagem regressiva

O texto do campo `countdown` aceita os seguintes verbos de modelo:

* `%TG`: String de data-hora alvo
* `%D`: Contagem de dias restantes até a data-hora alvo
* `%H`: Tempo restante em horas até a data-hora alvo
* `%h`: A hora (hh) do tempo restante (hh:mm:ss)
* `%M`: Tempo restante em minutos até a data-hora alvo
* `%m`: O minuto (mm) do tempo restante (hh:mm:ss)
* `%S`: Tempo restante em segundos até a data-hora alvo
* `%s`: O segundo (ss) do tempo restante (hh:mm:ss)

## ⏱️ Temporizador simples

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

Clique na janela do aplicativo `mclocks`, depois pressione a tecla `Ctrl + 1` para iniciar um temporizador de 1 minuto. Pressione `Ctrl + Alt + 1` para iniciar um temporizador de 10 minutos. As outras teclas numéricas funcionam da mesma forma. Até 5 temporizadores podem ser iniciados simultaneamente.

`Ctrl + p` para pausar / retomar os temporizadores.

`Ctrl + 0` para excluir o temporizador mais antigo. `Ctrl + Alt + 0` para excluir o temporizador mais recente.

🔔 NOTA: O relógio de contagem regressiva e o temporizador simples enviam notificação por padrão quando o temporizador é concluído. Se você não precisa de notificações, defina `withoutNotification: true` no `config.json`.

## 🔢 Exibir tempo Epoch

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

Clique na janela do aplicativo `mclocks`, depois pressione a tecla `Ctrl + e` para alternar a exibição do tempo Epoch.

## 🔄 Converter entre data-hora e tempo Epoch

Clique na janela do aplicativo `mclocks`, depois cole uma data-hora ou tempo Epoch, e um diálogo aparecerá para exibir os resultados da conversão. Também é possível copiar os resultados para a área de transferência. Se não quiser copiar, pressione `[No]` para apenas fechar o diálogo.

Ao colar com `Ctrl + v`, o valor (tempo Epoch) é tratado como segundos. Se usar `Ctrl + Alt + v`, é tratado como milissegundos, com `Ctrl + Alt + Shift + V` como microssegundos, e com `Ctrl + Alt + Shift + N + V` como nanossegundos e convertido adequadamente.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

Quando os valores de data-hora colados não incluem informações de fuso horário, eles são tratados como fuso horário local por padrão. Para tratá-los como um fuso horário específico, defina o fuso horário na opção convtz.

    "convtz": "UTC"

## 🔀 Recurso de conversão de texto

Clique na janela do aplicativo `mclocks`, depois use os seguintes atalhos de teclado para processar o texto da área de transferência e abri-lo em um editor:

* `Ctrl + i`: Coloca cada linha do texto da área de transferência entre aspas duplas e adiciona uma vírgula no final (exceto a última linha)
* `Ctrl + Shift + i`: Adiciona uma vírgula no final de cada linha (sem aspas) para condição IN de lista INT (exceto a última linha)

Linhas vazias são preservadas como estão em todas as operações.

(Este recurso de conversão de texto não tem nada a ver com relógios ou tempo, mas desenvolvedores de software podem achá-lo útil! 😊)

## ⌨️ Atalhos de teclado

### Mostrar ajuda

`F1` (Windows) ou `Cmd + Shift + /` (macOS) para abrir a página de ajuda (este README) no navegador

### Configuração, formatos de exibição

| Atalho | Descrição |
|----------|-------------|
| `Ctrl + o` | Abrir o arquivo `config.json` no editor |
| `Ctrl + f` | Alternar entre `format` e `format2` (se `format2` estiver definido no `config.json`) |
| `Ctrl + e` ou `Ctrl + u` | Alternar a exibição do tempo Epoch |

### Temporizador

| Atalho | Descrição |
|----------|-------------|
| `Ctrl + 1` a `Ctrl + 9` | Iniciar temporizador (1 minuto × tecla numérica) |
| `Ctrl + Alt + 1` a `Ctrl + Alt + 9` | Iniciar temporizador (10 minutos × tecla numérica) |
| `Ctrl + p` | Pausar / retomar todos os temporizadores |
| `Ctrl + 0` | Excluir o temporizador mais antigo |
| `Ctrl + Alt + 0` | Excluir o temporizador mais recente |

### Nota adesiva

| Atalho | Descrição |
|----------|-------------|
| `Ctrl + s` | Criar uma nova nota adesiva a partir do texto da área de transferência |

### Operações de data-hora da área de transferência

| Atalho | Descrição |
|----------|-------------|
| `Ctrl + c` | Copiar o texto atual do mclocks para a área de transferência |
| `Ctrl + v` | Converter o conteúdo da área de transferência (tempo Epoch como segundos, ou data-hora) |
| `Ctrl + Alt + v` | Converter o conteúdo da área de transferência (tempo Epoch como milissegundos) |
| `Ctrl + Alt + Shift + V` | Converter o conteúdo da área de transferência (tempo Epoch como microssegundos) |
| `Ctrl + Alt + Shift + N + V` | Converter o conteúdo da área de transferência (tempo Epoch como nanossegundos) |

### Conversão de texto

| Atalho | Descrição |
|----------|-------------|
| `Ctrl + i` | Colocar cada linha da área de transferência entre aspas duplas, adicionar vírgula no final e abrir no editor (exceto a última linha) |
| `Ctrl + Shift + i` | Adicionar vírgula no final de cada linha (sem aspas) para condição IN de lista INT e abrir no editor (exceto a última linha) |

## 📝 Nota adesiva

![sticky-note](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-sticky-note.png)

Clique na janela do aplicativo `mclocks`, depois pressione `Ctrl + s` para criar uma nota adesiva a partir do texto da área de transferência. Uma pequena janela flutuante abrirá com o conteúdo da área de transferência.

Cada nota adesiva possui:

* **Botão de alternância** (`▸` / `▾`): Expandir ou recolher a nota. No modo recolhido, apenas uma linha é exibida.
* **Botão de copiar** (`⧉`): Copiar o texto da nota para a área de transferência.
* **Botão de primeiro plano** (`⊤` / `⊥`): Alternar se a nota permanece acima de outras janelas. Esta configuração é salva por nota adesiva.
* **Botão de fechar** (`✖`): Excluir a nota adesiva e fechar sua janela.
* **Área de texto**: Editar livremente o conteúdo da nota. As alterações são salvas automaticamente.
* **Alça de redimensionamento**: Arraste o canto inferior direito para redimensionar a nota quando expandida.

As notas adesivas herdam as configurações de `font`, `size`, `color` e `forefront` do `config.json`. A configuração de primeiro plano pode ser substituída por nota adesiva usando o botão de primeiro plano; se não for substituída, o valor do `config.json` é usado. Sua posição, tamanho, estado de aberto/fechado e substituição de primeiro plano são persistidos, e todas as notas são automaticamente restauradas quando o `mclocks` reinicia.

O tamanho máximo de texto por nota adesiva é de 128 KB.

## 🌐 Servidor web

O `mclocks` sempre inicia um servidor web local integrado na inicialização. Se você configurar um campo `web` no `config.json`, também poderá servir arquivos estáticos do seu diretório:

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

* `root`: Caminho para o diretório com os arquivos a servir (obrigatório apenas ao usar hospedagem de arquivos estáticos)
* `port`: Número de porta preferido do servidor web principal (`>=2000`, padrão: `3030`). Se estiver em uso, o mclocks tenta portas decrescentes (`-1`) até encontrar uma livre.
* `openBrowserAtStart`: Se `true`, abre automaticamente a URL do servidor web no navegador padrão ao iniciar o `mclocks` (padrão: `false`)
* `dump`: Se `true`, habilita o endpoint `/dump` que retorna detalhes da requisição em JSON (padrão: `false`)
* `slow`: Se `true`, habilita o endpoint `/slow` que atrasa a resposta (padrão: `false`)
* `status`: Se `true`, habilita o endpoint `/status/{code}` que retorna códigos de status HTTP arbitrários (padrão: `false`)
* `content.markdown.allowRawHTML`: Se `true`, permite HTML bruto na renderização Markdown; se `false`, o HTML bruto é escapado como texto (padrão: `false`)
* `content.markdown.openExternalLinkInNewTab`: Links Markdown externos abrem em nova aba e internos na mesma; se `false`, todos os links Markdown abrem na mesma aba (padrão: `true`)
* `content.markdown.enablePreviewApi`: Se `true`, habilita `POST /preview` para pré-visualizar Markdown a partir da CLI no navegador (padrão: `false`).
* `editor`: Se definido e contiver `reposDir`, habilita o endpoint `/editor` que abre arquivos locais no editor a partir de URLs do GitHub no navegador (padrão: não definido)

### Visualizador de conteúdo por arrastar e soltar

Além da hospedagem de arquivos estáticos, o mclocks oferece um fluxo de visualização por arrastar e soltar:

* Solte um diretório na janela do relógio para abri-lo no visualizador web por meio de uma URL local temporária.
* Solte um único arquivo para abri-lo no visualizador web quando o tipo for suportado pelo visualizador de arquivos temporários.
* As URLs temporárias geradas são apenas locais e são descartadas ao sair do mclocks.

#### Modo de conteúdo

O visualizador web aceita opções de consulta `mode`, como `content`, `raw` e `source`.

* `content` (padrão): Serve o arquivo com o tipo de conteúdo detectado para renderização normal no navegador quando possível.
* `raw`: Retorna arquivos não binários como `text/plain` para exibir texto bruto sem renderização do navegador.
* `source`: Abre o layout do visualizador de código com resumo/barra lateral para formatos suportados e permite inspeção em texto simples para arquivos de texto não suportados.

O **Markdown** **detecta alterações automaticamente e atualiza o navegador em tempo real** (exibição renderizada no modo **`source`**).

### Endpoint /dump

Quando `dump: true` é definido na configuração `web`, o servidor web fornece um endpoint `/dump` que retorna detalhes da requisição como JSON.

O endpoint responde com um objeto JSON contendo:
* `method`: Método HTTP (ex., "GET", "POST")
* `path`: Caminho da requisição após `/dump/` (ex., "/test" para `/dump/test`)
* `query`: Parâmetros de consulta como um array de objetos chave-valor (ex., `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: Cabeçalhos da requisição como um array de objetos chave-valor (ex., `[{"Content-Type": "application/json"}]`)
* `body`: Corpo da requisição como string (se presente)
* `parsed_body`: Objeto JSON parseado se o Content-Type indica JSON, ou string de mensagem de erro se o parsing falhar

Acesse o endpoint dump em `http://127.0.0.1:3030/dump` ou qualquer caminho sob `/dump/` (ex., `/dump/test?key=value`).

### Endpoint /slow

Quando `slow: true` é definido na configuração `web`, o servidor web fornece um endpoint `/slow` que atrasa a resposta antes de retornar 200 OK.

O endpoint é acessível via qualquer método HTTP (GET, POST, etc.) e suporta os seguintes caminhos:

* `/slow`: Aguarda 30 segundos (padrão) e retorna 200 OK
* `/slow/120`: Aguarda 120 segundos (ou qualquer número especificado de segundos) e retorna 200 OK

O valor máximo permitido é 901 segundos (15 minutos + 1 segundo). Requisições que excedam este limite retornam um erro 400 Bad Request.

Este endpoint é útil para testar comportamento de timeout, tratamento de conexões ou simulação de condições de rede lentas.

Se um parâmetro de segundos inválido for fornecido (ex., `/slow/abc`), o endpoint retorna um erro 400 Bad Request.

### Endpoint /status

Quando `status: true` é definido na configuração `web`, o servidor web fornece um endpoint `/status/{code}` que retorna códigos de status HTTP arbitrários definidos nos padrões RFC (100-599).

O endpoint retorna uma resposta em texto simples com o código de status e sua frase correspondente, junto com cabeçalhos apropriados conforme exigido pela especificação HTTP.

**Exemplos:**
* `http://127.0.0.1:3030/status/200` - retorna 200 OK
* `http://127.0.0.1:3030/status/404` - retorna 404 Not Found
* `http://127.0.0.1:3030/status/500` - retorna 500 Internal Server Error
* `http://127.0.0.1:3030/status/418` - retorna 418 I'm a teapot (com mensagem especial)
* `http://127.0.0.1:3030/status/301` - retorna 301 Moved Permanently (com cabeçalho Location)

**Cabeçalhos específicos por status:**

O endpoint adiciona automaticamente cabeçalhos apropriados para códigos de status específicos:

* **3xx Redirecionamento** (301, 302, 303, 305, 307, 308): Adiciona cabeçalho `Location`
* **401 Unauthorized**: Adiciona cabeçalho `WWW-Authenticate`
* **405 Method Not Allowed**: Adiciona cabeçalho `Allow`
* **407 Proxy Authentication Required**: Adiciona cabeçalho `Proxy-Authenticate`
* **416 Range Not Satisfiable**: Adiciona cabeçalho `Content-Range`
* **426 Upgrade Required**: Adiciona cabeçalho `Upgrade`
* **429 Too Many Requests**: Adiciona cabeçalho `Retry-After` (60 segundos)
* **503 Service Unavailable**: Adiciona cabeçalho `Retry-After` (60 segundos)
* **511 Network Authentication Required**: Adiciona cabeçalho `WWW-Authenticate`

**Tratamento do corpo da resposta:**

* **204 No Content** e **304 Not Modified**: Retorna corpo de resposta vazio (conforme especificação HTTP)
* **418 I'm a teapot**: Retorna mensagem especial "I'm a teapot" em vez da frase de status padrão
* **Todos os outros códigos de status**: Retorna texto simples no formato `{code} {phrase}` (ex., "404 Not Found")

Este endpoint é útil para testar como seus aplicativos lidam com diferentes códigos de status HTTP, tratamento de erros, redirecionamentos, requisitos de autenticação e cenários de limitação de taxa.

### Endpoint /editor

Quando `web.editor.reposDir` é definido no arquivo de configuração, o servidor web fornece um endpoint `/editor` que permite abrir arquivos locais no seu editor diretamente a partir de URLs do GitHub no navegador.

**Configuração:**

Adicione o seguinte à sua configuração `web`:

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

* `reposDir`: Caminho para o diretório dos seus repositórios locais. Suporta `~` para expansão do diretório home (ex., `"~/repos"` no macOS ou `"C:/Users/username/repos"` no Windows). Este diretório deve existir.
* `includeHost`: Se `true`, a resolução do caminho local inclui o host original como diretório (ex., `{reposDir}/{host}/{owner}/{repo}/...`). Se `false`, resolve para `{reposDir}/{owner}/{repo}/...` (padrão: `false`).
* `command`: Nome do comando ou caminho para o executável do seu editor (padrão: `code`)
* `args`: Array de modelo de argumentos. Use os marcadores `{file}` e `{line}`. Se `#L...` não estiver presente na URL, `{line}` usa 1.

**Como funciona:**

1. Quando você acessa uma URL de arquivo do GitHub através do endpoint `/editor`, ele converte o caminho do GitHub em um caminho de arquivo local
2. O caminho do arquivo local é construído como: `{reposDir}/{owner}/{repository_name}/{file_path}`
3. Se o arquivo existir, ele o abre no seu editor no número de linha especificado usando o comando e argumentos configurados (padrão: `code -g {local_file_path}:{line_number}`)
4. Se o arquivo não existir, uma página de erro é exibida com um link para clonar o repositório

**Bookmarklet:**

Crie um bookmarklet para abrir rapidamente arquivos do GitHub no seu editor local. Substitua `3030` pelo seu número de porta configurado:

```javascript
javascript:(function(){var u=new URL(document.location.href);open('http://127.0.0.1:3030/editor/'+u.host+u.pathname+u.hash,'_blank');})()
```

**Suporte a número de linha:**

Você pode especificar um número de linha usando o fragmento hash na URL:
* `https://github.com/username/repo/blob/main/file.rs#L123` → Abre na linha 123

**Tratamento de erros:**

* Se o arquivo não existir localmente, a aba permanece aberta e exibe uma mensagem de erro com um link para clonar o repositório do GitHub
* Se o arquivo for aberto com sucesso, a aba fecha automaticamente
* Se `web.editor.reposDir` não estiver configurado ou não existir, o endpoint `/editor` não é habilitado (e você receberá um 404)

**Exemplo:**

1. Você está visualizando um arquivo no GitHub: `https://github.com/bayashi/mclocks/blob/main/src/app.js#L42`
2. Clique no bookmarklet ou navegue manualmente para: `http://127.0.0.1:3030/editor/bayashi/mclocks/blob/main/src/app.js#L42`
3. Se `~/repos/mclocks/src/app.js` existir no seu local, o VS Code o abre na linha 42
4. Se o arquivo não existir, uma página de erro é exibida com um link para `https://github.com/bayashi/mclocks` para clonar

----------

## 🧠 Servidor MCP do mclocks

O `mclocks` inclui um servidor MCP (Model Context Protocol) que permite a assistentes de IA como [Cursor](https://www.cursor.com/) e [Claude Desktop](https://claude.ai/download) responder "Que horas são?" em múltiplos fusos horários, e converter entre formatos de data-hora e timestamps Epoch. O servidor MCP usa automaticamente seu `config.json` do mclocks, então os fusos horários configurados no mclocks são refletidos nas respostas da IA.

### Pré-requisitos

* [Node.js](https://nodejs.org/) (v18 ou posterior)

Se você não tem o Node.js, instale-o a partir do site oficial.

### Configuração

Adicione o seguinte JSON ao seu arquivo de configuração MCP:

* **Cursor**: `.cursor/mcp.json` na raiz do seu projeto, ou global `~/.cursor/mcp.json`
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

Após salvar, reinicie o aplicativo. O servidor MCP será automaticamente baixado e iniciado. As seguintes ferramentas ficam disponíveis:

* **`current-time`** - Obter a hora atual nos seus fusos horários configurados
* **`local-time`** - Obter a hora local atual no fuso horário do usuário (da configuração `convtz` ou padrão do sistema)
* **`convert-time`** - Converter uma string de data-hora ou timestamp Epoch para múltiplos fusos horários
* **`next-weekday`** - Encontrar a data da próxima ocorrência de um dia da semana
* **`date-to-weekday`** - Obter o dia da semana para uma data específica
* **`days-until`** - Contar o número de dias de hoje até uma data especificada
* **`days-between`** - Contar o número de dias entre duas datas
* **`date-offset`** - Calcular a data N dias antes ou depois de uma data específica

### Como funciona com a configuração do mclocks

O servidor MCP lê automaticamente seu `config.json` do mclocks e usa:

* **`clocks`** - Os fusos horários definidos nos seus relógios são usados como destinos de conversão padrão
* **`convtz`** - Usado como o fuso horário de origem padrão ao converter strings de data-hora sem informação de fuso horário
* **`usetz`** - Habilita conversão estrita de fusos horários para offsets UTC historicamente precisos (ex., JST era +09:18 antes de 1888). Defina como `true` quando precisar converter datas-horas históricas com precisão

Se nenhum `config.json` for encontrado, o servidor recorre a um conjunto integrado de fusos horários comuns (UTC, America/New_York, America/Los_Angeles, Europe/London, Europe/Berlin, Asia/Tokyo, Asia/Shanghai, Asia/Kolkata, Australia/Sydney).

### Variáveis de ambiente

Se você quiser substituir as configurações do `config.json`, ou se não tiver um `config.json`, pode definir variáveis de ambiente na sua configuração MCP. As variáveis de ambiente têm prioridade sobre os valores no `config.json`.

| Variável | Descrição | Padrão |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | Caminho para o `config.json`. Não é necessário na maioria dos casos, pois o servidor detecta automaticamente a localização. | detecção automática |
| `MCLOCKS_LOCALE` | Locale para formatação de nomes de dias da semana, etc. (ex., `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Fuso horário de origem padrão para interpretar strings de data-hora sem informação de fuso horário (ex., `Asia/Tokyo`) | *(nenhum)* |
| `MCLOCKS_USETZ` | Definir como `true` para habilitar conversão estrita de fusos horários | `false` |

Exemplo:

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

### Exemplo de uso

Uma vez configurado, você pode perguntar ao seu assistente de IA coisas como:

* "Que horas são?" - Retorna a hora atual em todos os seus fusos horários configurados no mclocks
* "Que horas são em Jacarta?" - Retorna a hora atual em um fuso horário específico
* "Converta o epoch 1705312200 para data-hora"
* "Converta 2024-01-15T10:30:00Z para Asia/Tokyo"
* "Qual a data da próxima sexta-feira?"
* "Que dia da semana é 25 de dezembro de 2026?"
* "Quantos dias faltam para o Natal?"
* "Quantos dias entre 1 de janeiro de 2026 e 31 de dezembro de 2026?"
* "Qual a data 90 dias após 1 de abril de 2026?"

----------

## Licença

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Autor

Dai Okabayashi: https://github.com/bayashi
