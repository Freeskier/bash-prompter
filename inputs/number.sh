#!/bin/bash

[[ -n "${_INPUT_NUMBER_SH_LOADED:-}" ]] && return
_INPUT_NUMBER_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

_number_clamp() {
    local value="$1"
    local min="$2"
    local max="$3"

    if [ -n "$min" ] && [[ "$min" =~ ^-?[0-9]+$ ]] && [ "$value" -lt "$min" ]; then
        echo "$min"
        return
    fi
    if [ -n "$max" ] && [[ "$max" =~ ^-?[0-9]+$ ]] && [ "$value" -gt "$max" ]; then
        echo "$max"
        return
    fi
    echo "$value"
}

_number_is_valid() {
    local value="$1"
    local min="$2"
    local max="$3"

    if [[ ! "$value" =~ ^-?[0-9]+$ ]]; then
        return 1
    fi
    if [ -n "$min" ] && [[ "$min" =~ ^-?[0-9]+$ ]] && [ "$value" -lt "$min" ]; then
        return 1
    fi
    if [ -n "$max" ] && [[ "$max" =~ ^-?[0-9]+$ ]] && [ "$value" -gt "$max" ]; then
        return 1
    fi
    return 0
}

input_number() {
    local prompt="$1"
    local variable="$2"
    local min="${3:-}"
    local max="${4:-}"
    local step="${5:-1}"
    local default="${6:-}"

    prompt=$(interpolate "$prompt")
    min=$(interpolate "$min")
    max=$(interpolate "$max")
    step=$(interpolate "$step")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Digits only, ↑↓ to change, Enter to confirm)${NC}"

    local value="$default"
    if [ -z "$value" ]; then
        if [ -n "$min" ] && [[ "$min" =~ ^-?[0-9]+$ ]]; then
            value="$min"
        else
            value="0"
        fi
    fi

    tput civis
    trap 'tput cnorm; exit 130' INT

    draw_line() {
        echo -ne "\r"
        tput el
        if _number_is_valid "$value" "$min" "$max"; then
            echo -ne "  ${YELLOW}>${NC} ${GREEN}${value}${NC}"
        else
            echo -ne "  ${YELLOW}>${NC} ${RED}${BOLD}${value}${NC}"
        fi
    }

    draw_line

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'A')
                        if [[ "$value" =~ ^-?[0-9]+$ ]]; then
                            value=$((value + step))
                            value=$(_number_clamp "$value" "$min" "$max")
                        fi
                        draw_line
                        ;;
                    'B')
                        if [[ "$value" =~ ^-?[0-9]+$ ]]; then
                            value=$((value - step))
                            value=$(_number_clamp "$value" "$min" "$max")
                        fi
                        draw_line
                        ;;
                esac
            fi
        elif [[ $key == $'\177' ]] || [[ $key == $'\b' ]]; then
            value="${value%?}"
            if [ -z "$value" ]; then
                value="0"
            fi
            draw_line
        elif [[ $key =~ [0-9] ]]; then
            if [ "$value" = "0" ]; then
                value="$key"
            else
                value+="$key"
            fi
            draw_line
        elif [[ $key == "-" ]]; then
            if [[ "$value" =~ ^-?[0-9]+$ ]]; then
                if [[ "$value" == -* ]]; then
                    value="${value#-}"
                else
                    value="-$value"
                fi
                draw_line
            fi
        elif [[ $key == "" ]]; then
            if _number_is_valid "$value" "$min" "$max"; then
                break
            fi
            echo -e "\n  ${RED}✗${NC} Invalid number"
            draw_line
        fi
    done

    trap - INT
    tput cnorm
    echo

    state_set "$variable" "$value"
}

input_number_inline() {
    local label="$1"
    local min="${2:-}"
    local max="${3:-}"
    local step="${4:-1}"
    local default="${5:-}"

    min=$(interpolate "$min")
    max=$(interpolate "$max")
    step=$(interpolate "$step")
    default=$(interpolate "$default")

    local value="$default"
    if [ -z "$value" ]; then
        if [ -n "$min" ] && [[ "$min" =~ ^-?[0-9]+$ ]]; then
            value="$min"
        else
            value="0"
        fi
    fi

    draw_inline() {
        inline_on_change "$value"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        if _number_is_valid "$value" "$min" "$max"; then
            echo -ne "${GREEN}${value}${NC}"
        else
            echo -ne "${RED}${BOLD}${value}${NC}"
        fi
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
                    'A')
                        if [[ "$value" =~ ^-?[0-9]+$ ]]; then
                            value=$((value + step))
                            value=$(_number_clamp "$value" "$min" "$max")
                        fi
                        draw_inline
                        ;;
                    'B')
                        if [[ "$value" =~ ^-?[0-9]+$ ]]; then
                            value=$((value - step))
                            value=$(_number_clamp "$value" "$min" "$max")
                        fi
                        draw_inline
                        ;;
                esac
            fi
        elif [[ $key =~ [0-9] ]]; then
            if [ "$value" = "0" ]; then
                value="$key"
            else
                value+="$key"
            fi
            draw_inline
        elif [[ $key == $'\177' ]] || [[ $key == $'\b' ]]; then
            value="${value%?}"
            if [ -z "$value" ]; then
                value="0"
            fi
            draw_inline
        elif [[ $key == $'\x17' ]]; then
            value="${value%[0-9]*}"
            value="${value//[[:space:]]/}"
            if [ -z "$value" ]; then
                value="0"
            fi
            draw_inline
        elif [[ $key == "-" ]]; then
            if [[ "$value" =~ ^-?[0-9]+$ ]]; then
                if [[ "$value" == -* ]]; then
                    value="${value#-}"
                else
                    value="-$value"
                fi
                draw_inline
            fi
        elif [[ $key == "" ]]; then
            if _number_is_valid "$value" "$min" "$max"; then
                break
            fi
            local err="Validation failed"
            if [[ ! "$value" =~ ^-?[0-9]+$ ]]; then
                err="Invalid number"
            else
                err="Value out of range"
            fi
            inline_error "$err" "$label"
            draw_inline
        fi
    done

    echo
    INPUT_VALUE="$value"
}
