#!/bin/bash

[[ -n "${_COMPONENT_TABLE_SH_LOADED:-}" ]] && return
_COMPONENT_TABLE_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_table() {
    local prompt="$1"
    local variable="$2"
    local fields_count="$3"
    shift 3

    prompt=$(interpolate "$prompt")

    print_step "$prompt"
    echo -e "${DIM}  (Use arrows to move, Enter to finish)${NC}"

    # Parse field definitions
    declare -a field_names=()
    declare -a field_inputs=()
    declare -a field_displays=()
    declare -a field_placeholders=()
    declare -a field_patterns=()
    declare -a field_on_errors=()
    declare -a field_min_lengths=()
    declare -a field_mins=()
    declare -a field_maxs=()
    declare -a field_steps=()
    declare -a field_defaults=()
    declare -a field_formats=()
    declare -a field_options=()
    declare -a field_actives=()
    declare -a field_inactives=()

    for ((field_idx = 0; field_idx < fields_count; field_idx++)); do
        field_names+=("$1")
        field_inputs+=("$2")
        field_displays+=("${3:-$1}")
        field_placeholders+=("${4:-}")
        field_patterns+=("${5:-}")
        field_on_errors+=("${6:-}")
        field_min_lengths+=("${7:-}")
        field_mins+=("${8:-}")
        field_maxs+=("${9:-}")
        field_steps+=("${10:-}")
        field_defaults+=("${11:-}")
        field_formats+=("${12:-}")
        field_options+=("${13:-}")
        field_actives+=("${14:-}")
        field_inactives+=("${15:-}")
        local field_validate_count="${16:-0}"
        shift 16

        if [ "$field_validate_count" -gt 0 ]; then
            local v_idx
            for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                shift 2
            done
        fi
    done

    # Storage
    local records_count=1
    local selected_row=0
    local selected_col=0
    declare -a col_widths=()
    local col_signature=""
    local col_changed=0
    local last_lines=0

    # Calculate widths (cell inner width + 2 for padding/brackets)
    recalc_column_widths() {
        local prev_signature="$col_signature"
        col_widths=()
        local i
        for ((i=0; i<fields_count; i++)); do
            local header="${field_displays[$i]}"
            local width=${#header}
            if [ $records_count -gt 0 ]; then
                local r
                for ((r=0; r<records_count; r++)); do
                    local field_name="${field_names[$i]}"
                    local value
                    value=$(state_get "__table_tmp_${r}_${field_name}")
                    local vlen=${#value}
                    if [ $vlen -gt $width ]; then
                        width=$vlen
                    fi
                done
            fi
            if [ $width -lt 3 ]; then
                width=3
            fi
            col_widths+=("$((width + 2))")
        done
        col_signature="$(printf "%s," "${col_widths[@]}")"
        if [ "$col_signature" != "$prev_signature" ]; then
            col_changed=1
        else
            col_changed=0
        fi
    }

    pad_cell() {
        local value="$1"
        local inner_width="$2"
        local out="${value:0:$inner_width}"
        local len=${#out}
        while [ $len -lt $inner_width ]; do
            out+=" "
            len=$((len + 1))
        done
        printf "%s" "$out"
    }

    draw_border() {
        local left="$1"
        local mid="$2"
        local right="$3"
        printf "  %s" "$left"
        local i
        for ((i=0; i<fields_count; i++)); do
            local width="${col_widths[$i]}"
            local j
            for ((j=0; j<width; j++)); do
                printf "─"
            done
            if [ $i -lt $((fields_count - 1)) ]; then
                printf "%s" "$mid"
            fi
        done
        printf "%s\n" "$right"
    }

    draw_row() {
        local row_idx="$1"
        printf "  │"
        local c
        for ((c=0; c<fields_count; c++)); do
            local field_name="${field_names[$c]}"
            local width="${col_widths[$c]}"
            local inner_width=$((width - 2))
            local value=""
            if [ $row_idx -ge 0 ]; then
                value=$(state_get "__table_tmp_${row_idx}_${field_name}")
            fi
            local cell=""
            if [ $row_idx -ge 0 ] && [ $row_idx -eq $selected_row ] && [ $c -eq $selected_col ]; then
                cell="[$(pad_cell "$value" "$inner_width")]"
            else
                cell=" $(pad_cell "$value" "$inner_width") "
            fi
            printf "%s│" "$cell"
        done
        echo
    }

    draw_header() {
        printf "  │"
        local i
        for ((i=0; i<fields_count; i++)); do
            local width="${col_widths[$i]}"
            local inner_width=$((width - 2))
            local header="${field_displays[$i]}"
            header=$(interpolate "$header")
            printf " ${CYAN}%s${NC} │" "$(pad_cell "$header" "$inner_width")"
        done
        echo
    }

    draw_button_bar() {
        tput el
        printf "  ${GREEN}[Enter]${NC} Finish\n"
    }

    draw_table() {
        recalc_column_widths

        # Top border
        draw_border "┌" "┬" "┐"
        # Header
        draw_header
        # Header separator
        draw_border "├" "┼" "┤"

        local r
        for ((r=0; r<records_count; r++)); do
            draw_row "$r"
        done

        # Bottom border
        draw_border "└" "┴" "┘"
        # Button bar
        draw_button_bar

        # Track line count for redraw
        last_lines=$((records_count + 5))
    }

    # Hide cursor
    cursor_hide
    trap 'cursor_show; exit 130' INT

    draw_table

    while true; do
        IFS= read -rsn1 key

        if [[ $key == $'\x1b' ]]; then
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true

            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'A') # Up
                        if [ $selected_row -gt 0 ]; then
                            selected_row=$((selected_row - 1))
                            draw_table
                        fi
                        ;;
                    'B') # Down
                        if [ $selected_row -lt $((records_count - 1)) ]; then
                            selected_row=$((selected_row + 1))
                            draw_table
                        fi
                        ;;
                    'C') # Right
                        if [ $fields_count -gt 0 ] && [ $selected_col -lt $((fields_count - 1)) ]; then
                            selected_col=$((selected_col + 1))
                            draw_table
                        fi
                        ;;
                    'D') # Left
                        if [ $fields_count -gt 0 ] && [ $selected_col -gt 0 ]; then
                            selected_col=$((selected_col - 1))
                            draw_table
                        fi
                        ;;
                esac
            fi
        elif [[ $key == "" ]]; then
            break
        fi
    done

    trap - INT
    cursor_show

    # Save records to state
    state_set "${variable}_count" "$records_count"
    local r f
    for ((r=0; r<records_count; r++)); do
        for ((f=0; f<fields_count; f++)); do
            local field_name="${field_names[$f]}"
            local value
            value=$(state_get "__table_tmp_${r}_${field_name}")
            state_set "${variable}_${r}_${field_name}" "$value"
            state_delete "__table_tmp_${r}_${field_name}"
        done
    done

    print_success "Added $records_count row(s)"
}