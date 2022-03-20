#!/bin/zsh

cargo build && ./target/debug/quant  --networktype=mainnet --tradetype=buysell --configcheck

