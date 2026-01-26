#!/bin/bash

[[ -n "${_VALIDATOR_SH_LOADED:-}" ]] && return
_VALIDATOR_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"

validate() {
    local input_type="$1"
    local value="$2"
    shift 2

    case "$input_type" in
        text)
            _validate_text "$value" "$@"
            return $?
            ;;
        password)
            _validate_password "$value" "$@"
            return $?
            ;;
        email)
            _validate_email "$value" "$@"
            return $?
            ;;
        url)
            _validate_url "$value" "$@"
            return $?
            ;;
        ip)
            _validate_ip "$value" "$@"
            return $?
            ;;
        slider)
            _validate_slider "$value" "$@"
            return $?
            ;;
        color)
            _validate_color "$value" "$@"
            return $?
            ;;
        *)
            echo "Unknown input type: $input_type"
            return 1
            ;;
    esac
}

_validate_text() {
    local value="${1:-}"
    local pattern="${2:-}"
    local error_msg="${3:-Validation failed}"

    if [ -z "$pattern" ]; then
        return 0
    fi

    if [[ ! "$value" =~ $pattern ]]; then
        echo "$error_msg"
        return 1
    fi

    return 0
}

_validate_password() {
    local value="$1"
    local min_length="${2:-8}"
    local error_msg="${3:-Password is too short}"

    if [ ${#value} -lt $min_length ]; then
        echo "$error_msg"
        return 1
    fi

    return 0
}

_validate_url() {
    local value="$1"
    local pattern="^https?://[a-zA-Z0-9.-]+.*"
    local error_msg="${2:-Nieprawidłowy format URL}"

    if [[ ! "$value" =~ $pattern ]]; then
        echo "$error_msg"
        return 1
    fi

    return 0
}

_validate_ip() {
    local value="$1"
    local pattern="^([0-9]{1,3}\.){3}[0-9]{1,3}$"
    local error_msg="${2:-Nieprawidłowy format IP}"

    if [[ ! "$value" =~ $pattern ]]; then
        echo "$error_msg"
        return 1
    fi

    return 0
}

_validate_slider() {
    local value="$1"
    local min="${2:-0}"
    local max="${3:-100}"
    local error_msg="${4:-Value out of range}"

    # Skip validation if min/max are empty or not numbers
    if [ -z "$min" ] || [ -z "$max" ]; then
        return 0
    fi

    # Use arithmetic comparison
    if (( value < min || value > max )); then
        echo "$error_msg"
        return 1
    fi

    return 0
}

_validate_color() {
    local value="$1"
    local error_msg="${2:-Invalid color format}"

    if [[ ! "$value" =~ ^#?[0-9A-Fa-f]{6}$ ]]; then
        echo "$error_msg"
        return 1
    fi

    return 0
}

_validate_email() {
    local value="$1"
    local pattern="^.+@.+\..+$"
    local error_msg="${2:-Nieprawidłowy format email}"

    if [[ ! "$value" =~ $pattern ]]; then
        echo "$error_msg"
        return 1
    fi

    return 0
}
