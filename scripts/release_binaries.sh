#!/bin/bash

##
#  Constellation
#
#  Pluggable authoritative DNS server
#  Copyright: 2020, Valerian Saliou <valerian@valeriansaliou.name>
#  License: Mozilla Public License v2.0 (MPL v2.0)
##

# Read arguments
while [ "$1" != "" ]; do
    argument_key=`echo $1 | awk -F= '{print $1}'`
    argument_value=`echo $1 | awk -F= '{print $2}'`

    case $argument_key in
        -v | --version)
            # Notice: strip any leading 'v' to the version number
            CONSTELLATION_VERSION="${argument_value/v}"
            ;;
        *)
            echo "Unknown argument received: '$argument_key'"
            exit 1
            ;;
    esac

    shift
done

# Ensure release version is provided
if [ -z "$CONSTELLATION_VERSION" ]; then
  echo "No Constellation release version was provided, please provide it using '--version'"

  exit 1
fi

# Define release pipeline
function release_for_architecture {
    final_tar="v$CONSTELLATION_VERSION-$1.tar.gz"

    rm -rf ./constellation/ && \
        cross build --target "$2" --release && \
        mkdir ./constellation && \
        cp -p "target/$2/release/constellation" ./constellation/ && \
        cp ./config.cfg constellation/ && \
        tar --owner=0 --group=0 -czvf "$final_tar" ./constellation && \
        rm -r ./constellation/
    release_result=$?

    if [ $release_result -eq 0 ]; then
        echo "Result: Packed architecture: $1 to file: $final_tar"
    fi

    return $release_result
}

# Run release tasks
ABSPATH=$(cd "$(dirname "$0")"; pwd)
BASE_DIR="$ABSPATH/../"

rc=0

pushd "$BASE_DIR" > /dev/null
    echo "Executing release steps for Constellation v$CONSTELLATION_VERSION..."

    release_for_architecture "x86_64" "x86_64-unknown-linux-musl" && \
        release_for_architecture "armv7" "armv7-unknown-linux-musleabihf"
    rc=$?

    if [ $rc -eq 0 ]; then
        echo "Success: Done executing release steps for Constellation v$CONSTELLATION_VERSION"
    else
        echo "Error: Failed executing release steps for Constellation v$CONSTELLATION_VERSION"
    fi
popd > /dev/null

exit $rc
