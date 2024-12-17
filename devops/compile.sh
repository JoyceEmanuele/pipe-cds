#!/bin/bash

RUSTFLAGS=-Awarnings cargo build --release || exit 1

./target/release/computed-data-service --test-config ./configfile_example.json5 || exit 1
./target/release/computed-data-service --test-config

mv computed-data-service computed-data-service-`date '+%FT%T'`
cp ./target/release/computed-data-service .
