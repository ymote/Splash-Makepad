#!/bin/bash
# Switch the kit-host catalog to a route on the OnePlus 6 and relaunch.
#   ./route.sh button
A=~/Library/Android/sdk/platform-tools/adb
D=cfb7c9e3          # OnePlus 6 only -- never bf0a4730 (the 6T)
$A -s $D shell "echo ${1:-button} > /data/local/tmp/kit_host.route"
$A -s $D shell appops set --uid dev.makepad.kit_host COARSE_LOCATION deny
$A -s $D shell appops set --uid dev.makepad.kit_host FINE_LOCATION deny
$A -s $D shell am start -S -n dev.makepad.kit_host/.MakepadApp >/dev/null
echo "kit-host -> ${1:-button}"
