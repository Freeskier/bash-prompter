#!/bin/bash

[[ -n "${_INPUT_LIST_SH_LOADED:-}" ]] && return
_INPUT_LIST_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

_list_render_value() {
    local value="$1"
    local separator="$2"
    local out=""
    local i=0
    local sep_len=${#separator}
    local val_len=${#value}

    while [ $i -lt $val_len ]; do
        if [ $sep_len -gt 0 ] && [ "${value:$i:$sep_len}" = "$separator" ]; then
            out+="${YELLOW}${separator}${NC}"
            i=$((i + sep_len))
        else
            out+="${value:$i:1}"
            i=$((i + 1))
        fi
    done

    echo -ne "$out"
}

_list_split_values() {
    local value="$1"
    local separator="$2"
    local tmp="$value"

    if [ -n "$separator" ]; then
        tmp="${tmp//$separator/$'\n'}"
    fi

    local -a items=()
    while IFS= read -r line; do
        local trimmed
        trimmed=$(echo "$line" | xargs)
        if [ -n "$trimmed" ]; then
            items+=("$trimmed")
        fi
    done <<< "$tmp"

    echo "${items[@]}"
}

_list_join_values() {
    local separator="$1"
    shift
    local -a items=("$@")
    local joined=""
    local idx

    for ((idx=0; idx<${#items[@]}; idx++)); do
        if [ $idx -gt 0 ]; then
            joined+="${separator}"
        fi
        joined+="${items[$idx]}"
    done

    echo "$joined"
}

_input_list_common() {
    local label="$1"
    local separator="$2"
    local default="$3"
    local inline="${4:-false}"
    local value="$default"
    local has_placeholder=true
    if [ -n "$value" ]; then
        has_placeholder=false
    fi

    draw_line() {
        inline_on_change "$value"
        inline_clear
        if [ "$inline" = "true" ]; then
            inline_prefix "$label"
        else
            echo -ne "  "
        fi
        if [ "$inline" = "true" ]; then
            inline_wrap_start
        fi
        if [ $has_placeholder = true ]; then
            echo -ne "${DIM}${default}${NC}"
        else
            _list_render_value "$value" "$separator"
        fi
        if [ "$inline" = "true" ]; then
            inline_wrap_end
            inline_suffix
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
                    'C'|'D')
                        # Ignore arrows for now
                        ;;
                esac
            fi
        elif [[ $key == $'\177' ]] || [[ $key == $'\b' ]]; then
            if [ ${#value} -gt 0 ]; then
                value="${value%?}"
                has_placeholder=false
                if [ -z "$value" ]; then
                    has_placeholder=true
                fi
                draw_line
            fi
        elif [[ $key == $'\x17' ]]; then
            if [ ${#value} -gt 0 ]; then
                while [[ "$value" == *" " ]]; do
                    value="${value% }"
                done
                if [[ "$separator" != "" ]]; then
                    value="${value%${separator}*}"
                else
                    while [ -n "$value" ] && [[ "${value: -1}" != " " ]]; do
                        value="${value%?}"
                    done
                fi
                while [[ "$value" == *" " ]]; do
                    value="${value% }"
                done
                if [ -z "$value" ]; then
                    has_placeholder=true
                fi
                draw_line
            fi
        elif [[ $key == "" ]]; then
            break
        else
            if [ $has_placeholder = true ]; then
                value=""
                has_placeholder=false
            fi
            value+="$key"
            draw_line
        fi
    done

    echo
    INPUT_VALUE="$value"
}

input_list() {
    local prompt="$1"
    local variable="$2"
    local separator="${3:-,}"
    local default="${4:-}"

    prompt=$(interpolate "$prompt")
    separator=$(interpolate "$separator")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Use '${separator}' as separator, Enter to confirm)${NC}"

    _input_list_common "$prompt" "$separator" "$default" "false"

    local raw="$INPUT_VALUE"
    local -a items
    read -r -a items <<< "$(_list_split_values "$raw" "$separator")"
    local joined
    joined="$(_list_join_values "$separator" "${items[@]}")"

    state_set "$variable" "$joined"
    state_set "${variable}_count" "${#items[@]}"
    local idx
    for ((idx=0; idx<${#items[@]}; idx++)); do
        state_set "${variable}_${idx}" "${items[$idx]}"
    done
}

input_list_inline() {
    local label="$1"
    local separator="${2:-,}"
    local default="${3:-}"

    separator=$(interpolate "$separator")
    default=$(interpolate "$default")

    _input_list_common "$label" "$separator" "$default" "true"
}
