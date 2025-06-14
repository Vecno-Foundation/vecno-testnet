## WASM32 bindings for Vecno SDK

Vecno WASM32 bindings offer direct integration of Rust code and Vecno
codebase within JavaScript and TypeScript environments such as Node.js and Web Browsers.

Please note that while WASM directly binds JavaScript and Rust resources, their names on JavaScript side
are different from their name in Rust as they conform to the 'camelCase' convention in JavaScript and
to the 'snake_case' convention in Rust.

The WASM32 bindings can be used in both TypeScript and JavaScript environments, where in JavaScript
types will not be constrained by TypeScript type definitions.

## Interfaces

The SDK is currently separated into the following top-level categories:

- **RPC API** — RPC API for the Vecno node using WebSockets.
- **Wallet SDK** — Bindings for primitives related to key management and transactions.
- **Wallet API** — API for the Vecno Wallet framework.

## WASM32 SDK release packages

The SDK is built as 4 packages for Web Browsers as follows:

- KeyGen - Key & Address Generation only
- RPC - RPC only
- Core - RPC + Key & Address Generation + Wallet SDK
- Full - Full SDK + Integrated Wallet
  For NodeJS, the SDK is built as a single package containing all features.

## SDK folder structure

The following is a brief overview of the SDK folder structure (as available in the release):

- `web/vecno` - **full** Vecno WASM32 SDK bindings for use in web browsers.
- `web/vecno-rpc` - only the RPC bindings for use in web browsers (reduced WASM binary size).
- `nodejs/vecno` - **full** Vecno WASM32 SDK bindings for use with NodeJS.
- `docs` - Vecno WASM32 SDK documentation.
- `examples` folders contain examples for NodeJS and web browsers.
- `examples/data` - folder user by examples for configuration and wallet data storage.
- `examples/javascript` - JavaScript examples.
- `examples/javascript/general` - General SDK examples (keys & derivation, addresses, encryption, etc.).
- `examples/javascript/transactions` - Creating, sending and receiving transactions.
- `examples/javascript/wallet` - Interfacing with the Vecno Wallet framework.
- `examples/typescript` - TypeScript examples.

If you are using JavaScript and Visual Studio Code, it is highly recommended you replicate
the `jsconfig.json` configuration file as is done in the SDK examples. This file allows
Visual Studio to provide TypeScript-like code completion, type checking and documentation.

Included documentation in the release can be accessed by loading the `docs/vecno/index.html`
file in a web browser.

## Building from Source

To build the WASM32 SDK from source, you need to have the Rust environment installed. To do that,
follow instructions in the [Vecno README](https://github.com/Vecno-Foundation/vecnod).

Once you have Rust installed, you can build the WASM32 SDK as follows:

- `./build-release` - build the release version of the WASM32 SDK + Docs. The release version also contains `debug` builds of the libraries.
- `./build-web` - build the web package (ES6 module)
- `./build-node` - build the NodeJS package (CommonJS module)
- `./build-docs` - runs `build-web` and then generates TypeDoc documentation from the resulting build.

Please note that to build from source, you need to have TypeDoc installed globally via `npm install -g typedoc` (see below).

## Running Web examples

**IMPORTANT:** To view web examples, you need to serve them from a local web server and
serve them from the root of the SDK folder (`vecno-wasm32-sdk` if using a redistributable or
`vecno/wasm` if building from source). This is because examples use relative paths.
WASM32 currently can not be loaded using the `file://` protocol.

You can use any web server of your choice. If you don't have one, you can run one as follows:

```bash
cargo install http-server
http-server
```

Access the examples at  [http://localhost:7878/examples/web/index.html](http://localhost:7878/examples/web/index.html).
(Make sure to change the port if you are using a different server. Many servers will serve on
[http://localhost:8000/examples/web/index.html](http://localhost:8000/examples/web/index.html) by default)

If building from source, you must run `build-release` or `build-web` scripts before running the examples.

## Running NodeJs examples

This applies to running examples while building the project from source as some dependencies are instantiated as a part of the build process. You just need to run `node init` to initialize a local config.

NOTES:

- `npm install` will install NodeJs types for TypeScript and W3C websocket modules
- `npm install -g typedoc` is needed for the release build to generate documentation
- `node init` creates a local `examples/data/config.json` that contains a private key (mnemonic) use across NodeJS examples. You can override address used in some examples by specifying the address as a command line argument.
- Majority of examples will accept following arguments: `node <script> [address] [mainnet|testnet-10|testnet-11] [--address <address>] [--network <mainnet|testnet-10|testnet-11>] [--encoding <borsh|json>]`.

  By default all wRPC connections use Borsh binary encoding.

Example:

```bash
cd wasm
./build-release
cd examples
npm install
node init
node nodejs/javascript/general/rpc.js
```

## Using RPC

There are multiple ways to use RPC:

- Control over WebSocket-framed JSON-RPC protocol (you have to manually handle serialization)
- Use `RpcClient` class that handles the connectivity automatically and provides RPC interfaces in a form of async function calls.

**NODEJS:** To use WASM RPC client in the Node.js environment, you need to introduce a W3C WebSocket object
before loading the WASM32 library. The compatible WebSocket library is [WebSocket](https://www.npmjs.com/package/websocket) and is included in the `vecno` NPM package. `vecno` package is a wrapper around `vecno-wasm` that imports and installs this WebSocket shim in the `globalThis` object and then re-exports `vecno-wasm` exports.

## Loading in a Web App

```html
<html>
    <head>
        <script type="module">
            import * as vecno from './vecno/vecno-wasm.js';
            (async () => {
                await vecno.default('./vecno/vecno-wasm_bg.wasm');
                console.log(vecno.version());
                // ...
            })();
        </script>
    </head>
    <body></body>
</html>
```

## Loading in a Node.js App

```javascript
//
// W3C WebSocket module shim
// this is provided by NPM `vecno` module and is only needed
// if you are building WASM libraries for NodeJS from source
//
// @ts-ignore
// globalThis.WebSocket = require('websocket').w3cwebsocket;
//

let {
    RpcClient,
    Encoding,
    initConsolePanicHook
} = require('./vecno');

// enabling console panic hooks allows WASM to print panic details to console
// initConsolePanicHook();
// enabling browser panic hooks will create a full-page DIV with panic details
// this is useful for mobile devices where console is not available
// initBrowserPanicHook();
```

```javascript
// if port is not specified, it will use the default port for the specified network
const rpc = new RpcClient({
    url: "127.0.0.1", 
    encoding: Encoding.Borsh, 
    network : "testnet-10"
});

(async () => {
    try {
        await rpc.connect();
        let info = await rpc.getInfo();
        console.log(info);
    } finally {
        await rpc.disconnect();
    }
})();
```

For more details, please follow the **integrating with Vecno** guide.

## Creating Documentation

Please note that to build documentation from source you need to have the Rust environment installed.
The build script will first build the WASM32 SDK and then generate typedoc documentation from it.

You can build documentation from source as follows:

```bash
npm install -g typedoc
./build-docs
```

The resulting documentation will be located in `docs/typedoc/`
