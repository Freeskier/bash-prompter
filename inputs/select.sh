#!/bin/bash

[[ -n "${_INPUT_SELECT_SH_LOADED:-}" ]] && return
_INPUT_SELECT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

input_select() {
    local prompt="$1"
    local variable="$2"
    shift 2
    local OPTIONS_ARRAY=("$@")

    prompt=$(interpolate "$prompt")

    print_step "$prompt"
    echo -e "${DIM}  (Use ↑↓ arrows, Enter to select)${NC}"

    local selected_index=0
    local num_options=${#OPTIONS_ARRAY[@]}

    if [ -z "${INPUT_INLINE_KEEP_CURSOR:-}" ]; then
        tput civis
        trap 'tput cnorm; exit 130' INT
    fi

    for idx in "${!OPTIONS_ARRAY[@]}"; do
        if [ $idx -eq $selected_index ]; then
            echo -e "  ${YELLOW}>${NC} ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
        else
            echo -e "    ${OPTIONS_ARRAY[$idx]}"
        fi
    done

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true

            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'A')
                        if [ $selected_index -gt 0 ]; then
                            selected_index=$((selected_index - 1))
                        fi
                        ;;
                    'B')
                        if [ $selected_index -lt $((num_options - 1)) ]; then
                            selected_index=$((selected_index + 1))
                        fi
                        ;;
                esac

                tput cuu $num_options
                for idx in "${!OPTIONS_ARRAY[@]}"; do
                    tput el
                    if [ $idx -eq $selected_index ]; then
                        echo -e "  ${YELLOW}>${NC} ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
                    else
                        echo -e "    ${OPTIONS_ARRAY[$idx]}"
                    fi
                done
            fi
        elif [[ $key == "" ]]; then
            break
        fi
    done

    if [ -z "${INPUT_INLINE_KEEP_CURSOR:-}" ]; then
        trap - INT
        tput cnorm
    fi

    local selected="${OPTIONS_ARRAY[$selected_index]}"
    print_success "Selected: $selected"
    state_set "$variable" "$selected"
}

input_select_inline() {
    local label="$1"
    shift 1
    local OPTIONS_ARRAY=("$@")

    local selected_index=0
    local num_options=${#OPTIONS_ARRAY[@]}

    if [ -z "${INPUT_INLINE_KEEP_CURSOR:-}" ]; then
        tput civis
        trap 'tput cnorm; return 130' INT
    fi

    draw_inline() {
        local value="${OPTIONS_ARRAY[$selected_index]}"
        inline_on_change "$value"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        echo -ne "${DIM}${BOLD}<${NC} ${BOLD}${value}${NC} ${DIM}${BOLD}>${NC}"
        inline_wrap_end
        inline_suffix
    }

    draw_inline

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true

            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'C'|'B')
                        selected_index=$(( (selected_index + 1) % num_options ))
                        draw_inline
                        ;;
                    'D'|'A')
                        selected_index=$(( (selected_index - 1 + num_options) % num_options ))
                        draw_inline
                        ;;
                esac
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
    INPUT_VALUE="${OPTIONS_ARRAY[$selected_index]}"
}
