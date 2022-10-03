#!/usr/bin/env bash
# Exports the following from parsing $1:
#   on: original name
#   dn: directory of "." if none
#   fn: filename with extension
#   bn: filename without exentsion
#   ext: extension of filename or empty if none

export on dn fn bn ext

on="$1"

# if $n has no path "." is returned
dn="$(dirname "$on")"

# Get the "filename" including the extenstion
fn="$(basename "$on")"

# Get the extentsion and basename (filename without extension)
ext="${fn##*.}"
if [[ "$ext" == "$fn" ]]; then
  # There was no extension and basename is fn
  ext=""
  bn="$fn"
else
  # There is an extention, extract the basename without extension
  bn="$(basename "$on" ".$ext")"
fi

# Make a fullname with new extension
function mkfullname () {
    echo "$1/$2.$3"
}

# echo on: $on
# echo dn: $dn
# echo bn: $bn
# echo ext: $ext
# fullname=$(mkfullname $dn $bn pbudf.csv)
# echo fullname: $fullname

