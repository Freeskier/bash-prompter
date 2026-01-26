#!/bin/bash

[[ -n "${_COMPONENT_MULTISELECT_SH_LOADED:-}" ]] && return
_COMPONENT_MULTISELECT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_multiselect() {
    local prompt="$1"
    local variable="$2"
    shift 2
    local OPTIONS_ARRAY=("$@")

    prompt=$(interpolate "$prompt")

    print_step "$prompt"
    echo -e "${DIM}  (Use ↑↓ arrows, Space to toggle, Enter to confirm)${NC}"

    local selected_index=0
    local num_options=${#OPTIONS_ARRAY[@]}

    declare -a is_selected
    for ((idx=0; idx<num_options; idx++)); do
        is_selected[$idx]=0
    done

    cursor_hide
    trap 'cursor_show; exit 130' INT

    for idx in "${!OPTIONS_ARRAY[@]}"; do
        local checkbox=" "
        if [ ${is_selected[$idx]} -eq 1 ]; then
            checkbox="${GREEN}✓${NC}"
        fi

        if [ $idx -eq $selected_index ]; then
            echo -e "  ${YELLOW}>${NC} [${checkbox}] ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
        else
            echo -e "    [${checkbox}] ${OPTIONS_ARRAY[$idx]}"
        fi
    done

    while true; do
        IFS= read -rsn1 key

        local should_redraw=0

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true

            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'A')
                        if [ $selected_index -gt 0 ]; then
                            selected_index=$((selected_index - 1))
                            should_redraw=1
                        fi
                        ;;
                    'B')
                        if [ $selected_index -lt $((num_options - 1)) ]; then
                            selected_index=$((selected_index + 1))
                            should_redraw=1
                        fi
                        ;;
                esac
            fi
        elif [[ $key == " " ]]; then
            if [ ${is_selected[$selected_index]} -eq 1 ]; then
                is_selected[$selected_index]=0
            else
                is_selected[$selected_index]=1
            fi
            should_redraw=1
        elif [[ $key == "" ]]; then
            break
        fi

        if [ $should_redraw -eq 1 ]; then
            tput cuu $num_options
            for idx in "${!OPTIONS_ARRAY[@]}"; do
                tput el
                local checkbox=" "
                if [ ${is_selected[$idx]} -eq 1 ]; then
                    checkbox="${GREEN}✓${NC}"
                fi

                if [ $idx -eq $selected_index ]; then
                    echo -e "  ${YELLOW}>${NC} [${checkbox}] ${BOLD}${OPTIONS_ARRAY[$idx]}${NC}"
                else
                    echo -e "    [${checkbox}] ${OPTIONS_ARRAY[$idx]}"
                fi
            done
        fi
    done

    trap - INT
    cursor_show

    local selected=()
    for ((idx=0; idx<num_options; idx++)); do
        if [ ${is_selected[$idx]} -eq 1 ]; then
            selected+=("${OPTIONS_ARRAY[$idx]}")
        fi
    done

    if [ ${#selected[@]} -gt 0 ]; then
        print_success "Selected: ${selected[*]}"

        # Store as flat indexed array for loop compatibility
        state_set "${variable}_count" "${#selected[@]}"
        for ((idx=0; idx<${#selected[@]}; idx++)); do
            state_set "${variable}_${idx}" "${selected[$idx]}"
        done
    else
        print_info "Nothing selected"
        state_set "${variable}_count" "0"
    fi
}