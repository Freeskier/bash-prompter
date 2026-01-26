#!/bin/bash

[[ -n "${_INPUT_URL_SH_LOADED:-}" ]] && return
_INPUT_URL_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/text.sh"

input_url() {
    local prompt="$1"
    local variable="$2"
    local placeholder="${3:-https://example.com}"
    local default="$4"
    local on_error="Nieprawidłowy format URL (musi zaczynać się od http:// lub https://)"

    input_text "$prompt" "$variable" "$placeholder" "$default" --type url --type-error "$on_error"
}
