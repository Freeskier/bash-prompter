#!/bin/bash

[[ -n "${_INPUT_EMAIL_SH_LOADED:-}" ]] && return
_INPUT_EMAIL_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/text.sh"

input_email() {
    local prompt="$1"
    local variable="$2"
    local placeholder="${3:-user@example.com}"
    local default="$4"
    local on_error="Nieprawidłowy format email"

    input_text "$prompt" "$variable" "$placeholder" "$default" --type email --type-error "$on_error"
}
