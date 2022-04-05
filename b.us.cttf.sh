#!/usr/bin/env bash

source ./filename.sh

#cmd=binance-cli
cmd="cargo run --release"
$cmd pbudf --no-usd-value-needed -f $n -o $(mkfullname $dn $bn pbudf.csv)
$cmd ttffbudf -f $(mkfullname $dn $bn pbudf.csv) -o $(mkfullname $dn $bn ttf.csv)
$cmd cttf -f $(mkfullname $dn $bn ttf.csv) -o $(mkfullname $dn $bn cttf.csv)
