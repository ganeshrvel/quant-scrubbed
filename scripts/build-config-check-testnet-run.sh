#!/bin/zsh

cargo build && ./target/debug/quant  --networktype=testnet --tradetype=buysell --configcheck

