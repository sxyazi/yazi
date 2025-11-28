#!/bin/bash

arch="apple"
min="11.0"
sdk="11.3"
plaintext=0
while getopts a:m:s:p opt
do
   case "$opt" in
       a) arch="$OPTARG" ;;
       m) min="$OPTARG" ;;
       s) sdk="$OPTARG" ;;
       p) plaintext=1 ;; # For debugging
       *) echo >&2 "unknown flag $opt"
          exit 1
          ;;
   esac
done

buildargs=(--build-arg MACMIN="$min" --build-arg OWNSDK="$sdk")

case "$arch" in
    apple) buildargs+=(--build-arg ARCH="aarch64") ;;
    intel) buildargs+=(--build-arg ARCH="x86_64") ;;
    *) echo >&2 "unknown architecture $arch"
       exit 2
       ;;
esac

if [[ $plaintext = 1 ]] ; then
    buildargs+=(--progress=plain)
fi

dir="$( dirname "$0" )"
docker build -o "platform/built/$(basename "$dir")/$arch/" -f "$dir/Dockerfile" "${buildargs[@]}" .
