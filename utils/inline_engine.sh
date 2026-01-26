#!/bin/bash

[[ -n "${_INLINE_ENGINE_SH_LOADED:-}" ]] && return
_INLINE_ENGINE_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/colors.sh"

inline_on_change() {
    if [ -n "${INPUT_INLINE_ON_CHANGE:-}" ]; then
        "${INPUT_INLINE_ON_CHANGE}" "$1"
    fi
}

inline_clear() {
    echo -ne "\r"
    tput el
}

inline_prefix() {
    local label="$1"
    if [ -n "$label" ]; then
        echo -ne "${CYAN}  ${label}:${NC} "
    elif [ -n "${INPUT_INLINE_PREFIX:-}" ]; then
        echo -ne "${INPUT_INLINE_PREFIX}"
    fi
}

inline_wrap_start() {
    if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
        echo -ne "${YELLOW}[${NC}"
    fi
}

inline_wrap_end() {
    if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
        echo -ne "${YELLOW}]${NC}"
    fi
}

inline_suffix() {
    if [ -n "${INPUT_INLINE_SUFFIX:-}" ]; then
        echo -ne "${INPUT_INLINE_SUFFIX}"
    fi
}

inline_error() {
    local msg="$1"
    local label="${2:-}"
    inline_clear
    inline_prefix "$label"
    inline_wrap_start
    echo -ne "${RED}${BOLD}✗ ${msg}${NC}"
    inline_wrap_end
    inline_suffix
    sleep 1
    inline_clear
}
