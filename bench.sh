#!/bin/bash

rm -rf ~/.cache/absentis

echo "Runtime without db"

NO_CACHE="$(time ./target/release/absentis -V txs2.csv --to 6000000 --address fb6916095ca1df60bb79ce92ce3ea74c37c5d359)"

echo "Runtime with db"
CACHE="$(time ./target/release/absentis -V txs2.csv --to 6000000 --address fb6916095ca1df60bb79ce92ce3ea74c37c5d359)"

