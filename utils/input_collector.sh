#!/bin/bash

[[ -n "${_INPUT_COLLECTOR_SH_LOADED:-}" ]] && return
_INPUT_COLLECTOR_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/input_reader.sh"
source "$(dirname "${BASH_SOURCE[0]}")/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/validator.sh"

collect_input() {
    local label="$1"
    local input_type="$2"
    shift 2

    local -A params
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        case "$1" in
            --placeholder)
                params[placeholder]="$2"
                shift 2
                ;;
            --options)
                params[options]="$2"
                shift 2
                ;;
            --no-newline)
                params[no_newline]="true"
                shift
                ;;
            --min-length)
                params[min_length]="$2"
                shift 2
                ;;
            --min)
                params[min]="$2"
                shift 2
                ;;
            --max)
                params[max]="$2"
                shift 2
                ;;
            --step)
                params[step]="$2"
                shift 2
                ;;
            --unit)
                params[unit]="$2"
                shift 2
                ;;
            --default)
                params[default]="$2"
                shift 2
                ;;
            --format)
                params[format]="$2"
                shift 2
                ;;
            --separator)
                params[separator]="$2"
                shift 2
                ;;
            --active)
                params[active]="$2"
                shift 2
                ;;
            --inactive)
                params[inactive]="$2"
                shift 2
                ;;
            --validate)
                validate_patterns+=("$2")
                validate_errors+=("$3")
                shift 3
                ;;
            *)
                shift
                ;;
        esac
    done

    local input_value=""

    local inline_active=false
    local -a validate_args=()
    if [ ${#validate_patterns[@]} -gt 0 ]; then
        local v_idx
        for ((v_idx=0; v_idx<${#validate_patterns[@]}; v_idx++)); do
            validate_args+=("${validate_patterns[$v_idx]}" "${validate_errors[$v_idx]}")
        done
    fi

    case "$input_type" in
        text)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
            if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
                input_text_inline "$label" "${params[placeholder]:-}" "${params[default]:-}" "${validate_args[@]}"
            else
                echo -ne "${CYAN}  ${label}:${NC} "
                read_with_placeholder "${params[placeholder]:-}" "" "${params[no_newline]:-false}"
                INPUT_VALUE="$INPUT_VALUE"
            fi
            input_value="$INPUT_VALUE"
            ;;
        url)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
            if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
                input_text_inline "$label" "${params[placeholder]:-https://example.com}" "${params[default]:-}" --type url --type-error "Nieprawidłowy format URL" "${validate_args[@]}"
            else
                echo -ne "${CYAN}  ${label}:${NC} "
                read_with_placeholder "${params[placeholder]:-https://example.com}" "" "${params[no_newline]:-false}"
            fi
            input_value="$INPUT_VALUE"
            ;;
        email)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
            if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
                input_text_inline "$label" "${params[placeholder]:-user@example.com}" "${params[default]:-}" --type email --type-error "Nieprawidłowy format email" "${validate_args[@]}"
            else
                echo -ne "${CYAN}  ${label}:${NC} "
                read_with_placeholder "${params[placeholder]:-user@example.com}" "" "${params[no_newline]:-false}"
            fi
            input_value="$INPUT_VALUE"
            ;;
        ip)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/ip.sh"
            input_ip_inline "$label" "${params[placeholder]:-192.168.1.1}" "${params[default]:-}"
            input_value="$INPUT_VALUE"
            ;;
        select)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/select.sh"
            if [ -z "${params[options]:-}" ]; then
                INPUT_VALUE=""
                return
            fi
            IFS=',' read -ra _select_opts <<< "${params[options]}"
            local i
            for i in "${!_select_opts[@]}"; do
                _select_opts[$i]=$(echo "${_select_opts[$i]}" | xargs)
            done
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_select_inline "$label" "${_select_opts[@]}"
            input_value="$INPUT_VALUE"
            ;;
        slider)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/slider.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_slider_inline "$label" "${params[min]:-0}" "${params[max]:-100}" "${params[step]:-1}" "${params[default]:-}" "${params[unit]:-}"
            input_value="$INPUT_VALUE"
            ;;
        number)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/number.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_number_inline "$label" "${params[min]:-}" "${params[max]:-}" "${params[step]:-1}" "${params[default]:-}"
            input_value="$INPUT_VALUE"
            ;;
        url)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_text_inline "$label" "${params[placeholder]:-https://example.com}" "${params[default]:-}" --type url --type-error "Nieprawidłowy format URL"
            input_value="$INPUT_VALUE"
            ;;
        email)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_text_inline "$label" "${params[placeholder]:-user@example.com}" "${params[default]:-}" --type email --type-error "Nieprawidłowy format email"
            input_value="$INPUT_VALUE"
            ;;
        date)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/date.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_date_inline "$label" "${params[format]:-YYYY-MM-DD}" "${params[default]:-}"
            input_value="$INPUT_VALUE"
            ;;
        color)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/color.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_color_inline "$label" "${params[default]:-#000000}"
            input_value="$INPUT_VALUE"
            ;;
        list)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/list.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_list_inline "$label" "${params[separator]:-,}" "${params[default]:-}"
            input_value="$INPUT_VALUE"
            ;;
        toggle)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/toggle.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            export INPUT_INLINE_KEEP_CURSOR=1
            input_toggle_inline "$label" "${params[active]:-active}" "${params[inactive]:-inactive}" "${params[default]:-false}"
            input_value="$INPUT_VALUE"
            ;;
        bool)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/checkbox.sh"
            inline_active=true
            export INPUT_INLINE_ACTIVE=1
            input_bool_inline "$label" "${params[default]:-false}"
            input_value="$INPUT_VALUE"
            ;;
        bool_text)
            # Text-based bool input (fallback)
            echo -ne "${CYAN}  ${label}:${NC} ${DIM}(true/false)${NC} "
            read_with_placeholder "${params[placeholder]:-${params[default]:-false}}" "" "${params[no_newline]:-false}"
            local raw_value="$INPUT_VALUE"

            # Normalize boolean values
            case "$raw_value" in
                t|T|true|TRUE|y|Y|yes|YES|1)
                    input_value="true"
                    ;;
                f|F|false|FALSE|n|N|no|NO|0|"")
                    input_value="false"
                    ;;
                *)
                    input_value="$raw_value"  # Let validation handle invalid values
                    ;;
            esac
            ;;
        password)
            if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
                # No inline password UI yet, fallback to standard prompt line.
                echo -ne "${CYAN}  ${label}:${NC} "
                read_with_placeholder "${params[placeholder]:-}" "*" "${params[no_newline]:-false}"
                input_value="$INPUT_VALUE"
            else
                echo -ne "${CYAN}  ${label}:${NC} "
                read_with_placeholder "${params[placeholder]:-}" "*" "${params[no_newline]:-false}"
                input_value="$INPUT_VALUE"
            fi
            ;;
        *)
            echo -ne "${CYAN}  ${label}:${NC} "
            read_with_placeholder "${params[placeholder]:-}" "" "${params[no_newline]:-false}"
            input_value="$INPUT_VALUE"
            ;;
    esac

    if [ "$inline_active" = true ]; then
        unset INPUT_INLINE_ACTIVE
        unset INPUT_INLINE_KEEP_CURSOR
    fi
    INPUT_VALUE="$input_value"
}

_show_inline_error() {
    local msg="$1"
    local no_newline="${2:-false}"
    if [ "$no_newline" = "true" ]; then
        tput cr
        tput el
        printf "  ${RED}✗${NC} %s" "$msg"
        sleep 1
        tput cr
        tput el
        return
    fi
    tput cuu 1
    tput el
    echo -e "  ${RED}✗${NC} $msg"
    sleep 1
    tput cuu 1
    tput el
}

collect_and_validate() {
    local label="$1"
    local input_type="$2"
    shift 2

    local -a args=("$@")

    while true; do
        collect_input "$label" "$input_type" "${args[@]}"
        local input_value="$INPUT_VALUE"

        if validate_collected_input "$input_type" "$input_value" "${args[@]}"; then
            INPUT_VALUE="$input_value"
            return 0
        else
            local error_msg="$INPUT_VALUE"
            local no_newline="false"
            local idx=0
            while [ $idx -lt ${#args[@]} ]; do
                if [ "${args[$idx]}" = "--no-newline" ]; then
                    no_newline="true"
                    break
                fi
                idx=$((idx + 1))
            done
            if [ -n "${INPUT_INLINE_ACTIVE:-}" ]; then
                inline_error "$error_msg" "$label"
            else
                _show_inline_error "$error_msg" "$no_newline"
            fi
        fi
    done
}

validate_collected_input() {
    local input_type="$1"
    local value="$2"
    shift 2

    if [ "$input_type" = "slider" ] || [ "$input_type" = "date" ]; then
        return 0
    fi

    local -A params
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        case "$1" in
            --min-length)
                params[min_length]="$2"
                shift 2
                ;;
            --min)
                params[min]="$2"
                shift 2
                ;;
            --max)
                params[max]="$2"
                shift 2
                ;;
            --validate)
                validate_patterns+=("$2")
                validate_errors+=("$3")
                shift 3
                ;;
            *)
                shift
                ;;
        esac
    done

    local error_msg
    case "$input_type" in
        bool)
            # Validate boolean values
            if [[ "$value" != "true" && "$value" != "false" ]]; then
                error_msg="${params[on_error]:-Invalid boolean value. Use true/false, t/f, y/n, or 1/0}"
                local exit_code=1
            else
                error_msg=""
                local exit_code=0
            fi
            ;;
        select)
            error_msg=""
            local exit_code=0
            ;;
        color)
            error_msg=$(validate "color" "$value" "${params[on_error]:-Invalid color format}")
            local exit_code=$?
            ;;
        list)
            error_msg=""
            local exit_code=0
            ;;
        toggle)
            error_msg=""
            local exit_code=0
            ;;
        password)
            error_msg=$(validate "password" "$value" "${params[min_length]:-8}" "${params[on_error]:-}")
            local exit_code=$?
            ;;
        slider)
            error_msg=$(validate "slider" "$value" "${params[min]:-0}" "${params[max]:-100}" "${params[on_error]:-}")
            local exit_code=$?
            ;;
        *)
            error_msg=$(validate "$input_type" "$value")
            local exit_code=$?
            ;;
    esac

    if [ $exit_code -eq 0 ] && [ ${#validate_patterns[@]} -gt 0 ]; then
        local v_idx
        for ((v_idx=0; v_idx<${#validate_patterns[@]}; v_idx++)); do
            local v_pattern="${validate_patterns[$v_idx]}"
            local v_error="${validate_errors[$v_idx]}"
            if [ -n "$v_pattern" ] && [[ ! "$value" =~ $v_pattern ]]; then
                error_msg="${v_error:-Validation failed}"
                exit_code=1
                break
            fi
        done
    fi

    INPUT_VALUE="$error_msg"
    return $exit_code
}
