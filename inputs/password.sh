#!/bin/bash

[[ -n "${_INPUT_PASSWORD_SH_LOADED:-}" ]] && return
_INPUT_PASSWORD_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_reader.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

input_password() {
    local prompt="$1"
    local variable="$2"
    local placeholder="$3"
    shift 3
    local -a validate_patterns=()
    local -a validate_errors=()
    while [ $# -gt 0 ]; do
        validate_patterns+=("$1")
        validate_errors+=("$2")
        shift 2
    done

    prompt=$(interpolate "$prompt")
    placeholder=$(interpolate "$placeholder")

    echo -ne "\n${BLUE}▶${NC} ${BOLD}${prompt}${NC}\n"

    while true; do
        echo -ne "  ${YELLOW}> ${NC}"

        read_with_placeholder "$placeholder" "*" "true"
        local result="$INPUT_VALUE"

        if [ -z "$result" ]; then
            _show_inline_error "Password cannot be empty" true
            continue
        fi

        if [ ${#validate_patterns[@]} -gt 0 ]; then
            local ok=true
            local v_idx
            for ((v_idx=0; v_idx<${#validate_patterns[@]}; v_idx++)); do
                local v_pattern="${validate_patterns[$v_idx]}"
                local v_error="${validate_errors[$v_idx]}"
                if [ -n "$v_pattern" ] && [[ ! "$result" =~ $v_pattern ]]; then
                    _show_inline_error "${v_error:-Validation failed}" true
                    ok=false
                    break
                fi
            done
            if [ "$ok" != true ]; then
                continue
            fi
        fi

        echo
        state_set "$variable" "$result"
        break
    done
}
