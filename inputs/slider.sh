#!/bin/bash

[[ -n "${_INPUT_SLIDER_SH_LOADED:-}" ]] && return
_INPUT_SLIDER_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

input_slider() {
    local prompt="$1"
    local variable="$2"
    local min="${3:-0}"
    local max="${4:-100}"
    local step="${5:-1}"
    local default="${6:-}"
    local unit="${7:-}"
    shift 7
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        validate_patterns+=("$1")
        validate_errors+=("$2")
        shift 2
    done

    prompt=$(interpolate "$prompt")
    min=$(interpolate "$min")
    max=$(interpolate "$max")
    step=$(interpolate "$step")
    default=$(interpolate "$default")
    unit=$(interpolate "$unit")

    local current=$default
    if [ -z "$current" ]; then
        current=$(( (min + max) / 2 ))
    fi

    print_step "$prompt"
    echo -e "${DIM}  (Use ←→ arrows, Enter to confirm)${NC}"

    tput civis
    trap 'tput cnorm; exit 130' INT

    local bar_width=20

    _slider_is_valid() {
        local value="$1"
        if [ ${#validate_patterns[@]} -eq 0 ]; then
            return 0
        fi
        local v_idx
        for ((v_idx=0; v_idx<${#validate_patterns[@]}; v_idx++)); do
            local v_pattern="${validate_patterns[$v_idx]}"
            if [ -n "$v_pattern" ] && [[ ! "$value" =~ $v_pattern ]]; then
                return 1
            fi
        done
        return 0
    }

    draw_slider_line() {
        local value=$1
        local range=$((max - min))
        local position=$(( (value - min) * bar_width / range ))

        echo -ne "\r"
        tput el
        local bar="${YELLOW}[${NC}"
        for ((idx=0; idx<bar_width; idx++)); do
            if [ $idx -eq $position ]; then
                bar+="${GREEN}◉${NC}"
            elif [ $idx -lt $position ]; then
                bar+="${GREEN}━${NC}"
            else
                bar+="━"
            fi
        done
        if _slider_is_valid "$value"; then
            bar+="${YELLOW}]${NC} ${BOLD}${value}${NC}"
        else
            bar+="${YELLOW}]${NC} ${RED}${BOLD}${value}${NC}"
        fi
        if [ -n "$unit" ]; then
            bar+=" ${DIM}[${unit}]${NC}"
        fi

        echo -ne "  $bar"
    }

    draw_slider_line $current

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true

            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'C')
                        if [ $current -lt $max ]; then
                            current=$((current + step))
                            if [ $current -gt $max ]; then
                                current=$max
                            fi
                            draw_slider_line $current
                        fi
                        ;;
                    'D')
                        if [ $current -gt $min ]; then
                            current=$((current - step))
                            if [ $current -lt $min ]; then
                                current=$min
                            fi
                            draw_slider_line $current
                        fi
                        ;;
                esac
            fi
        elif [[ $key =~ [0-9] ]]; then
            local input_str="$key"
            while true; do
                local input_num=$input_str
                if [[ "$input_num" =~ ^[0-9]+$ ]] && [ "$input_num" -ge "$min" ] && [ "$input_num" -le "$max" ]; then
                    current=$input_num
                fi
                draw_slider_line $current
                IFS= read -rsn1 -t 0.4 next_key || true
                if [[ -z "$next_key" ]]; then
                    break
                fi
                if [[ $next_key =~ [0-9] ]]; then
                    input_str+="$next_key"
                elif [[ $next_key == $'\177' ]] || [[ $next_key == $'\b' ]]; then
                    input_str="${input_str%?}"
                    if [ -z "$input_str" ]; then
                        break
                    fi
                elif [[ $next_key == "" ]]; then
                    break
                else
                    break
                fi
            done
        elif [[ $key == "" ]]; then
            if ! _slider_is_valid "$current"; then
                local err="Validation failed"
                local v_idx
                for ((v_idx=0; v_idx<${#validate_errors[@]}; v_idx++)); do
                    local v_error="${validate_errors[$v_idx]}"
                    if [ -n "$v_error" ]; then
                        err="$v_error"
                        break
                    fi
                done
                _show_inline_error "$err" true
                draw_slider_line $current
                continue
            fi
            break
        fi
    done

    trap - INT
    tput cnorm
    echo

    state_set "$variable" "$current"
}

input_slider_inline() {
    local label="$1"
    local min="${2:-0}"
    local max="${3:-100}"
    local step="${4:-1}"
    local default="${5:-}"
    local unit="${6:-}"
    shift 6
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        validate_patterns+=("$1")
        validate_errors+=("$2")
        shift 2
    done

    local current=$default
    if [ -z "$current" ]; then
        current=$(( (min + max) / 2 ))
    fi

    local bar_width=15

    draw_inline() {
        local value=$1
        local range=$((max - min))
        local position=$(( (value - min) * bar_width / range ))

        inline_on_change "$value"
        inline_clear
        local bar=""
        for ((idx=0; idx<bar_width; idx++)); do
            if [ $idx -eq $position ]; then
                bar+="${GREEN}◉${NC}"
            elif [ $idx -lt $position ]; then
                bar+="${GREEN}━${NC}"
            else
                bar+="━"
            fi
        done
        if [ ${#validate_patterns[@]} -eq 0 ] || [[ "$value" =~ ${validate_patterns[0]} ]]; then
            bar+=" ${BOLD}${value}${NC}"
        else
            bar+=" ${RED}${BOLD}${value}${NC}"
        fi
        if [ -n "$unit" ]; then
            bar+=" ${DIM}[${unit}]${NC}"
        fi

        inline_prefix "$label"
        inline_wrap_start
        echo -ne "$bar"
        inline_wrap_end
        inline_suffix
    }

    draw_inline $current

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true

            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'C')
                        if [ $current -lt $max ]; then
                            current=$((current + step))
                            if [ $current -gt $max ]; then
                                current=$max
                            fi
                            draw_inline $current
                        fi
                        ;;
                    'D')
                        if [ $current -gt $min ]; then
                            current=$((current - step))
                            if [ $current -lt $min ]; then
                                current=$min
                            fi
                            draw_inline $current
                        fi
                        ;;
                esac
            fi
        elif [[ $key =~ [0-9] ]]; then
            local input_str="$key"
            while true; do
                local input_num=$input_str
                if [[ "$input_num" =~ ^[0-9]+$ ]] && [ "$input_num" -ge "$min" ] && [ "$input_num" -le "$max" ]; then
                    current=$input_num
                fi
                draw_inline $current
                IFS= read -rsn1 -t 0.4 next_key || true
                if [[ -z "$next_key" ]]; then
                    break
                fi
                if [[ $next_key =~ [0-9] ]]; then
                    input_str+="$next_key"
                elif [[ $next_key == $'\177' ]] || [[ $next_key == $'\b' ]]; then
                    input_str="${input_str%?}"
                    if [ -z "$input_str" ]; then
                        break
                    fi
                elif [[ $next_key == "" ]]; then
                    break
                else
                    break
                fi
            done
        elif [[ $key == "" ]]; then
            if [ ${#validate_patterns[@]} -gt 0 ]; then
                local ok=true
                local v_idx
                for ((v_idx=0; v_idx<${#validate_patterns[@]}; v_idx++)); do
                    local v_pattern="${validate_patterns[$v_idx]}"
                    if [ -n "$v_pattern" ] && [[ ! "$current" =~ $v_pattern ]]; then
                        local err="${validate_errors[$v_idx]:-Validation failed}"
                        inline_error "$err" "$label"
                        draw_inline $current
                        ok=false
                        break
                    fi
                done
                if [ "$ok" != true ]; then
                    continue
                fi
            fi
            break
        fi
    done

    echo
    INPUT_VALUE="$current"
}
