npm install -g javy-cli

curl https://wasmtime.dev/install.sh -sSf | bash

javy compile t01.js -o t01.wasm

time echo '{ "n": 2, "bar": "baz" }' | wasmtime t01.wasm

javy compile fib.js -o fib.wasm

time wasmtime fib.wasm

> 102334155
> real    0m36.842s
