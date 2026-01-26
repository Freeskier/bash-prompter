#!/bin/bash

[[ -n "${_INPUT_TOGGLE_SH_LOADED:-}" ]] && return
_INPUT_TOGGLE_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

input_toggle() {
    local prompt="$1"
    local variable="$2"
    local active_label="${3:-active}"
    local inactive_label="${4:-inactive}"
    local default="${5:-false}"

    prompt=$(interpolate "$prompt")
    active_label=$(interpolate "$active_label")
    inactive_label=$(interpolate "$inactive_label")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Use ←→ to toggle, Enter to confirm)${NC}"

    local selected=false
    if [ "$default" = "true" ] || [ "$default" = "yes" ] || [ "$default" = "1" ] || [ "$default" = "$active_label" ]; then
        selected=true
    fi

    tput civis
    trap 'tput cnorm; exit 130' INT

    draw_line() {
        local left="${inactive_label}"
        local right="${active_label}"
        if [ "$selected" = true ]; then
            right="${GREEN}${UNDERLINE}${BOLD}${active_label}${NC}"
            left="${DIM}${inactive_label}${NC}"
        else
            left="${GREEN}${UNDERLINE}${BOLD}${inactive_label}${NC}"
            right="${DIM}${active_label}${NC}"
        fi
        echo -ne "\r  ${left}${NC} / ${right}${NC}"
    }

    draw_line

    while true; do
        IFS= read -rsn1 key
        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'C'|'D')
                        selected=$([ "$selected" = true ] && echo false || echo true)
                        draw_line
                        ;;
                esac
            fi
        elif [[ $key == " " ]]; then
            selected=$([ "$selected" = true ] && echo false || echo true)
            draw_line
        elif [[ $key == "" ]]; then
            break
        fi
    done

    trap - INT
    tput cnorm
    echo

    if [ "$selected" = true ]; then
        state_set "$variable" "true"
    else
        state_set "$variable" "false"
    fi
}

input_toggle_inline() {
    local label="$1"
    local active_label="${2:-active}"
    local inactive_label="${3:-inactive}"
    local default="${4:-false}"

    active_label=$(interpolate "$active_label")
    inactive_label=$(interpolate "$inactive_label")
    default=$(interpolate "$default")

    local selected=false
    if [ "$default" = "true" ] || [ "$default" = "yes" ] || [ "$default" = "1" ] || [ "$default" = "$active_label" ]; then
        selected=true
    fi

    tput civis
    trap 'tput cnorm; return 130' INT

    draw_inline() {
        inline_on_change "$selected"
        local left="${inactive_label}"
        local right="${active_label}"
        if [ "$selected" = true ]; then
            right="${GREEN}${UNDERLINE}${BOLD}${active_label}${NC}"
            left="${DIM}${inactive_label}${NC}"
        else
            left="${GREEN}${UNDERLINE}${BOLD}${inactive_label}${NC}"
            right="${DIM}${active_label}${NC}"
        fi
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        echo -ne "${left}${NC} / ${right}${NC}"
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
                    'C'|'D')
                        selected=$([ "$selected" = true ] && echo false || echo true)
                        draw_inline
                        ;;
                esac
            fi
        elif [[ $key == " " ]]; then
            selected=$([ "$selected" = true ] && echo false || echo true)
            draw_inline
        elif [[ $key == "" ]]; then
            break
        fi
    done

    trap - INT
    tput cnorm
    echo

    if [ "$selected" = true ]; then
        INPUT_VALUE="true"
    else
        INPUT_VALUE="false"
    fi
}
