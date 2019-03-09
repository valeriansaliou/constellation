#!/bin/bash

##
#  Constellation
#
#  Pluggable authoritative DNS server
#  Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
#  License: Mozilla Public License v2.0 (MPL v2.0)
##

ABSPATH=$(cd "$(dirname "$0")"; pwd)
BASE_DIR="$ABSPATH/../"

GEOIP_DB_URL="https://geolite.maxmind.com/download/geoip/database/GeoLite2-City.mmdb.gz"
GEOIP_DB_PATH="./GeoLite2-Country.mmdb"

rc=0

pushd "$BASE_DIR" > /dev/null
  pushd ./res/geo > /dev/null
    # Fail by default, if not marked as successful.
    cur_rc=1

    rm -f "$GEOIP_DB_PATH"
    wget -O - "$GEOIP_DB_URL" | gunzip -c > "$GEOIP_DB_PATH"
    cur_rc=$?

    if [ $rc -eq 0 ]; then rc=$cur_rc; fi
  popd > /dev/null
popd > /dev/null

exit $rc
