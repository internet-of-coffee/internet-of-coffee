#!/bin/sh

tools=$2
: ${tools:="../rasbpi"}

export SYSROOT="$tools/arm-bcm2708/arm-bcm2708hardfp-linux-gnueabi/arm-bcm2708hardfp-linux-gnueabi/sysroot"

#Include the cross copilation binaries
export PATH="$tools/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin":$PATH

#Set up our tools for anyting using these variables
#export CC="$tools/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin/gcc-sysroot"
export CC="$tools/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin/arm-linux-gnueabihf-gcc"
export AR="$tools/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/bin/arm-linux-gnueabihf-ar"

#Set target triple
flags="--target=arm-unknown-linux-gnueabihf"

echo $CC
echo $AR
echo $SYSROOT
echo $flags
echo $LD_LIBRARY_PATH

if [ "$1" != "-v" ]
then
	cargo $1 $flags
else
	rustc -vV
fi
