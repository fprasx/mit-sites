#!/bin/sh

logfile="~/Code/mit-sites/log/log-$(date +%y-%m-%d-%s)"
outfile="~/Code/mit-sites/out/out-$(date +%y-%m-%d-%s)"

touch "$logfile"
touch "$outfile"

RUST_LOG=info ~/.cargo/bin/cargo run --manifest-path "~/Code/mit-sites/Cargo.toml" -q -- 2> "$logfile" 1> "$outfile"

/usr/local/bin/rg '<https*://([\w.]+)\.mit\.edu' -oNr '$1'  "$logfile" >> "$outfile"

sort -u -o "$outfile" "$outfile"
