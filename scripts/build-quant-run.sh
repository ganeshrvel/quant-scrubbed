#!/bin/zsh

cargo build && ./target/debug/quant "$@"

