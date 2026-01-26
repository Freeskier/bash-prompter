#!/bin/bash

[[ -n "${_COMPONENT_RECORDS_SH_LOADED:-}" ]] && return
_COMPONENT_RECORDS_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_records() {
    local prompt="$1"
    local variable="$2"
    local fields_count="$3"
    shift 3

    prompt=$(interpolate "$prompt")

    print_step "$prompt"
    echo -e "${DIM}  (↑↓ select, 'i' insert, 'd' delete, Enter finish)${NC}"

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
    declare -A field_validate_counts=()
    declare -A field_validate_patterns=()
    declare -A field_validate_errors=()

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

        field_validate_counts["$field_idx"]="$field_validate_count"
        if [ "$field_validate_count" -gt 0 ]; then
            field_patterns[$field_idx]=""
            field_on_errors[$field_idx]=""
            local v_idx
            for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                field_validate_patterns["$field_idx,$v_idx"]="$1"
                field_validate_errors["$field_idx,$v_idx"]="$2"
                shift 2
            done
        fi
    done

    # Storage
    local records_count=0
    local selected_record=0
    declare -a col_widths=()
    local col_gap=2
    local col_signature=""
    local col_changed=0

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
            width=$((width + col_gap))
            col_widths+=("$width")
        done
        col_signature="$(printf "%s," "${col_widths[@]}")"
        if [ "$col_signature" != "$prev_signature" ]; then
            col_changed=1
        else
            col_changed=0
        fi
    }

    # Hide cursor
    cursor_hide
    trap 'cursor_show; exit 130' INT

    # Draw main screen (ONCE - headers are static!)
    draw_main_screen() {
        recalc_column_widths
        # Column headers (STATIC - never redrawn)
        tput el
        printf "     "
        local i
        for ((i=0; i<fields_count; i++)); do
            local header="${field_displays[$i]}"
            local width="${col_widths[$i]}"
            printf "${CYAN}%-*s${NC}" "$width" "$header"
        done
        echo ""

        # Records or empty message
        if [ $records_count -eq 0 ]; then
            tput el
            printf "     ${DIM}(no records)${NC}\n"
        else
            local r f
            for ((r=0; r<records_count; r++)); do
                tput el
                if [ $r -eq $selected_record ]; then
                    printf "  ${YELLOW}>${NC}  "
                else
                    printf "     "
                fi

                for ((f=0; f<fields_count; f++)); do
                    local field_name="${field_names[$f]}"
                    local value=$(state_get "__table_tmp_${r}_${field_name}")
                    local width="${col_widths[$f]}"
                    if [ $r -eq $selected_record ]; then
                        printf "${BOLD}%-*s${NC}" "$width" "$value"
                    else
                        printf "%-*s" "$width" "$value"
                    fi
                done
                echo ""
            done
        fi

        # Separator
        tput el
        local total_width=0
        for ((i=0; i<fields_count; i++)); do
            total_width=$((total_width + col_widths[$i]))
        done
        local sep_len=$((total_width + 5))
        printf "  ${DIM}"
        for ((i=0; i<sep_len; i++)); do
            printf "─"
        done
        printf "${NC}\n"

        # Button bar
        draw_button_bar
    }

    # Draw button bar only
    draw_button_bar() {
        tput el
        printf "  ${DIM}[i]${NC} New  ${DIM}[d]${NC} Delete"
        printf "%*s" 10 ""
        printf "${GREEN}[Enter]${NC} Finish\n"
    }

    # Redraw table in place (WITHOUT headers - they are static!)
    redraw_table() {
        recalc_column_widths
        if [ $col_changed -eq 1 ]; then
            local total_lines=$(( (records_count > 0 ? records_count : 1) + 2 ))
            tput cuu $total_lines
            draw_main_screen
            return
        fi
        # Calculate lines to go up: separator + button bar + records/empty
        local lines_up=2  # separator + button bar

        if [ $records_count -eq 0 ]; then
            lines_up=$((lines_up + 1))  # (no records) line
        else
            lines_up=$((lines_up + records_count))
        fi

        # Go up (skipping static headers!)
        tput cuu $lines_up

        # Redraw records/empty
        if [ $records_count -eq 0 ]; then
            tput el
            printf "     ${DIM}(no records)${NC}\n"
        else
            local r f
            for ((r=0; r<records_count; r++)); do
                tput el
                if [ $r -eq $selected_record ]; then
                    printf "  ${YELLOW}>${NC}  "
                else
                    printf "     "
                fi

                for ((f=0; f<fields_count; f++)); do
                    local field_name="${field_names[$f]}"
                    local value=$(state_get "__table_tmp_${r}_${field_name}")
                    local width="${col_widths[$f]}"
                    if [ $r -eq $selected_record ]; then
                        printf "${BOLD}%-*s${NC}" "$width" "$value"
                    else
                        printf "%-*s" "$width" "$value"
                    fi
                done
                echo ""
            done
        fi

        # Redraw separator
        tput el
        local total_width=0
        for ((i=0; i<fields_count; i++)); do
            total_width=$((total_width + col_widths[$i]))
        done
        local sep_len=$((total_width + 5))
        printf "  ${DIM}"
        for ((i=0; i<sep_len; i++)); do
            printf "─"
        done
        printf "${NC}\n"

        # Redraw button bar
        draw_button_bar
    }

    # Initial draw
    draw_main_screen

    # Main loop
    while true; do
        IFS= read -rsn1 key

        # NAVIGATION MODE
        if [[ $key == $'\x1b' ]]; then
            # Arrow keys
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true

            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'A') # Up
                        if [ $records_count -gt 0 ] && [ $selected_record -gt 0 ]; then
                            selected_record=$((selected_record - 1))
                            redraw_table
                        fi
                        ;;
                    'B') # Down
                        if [ $records_count -gt 0 ] && [ $selected_record -lt $((records_count - 1)) ]; then
                            selected_record=$((selected_record + 1))
                            redraw_table
                        fi
                        ;;
                esac
            fi
        elif [[ $key == "i" ]] || [[ $key == "I" ]]; then
            # Insert new record - use input_collector for each field
            # Go up to replace button bar line
            tput cuu 1
            tput el

            local all_valid=1
            local field_idx
            for ((field_idx=0; field_idx<fields_count; field_idx++)); do
                local field_name="${field_names[$field_idx]}"
                local field_input="${field_inputs[$field_idx]}"
                local field_display="${field_displays[$field_idx]}"
                local field_placeholder="${field_placeholders[$field_idx]}"
                local field_min_length="${field_min_lengths[$field_idx]}"
                local field_min="${field_mins[$field_idx]}"
                local field_max="${field_maxs[$field_idx]}"
                local field_step="${field_steps[$field_idx]}"
                local field_default="${field_defaults[$field_idx]}"
                local field_format="${field_formats[$field_idx]}"
                local field_option_list="${field_options[$field_idx]}"
                local field_active="${field_actives[$field_idx]}"
                local field_inactive="${field_inactives[$field_idx]}"

                field_display=$(interpolate "$field_display")
                field_placeholder=$(interpolate "$field_placeholder")
                field_default=$(interpolate "$field_default")

                # Use input_collector with no-echo mode for inline editing
                local -a validate_args=()
                local v_count="${field_validate_counts[$field_idx]:-0}"
                if [ "$v_count" -gt 0 ]; then
                    local v_idx
                    for ((v_idx=0; v_idx<v_count; v_idx++)); do
                        local v_pattern="${field_validate_patterns[$field_idx,$v_idx]}"
                        local v_error="${field_validate_errors[$field_idx,$v_idx]}"
                        v_pattern=$(interpolate "$v_pattern")
                        v_error=$(interpolate "$v_error")
                        validate_args+=(--validate "$v_pattern" "$v_error")
                    done
                fi

                export INPUT_INLINE_ACTIVE=1
                export INPUT_INLINE_KEEP_CURSOR=1
                collect_and_validate "$field_display" "$field_input" \
                    --placeholder "$field_placeholder" \
                    --options "$field_option_list" \
                    --active "$field_active" \
                    --inactive "$field_inactive" \
                    --min-length "$field_min_length" \
                    --min "$field_min" \
                    --max "$field_max" \
                    --step "$field_step" \
                    --default "$field_default" \
                    --format "$field_format" \
                    "${validate_args[@]}"
                unset INPUT_INLINE_ACTIVE
                unset INPUT_INLINE_KEEP_CURSOR

                local input_value="$INPUT_VALUE"

                # Store in temporary state
                state_set "__table_tmp_${records_count}_${field_name}" "$input_value"
                # Clear input line so next field overwrites in place
                tput cuu 1
                tput el
            done
            # Clear the input line before redraw
            tput el

            # Add record
            local was_empty=$((records_count == 0))
            records_count=$((records_count + 1))
            selected_record=$((records_count - 1))

            # Physical line management (like v1 template.sh):
            # - First record: replaces "(no records)" - no new line needed
            # - Subsequent records: need new physical line
            # if [ $was_empty -eq 0 ]; then
            #     #echo ""  # Insert new physical line to push history up
            # fi
            echo ""
            #tput cuu 1
            redraw_table

        elif [[ $key == "d" ]] || [[ $key == "D" ]]; then
            # Delete selected record
            if [ $records_count -gt 0 ]; then
                local r f
                # Shift all records after deleted one
                for ((r=selected_record; r<records_count-1; r++)); do
                    for ((f=0; f<fields_count; f++)); do
                        local field_name="${field_names[$f]}"
                        local next_value=$(state_get "__table_tmp_$((r+1))_${field_name}")
                        state_set "__table_tmp_${r}_${field_name}" "$next_value"
                    done
                done

                # Delete last record
                for ((f=0; f<fields_count; f++)); do
                    local field_name="${field_names[$f]}"
                    state_delete "__table_tmp_$((records_count-1))_${field_name}"
                done

                records_count=$((records_count - 1))

                if [ $selected_record -ge $records_count ] && [ $records_count -gt 0 ]; then
                    selected_record=$((records_count - 1))
                fi

                # Physical line management:
                # - If still have records: delete one physical line
                # - If now empty: will be handled by redraw (replaces last record with "(no records)")
                if [ $records_count -gt 0 ]; then
                    tput cuu 1
                    tput dl1  # Delete one physical line from terminal
                fi

                redraw_table
            fi

        elif [[ $key == "" ]]; then
            # Enter - finish
            break
        fi
    done

    # Restore cursor
    trap - INT
    cursor_show

    # Save records to state in flat indexed format
    state_set "${variable}_count" "$records_count"

    local r f
    for ((r=0; r<records_count; r++)); do
        for ((f=0; f<fields_count; f++)); do
            local field_name="${field_names[$f]}"
            local value=$(state_get "__table_tmp_${r}_${field_name}")
            state_set "${variable}_${r}_${field_name}" "$value"

            # Cleanup temp state
            state_delete "__table_tmp_${r}_${field_name}"
        done
    done

    print_success "Added $records_count record(s)"
}
