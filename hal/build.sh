#!/usr/bin/env bash
ACTION=${1:-build}
for mcutype in $(ls ../pac/ | grep '^atsam3') ; do
    echo -e "\n--==[ ${mcutype} ] ]==--" 
    cargo ${ACTION} --features ${mcutype}
done

