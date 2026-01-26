#!/bin/bash

[[ -n "${_COMPONENT_SELECT_SH_LOADED:-}" ]] && return
_COMPONENT_SELECT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../inputs/select.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"

component_select() {
    cursor_hide
    trap 'cursor_show; exit 130' INT
    export INPUT_INLINE_KEEP_CURSOR=1
    input_select "$@"
    unset INPUT_INLINE_KEEP_CURSOR
    trap - INT
    cursor_show
}
