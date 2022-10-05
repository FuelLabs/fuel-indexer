#!/bin/bash

cargo install wasm-snip

wasm-snip target/wasm32-unknown-unknown/release/${1} -o ./${1} -p __wbindgen
