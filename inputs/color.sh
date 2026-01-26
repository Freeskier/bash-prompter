#!/bin/bash

[[ -n "${_INPUT_COLOR_SH_LOADED:-}" ]] && return
_INPUT_COLOR_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

_color_parse_hex() {
    local hex="$1"
    hex="${hex#\#}"
    if [[ "$hex" =~ ^[0-9A-Fa-f]{3}$ ]]; then
        echo "${hex:0:1}${hex:0:1}${hex:1:1}${hex:1:1}${hex:2:1}${hex:2:1}"
        return 0
    fi
    if [[ "$hex" =~ ^[0-9A-Fa-f]{6}$ ]]; then
        echo "$hex"
        return 0
    fi
    echo "000000"
}

_color_hex_to_rgb() {
    local hex="$1"
    local r=$((16#${hex:0:2}))
    local g=$((16#${hex:2:2}))
    local b=$((16#${hex:4:2}))
    echo "$r $g $b"
}

_color_rgb_to_hex() {
    printf "%02X%02X%02X" "$1" "$2" "$3"
}

_color_preview_block() {
    local r="$1"
    local g="$2"
    local b="$3"
    printf "\033[48;2;%s;%s;%sm  \033[0m" "$r" "$g" "$b"
}

input_color() {
    local prompt="$1"
    local variable="$2"
    local default="${3:-#000000}"

    prompt=$(interpolate "$prompt")
    default=$(interpolate "$default")

    local hex=$(_color_parse_hex "$default")
    read -r r g b <<< "$(_color_hex_to_rgb "$hex")"
    local channel=0

    print_step "$prompt"
    echo -e "${DIM}  (Use ←→ to select channel, ↑↓ to change, Enter to confirm)${NC}"

    tput civis
    trap 'tput cnorm; exit 130' INT

    draw_line() {
        local hex_out="$(_color_rgb_to_hex "$r" "$g" "$b")"
        local block="$(_color_preview_block "$r" "$g" "$b")"
        local cr="${r}" cg="${g}" cb="${b}"
        if [ $channel -eq 0 ]; then cr="${BOLD}${r}${NC}"; fi
        if [ $channel -eq 1 ]; then cg="${BOLD}${g}${NC}"; fi
        if [ $channel -eq 2 ]; then cb="${BOLD}${b}${NC}"; fi
        echo -ne "\r  ${DIM}#${NC}${BOLD}${hex_out}${NC}  $block  ${DIM}R:${NC}${cr} ${DIM}G:${NC}${cg} ${DIM}B:${NC}${cb}  "
    }

    draw_line

    while true; do
        IFS= read -rsn1 key
        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'C')
                        channel=$(( (channel + 1) % 3 ))
                        ;;
                    'D')
                        channel=$(( (channel - 1 + 3) % 3 ))
                        ;;
                    'A')
                        if [ $channel -eq 0 ] && [ $r -lt 255 ]; then r=$((r+1)); fi
                        if [ $channel -eq 1 ] && [ $g -lt 255 ]; then g=$((g+1)); fi
                        if [ $channel -eq 2 ] && [ $b -lt 255 ]; then b=$((b+1)); fi
                        ;;
                    'B')
                        if [ $channel -eq 0 ] && [ $r -gt 0 ]; then r=$((r-1)); fi
                        if [ $channel -eq 1 ] && [ $g -gt 0 ]; then g=$((g-1)); fi
                        if [ $channel -eq 2 ] && [ $b -gt 0 ]; then b=$((b-1)); fi
                        ;;
                esac
                draw_line
            fi
        elif [[ $key == "" ]]; then
            break
        fi
    done

    trap - INT
    tput cnorm
    echo

    local final_hex="#$(_color_rgb_to_hex "$r" "$g" "$b")"
    state_set "$variable" "$final_hex"
}

input_color_inline() {
    local label="$1"
    local default="${2:-#000000}"

    default=$(interpolate "$default")
    local hex=$(_color_parse_hex "$default")
    read -r r g b <<< "$(_color_hex_to_rgb "$hex")"
    local channel=0

    if [ -z "${INPUT_INLINE_KEEP_CURSOR:-}" ]; then
        tput civis
        trap 'tput cnorm; return 130' INT
    fi

    draw_inline() {
        local hex_out="$(_color_rgb_to_hex "$r" "$g" "$b")"
        local block="$(_color_preview_block "$r" "$g" "$b")"
        local cr="${r}" cg="${g}" cb="${b}"
        if [ $channel -eq 0 ]; then cr="${BOLD}${r}${NC}"; fi
        if [ $channel -eq 1 ]; then cg="${BOLD}${g}${NC}"; fi
        if [ $channel -eq 2 ]; then cb="${BOLD}${b}${NC}"; fi
        inline_on_change "#${hex_out}"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        echo -ne "${DIM}#${NC}${BOLD}${hex_out}${NC} $block ${DIM}R:${NC}${cr} ${DIM}G:${NC}${cg} ${DIM}B:${NC}${cb}"
        inline_wrap_end
        inline_suffix
    }

    draw_inline

    while true; do
        IFS= read -rsn1 key
        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'C')
                        channel=$(( (channel + 1) % 3 ))
                        ;;
                    'D')
                        channel=$(( (channel - 1 + 3) % 3 ))
                        ;;
                    'A')
                        if [ $channel -eq 0 ] && [ $r -lt 255 ]; then r=$((r+1)); fi
                        if [ $channel -eq 1 ] && [ $g -lt 255 ]; then g=$((g+1)); fi
                        if [ $channel -eq 2 ] && [ $b -lt 255 ]; then b=$((b+1)); fi
                        ;;
                    'B')
                        if [ $channel -eq 0 ] && [ $r -gt 0 ]; then r=$((r-1)); fi
                        if [ $channel -eq 1 ] && [ $g -gt 0 ]; then g=$((g-1)); fi
                        if [ $channel -eq 2 ] && [ $b -gt 0 ]; then b=$((b-1)); fi
                        ;;
                esac
                draw_inline
            fi
        elif [[ $key == "" ]]; then
            break
        fi
    done

    if [ -z "${INPUT_INLINE_KEEP_CURSOR:-}" ]; then
        trap - INT
        tput cnorm
    fi
    echo

    INPUT_VALUE="#$(_color_rgb_to_hex "$r" "$g" "$b")"
}
