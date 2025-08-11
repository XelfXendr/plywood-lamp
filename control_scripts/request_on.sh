#!/bin/bash

color="[255, 244, 200]"
duration="10000"
address="$1"

script_dir=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_dir"

sed \
    -e "s|\$color|$color|g" \
    -e "s|\$duration|$duration|g" \
    set_body.json \ \
    | curl -H "Content-Type: application/json" -X POST -d @- "$address"
