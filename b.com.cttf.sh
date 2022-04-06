#!/usr/bin/env bash

set -euo pipefail

# Parse the filename
source "$(dirname "$0")/filename.sh" "$1"

#cmd=binance-cli
cmd="cargo run --release"
$cmd pbcthf --no-progress-info -f "$on" -o "$(mkfullname "$dn" "$bn" pbcthf.csv)" | tee "$(mkfullname "$dn" "$bn" pbcthf.csv.result.txt)"
$cmd ttffbcthf --no-progress-info -f "$(mkfullname "$dn" "$bn" pbcthf.csv)" -o "$(mkfullname "$dn" "$bn" ttf.csv)"
$cmd pttf --no-progress-info --no-usd-value-needed -f "$(mkfullname "$dn" "$bn" ttf.csv)" | tee "$(mkfullname "$dn" "$bn" ttf.csv.pttf.result.txt)"
$cmd cttf --no-progress-info -f "$(mkfullname "$dn" "$bn" ttf.csv)" -o "$(mkfullname "$dn" "$bn" cttf.csv)"
$cmd pttf --no-progress-info --no-usd-value-needed -f "$(mkfullname "$dn" "$bn" cttf.csv)" | tee "$(mkfullname "$dn" "$bn" cttf.csv.pttf.result.txt)"

echo Done "$0"
