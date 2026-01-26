#!/bin/bash

[[ -n "${_COMPONENT_SNIPPET_SH_LOADED:-}" ]] && return
_COMPONENT_SNIPPET_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/validator.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/ip.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/number.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/slider.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/select.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/date.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/color.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/list.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/toggle.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../inputs/checkbox.sh"

component_snippet() {
    local text="$1"
    local variable="$2"
    local fields_count="$3"
    shift 3

    local HL_BG='\033[48;5;236m'
    local HL_DIM='\033[38;5;245m'
    local HL_ACTIVE_BG='\033[48;5;238m'
    local HL_ACTIVE_FG='\033[38;5;220m'
    local HL_FILLED_BG='\033[48;5;236m'
    local HL_FILLED_FG='\033[37m'

    declare -a field_names=()
    declare -a field_types=()
    declare -a field_placeholders=()
    declare -a field_defaults=()
    declare -a field_mins=()
    declare -a field_maxs=()
    declare -a field_steps=()
    declare -a field_formats=()
    declare -a field_options=()
    declare -a field_actives=()
    declare -a field_inactives=()
    declare -A field_validate_counts=()
    declare -A field_validate_patterns=()
    declare -A field_validate_errors=()

    local idx
    for ((idx = 0; idx < fields_count; idx++)); do
        field_names+=("$1")
        field_types+=("$2")
        field_placeholders+=("${3:-}")
        field_defaults+=("${4:-}")
        field_mins+=("${5:-}")
        field_maxs+=("${6:-}")
        field_steps+=("${7:-}")
        field_formats+=("${8:-}")
        field_options+=("${9:-}")
        field_actives+=("${10:-}")
        field_inactives+=("${11:-}")
        local field_validate_count="${12:-0}"
        shift 12

        field_validate_counts["$idx"]="$field_validate_count"
        if [ "$field_validate_count" -gt 0 ]; then
            local v_idx
            for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                field_validate_patterns["$idx,$v_idx"]="$1"
                field_validate_errors["$idx,$v_idx"]="$2"
                shift 2
            done
        fi
    done

    text=$(interpolate "$text")

    print_step "Snippet"

    declare -g -A _SNIPPET_VALUES=()

    IFS=$'\n' read -r -d '' -a base_lines <<< "${text}"$'\0'
    local snippet_lines=${#base_lines[@]}

    find_placeholder_pos() {
        local name="$1"
        local placeholder="<${name}>"
        local line_idx=0
        local sep=$'\x1f'
        for line in "${base_lines[@]}"; do
            if [[ "$line" == *"$placeholder"* ]]; then
                local prefix="${line%%$placeholder*}"
                local suffix="${line#*$placeholder}"
                local col=${#prefix}
                printf "%s%s%s%s%s%s%s\n" "$line_idx" "$sep" "$col" "$sep" "$prefix" "$sep" "$suffix"
                return
            fi
            line_idx=$((line_idx + 1))
        done
        printf "%s%s%s%s%s%s%s\n" "-1" "$sep" "-1" "$sep" "" "$sep" ""
    }

    render_snippet() {
        local current_name="$1"
        local error_name="$2"
        local error_msg="$3"

        local out_lines=()
        local l
        for l in "${base_lines[@]}"; do
            local line="$l"
            local n
            for n in "${field_names[@]}"; do
                local placeholder="<${n}>"
                local val="${_SNIPPET_VALUES[$n]:-}"
                local replacement=""
                if [ -z "$val" ]; then
                    replacement="${HL_BG}${HL_DIM}${placeholder}${NC}"
                else
                    if [ "$n" = "$error_name" ] && [ -n "$error_msg" ]; then
                        replacement="${HL_BG}${RED}${BOLD}✗ ${error_msg}${NC}"
                    else
                        replacement="${HL_FILLED_BG}${HL_FILLED_FG}${val}${NC}"
                    fi
                fi
                line="${line//$placeholder/$replacement}"
            done
            out_lines+=("$line")
        done

        local i
        for ((i=0; i<snippet_lines; i++)); do
            echo -ne "\r"
            tput el
            echo -e "${out_lines[$i]}"
        done
        echo -ne "\r"
        tput el
        echo
    }

    render_snippet_lines() {
        local current_name="$1"
        local error_name="$2"
        local error_msg="$3"
        local skip_line="$4"

        local out_lines=()
        local l
        for l in "${base_lines[@]}"; do
            local line="$l"
            local n
            for n in "${field_names[@]}"; do
                local placeholder="<${n}>"
                local val="${_SNIPPET_VALUES[$n]:-}"
                local replacement=""
                if [ -z "$val" ]; then
                    replacement="${HL_BG}${HL_DIM}${placeholder}${NC}"
                else
                    if [ "$n" = "$error_name" ] && [ -n "$error_msg" ]; then
                        replacement="${HL_BG}${RED}${BOLD}✗ ${error_msg}${NC}"
                    else
                        replacement="${HL_FILLED_BG}${HL_FILLED_FG}${val}${NC}"
                    fi
                fi
                line="${line//$placeholder/$replacement}"
            done
            out_lines+=("$line")
        done

        local i
        for ((i=0; i<snippet_lines; i++)); do
            if [ "$i" -eq "$skip_line" ]; then
                echo -ne "\r"
                tput cud 1
                continue
            fi
            echo -ne "\r"
            tput el
            echo -e "${out_lines[$i]}"
        done
    }

    redraw_snippet() {
        tput cuu "$((snippet_lines + 1))"
        render_snippet "$1" "$2" "$3"
    }

    validate_field() {
        local field_idx="$1"
        local field_name="${field_names[$field_idx]}"
        local field_type="${field_types[$field_idx]}"
        local input_value="${_SNIPPET_VALUES[$field_name]}"
        local field_min="${field_mins[$field_idx]}"
        local field_max="${field_maxs[$field_idx]}"
        local field_options="${field_options[$field_idx]}"

        if [ -z "$input_value" ]; then
            echo "Value cannot be empty"
            return 1
        fi

        case "$field_type" in
            ip)
                if ! validate "ip" "$input_value" >/dev/null; then
                    echo "Nieprawidłowy format IP"
                    return 1
                fi
                ;;
            url)
                if ! validate "url" "$input_value" >/dev/null; then
                    echo "Nieprawidłowy format URL"
                    return 1
                fi
                ;;
            email)
                if ! validate "email" "$input_value" >/dev/null; then
                    echo "Nieprawidłowy format email"
                    return 1
                fi
                ;;
            color)
                if ! validate "color" "$input_value" >/dev/null; then
                    echo "Invalid color format"
                    return 1
                fi
                ;;
            number)
                if [[ ! "$input_value" =~ ^-?[0-9]+$ ]]; then
                    echo "Invalid number"
                    return 1
                fi
                if [ -n "$field_min" ] && [[ "$field_min" =~ ^-?[0-9]+$ ]] && [ "$input_value" -lt "$field_min" ]; then
                    echo "Value too small"
                    return 1
                fi
                if [ -n "$field_max" ] && [[ "$field_max" =~ ^-?[0-9]+$ ]] && [ "$input_value" -gt "$field_max" ]; then
                    echo "Value too large"
                    return 1
                fi
                ;;
            select)
                if [ -n "$field_options" ]; then
                    local ok=false
                    local opt
                    IFS=',' read -ra _opts <<< "$field_options"
                    for opt in "${_opts[@]}"; do
                        opt=$(echo "$opt" | xargs)
                        if [ "$input_value" = "$opt" ]; then
                            ok=true
                            break
                        fi
                    done
                    if [ "$ok" != true ]; then
                        echo "Invalid option"
                        return 1
                    fi
                fi
                ;;
        esac

        local v_count="${field_validate_counts[$field_idx]:-0}"
        if [ "$v_count" -gt 0 ]; then
            local v_idx
            for ((v_idx=0; v_idx<v_count; v_idx++)); do
                local v_pattern="${field_validate_patterns[$field_idx,$v_idx]}"
                local v_error="${field_validate_errors[$field_idx,$v_idx]}"
                v_pattern=$(interpolate "$v_pattern")
                v_error=$(interpolate "$v_error")
                if [ -n "$v_pattern" ] && [[ ! "$input_value" =~ $v_pattern ]]; then
                    echo "${v_error:-Validation failed}"
                    return 1
                fi
            done
        fi
        return 0
    }

    render_snippet "" "" ""

    cursor_hide
    trap 'cursor_show; exit 130' INT

    local current_idx=0
    local error_name=""
    local error_msg=""

    SNIPPET_ACTIVE_NAME=""
    SNIPPET_ACTIVE_LINE_IDX=0

    _snippet_on_change() {
        local val="$1"
        if [ -n "$SNIPPET_ACTIVE_NAME" ]; then
            _SNIPPET_VALUES["$SNIPPET_ACTIVE_NAME"]="$val"
            local up_to_top=$((SNIPPET_ACTIVE_LINE_IDX))
            if [ "$up_to_top" -gt 0 ]; then
                tput cuu "$up_to_top"
            fi
            echo -ne "\r"
            render_snippet_lines "$SNIPPET_ACTIVE_NAME" "" "" "$SNIPPET_ACTIVE_LINE_IDX"
            local back_up=$((snippet_lines - SNIPPET_ACTIVE_LINE_IDX))
            tput cuu "$back_up"
            echo -ne "\r"
        fi
    }

    while true; do
        local name="${field_names[$current_idx]}"
        local def="${field_defaults[$current_idx]}"
        def=$(interpolate "$def")
        _SNIPPET_VALUES["$name"]="${_SNIPPET_VALUES[$name]:-$def}"

        redraw_snippet "$name" "$error_name" "$error_msg"

        local pos
        pos="$(find_placeholder_pos "$name")"
        local line_idx=""
        local col_idx=""
        local prefix_text=""
        local suffix_text=""
        IFS=$'\x1f' read -r line_idx col_idx prefix_text suffix_text <<< "$pos"
        if [[ ! "$line_idx" =~ ^-?[0-9]+$ ]]; then
            line_idx=0
        fi

        local prefix="${prefix_text}"

        local field_type="${field_types[$current_idx]}"
        local placeholder="${field_placeholders[$current_idx]}"
        local min="${field_mins[$current_idx]}"
        local max="${field_maxs[$current_idx]}"
        local step="${field_steps[$current_idx]}"
        local format="${field_formats[$current_idx]}"
        local options="${field_options[$current_idx]}"
        local active="${field_actives[$current_idx]}"
        local inactive="${field_inactives[$current_idx]}"

        placeholder=$(interpolate "$placeholder")
        min=$(interpolate "$min")
        max=$(interpolate "$max")
        step=$(interpolate "$step")
        format=$(interpolate "$format")
        options=$(interpolate "$options")
        active=$(interpolate "$active")
        inactive=$(interpolate "$inactive")

        local -a validate_args=()
        local v_count="${field_validate_counts[$current_idx]:-0}"
        if [ "$v_count" -gt 0 ]; then
            local v_idx
            for ((v_idx=0; v_idx<v_count; v_idx++)); do
                local v_pattern="${field_validate_patterns[$current_idx,$v_idx]}"
                local v_error="${field_validate_errors[$current_idx,$v_idx]}"
                v_pattern=$(interpolate "$v_pattern")
                v_error=$(interpolate "$v_error")
                validate_args+=("$v_pattern" "$v_error")
            done
        fi

        while true; do
            SNIPPET_ACTIVE_NAME="$name"
            SNIPPET_ACTIVE_LINE_IDX="$line_idx"
            if [ "$line_idx" -ge 0 ]; then
                local lines_up=$((snippet_lines - line_idx + 1))
                tput cuu "$lines_up"
                echo -ne "\r"
                tput el
            fi

            export INPUT_INLINE_PREFIX="$prefix"
            export INPUT_INLINE_SUFFIX="$suffix_text"
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            export INPUT_INLINE_ON_CHANGE="_snippet_on_change"
            case "$field_type" in
                ip)
                    input_ip_inline "" "$placeholder" "${_SNIPPET_VALUES[$name]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                url|email)
                    input_text_inline "" "$placeholder" "${_SNIPPET_VALUES[$name]}" --type "$field_type" "${validate_args[@]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                text)
                    input_text_inline "" "$placeholder" "${_SNIPPET_VALUES[$name]}" "${validate_args[@]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                number)
                    input_number_inline "" "$min" "$max" "$step" "${_SNIPPET_VALUES[$name]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                slider)
                    input_slider_inline "" "$min" "$max" "$step" "${_SNIPPET_VALUES[$name]}" "" "${validate_args[@]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                select)
                    IFS=',' read -ra _opts <<< "$options"
                    input_select_inline "" "${_opts[@]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                date)
                    input_date_inline "" "${format:-YYYY-MM-DD}" "${_SNIPPET_VALUES[$name]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                color)
                    input_color_inline "" "${_SNIPPET_VALUES[$name]:-#000000}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                list)
                    input_list_inline "" "${format:-,}" "${_SNIPPET_VALUES[$name]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                toggle)
                    input_toggle_inline "" "${active:-active}" "${inactive:-inactive}" "${_SNIPPET_VALUES[$name]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                checkbox)
                    input_bool_inline "" "${_SNIPPET_VALUES[$name]:-false}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
                *)
                    input_text_inline "" "$placeholder" "${_SNIPPET_VALUES[$name]}" "${validate_args[@]}"
                    _SNIPPET_VALUES["$name"]="$INPUT_VALUE"
                    ;;
            esac
            unset INPUT_INLINE_PREFIX
            unset INPUT_INLINE_SUFFIX
            unset INPUT_INLINE_ACTIVE
            unset INPUT_INLINE_KEEP_CURSOR
            unset INPUT_INLINE_ON_CHANGE

            if [ "$line_idx" -ge 0 ]; then
                local lines_down=$((snippet_lines - line_idx))
                if [ "$lines_down" -gt 0 ]; then
                    tput cud "$lines_down"
                fi
            fi

            local err
            err="$(validate_field "$current_idx")"
            if [ -n "$err" ]; then
                error_name="$name"
                error_msg="$err"
                redraw_snippet "$name" "$error_name" "$error_msg"
                sleep 1
                error_name=""
                error_msg=""
                redraw_snippet "$name" "$error_name" "$error_msg"
                continue
            fi
            error_name=""
            error_msg=""
            redraw_snippet "$name" "$error_name" "$error_msg"
            break
        done

        if [ $current_idx -lt $((fields_count - 1)) ]; then
            current_idx=$((current_idx + 1))
        else
            break
        fi
    done

    trap - INT
    cursor_show

    local final="$text"
    for name in "${field_names[@]}"; do
        local placeholder="<${name}>"
        local value="${_SNIPPET_VALUES[$name]}"
        final="${final//$placeholder/$value}"
        state_set "${variable}_${name}" "$value"
    done

    state_set "$variable" "$final"
}