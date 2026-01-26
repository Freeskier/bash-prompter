#!/bin/bash

set -uo pipefail

trap 'echo -e "\n\nPrzerwano przez użytkownika"; exit 130' INT

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "${SCRIPT_DIR}/core/runner.sh"

main() {
    local yaml_file="${1:-}"

    if [ -z "$yaml_file" ]; then
        print_error "Usage: $0 <config.yml>"
        exit 1
    fi

    if [ ! -f "$yaml_file" ]; then
        print_error "File not found: $yaml_file"
        exit 1
    fi

    run_steps "$yaml_file"
}

main "$@"
