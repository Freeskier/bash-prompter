#!/bin/bash

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_reader.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/validator.sh"

input_text() {
    local prompt="$1"
    local variable="$2"
    local placeholder="$3"
    local default="$4"
    shift 4
    local validate_type=""
    local validate_type_error=""
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        if [ "$1" = "--type" ]; then
            validate_type="$2"
            shift 2
            continue
        fi
        if [ "$1" = "--type-error" ]; then
            validate_type_error="$2"
            shift 2
            continue
        fi
        validate_patterns+=("$1")
        validate_errors+=("$2")
        shift 2
    done

    prompt=$(interpolate "$prompt")
    placeholder=$(interpolate "$placeholder")
    default=$(interpolate "$default")

    echo -ne "\n${BLUE}▶${NC} ${BOLD}${prompt}${NC}"
    if [ -n "$default" ]; then
        echo -ne " ${DIM}[default: ${default}]${NC}"
    fi
    echo ""

    _text_is_valid() {
        local value="$1"
        local type="${validate_type}"
        if [ -n "$type" ]; then
            if ! validate "$type" "$value" >/dev/null; then
                return 1
            fi
        fi
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

    while true; do
        local input_value=""
        local has_placeholder=true

        draw_line() {
            echo -ne "\r"
            tput el
            echo -ne "  ${YELLOW}>${NC} "
            if [ -z "$input_value" ] && [ -n "$placeholder" ] && [ "$has_placeholder" = true ]; then
                echo -ne "${DIM}${placeholder}${NC}"
                return
            fi
            if [ -z "$input_value" ]; then
                return
            fi
            if _text_is_valid "$input_value"; then
                echo -ne "${GREEN}${input_value}${NC}"
            else
                echo -ne "${RED}${input_value}${NC}"
            fi
        }

        draw_line

        while true; do
            IFS= read -rsn1 char
        if [[ $char == $'\0' ]] || [[ $char == "" ]]; then
            echo
            break
        elif [[ $char == $'\x17' ]]; then
            if [ ${#input_value} -gt 0 ]; then
                while [[ "$input_value" == *" " ]]; do
                    input_value="${input_value% }"
                done
                while [ -n "$input_value" ] && [[ "${input_value: -1}" != " " ]]; do
                    input_value="${input_value%?}"
                done
                while [[ "$input_value" == *" " ]]; do
                    input_value="${input_value% }"
                done
            fi
            if [ -z "$input_value" ]; then
                has_placeholder=true
            fi
            draw_line
        elif [[ $char == $'\177' ]] || [[ $char == $'\b' ]]; then
            if [ ${#input_value} -gt 0 ]; then
                input_value="${input_value%?}"
            fi
                if [ -z "$input_value" ]; then
                    has_placeholder=true
                fi
                draw_line
            else
                if [ "$has_placeholder" = true ]; then
                    input_value=""
                    has_placeholder=false
                fi
                input_value+="$char"
                draw_line
            fi
        done

        if [ -z "$input_value" ] && [ -n "$default" ]; then
            input_value="$default"
        fi

        if [ -z "$input_value" ]; then
            echo -e "  ${RED}✗${NC} Value cannot be empty"
            continue
        fi

        if _text_is_valid "$input_value"; then
            state_set "$variable" "$input_value"
            break
        fi

        local error_msg="Validation failed"
        if [ -n "$validate_type" ]; then
            if [ -n "$validate_type_error" ]; then
                error_msg="$validate_type_error"
            else
                local type_err
                type_err="$(validate "$validate_type" "$input_value" 2>/dev/null)"
                if [ -n "$type_err" ]; then
                    error_msg="$type_err"
                fi
            fi
        fi
        local v_idx
        for ((v_idx=0; v_idx<${#validate_errors[@]}; v_idx++)); do
            local v_error="${validate_errors[$v_idx]}"
            if [ -n "$v_error" ]; then
                error_msg="$v_error"
                break
            fi
        done
        echo -e "  ${RED}✗${NC} $error_msg"
    done
}

input_text_inline() {
    local label="$1"
    local placeholder="$2"
    local default="$3"
    shift 3
    local validate_type=""
    local validate_type_error=""
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        if [ "$1" = "--type" ]; then
            validate_type="$2"
            shift 2
            continue
        fi
        if [ "$1" = "--type-error" ]; then
            validate_type_error="$2"
            shift 2
            continue
        fi
        validate_patterns+=("$1")
        validate_errors+=("$2")
        shift 2
    done

    placeholder=$(interpolate "$placeholder")
    default=$(interpolate "$default")

    local input_value="$default"
    local has_placeholder=true
    if [ -n "$input_value" ]; then
        has_placeholder=false
    fi

    _text_is_valid() {
        local value="$1"
        local type="${validate_type}"
        if [ -n "$type" ]; then
            if ! validate "$type" "$value" >/dev/null; then
                return 1
            fi
        fi
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

    draw_inline() {
        inline_on_change "$input_value"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        if [ -z "$input_value" ] && [ -n "$placeholder" ] && [ "$has_placeholder" = true ]; then
            echo -ne "${DIM}${placeholder}${NC}"
            inline_wrap_end
            inline_suffix
            return
        fi
        if _text_is_valid "$input_value"; then
            echo -ne "${GREEN}${input_value}${NC}"
        else
            echo -ne "${RED}${BOLD}${input_value}${NC}"
        fi
        inline_wrap_end
        inline_suffix
    }

    draw_inline

    while true; do
        IFS= read -rsn1 char
        if [[ $char == "" ]]; then
            if ! _text_is_valid "$input_value"; then
                local error_msg="Validation failed"
                if [ -n "$validate_type" ]; then
                    if [ -n "$validate_type_error" ]; then
                        error_msg="$validate_type_error"
                    else
                        local type_err
                        type_err="$(validate "$validate_type" "$input_value" 2>/dev/null)"
                        if [ -n "$type_err" ]; then
                            error_msg="$type_err"
                        fi
                    fi
                fi
                local v_idx
                for ((v_idx=0; v_idx<${#validate_errors[@]}; v_idx++)); do
                    local v_error="${validate_errors[$v_idx]}"
                    if [ -n "$v_error" ]; then
                        error_msg="$v_error"
                        break
                    fi
                done
                inline_error "$error_msg" "$label"
                draw_inline
                continue
            fi
            break
        elif [[ $char == $'\x17' ]]; then
            if [ ${#input_value} -gt 0 ]; then
                while [[ "$input_value" == *" " ]]; do
                    input_value="${input_value% }"
                done
                while [ -n "$input_value" ] && [[ "${input_value: -1}" != " " ]]; do
                    input_value="${input_value%?}"
                done
                while [[ "$input_value" == *" " ]]; do
                    input_value="${input_value% }"
                done
            fi
            if [ -z "$input_value" ]; then
                has_placeholder=true
            fi
            draw_inline
        elif [[ $char == $'\177' ]] || [[ $char == $'\b' ]]; then
            if [ ${#input_value} -gt 0 ]; then
                input_value="${input_value%?}"
            fi
            if [ -z "$input_value" ]; then
                has_placeholder=true
            fi
            draw_inline
        else
            if [ "$has_placeholder" = true ]; then
                input_value=""
                has_placeholder=false
            fi
            input_value+="$char"
            draw_inline
        fi
    done

    echo
    INPUT_VALUE="$input_value"
}
