#!/usr/bin/env bash

source ./filename.sh

#cmd=binance-cli
cmd="cargo run --release"
$cmd pbcthf -f $n -o $(mkfullname $dn $bn pbcthf.csv)
$cmd ttffbcthf -f $(mkfullname $dn $bn pbcthf.csv) -o $(mkfullname $dn $bn ttf.csv)
$cmd cttf -f $(mkfullname $dn $bn ttf.csv) -o $(mkfullname $dn $bn cttf.csv)
