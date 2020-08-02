#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACTION=${1:-build}
for mcutype in $(ls ${DIR}/../pac/ | grep '^atsam3') ; do
    echo -e "\n--==[ ${mcutype} ]==--" 
    cargo ${ACTION} --features ${mcutype:2}
done

