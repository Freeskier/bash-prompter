#!/bin/bash

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

output_info() {
    local value="$1"

    value=$(interpolate "$value")

    print_info "$value"
}
