#!/bin/bash

[[ -n "${_COMPONENT_OBJECT_SH_LOADED:-}" ]] && return
_COMPONENT_OBJECT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_object() {
    local prompt="$1"
    local variable="$2"
    local fields_count="$3"
    shift 3

    prompt=$(interpolate "$prompt")

    cursor_hide
    trap 'cursor_show; exit 130' INT
    export INPUT_INLINE_KEEP_CURSOR=1

    print_step "$prompt"

    declare -A object_data

    for ((field_idx = 0; field_idx < fields_count; field_idx++)); do
        local field_name="$1"
        local field_input="$2"
        local field_display="${3:-}"
        local field_placeholder="${4:-}"
        local field_pattern="${5:-}"
        local field_on_error="${6:-}"
        local field_min_length="${7:-}"
        local field_min="${8:-}"
        local field_max="${9:-}"
        local field_step="${10:-}"
        local field_default="${11:-}"
        local field_format="${12:-}"
        local field_options="${13:-}"
        local field_active="${14:-}"
        local field_inactive="${15:-}"
        local field_validate_count="${16:-0}"
        shift 16

        local -a validate_args=()
        if [ "$field_validate_count" -gt 0 ]; then
            local v_idx
            for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                local v_pattern="$1"
                local v_error="$2"
                shift 2
                v_pattern=$(interpolate "$v_pattern")
                v_error=$(interpolate "$v_error")
                validate_args+=(--validate "$v_pattern" "$v_error")
            done
        fi

        field_display=$(interpolate "$field_display")
        field_placeholder=$(interpolate "$field_placeholder")

        local display_label="${field_display:-$field_name}"

        local had_error=false
        local needs_no_newline=false
        case "$field_input" in
            text|email|password|url|ip|bool_text)
                needs_no_newline=true
                ;;
        esac

        while true; do
            if [ "$had_error" = true ]; then
                tput cuu 1
                tput el
            fi
            collect_and_validate "$display_label" "$field_input" \
                --placeholder "$field_placeholder" \
                --options "$field_options" \
                --active "$field_active" \
                --inactive "$field_inactive" \
                $( [ "$needs_no_newline" = true ] && echo --no-newline ) \
                --min-length "$field_min_length" \
                --min "$field_min" \
                --max "$field_max" \
                --step "$field_step" \
                --default "$field_default" \
                --format "$field_format" \
                "${validate_args[@]}"

            local input_value="$INPUT_VALUE"
            if [ -z "$input_value" ]; then
                _show_inline_error "Value cannot be empty" "$needs_no_newline"
                had_error=true
                continue
            fi

            # Replace input UI with the final value line
            tput cuu 1
            tput el
            printf "  ${CYAN}%s:${NC} %s\n" "$display_label" "$input_value"

            object_data["$field_name"]="$input_value"
            break
        done
    done

    for key in "${!object_data[@]}"; do
        state_set "${variable}_${key}" "${object_data[$key]}"
    done

    unset INPUT_INLINE_KEEP_CURSOR
    trap - INT
    cursor_show
}
