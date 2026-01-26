#!/bin/bash

[[ -n "${_INPUT_DATE_SH_LOADED:-}" ]] && return
_INPUT_DATE_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/inline_engine.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

input_date() {
    local prompt="$1"
    local variable="$2"
    local format="${3:-YYYY-MM-DD HH:mm}"
    local default="${4:-}"

    prompt=$(interpolate "$prompt")
    format=$(interpolate "$format")
    default=$(interpolate "$default")

    print_step "$prompt"
    echo -e "${DIM}  (Use ←→ arrows to move, ↑↓ to change, Enter to confirm)${NC}"

    local fields=()
    local separators=()
    local current=""

    for ((idx=0; idx<${#format}; idx++)); do
        local char="${format:$idx:1}"
        if [[ "$char" =~ [YMDHmS] ]]; then
            current+="$char"
        else
            if [ -n "$current" ]; then
                fields+=("$current")
                current=""
            fi
            separators+=("$char")
        fi
    done
    if [ -n "$current" ]; then
        fields+=("$current")
    fi

    declare -a values
    local today=$(date +"%Y-%m-%d %H:%M:%S")
    for field in "${fields[@]}"; do
        case "$field" in
            YYYY) values+=($(date -d "$today" +"%Y")) ;;
            MM) values+=($(date -d "$today" +"%m")) ;;
            DD) values+=($(date -d "$today" +"%d")) ;;
            HH) values+=($(date -d "$today" +"%H")) ;;
            mm) values+=($(date -d "$today" +"%M")) ;;
            SS) values+=($(date -d "$today" +"%S")) ;;
        esac
    done

    local current_field=0
    local num_fields=${#fields[@]}

    tput civis
    trap 'tput cnorm; exit 130' INT

    draw_date() {
        echo -ne "\r  ${CYAN}[${NC}"
        for ((idx=0; idx<num_fields; idx++)); do
            if [ $idx -eq $current_field ]; then
                echo -ne "${YELLOW}${BOLD}${values[$idx]}${NC}"
            else
                echo -ne "${DIM}${values[$idx]}${NC}"
            fi
            if [ $idx -lt ${#separators[@]} ]; then
                echo -ne "${DIM}${separators[$idx]}${NC}"
            fi
        done
        echo -ne "${CYAN}]${NC}"
    }

    draw_date

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true

            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'C')
                        if [ $current_field -lt $((num_fields - 1)) ]; then
                            current_field=$((current_field + 1))
                        fi
                        draw_date
                        ;;
                    'D')
                        if [ $current_field -gt 0 ]; then
                            current_field=$((current_field - 1))
                        fi
                        draw_date
                        ;;
                    'A')
                        local val=${values[$current_field]}
                        local field_type="${fields[$current_field]}"
                        case "$field_type" in
                            YYYY) val=$((val + 1)) ;;
                            MM) val=$((val % 12 + 1)) ;;
                            DD) val=$((val % 31 + 1)) ;;
                            HH) val=$((val % 24 + 1)) ;;
                            mm|SS) val=$((val % 60 + 1)) ;;
                        esac
                        values[$current_field]=$(printf "%0${#field_type}d" $val)
                        draw_date
                        ;;
                    'B')
                        local val=${values[$current_field]}
                        local field_type="${fields[$current_field]}"
                        case "$field_type" in
                            YYYY) [ $val -gt 1 ] && val=$((val - 1)) ;;
                            MM) val=$(((val - 2 + 12) % 12 + 1)) ;;
                            DD) val=$(((val - 2 + 31) % 31 + 1)) ;;
                            HH) val=$(((val - 1 + 24) % 24)) ;;
                            mm|SS) val=$(((val - 1 + 60) % 60)) ;;
                        esac
                        values[$current_field]=$(printf "%0${#field_type}d" $val)
                        draw_date
                        ;;
                esac
            fi
        elif [[ $key =~ [0-9] ]]; then
            local field_type="${fields[$current_field]}"
            local max_len=${#field_type}
            local input_str="$key"

            # Show preview with underscores
            local preview="$input_str"
            for ((p=${#input_str}; p<max_len; p++)); do
                preview+="_"
            done
            values[$current_field]="$preview"
            draw_date

            while [ ${#input_str} -lt $max_len ]; do
                IFS= read -rsn1 next_key
                if [[ $next_key =~ [0-9] ]]; then
                    input_str+="$next_key"
                    # Update preview
                    preview="$input_str"
                    for ((p=${#input_str}; p<max_len; p++)); do
                        preview+="_"
                    done
                    values[$current_field]="$preview"
                    draw_date
                else
                    break
                fi
            done

            local input_num=$((10#$input_str))
            case "$field_type" in
                YYYY)
                    if [ $input_num -ge 1 ] && [ $input_num -le 9999 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                MM)
                    if [ $input_num -ge 1 ] && [ $input_num -le 12 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                DD)
                    if [ $input_num -ge 1 ] && [ $input_num -le 31 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                HH)
                    if [ $input_num -ge 0 ] && [ $input_num -le 23 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                mm|SS)
                    if [ $input_num -ge 0 ] && [ $input_num -le 59 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
            esac
            draw_date
        elif [[ $key == "" ]]; then
            break
        fi
    done

    # Final redraw with all fields gray
    echo -ne "\r  ${CYAN}[${NC}"
    for ((idx=0; idx<num_fields; idx++)); do
        echo -ne "${DIM}${values[$idx]}${NC}"
        if [ $idx -lt ${#separators[@]} ]; then
            echo -ne "${DIM}${separators[$idx]}${NC}"
        fi
    done
    echo -ne "${CYAN}]${NC}"

    trap - INT
    tput cnorm
    echo

    local result=""
    for ((idx=0; idx<num_fields; idx++)); do
        result+="${values[$idx]}"
        if [ $idx -lt ${#separators[@]} ]; then
            result+="${separators[$idx]}"
        fi
    done

    state_set "$variable" "$result"
}

input_date_inline() {
    local label="$1"
    local format="${2:-YYYY-MM-DD HH:mm}"
    local default="${3:-}"

    local fields=()
    local separators=()
    local current=""

    for ((idx=0; idx<${#format}; idx++)); do
        local char="${format:$idx:1}"
        if [[ "$char" =~ [YMDHmS] ]]; then
            current+="$char"
        else
            if [ -n "$current" ]; then
                fields+=("$current")
                current=""
            fi
            separators+=("$char")
        fi
    done
    if [ -n "$current" ]; then
        fields+=("$current")
    fi

    declare -a values
    local today=$(date +"%Y-%m-%d %H:%M:%S")
    for field in "${fields[@]}"; do
        case "$field" in
            YYYY) values+=($(date -d "$today" +"%Y")) ;;
            MM) values+=($(date -d "$today" +"%m")) ;;
            DD) values+=($(date -d "$today" +"%d")) ;;
            HH) values+=($(date -d "$today" +"%H")) ;;
            mm) values+=($(date -d "$today" +"%M")) ;;
            SS) values+=($(date -d "$today" +"%S")) ;;
        esac
    done

    local current_field=0
    local num_fields=${#fields[@]}

    _date_is_valid_field() {
        local field="$1"
        local value="$2"
        if [[ ! "$value" =~ ^[0-9]+$ ]]; then
            return 1
        fi
        case "$field" in
            YYYY) [ "$value" -ge 1 ] && [ "$value" -le 9999 ] ;;
            MM) [ "$value" -ge 1 ] && [ "$value" -le 12 ] ;;
            DD) [ "$value" -ge 1 ] && [ "$value" -le 31 ] ;;
            HH) [ "$value" -ge 0 ] && [ "$value" -le 23 ] ;;
            mm) [ "$value" -ge 0 ] && [ "$value" -le 59 ] ;;
            SS) [ "$value" -ge 0 ] && [ "$value" -le 59 ] ;;
            *) return 1 ;;
        esac
    }

    _date_is_valid() {
        local idx
        for ((idx=0; idx<num_fields; idx++)); do
            if ! _date_is_valid_field "${fields[$idx]}" "${values[$idx]}"; then
                return 1
            fi
        done
        return 0
    }

    draw_inline() {
        local formatted=""
        for ((idx=0; idx<num_fields; idx++)); do
            formatted+="${values[$idx]}"
            if [ $idx -lt ${#separators[@]} ]; then
                formatted+="${separators[$idx]}"
            fi
        done
        inline_on_change "$formatted"
        inline_clear
        inline_prefix "$label"
        inline_wrap_start
        for ((idx=0; idx<num_fields; idx++)); do
            if ! _date_is_valid_field "${fields[$idx]}" "${values[$idx]}"; then
                echo -ne "${RED}${BOLD}${values[$idx]}${NC}"
            elif [ $idx -eq $current_field ]; then
                echo -ne "${YELLOW}${BOLD}${values[$idx]}${NC}"
            else
                echo -ne "${DIM}${values[$idx]}${NC}"
            fi
            if [ $idx -lt ${#separators[@]} ]; then
                echo -ne "${DIM}${separators[$idx]}${NC}"
            fi
        done
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
                    'C')
                        if [ $current_field -lt $((num_fields - 1)) ]; then
                            current_field=$((current_field + 1))
                        fi
                        draw_inline
                        ;;
                    'D')
                        if [ $current_field -gt 0 ]; then
                            current_field=$((current_field - 1))
                        fi
                        draw_inline
                        ;;
                    'A')
                        local val=${values[$current_field]}
                        local field_type="${fields[$current_field]}"
                        case "$field_type" in
                            YYYY) val=$((val + 1)) ;;
                            MM) val=$((val % 12 + 1)) ;;
                            DD) val=$((val % 31 + 1)) ;;
                            HH) val=$((val % 24 + 1)) ;;
                            mm|SS) val=$((val % 60 + 1)) ;;
                        esac
                        values[$current_field]=$(printf "%0${#field_type}d" $val)
                        draw_inline
                        ;;
                    'B')
                        local val=${values[$current_field]}
                        local field_type="${fields[$current_field]}"
                        case "$field_type" in
                            YYYY) [ $val -gt 1 ] && val=$((val - 1)) ;;
                            MM) val=$(((val - 2 + 12) % 12 + 1)) ;;
                            DD) val=$(((val - 2 + 31) % 31 + 1)) ;;
                            HH) val=$(((val - 1 + 24) % 24)) ;;
                            mm|SS) val=$(((val - 1 + 60) % 60)) ;;
                        esac
                        values[$current_field]=$(printf "%0${#field_type}d" $val)
                        draw_inline
                        ;;
                esac
            fi
        elif [[ $key =~ [0-9] ]]; then
            local field_type="${fields[$current_field]}"
            local max_len=${#field_type}
            local input_str="$key"

            # Show preview with underscores
            local preview="$input_str"
            for ((p=${#input_str}; p<max_len; p++)); do
                preview+="_"
            done
            values[$current_field]="$preview"
            draw_inline

            while [ ${#input_str} -lt $max_len ]; do
                IFS= read -rsn1 next_key
                if [[ $next_key =~ [0-9] ]]; then
                    input_str+="$next_key"
                    # Update preview
                    preview="$input_str"
                    for ((p=${#input_str}; p<max_len; p++)); do
                        preview+="_"
                    done
                    values[$current_field]="$preview"
                    draw_inline
                else
                    break
                fi
            done

            local input_num=$((10#$input_str))
            case "$field_type" in
                YYYY)
                    if [ $input_num -ge 1 ] && [ $input_num -le 9999 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                MM)
                    if [ $input_num -ge 1 ] && [ $input_num -le 12 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                DD)
                    if [ $input_num -ge 1 ] && [ $input_num -le 31 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                HH)
                    if [ $input_num -ge 0 ] && [ $input_num -le 23 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
                mm|SS)
                    if [ $input_num -ge 0 ] && [ $input_num -le 59 ]; then
                        values[$current_field]=$(printf "%0${max_len}d" $input_num)
                    fi
                    ;;
            esac
            draw_inline
        elif [[ $key == "" ]]; then
            if ! _date_is_valid; then
                inline_error "Nieprawidłowy format daty" "$label"
                draw_inline
                continue
            fi
            break
        fi
    done

    # Final redraw with all fields gray
    echo -ne "\r${CYAN}  ${label}: [${NC}"
    for ((idx=0; idx<num_fields; idx++)); do
        echo -ne "${DIM}${values[$idx]}${NC}"
        if [ $idx -lt ${#separators[@]} ]; then
            echo -ne "${DIM}${separators[$idx]}${NC}"
        fi
    done
    echo -ne "${CYAN}]${NC}"

    echo

    local result=""
    for ((idx=0; idx<num_fields; idx++)); do
        result+="${values[$idx]}"
        if [ $idx -lt ${#separators[@]} ]; then
            result+="${separators[$idx]}"
        fi
    done

    INPUT_VALUE="$result"
}
