#!/bin/bash

color="[255, 244, 200]"
minutes="[540, 600, 1260, 1320]"
current_time="$1"
address="$2"

script_dir=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_dir"

sed \
    -e "s|\$color|$color|g" \
    -e "s|\$minutes|$minutes|g" \
    -e "s|\$current_time|$current_time|g" \
    cycle_body.json \
    | curl -H "Content-Type: application/json" -X POST -d @- "$address"