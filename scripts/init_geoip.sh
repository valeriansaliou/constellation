#!/bin/bash

##
#  Constellation
#
#  Pluggable authoritative DNS server
#  Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
#  License: Mozilla Public License v2.0 (MPL v2.0)
##

# Read arguments
while [ "$1" != "" ]; do
    argument_key=`echo $1 | awk -F= '{print $1}'`
    argument_value=`echo $1 | awk -F= '{print $2}'`

    case $argument_key in
        -l | --license_key)
            GEOIP_LICENSE_KEY="$argument_value"
            ;;
        *)
            echo "Unknown argument received: '$argument_key'"
            exit 1
            ;;
    esac

    shift
done

# Ensure a license key is provided
if [ -z "$GEOIP_LICENSE_KEY" ]; then
  echo "No GeoIP license key was provided, please provide it using '--license_key'"

  exit 1
fi

# Run initialization script
ABSPATH=$(cd "$(dirname "$0")"; pwd)
BASE_DIR="$ABSPATH/../"

GEOIP_DB_URL="https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-Country&suffix=tar.gz&license_key=$GEOIP_LICENSE_KEY"
GEOIP_DB_NAME="GeoLite2-Country.mmdb"

GEOIP_DB_PATH="./"
GEOIP_DB_PATH_NAME_FILE="$GEOIP_DB_PATH$GEOIP_DB_NAME"
GEOIP_DB_PATH_TEMP_DIRECTORY="$GEOIP_DB_PATH/tmp"

rc=0

pushd "$BASE_DIR" > /dev/null
  pushd ./res/geo > /dev/null
    # Clear any residual temporary path (before we start)
    rm -rf "$GEOIP_DB_PATH_TEMP_DIRECTORY"
    mkdir "$GEOIP_DB_PATH_TEMP_DIRECTORY"

    # Download up-to-date database (to temporary directory)
    wget -O - "$GEOIP_DB_URL" | tar xvz --strip-components=1 --directory="$GEOIP_DB_PATH_TEMP_DIRECTORY"
    cur_rc=$?

    # May we move the database to its final location? (download succeeded)
    if [ $cur_rc -eq 0 ]; then
      if [ -f "$GEOIP_DB_PATH_TEMP_DIRECTORY/$GEOIP_DB_NAME" ]; then
        rm -f "$GEOIP_DB_PATH_NAME_FILE"

        mv "$GEOIP_DB_PATH_TEMP_DIRECTORY/$GEOIP_DB_NAME" "$GEOIP_DB_PATH_NAME_FILE"
        cur_rc=$?
      else
        echo "Warning: downloaded GeoIP database does not contain the target file '$GEOIP_DB_NAME'. Could not update local database."
      fi
    fi

    # Remove temporary path (once we are done)
    rm -rf "$GEOIP_DB_PATH_TEMP_DIRECTORY"

    # Override final return code?
    if [ $rc -eq 0 ]; then rc=$cur_rc; fi

    # Database file does not exist?
    if [ ! -f "$GEOIP_DB_PATH_NAME_FILE" ]; then
      echo "Error: GeoIP database file '$GEOIP_DB_PATH_NAME_FILE' does not exist locally."

      rc=1
    fi
  popd > /dev/null
popd > /dev/null

exit $rc
