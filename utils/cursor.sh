#!/bin/bash

[[ -n "${_CURSOR_SH_LOADED:-}" ]] && return
_CURSOR_SH_LOADED=1

cursor_hide() {
    if [ -z "${_CURSOR_HIDE_COUNT:-}" ]; then
        _CURSOR_HIDE_COUNT=0
    fi
    _CURSOR_HIDE_COUNT=$((_CURSOR_HIDE_COUNT + 1))
    if [ "$_CURSOR_HIDE_COUNT" -eq 1 ]; then
        tput civis
    fi
}

cursor_show() {
    if [ -z "${_CURSOR_HIDE_COUNT:-}" ]; then
        _CURSOR_HIDE_COUNT=0
    fi
    if [ "$_CURSOR_HIDE_COUNT" -gt 0 ]; then
        _CURSOR_HIDE_COUNT=$((_CURSOR_HIDE_COUNT - 1))
    fi
    if [ "$_CURSOR_HIDE_COUNT" -eq 0 ]; then
        tput cnorm
    fi
}
