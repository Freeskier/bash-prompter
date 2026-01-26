#!/bin/bash

[[ -n "${_COMPONENT_RADIO_GROUP_SH_LOADED:-}" ]] && return
_COMPONENT_RADIO_GROUP_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_radio_group() {
    local prompt="$1"
    local variable="$2"
    local default="$3"
    shift 3
    local OPTIONS_ARRAY=("$@")

    prompt=$(interpolate "$prompt")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Use ↑↓ arrows to navigate, Enter to select)${NC}"

    local cursor_index=0
    local selected_option_index=0

    if [ -n "$default" ]; then
        for j in "${!OPTIONS_ARRAY[@]}"; do
            if [ "${OPTIONS_ARRAY[$j]}" = "$default" ]; then
                cursor_index=$j
                selected_option_index=$j
                break
            fi
        done
    fi

    local num_options=${#OPTIONS_ARRAY[@]}

    cursor_hide
    trap 'cursor_show; exit 130' INT

    for idx in "${!OPTIONS_ARRAY[@]}"; do
        local is_cursor=$([ $idx -eq $cursor_index ] && echo 1 || echo 0)
        local is_this_selected=$([ $idx -eq $selected_option_index ] && echo 1 || echo 0)

        if [ $is_cursor -eq 1 ]; then
            if [ $is_this_selected -eq 1 ]; then
                echo -e "  ${YELLOW}>${NC} ${GREEN}◉${NC} ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
            else
                echo -e "  ${YELLOW}>${NC} ○ ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
            fi
        else
            if [ $is_this_selected -eq 1 ]; then
                echo -e "    ${GREEN}◉${NC} ${OPTIONS_ARRAY[$idx]}"
            else
                echo -e "    ○ ${OPTIONS_ARRAY[$idx]}"
            fi
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
                        if [ $cursor_index -gt 0 ]; then
                            cursor_index=$((cursor_index - 1))
                        fi
                        ;;
                    'B')
                        if [ $cursor_index -lt $((num_options - 1)) ]; then
                            cursor_index=$((cursor_index + 1))
                        fi
                        ;;
                esac

                tput cuu $num_options
                for idx in "${!OPTIONS_ARRAY[@]}"; do
                    tput el
                    local is_cursor=$([ $idx -eq $cursor_index ] && echo 1 || echo 0)
                    local is_this_selected=$([ $idx -eq $selected_option_index ] && echo 1 || echo 0)

                    if [ $is_cursor -eq 1 ]; then
                        if [ $is_this_selected -eq 1 ]; then
                            echo -e "  ${YELLOW}>${NC} ${GREEN}◉${NC} ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
                        else
                            echo -e "  ${YELLOW}>${NC} ○ ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
                        fi
                    else
                        if [ $is_this_selected -eq 1 ]; then
                            echo -e "    ${GREEN}◉${NC} ${OPTIONS_ARRAY[$idx]}"
                        else
                            echo -e "    ○ ${OPTIONS_ARRAY[$idx]}"
                        fi
                    fi
                done
            fi
        elif [[ $key == " " ]]; then
            selected_option_index=$cursor_index
            tput cuu $num_options
            for idx in "${!OPTIONS_ARRAY[@]}"; do
                tput el
                local is_cursor=$([ $idx -eq $cursor_index ] && echo 1 || echo 0)
                local is_this_selected=$([ $idx -eq $selected_option_index ] && echo 1 || echo 0)

                if [ $is_cursor -eq 1 ]; then
                    if [ $is_this_selected -eq 1 ]; then
                        echo -e "  ${YELLOW}>${NC} ${GREEN}◉${NC} ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
                    else
                        echo -e "  ${YELLOW}>${NC} ○ ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
                    fi
                else
                    if [ $is_this_selected -eq 1 ]; then
                        echo -e "    ${GREEN}◉${NC} ${OPTIONS_ARRAY[$idx]}"
                    else
                        echo -e "    ○ ${OPTIONS_ARRAY[$idx]}"
                    fi
                fi
            done
        elif [[ $key == "" ]]; then
            selected_option_index=$cursor_index
            break
        fi
    done

    trap - INT
    cursor_show

    local selected="${OPTIONS_ARRAY[$selected_option_index]}"
    state_set "$variable" "$selected"
}