RUST_LOG=info cargo run -- > out.txt 2>&1

echo "Extensions found:"
rg "\.\w*\$" -o out.txt | sort -u

echo "sites found"
rg '<https*://([\w.]+)\.mit\.edu' -oNr '$1'  out.txt | sort -u