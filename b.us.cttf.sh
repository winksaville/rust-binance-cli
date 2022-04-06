#!/usr/bin/env bash

set -euo pipefail

# Parse the filename
source "$(dirname "$0")/filename.sh" "$1"

#cmd=binance-cli
cmd="cargo run --release"
$cmd pbudf --no-progress-info --no-usd-value-needed -f "$on" -o "$(mkfullname "$dn" "$bn" pbudf.csv)" | tee "$(mkfullname "$dn" "$bn" pbudf.csv.result.txt)"
$cmd ttffbudf --no-progress-info -f "$(mkfullname "$dn" "$bn" pbudf.csv)" -o "$(mkfullname "$dn" "$bn" ttf.csv)"
$cmd pttf --no-progress-info --no-usd-value-needed -f "$(mkfullname "$dn" "$bn" ttf.csv)" | tee "$(mkfullname "$dn" "$bn" ttf.csv.pttf.result.txt)"
$cmd cttf --no-progress-info -f "$(mkfullname "$dn" "$bn" ttf.csv)" -o "$(mkfullname "$dn" "$bn" cttf.csv)"
$cmd pttf --no-progress-info --no-usd-value-needed -f "$(mkfullname "$dn" "$bn" cttf.csv)" | tee "$(mkfullname "$dn" "$bn" cttf.csv.pttf.result.txt)"

echo Done "$0"
