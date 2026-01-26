#!/bin/bash

[[ -n "${_INPUT_READER_SH_LOADED:-}" ]] && return
_INPUT_READER_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/colors.sh"

INPUT_VALUE=""

read_with_placeholder() {
    local placeholder="$1"
    local mask_char="${2:-}"
    local no_newline="${3:-false}"

    if [ -n "$placeholder" ]; then
        echo -ne "${DIM}${placeholder}${NC}"
        local placeholder_len=${#placeholder}
        for ((j=0; j<placeholder_len; j++)); do
            echo -ne "\b"
        done
    fi

    INPUT_VALUE=""
    local first_char=true

    while true; do
        IFS= read -rsn1 char

        if [[ $char == $'\0' ]] || [[ $char == "" ]]; then
            if [ "$no_newline" != "true" ]; then
                echo
            fi
            break
        elif [[ $char == $'\x17' ]]; then
            if [ ${#INPUT_VALUE} -gt 0 ]; then
                local original_len=${#INPUT_VALUE}
                # Trim trailing spaces
                while [[ "$INPUT_VALUE" == *" " ]]; do
                    INPUT_VALUE="${INPUT_VALUE% }"
                done
                # Remove last word
                while [ -n "$INPUT_VALUE" ] && [[ "${INPUT_VALUE: -1}" != " " ]]; do
                    INPUT_VALUE="${INPUT_VALUE%?}"
                done
                # Trim trailing spaces
                while [[ "$INPUT_VALUE" == *" " ]]; do
                    INPUT_VALUE="${INPUT_VALUE% }"
                done
                local new_len=${#INPUT_VALUE}
                local to_erase=$((original_len - new_len))
                for ((i=0; i<to_erase; i++)); do
                    echo -ne "\b \b"
                done
            fi
        elif [[ $char == $'\177' ]] || [[ $char == $'\b' ]]; then
            if [ ${#INPUT_VALUE} -gt 0 ]; then
                INPUT_VALUE="${INPUT_VALUE%?}"
                echo -ne "\b \b"
            fi
        else
            if [ "$first_char" = true ] && [ -n "$placeholder" ]; then
                local placeholder_len=${#placeholder}
                for ((j=0; j<placeholder_len; j++)); do
                    echo -ne " "
                done
                for ((j=0; j<placeholder_len; j++)); do
                    echo -ne "\b"
                done
                first_char=false
            fi
            INPUT_VALUE+="$char"
            if [ -n "$mask_char" ]; then
                echo -ne "${DIM}${mask_char}${NC}"
            else
                echo -n "$char"
            fi
        fi
    done
}
