#!/bin/bash

[[ -n "${_INPUT_IP_SH_LOADED:-}" ]] && return
_INPUT_IP_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/text.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"

input_ip() {
    local prompt="$1"
    local variable="$2"
    local placeholder="${3:-192.168.1.1}"
    local default="$4"

    prompt=$(interpolate "$prompt")
    placeholder=$(interpolate "$placeholder")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Use ←→ to move, ↑↓ to change, digits to type, Enter to confirm)${NC}"

    local -a octet_str=("0" "0" "0" "0")
    local -a octet_touched=(0 0 0 0)
    local idx=0

    if [ -n "$default" ] && [[ "$default" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]]; then
        IFS='.' read -r -a octet_str <<< "$default"
        octet_touched=(0 0 0 0)
    elif [ -n "$placeholder" ] && [[ "$placeholder" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]]; then
        IFS='.' read -r -a octet_str <<< "$placeholder"
        octet_touched=(0 0 0 0)
    fi

    is_octet_valid() {
        local v="$1"
        if [ -z "$v" ]; then
            return 1
        fi
        if [[ ! "$v" =~ ^[0-9]{1,3}$ ]]; then
            return 1
        fi
        if [ "$v" -gt 255 ]; then
            return 1
        fi
        return 0
    }

    draw_line() {
        if [ -n "${INPUT_INLINE_ON_CHANGE:-}" ]; then
            "${INPUT_INLINE_ON_CHANGE}" "${octet_str[0]}.${octet_str[1]}.${octet_str[2]}.${octet_str[3]}"
        fi
        echo -ne "\r"
        tput el
        echo -ne "  ${YELLOW}>${NC} "
        local i
        for ((i=0; i<4; i++)); do
            local part="${octet_str[$i]}"
            local color="$GREEN"
            if ! is_octet_valid "$part"; then
                color="${RED}${BOLD}"
            fi
            if [ $i -eq $idx ]; then
                echo -ne "${YELLOW}${BOLD}${part}${NC}"
            else
                echo -ne "${color}${part}${NC}"
            fi
            if [ $i -lt 3 ]; then
                echo -ne "."
            fi
        done
    }

    tput civis
    trap 'tput cnorm; exit 130' INT

    draw_line

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'C')
                        if [ $idx -lt 3 ]; then idx=$((idx + 1)); fi
                        draw_line
                        ;;
                    'D')
                        if [ $idx -gt 0 ]; then idx=$((idx - 1)); fi
                        draw_line
                        ;;
                    'A')
                        local v="${octet_str[$idx]}"
                        if ! is_octet_valid "$v"; then v=0; fi
                        v=$((v + 1))
                        if [ $v -gt 255 ]; then v=0; fi
                        octet_str[$idx]="$v"
                        octet_touched[$idx]=1
                        draw_line
                        ;;
                    'B')
                        local v="${octet_str[$idx]}"
                        if ! is_octet_valid "$v"; then v=0; fi
                        v=$((v - 1))
                        if [ $v -lt 0 ]; then v=255; fi
                        octet_str[$idx]="$v"
                        octet_touched[$idx]=1
                        draw_line
                        ;;
                esac
            fi
        elif [[ $key == "." ]]; then
            if [ $idx -lt 3 ]; then idx=$((idx + 1)); fi
            draw_line
        elif [[ $key == $'\177' ]] || [[ $key == $'\b' ]]; then
            if [ -n "${octet_str[$idx]}" ]; then
                octet_str[$idx]="${octet_str[$idx]%?}"
                octet_touched[$idx]=1
                draw_line
            fi
        elif [[ $key == $'\x17' ]]; then
            octet_str[$idx]=""
            octet_touched[$idx]=1
            draw_line
        elif [[ $key =~ [0-9] ]]; then
            local cur="${octet_str[$idx]}"
            if [ "${octet_touched[$idx]}" -eq 0 ] || [ ${#cur} -ge 3 ]; then
                cur="$key"
            else
                cur+="$key"
            fi
            if [ ${#cur} -le 3 ]; then
                octet_str[$idx]="$cur"
                octet_touched[$idx]=1
            fi
            draw_line
        elif [[ $key == "" ]]; then
            local all_ok=true
            local i
            for ((i=0; i<4; i++)); do
                if ! is_octet_valid "${octet_str[$i]}"; then
                    all_ok=false
                    break
                fi
            done
            if [ "$all_ok" = true ]; then
                break
            fi
            inline_error "Nieprawidłowy format IP" "$label"
            draw_line
        fi
    done

    trap - INT
    tput cnorm
    echo

    state_set "$variable" "${octet_str[0]}.${octet_str[1]}.${octet_str[2]}.${octet_str[3]}"
}

input_ip_inline() {
    local label="$1"
    local placeholder="${2:-192.168.1.1}"
    local default="${3:-}"

    placeholder=$(interpolate "$placeholder")
    default=$(interpolate "$default")

    local -a octet_str=("0" "0" "0" "0")
    local -a octet_touched=(0 0 0 0)
    local idx=0

    if [ -n "$default" ] && [[ "$default" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]]; then
        IFS='.' read -r -a octet_str <<< "$default"
        octet_touched=(0 0 0 0)
    elif [ -n "$placeholder" ] && [[ "$placeholder" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]]; then
        IFS='.' read -r -a octet_str <<< "$placeholder"
        octet_touched=(0 0 0 0)
    fi

    is_octet_valid() {
        local v="$1"
        if [ -z "$v" ]; then
            return 1
        fi
        if [[ ! "$v" =~ ^[0-9]{1,3}$ ]]; then
            return 1
        fi
        if [ "$v" -gt 255 ]; then
            return 1
        fi
        return 0
    }

    draw_line() {
        inline_on_change "${octet_str[0]}.${octet_str[1]}.${octet_str[2]}.${octet_str[3]}"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        local i
        for ((i=0; i<4; i++)); do
            local part="${octet_str[$i]}"
            local color="$GREEN"
            if ! is_octet_valid "$part"; then
                color="${RED}${BOLD}"
            fi
            if [ $i -eq $idx ]; then
                if ! is_octet_valid "$part"; then
                    echo -ne "${RED}${BOLD}${part}${NC}"
                else
                    echo -ne "${YELLOW}${BOLD}${part}${NC}"
                fi
            else
                echo -ne "${color}${part}${NC}"
            fi
            if [ $i -lt 3 ]; then
                echo -ne "."
            fi
        done
        inline_wrap_end
        inline_suffix
    }

    draw_line

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'C')
                        if [ $idx -lt 3 ]; then idx=$((idx + 1)); fi
                        draw_line
                        ;;
                    'D')
                        if [ $idx -gt 0 ]; then idx=$((idx - 1)); fi
                        draw_line
                        ;;
                    'A')
                        local v="${octet_str[$idx]}"
                        if ! is_octet_valid "$v"; then v=0; fi
                        v=$((v + 1))
                        if [ $v -gt 255 ]; then v=0; fi
                        octet_str[$idx]="$v"
                        octet_touched[$idx]=1
                        draw_line
                        ;;
                    'B')
                        local v="${octet_str[$idx]}"
                        if ! is_octet_valid "$v"; then v=0; fi
                        v=$((v - 1))
                        if [ $v -lt 0 ]; then v=255; fi
                        octet_str[$idx]="$v"
                        octet_touched[$idx]=1
                        draw_line
                        ;;
                esac
            fi
        elif [[ $key == "." ]]; then
            if [ $idx -lt 3 ]; then idx=$((idx + 1)); fi
            draw_line
        elif [[ $key == $'\177' ]] || [[ $key == $'\b' ]]; then
            if [ -n "${octet_str[$idx]}" ]; then
                octet_str[$idx]="${octet_str[$idx]%?}"
                octet_touched[$idx]=1
                draw_line
            fi
        elif [[ $key =~ [0-9] ]]; then
            local cur="${octet_str[$idx]}"
            if [ "${octet_touched[$idx]}" -eq 0 ] || [ ${#cur} -ge 3 ]; then
                cur="$key"
            else
                cur+="$key"
            fi
            if [ ${#cur} -le 3 ]; then
                octet_str[$idx]="$cur"
                octet_touched[$idx]=1
            fi
            draw_line
        elif [[ $key == "" ]]; then
            local all_ok=true
            local i
            for ((i=0; i<4; i++)); do
                if ! is_octet_valid "${octet_str[$i]}"; then
                    all_ok=false
                    break
                fi
            done
            if [ "$all_ok" = true ]; then
                break
            fi
            inline_error "Nieprawidłowy format IP" "$label"
            draw_line
        fi
    done

    echo
    INPUT_VALUE="${octet_str[0]}.${octet_str[1]}.${octet_str[2]}.${octet_str[3]}"
}
