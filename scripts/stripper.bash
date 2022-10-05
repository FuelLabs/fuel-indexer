#!/bin/bash

# Prerequisites:
#
#   cargo install wasm-snip
#
# Usage:
# 
# sh scripts/stripper.sh my_wasm_module_name.wasm

wasm-snip target/wasm32-unknown-unknown/release/${1} -o ./${1} -p __wbindgen
