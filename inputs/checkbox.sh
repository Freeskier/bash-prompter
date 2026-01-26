#!/bin/bash

[[ -n "${_INPUT_BOOL_SH_LOADED:-}" ]] && return
_INPUT_BOOL_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

input_bool() {
    local prompt="$1"
    local variable="$2"
    local placeholder="${3:-}"
    local default="${4:-false}"

    prompt=$(interpolate "$prompt")
    placeholder=$(interpolate "$placeholder")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Space to toggle, Enter to confirm)${NC}"

    local selected=false
    if [ "$default" = "true" ] || [ "$default" = "yes" ] || [ "$default" = "1" ]; then
        selected=true
    fi

    # Hide cursor
    tput civis

    # Trap to restore cursor on Ctrl+C
    trap 'tput cnorm; exit 130' INT

    draw_checkbox() {
        if [ "$selected" = true ]; then
            echo -e "  ${DIM}[${NC}${GREEN}✓${NC}${DIM}]${NC}"
        else
            echo -e "  ${DIM}[${NC}${RED}X${NC}${DIM}]${NC}"
        fi
    }

    draw_checkbox

    # Read keys
    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 key2 || true
            IFS= read -rsn1 -t 0.001 key3 || true
            if [[ $key2 == '[' ]]; then
                case "$key3" in
                    'A'|'B'|'C'|'D')
                        selected=$([ "$selected" = true ] && echo false || echo true)
                        tput cuu 1
                        tput el
                        draw_checkbox
                        ;;
                esac
            fi
        elif [[ $key == " " ]]; then
            selected=$([ "$selected" = true ] && echo false || echo true)
            tput cuu 1
            tput el
            draw_checkbox
        elif [[ $key == "t" ]] || [[ $key == "T" ]] || [[ $key == "y" ]] || [[ $key == "Y" ]]; then
            selected=true
            tput cuu 1
            tput el
            draw_checkbox
        elif [[ $key == "f" ]] || [[ $key == "F" ]] || [[ $key == "n" ]] || [[ $key == "N" ]]; then
            selected=false
            tput cuu 1
            tput el
            draw_checkbox
        elif [[ $key == "" ]]; then
            # Enter
            break
        fi
    done

    # Show cursor and remove trap
    trap - INT
    tput cnorm

    # Return result and store in state
    if [ "$selected" = true ]; then
        state_set "$variable" "true"
    else
        state_set "$variable" "false"
    fi
}

input_bool_inline() {
    local label="$1"
    local default="${2:-false}"

    local selected=false
    if [ "$default" = "true" ] || [ "$default" = "yes" ] || [ "$default" = "1" ]; then
        selected=true
    fi

    draw_inline() {
        inline_on_change "$selected"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        if [ "$selected" = true ]; then
            echo -ne "${GREEN}✓${NC}"
        else
            echo -ne "${RED}X${NC}"
        fi
        inline_wrap_end
        inline_suffix
    }

    draw_inline

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true

            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'A'|'B'|'C'|'D') # Any arrow - toggle
                        selected=$([ "$selected" = true ] && echo false || echo true)
                        draw_inline
                        ;;
                esac
            fi
        elif [[ $key == " " ]]; then
            selected=$([ "$selected" = true ] && echo false || echo true)
            draw_inline
        elif [[ $key == "t" ]] || [[ $key == "T" ]] || [[ $key == "y" ]] || [[ $key == "Y" ]]; then
            selected=true
            draw_inline
        elif [[ $key == "f" ]] || [[ $key == "F" ]] || [[ $key == "n" ]] || [[ $key == "N" ]]; then
            selected=false
            draw_inline
        elif [[ $key == "" ]]; then
            break
        fi
    done

    echo

    if [ "$selected" = true ]; then
        INPUT_VALUE="true"
    else
        INPUT_VALUE="false"
    fi
}
