# How to develop mclocks

## How to develop on Windows

### install node

<https://nodejs.org/ja/download/>

```
$ which node

CommandType     Name                                               Version    Source
-----------     ----                                               -------    ------
Application     node.exe                                           18.12.1.0  C:\Program Files\nodejs\node.exe

$ which npm

CommandType     Name                                               Version    Source
-----------     ----                                               -------    ------
Application     npm.cmd                                            0.0.0.0    C:\Program Files\nodejs\npm.cmd

$ node -v
v18.12.1
```

### create PR

Invoke mclocks:

    git clone git@github.com:bayashi/mclocks.git
    cd mclocks
    git checkout -b example-branch
    npm install
    electron src --debug

And you can change code.

Build locally:

    node_modules/.bin/electron-builder --win

When the build gets success, you can see the mclocks in the `dist` directory.
