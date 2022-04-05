#!/usr/bin/env bash
# converts first parameter, $1 to:
# n: original parameter
# dn: diretory of "." if none
# fn: filename with extension
# bn: filename without exentsion
# ext: extension of filename or empty if none

n=$1

# if $n has not path "." is returned
dn="$(dirname "$n")"

# Get the "filename" including the extenstion
fn="$(basename "$n")"

# Get the extentsion and basename (filename without extension)
ext="${fn##*.}"
if [[ "$ext" == "$fn" ]]; then
  # There was no extension and basename is fn
  ext=""
  bn="$fn"
else
  # There is an extention, extract the basename without extension
  bn="$(basename "$n" ".$ext")"
fi

# Make a fullname with new extension
function mkfullname () {
    echo "$1/$2.$3"
}

# echo n: $n
# echo dn: $dn
# echo bn: $bn
# echo ext: $ext
# fullname=$(mkfullname $dn $bn pbudf.csv)
# echo fullname: $fullname

