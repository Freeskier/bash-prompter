#!/bin/bash

[[ -n "${_COMPONENT_CONFIRM_SH_LOADED:-}" ]] && return
_COMPONENT_CONFIRM_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_confirm() {
    local prompt="$1"
    local variable="$2"
    local default="${3:-false}"

    prompt=$(interpolate "$prompt")

    print_step "$prompt"
    echo -e "${DIM}  (Use ←→ arrows or Y/N, Enter to confirm)${NC}"
    # Set default selection
    local selected=0
    if [ "$default" = "true" ] || [ "$default" = "yes" ] || [ "$default" = "y" ]; then
        selected=0  # Yes is default
    else
        selected=1  # No is default
    fi

    # Hide cursor
    cursor_hide

    # Trap to restore cursor on Ctrl+C
    trap 'cursor_show; exit 130' INT

    # Draw initial buttons
    if [ $selected -eq 0 ]; then
        echo -e "  ${MAGENTA}${BOLD}[ Yes ]${NC}  ${DIM}[ No ]${NC}"
    else
        echo -e "  ${DIM}[ Yes ]${NC}  ${MAGENTA}${BOLD}[ No ]${NC}"
    fi

    # Read keys
    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            # Read escape sequence
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true

            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'C'|'D') # Right or Left arrow
                        selected=$((1 - selected))  # Toggle between 0 and 1
                        # Redraw
                        tput cuu 1
                        tput el
                        if [ $selected -eq 0 ]; then
                            echo -e "  ${MAGENTA}${BOLD}[ Yes ]${NC}  ${DIM}[ No ]${NC}"
                        else
                            echo -e "  ${DIM}[ Yes ]${NC}  ${MAGENTA}${BOLD}[ No ]${NC}"
                        fi
                        ;;
                esac
            fi
        elif [[ $key == "y" ]] || [[ $key == "Y" ]]; then
            selected=0
            # Redraw
            tput cuu 1
            tput el
            echo -e "  ${MAGENTA}${BOLD}[ Yes ]${NC}  ${DIM}[ No ]${NC}"
        elif [[ $key == "n" ]] || [[ $key == "N" ]]; then
            selected=1
            # Redraw
            tput cuu 1
            tput el
            echo -e "  ${DIM}[ Yes ]${NC}  ${MAGENTA}${BOLD}[ No ]${NC}"
        elif [[ $key == "" ]]; then
            # Enter
            break
        fi
    done

    # Show cursor and remove trap
    trap - INT
    cursor_show

    # Return result and store in state
    if [ $selected -eq 0 ]; then
        print_success "Yes"
        state_set "$variable" "true"
    else
        print_success "No"
        state_set "$variable" "false"
    fi
}