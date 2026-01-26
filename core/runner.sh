#!/bin/bash

source "$(dirname "${BASH_SOURCE[0]}")/parser.sh"
source "$(dirname "${BASH_SOURCE[0]}")/state.sh"
source "$(dirname "${BASH_SOURCE[0]}")/conditions.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/spinner.sh"

run_steps() {
    local yaml_file="$1"

    if ! yaml_parse_file "$yaml_file"; then
        print_error "Failed to parse YAML file"
        return 1
    fi

    local step_count=$(yaml_get_step_count)

    _run_steps_with_prefix "$step_count" "steps"
}

# Execute steps with a given prefix (for recursive execution)
# Args: $1 - step count, $2 - prefix
_run_steps_with_prefix() {
    local step_count="$1"
    local prefix="$2"
    local i

    for ((i = 0; i < step_count; i++)); do
        local step_type=$(yaml_get_step_type "$i" "$prefix")
        local step_subtype=$(yaml_get_step_subtype "$i" "$prefix")

        case "$step_type" in
            input)
                _run_input_step "$i" "$step_subtype" "$prefix"
                ;;
            output)
                _run_output_step "$i" "$step_subtype" "$prefix"
                ;;
            command)
                _run_command_step "$i" "$prefix"
                ;;
            script)
                _run_script_step "$i" "$prefix"
                ;;
            pipeline)
                _run_pipeline_step "$i" "$prefix"
                ;;
            component)
                _run_component_step "$i" "$step_subtype" "$prefix"
                ;;
            condition)
                _run_condition_step "$i" "$prefix"
                ;;
            loop)
                _run_loop_step "$i" "$prefix"
                ;;
            *)
                print_error "Unknown step type: $step_type"
                return 1
                ;;
        esac
    done
}

_apply_validate_rules() {
    local value="$1"
    shift
    while [ $# -gt 0 ]; do
        local pattern="$1"
        local error="$2"
        if [ -n "$pattern" ] && [[ ! "$value" =~ $pattern ]]; then
            if [ -n "$error" ]; then
                print_error "$error"
            else
                print_error "Validation failed"
            fi
            return 1
        fi
        shift 2
    done
    return 0
}

_run_condition_step() {
    local step_idx="$1"
    local prefix="$2"

    local condition=$(yaml_get "${prefix}_${step_idx}_if")
    local nested_steps_count=$(yaml_get "${prefix}_${step_idx}_steps_count")

    if [ -z "$condition" ]; then
        print_error "Condition block missing 'if' expression"
        return 1
    fi

    # Evaluate the condition
    if evaluate_condition "$condition"; then
        # Condition is true - execute nested steps
        local nested_prefix="${prefix}_${step_idx}_steps"
        _run_steps_with_prefix "$nested_steps_count" "$nested_prefix"
    fi
    # If condition is false, skip the nested steps
}

_run_loop_step() {
    local step_idx="$1"
    local prefix="$2"

    local items_raw=$(yaml_get "${prefix}_${step_idx}_items")
    local loop_var=$(yaml_get "${prefix}_${step_idx}_variable")
    local separator=$(yaml_get "${prefix}_${step_idx}_separator" ",")
    local nested_steps_count=$(yaml_get "${prefix}_${step_idx}_steps_count")

    if [ -z "$loop_var" ]; then
        print_error "Loop block missing 'variable' field"
        return 1
    fi

    # Build items array
    local items_array=()

    # Check if items is a variable from state (has _count)
    if [ -n "$items_raw" ] && state_has "${items_raw}_count"; then
        # It's a flat indexed array from state
        local count=$(state_get "${items_raw}_count")
        for ((idx=0; idx<count; idx++)); do
            items_array+=("$idx")  # Store indices, not values
        done
        local items_source="$items_raw"
        local is_flat_array=true
    elif [ -n "$items_raw" ]; then
        # It's a simple string variable - split by separator
        local items_string=$(state_get "$items_raw")
        if [ -n "$items_string" ]; then
            IFS="$separator" read -ra items_array <<< "$items_string"
        fi
        local is_flat_array=false
    else
        # Static list from YAML (items_0, items_1, ...)
        local items_count=$(yaml_get "${prefix}_${step_idx}_items_count" "0")
        for ((idx=0; idx<items_count; idx++)); do
            items_array+=("$(yaml_get "${prefix}_${step_idx}_items_${idx}")")
        done
        local is_flat_array=false
    fi

    # Execute loop
    local total=${#items_array[@]}

    # Set loop context for interpolation
    state_set "__current_loop_var" "$loop_var"
    state_set "__current_loop_parent" "${items_source:-}"

    for ((idx=0; idx<total; idx++)); do
        # Set loop context variables
        state_set "loop_index0" "$idx"
        state_set "loop_index" "$((idx + 1))"
        state_set "loop_total" "$total"
        state_set "loop_first" "$([[ $idx -eq 0 ]] && echo "true" || echo "false")"
        state_set "loop_last" "$([[ $idx -eq $((total-1)) ]] && echo "true" || echo "false")"
        state_set "__current_loop_idx" "$idx"

        if [ "$is_flat_array" = true ]; then
            # Flat indexed array - copy all fields to loop_var prefix
            local item_idx="${items_array[$idx]}"

            # First, get the base value (for simple arrays like multiselect)
            local base_value=$(state_get "${items_source}_${item_idx}")
            if [ -n "$base_value" ]; then
                state_set "$loop_var" "$base_value"
            fi

            # Then find all nested fields for this item (for array of objects)
            local found_fields=false
            for key in $(state_keys); do
                if [[ "$key" =~ ^${items_source}_${item_idx}_(.+)$ ]]; then
                    local field_name="${BASH_REMATCH[1]}"
                    local value=$(state_get "$key")
                    state_set "${loop_var}_${field_name}" "$value"
                    found_fields=true
                fi
            done
        else
            # Simple value
            local item="${items_array[$idx]}"
            state_set "$loop_var" "$item"
        fi

        # Execute nested steps
        local nested_prefix="${prefix}_${step_idx}_steps"
        _run_steps_with_prefix "$nested_steps_count" "$nested_prefix"
    done

    # Cleanup loop variables
    state_delete "$loop_var"
    state_delete "loop_index0"
    state_delete "loop_index"
    state_delete "loop_total"
    state_delete "loop_first"
    state_delete "loop_last"
    state_delete "__current_loop_var"
    state_delete "__current_loop_parent"
    state_delete "__current_loop_idx"

    # Cleanup loop_var fields if flat array
    if [ "$is_flat_array" = true ]; then
        for key in $(state_keys); do
            if [[ "$key" =~ ^${loop_var}_.+ ]]; then
                state_delete "$key"
            fi
        done
    fi
}

_run_input_step() {
    local step_idx="$1"
    local input_type="$2"
    local prefix="${3:-steps}"

    local prompt=$(yaml_get "${prefix}_${step_idx}_prompt")
    local variable=$(yaml_get "${prefix}_${step_idx}_variable")
    local placeholder=$(yaml_get "${prefix}_${step_idx}_placeholder")
    local default=$(yaml_get "${prefix}_${step_idx}_default")
    local validate_count=$(yaml_get "${prefix}_${step_idx}_validate_count" "0")

    local -a validate_args=()
    if [ "$validate_count" -gt 0 ]; then
        local v_idx
        for ((v_idx=0; v_idx<validate_count; v_idx++)); do
            local v_pattern=$(yaml_get "${prefix}_${step_idx}_validate_${v_idx}_pattern")
            local v_error=$(yaml_get "${prefix}_${step_idx}_validate_${v_idx}_error")
            v_pattern=$(interpolate "$v_pattern")
            v_error=$(interpolate "$v_error")
            validate_args+=("$v_pattern" "$v_error")
        done
    fi

    # Interpolate variable name for dynamic variables in loops
    variable=$(interpolate "$variable")

    case "$input_type" in
        text)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/text.sh"
            input_text "$prompt" "$variable" "$placeholder" "$default" "${validate_args[@]}"
            ;;
        select)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/select.sh"
            local options_count=$(yaml_get_step_options_count "$step_idx" "$prefix")
            if [ -n "$options_count" ] && [ "$options_count" -gt 0 ]; then
                local opts=()
                local opt_idx
                for ((opt_idx=0; opt_idx<options_count; opt_idx++)); do
                    opts+=("$(yaml_get "${prefix}_${step_idx}_options_${opt_idx}")")
                done
                if [ ${#validate_args[@]} -gt 0 ]; then
                    while true; do
                        input_select "$prompt" "$variable" "${opts[@]}"
                        local val="$(state_get "$variable")"
                        if _apply_validate_rules "$val" "${validate_args[@]}"; then
                            break
                        fi
                    done
                else
                    input_select "$prompt" "$variable" "${opts[@]}"
                fi
            else
                local options=$(yaml_get "${prefix}_${step_idx}_options")
                IFS=',' read -ra opts <<< "$options"
                local j
                for j in "${!opts[@]}"; do
                    opts[$j]=$(echo "${opts[$j]}" | xargs)
                done
                if [ ${#validate_args[@]} -gt 0 ]; then
                    while true; do
                        input_select "$prompt" "$variable" "${opts[@]}"
                        local val="$(state_get "$variable")"
                        if _apply_validate_rules "$val" "${validate_args[@]}"; then
                            break
                        fi
                    done
                else
                    input_select "$prompt" "$variable" "${opts[@]}"
                fi
            fi
            ;;
        password)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/password.sh"
            input_password "$prompt" "$variable" "$placeholder" "${validate_args[@]}"
            ;;
        email)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/email.sh"
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_email "$prompt" "$variable" "$placeholder" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_email "$prompt" "$variable" "$placeholder" "$default"
            fi
            ;;
        url)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/url.sh"
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_url "$prompt" "$variable" "$placeholder" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_url "$prompt" "$variable" "$placeholder" "$default"
            fi
            ;;
        ip)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/ip.sh"
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_ip "$prompt" "$variable" "$placeholder" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_ip "$prompt" "$variable" "$placeholder" "$default"
            fi
            ;;
        number)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/number.sh"
            local min=$(yaml_get "${prefix}_${step_idx}_min")
            local max=$(yaml_get "${prefix}_${step_idx}_max")
            local step=$(yaml_get "${prefix}_${step_idx}_step")
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            input_number "$prompt" "$variable" "$min" "$max" "$step" "$default"
            ;;
        slider)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/slider.sh"
            local min=$(yaml_get "${prefix}_${step_idx}_min")
            local max=$(yaml_get "${prefix}_${step_idx}_max")
            local step=$(yaml_get "${prefix}_${step_idx}_step")
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            local unit=$(yaml_get "${prefix}_${step_idx}_unit")
            input_slider "$prompt" "$variable" "$min" "$max" "$step" "$default" "$unit" "${validate_args[@]}"
            ;;
        date)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/date.sh"
            local format=$(yaml_get "${prefix}_${step_idx}_format")
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_date "$prompt" "$variable" "$format" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_date "$prompt" "$variable" "$format" "$default"
            fi
            ;;
        color)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/color.sh"
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_color "$prompt" "$variable" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_color "$prompt" "$variable" "$default"
            fi
            ;;
        list)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/list.sh"
            local separator=$(yaml_get "${prefix}_${step_idx}_separator" ",")
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_list "$prompt" "$variable" "$separator" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_list "$prompt" "$variable" "$separator" "$default"
            fi
            ;;
        toggle)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/toggle.sh"
            local active=$(yaml_get "${prefix}_${step_idx}_active" "active")
            local inactive=$(yaml_get "${prefix}_${step_idx}_inactive" "inactive")
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_toggle "$prompt" "$variable" "$active" "$inactive" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_toggle "$prompt" "$variable" "$active" "$inactive" "$default"
            fi
            ;;
        bool)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/checkbox.sh"
            local default=$(yaml_get "${prefix}_${step_idx}_default" "false")
            if [ ${#validate_args[@]} -gt 0 ]; then
                while true; do
                    input_bool "$prompt" "$variable" "$placeholder" "$default"
                    local val="$(state_get "$variable")"
                    if _apply_validate_rules "$val" "${validate_args[@]}"; then
                        break
                    fi
                done
            else
                input_bool "$prompt" "$variable" "$placeholder" "$default"
            fi
            ;;
        *)
            print_error "Unknown input type: $input_type"
            return 1
            ;;
    esac
}

_run_output_step() {
    local step_idx="$1"
    local output_type="$2"
    local prefix="${3:-steps}"

    local value=$(yaml_get "${prefix}_${step_idx}_value")

    case "$output_type" in
        info)
            source "$(dirname "${BASH_SOURCE[0]}")/../outputs/info.sh"
            output_info "$value"
            ;;
        *)
            print_error "Unknown output type: $output_type"
            return 1
            ;;
    esac
}

_run_command_step() {
    local step_idx="$1"
    local prefix="${2:-steps}"

    local command=$(yaml_get "${prefix}_${step_idx}_command")
    local variable=$(yaml_get "${prefix}_${step_idx}_variable")
    local message=$(yaml_get "${prefix}_${step_idx}_message")
    local spinner=$(yaml_get "${prefix}_${step_idx}_spinner" "false")
    local show_output=$(yaml_get "${prefix}_${step_idx}_show_output" "false")
    local log_lines=$(yaml_get "${prefix}_${step_idx}_log_lines" "3")

    # Interpolate both command and variable name
    command=$(interpolate "$command")
    variable=$(interpolate "$variable")
    message=$(interpolate "$message")

    local label="${message:-$command}"
    local exit_code=0

    if [ "$spinner" = "true" ]; then
        local tmp_out
        tmp_out="$(mktemp)"
        run_with_spinner "$label" "$command" "$YELLOW" "$show_output" "$log_lines" "$tmp_out"
        exit_code=$?

        if [ $exit_code -eq 0 ] && [ -n "$variable" ]; then
            state_set "$variable" "$(cat "$tmp_out")"
        fi
        rm -f "$tmp_out"
        return $exit_code
    fi

    if [ "$show_output" = "true" ]; then
        local tmp_out
        tmp_out="$(mktemp)"
        bash -c "$command" 2>&1 | tee "$tmp_out"
        exit_code=${PIPESTATUS[0]}

        if [ $exit_code -eq 0 ]; then
            if [ -n "$variable" ]; then
                state_set "$variable" "$(cat "$tmp_out")"
            fi
        else
            print_error "Command failed with exit code $exit_code"
            print_dim "$(tail -n 5 "$tmp_out")"
            rm -f "$tmp_out"
            return $exit_code
        fi
        rm -f "$tmp_out"
        return 0
    fi

    local output
    output=$(eval "$command" 2>&1)
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        if [ -n "$variable" ]; then
            state_set "$variable" "$output"
        fi
    else
        print_error "Command failed with exit code $exit_code"
        print_dim "$output"
        return $exit_code
    fi
}

_run_pipeline_step() {
    local step_idx="$1"
    local prefix="${2:-steps}"

    local message=$(yaml_get "${prefix}_${step_idx}_message")
    local show_output=$(yaml_get "${prefix}_${step_idx}_show_output" "true")
    local log_lines=$(yaml_get "${prefix}_${step_idx}_log_lines" "3")
    local commands_count=$(yaml_get "${prefix}_${step_idx}_commands_count" "0")

    message=$(interpolate "$message")

    if [ "$commands_count" -le 0 ]; then
        print_error "Pipeline block missing 'commands' list"
        return 1
    fi

    if [ -n "$message" ]; then
        print_step "$message"
    fi

    local idx
    for ((idx=0; idx<commands_count; idx++)); do
        local name=$(yaml_get "${prefix}_${step_idx}_commands_${idx}_name")
        local run=$(yaml_get "${prefix}_${step_idx}_commands_${idx}_run")
        local variable=$(yaml_get "${prefix}_${step_idx}_commands_${idx}_variable")
        local item_show_output=$(yaml_get "${prefix}_${step_idx}_commands_${idx}_show_output")
        local item_log_lines=$(yaml_get "${prefix}_${step_idx}_commands_${idx}_log_lines")

        name=$(interpolate "$name")
        run=$(interpolate "$run")
        variable=$(interpolate "$variable")

        if [ -z "$run" ]; then
            print_error "Pipeline command missing 'run'"
            return 1
        fi

        if [ -z "$item_show_output" ]; then
            item_show_output="$show_output"
        fi
        if [ -z "$item_log_lines" ]; then
            item_log_lines="$log_lines"
        fi

        local step_num=$((idx + 1))
        local label="[Step ${step_num}/${commands_count}] ${name:-$run}"

        local tmp_out
        tmp_out="$(mktemp)"
        run_with_spinner_silent "$label" "$run" "$YELLOW" "$item_show_output" "$item_log_lines" "$tmp_out"
        local exit_code=$?

        if [ $exit_code -eq 0 ]; then
            if [ -n "$variable" ]; then
                state_set "$variable" "$(cat "$tmp_out")"
            fi
            print_success "${label} Done!"
        else
            print_error "${label} Failed!"
            if [ -s "$tmp_out" ]; then
                print_dim "Last output:"
                tail -n 5 "$tmp_out" 2>/dev/null | sed 's/^/  /'
            fi
            rm -f "$tmp_out"
            return $exit_code
        fi

        rm -f "$tmp_out"
    done
}

_run_script_step() {
    local step_idx="$1"
    local prefix="${2:-steps}"

    local script_path=$(yaml_get "${prefix}_${step_idx}_script")
    local args=$(yaml_get "${prefix}_${step_idx}_args")
    local variable=$(yaml_get "${prefix}_${step_idx}_variable")
    local workdir=$(yaml_get "${prefix}_${step_idx}_workdir")

    # Interpolate script path, args, workdir, and variable name
    script_path=$(interpolate "$script_path")
    args=$(interpolate "$args")
    variable=$(interpolate "$variable")
    workdir=$(interpolate "$workdir")

    if [ -z "$script_path" ]; then
        print_error "Script step missing 'script' path"
        return 1
    fi

    if [ ! -f "$script_path" ]; then
        print_error "Script not found: $script_path"
        return 1
    fi

    # Build args array (supports quoted args)
    local -a arg_array=()
    if [ -n "$args" ]; then
        eval "arg_array=($args)"
    fi

    # Export state vars into the shell so scripts can use them directly
    local -A _sourced_denylist=(
        [PATH]=1 [HOME]=1 [PWD]=1 [OLDPWD]=1 [SHELL]=1
        [TERM]=1 [USER]=1 [LOGNAME]=1 [SHLVL]=1 [LANG]=1 [IFS]=1
    )
    for key in $(state_keys); do
        if [[ "$key" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] && [[ -z "${_sourced_denylist[$key]:-}" ]]; then
            printf -v "$key" '%s' "$(state_get "$key")"
            export "$key"
        fi
    done
    export SONDA_STATE_KEYS="$(state_keys)"

    # Snapshot exported vars before sourcing
    local -A _exports_before=()
    while IFS= read -r line; do
        if [[ "$line" =~ ^declare\ -x\ ([A-Za-z_][A-Za-z0-9_]*)=(.*)$ ]]; then
            local k="${BASH_REMATCH[1]}"
            local v="${BASH_REMATCH[2]}"
            v=$(printf '%s' "$v" | sed 's/^"//; s/"$//')
            _exports_before["$k"]="$v"
        elif [[ "$line" =~ ^declare\ -x\ ([A-Za-z_][A-Za-z0-9_]*)$ ]]; then
            local k="${BASH_REMATCH[1]}"
            _exports_before["$k"]=""
        fi
    done < <(export -p)

    # Source in current shell so assignments update state naturally
    local saved_cwd="$PWD"
    if [ -n "$workdir" ]; then
        cd "$workdir" || return 1
    fi

    local exit_code=0
    if [ -n "$variable" ]; then
        local tmp_out
        tmp_out="$(mktemp)"
        exec 3>&1 4>&2
        source "$script_path" "${arg_array[@]}" >"$tmp_out" 2>&1
        exit_code=$?
        exec 1>&3 2>&4 3>&- 4>&-

        local output=""
        if [ -f "$tmp_out" ]; then
            output="$(cat "$tmp_out")"
            rm -f "$tmp_out"
        fi

        if [ $exit_code -eq 0 ]; then
            state_set "$variable" "$output"
        else
            print_error "Script failed with exit code $exit_code"
            print_dim "$output"
            cd "$saved_cwd" || true
            return $exit_code
        fi
    else
        source "$script_path" "${arg_array[@]}"
        exit_code=$?
        if [ $exit_code -ne 0 ]; then
            print_error "Script failed with exit code $exit_code"
            cd "$saved_cwd" || true
            return $exit_code
        fi
    fi

    # Capture exported vars after sourcing and sync to state (only changed or new)
    while IFS= read -r line; do
        if [[ "$line" =~ ^declare\ -x\ ([A-Za-z_][A-Za-z0-9_]*)=(.*)$ ]]; then
            local k="${BASH_REMATCH[1]}"
            local v="${BASH_REMATCH[2]}"
            v=$(printf '%s' "$v" | sed 's/^"//; s/"$//')
            if [[ -z "${_sourced_denylist[$k]:-}" ]] && [[ "$k" != _* ]]; then
                if [[ -z "${_exports_before[$k]+x}" ]] || [[ "${_exports_before[$k]}" != "$v" ]]; then
                    state_set "$k" "$v"
                fi
            fi
        elif [[ "$line" =~ ^declare\ -x\ ([A-Za-z_][A-Za-z0-9_]*)$ ]]; then
            local k="${BASH_REMATCH[1]}"
            if [[ -z "${_sourced_denylist[$k]:-}" ]] && [[ "$k" != _* ]]; then
                if [[ -z "${_exports_before[$k]+x}" ]]; then
                    state_set "$k" ""
                fi
            fi
        fi
    done < <(export -p)

    cd "$saved_cwd" || true
}

_run_component_step() {
    local step_idx="$1"
    local component_type="$2"
    local prefix="${3:-steps}"

    local prompt=$(yaml_get "${prefix}_${step_idx}_prompt")
    local variable=$(yaml_get "${prefix}_${step_idx}_variable")

    # Interpolate variable name for dynamic variables in loops
    variable=$(interpolate "$variable")

    case "$component_type" in
        object)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/object.sh"

            local fields_count=$(yaml_get_step_fields_count "$step_idx" "$prefix")
            local field_args=()
            local field_idx

            for ((field_idx = 0; field_idx < fields_count; field_idx++)); do
                local field_name=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_variable")
                if [ -z "$field_name" ]; then
                    print_error "Object field missing 'variable' key"
                    return 1
                fi
                local field_input=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_input")
                local field_display=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_display")
                local field_placeholder=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_placeholder")
                local field_min_length=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min_length")
                local field_min=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min")
                local field_max=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_max")
                local field_step=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_step")
                local field_default=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_default")
                local field_format=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_format")
                local field_options=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options")
                local field_active=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_active")
                local field_inactive=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_inactive")
                local field_validate_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_count")
                field_validate_count="${field_validate_count:-0}"
                local field_options_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_count")

                if [ -n "$field_options_count" ] && [ "$field_options_count" -gt 0 ]; then
                    local opts=()
                    local opt_idx
                    for ((opt_idx=0; opt_idx<field_options_count; opt_idx++)); do
                        opts+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_${opt_idx}")")
                    done
                    field_options=$(IFS=','; echo "${opts[*]}")
                fi

                field_args+=("$field_name" "$field_input" "$field_display" "$field_placeholder" "" "" "$field_min_length" "$field_min" "$field_max" "$field_step" "$field_default" "$field_format" "$field_options" "$field_active" "$field_inactive" "$field_validate_count")
                if [ "$field_validate_count" -gt 0 ]; then
                    local v_idx
                    for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_pattern")")
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_error")")
                    done
                fi
            done

            component_object "$prompt" "$variable" "$fields_count" "${field_args[@]}"
            ;;
        select)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/select.sh"
            local options_count=$(yaml_get_step_options_count "$step_idx" "$prefix")
            if [ -n "$options_count" ] && [ "$options_count" -gt 0 ]; then
                local opts=()
                local opt_idx
                for ((opt_idx=0; opt_idx<options_count; opt_idx++)); do
                    opts+=("$(yaml_get "${prefix}_${step_idx}_options_${opt_idx}")")
                done
                component_select "$prompt" "$variable" "${opts[@]}"
            else
                local options=$(yaml_get "${prefix}_${step_idx}_options")
                IFS=',' read -ra opts <<< "$options"
                local j
                for j in "${!opts[@]}"; do
                    opts[$j]=$(echo "${opts[$j]}" | xargs)
                done
                component_select "$prompt" "$variable" "${opts[@]}"
            fi
            ;;
        multiselect)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/multiselect.sh"
            local options_count=$(yaml_get_step_options_count "$step_idx" "$prefix")
            if [ -n "$options_count" ] && [ "$options_count" -gt 0 ]; then
                local opts=()
                local opt_idx
                for ((opt_idx=0; opt_idx<options_count; opt_idx++)); do
                    opts+=("$(yaml_get "${prefix}_${step_idx}_options_${opt_idx}")")
                done
                component_multiselect "$prompt" "$variable" "${opts[@]}"
            else
                local options=$(yaml_get "${prefix}_${step_idx}_options")
                IFS=',' read -ra opts <<< "$options"
                local j
                for j in "${!opts[@]}"; do
                    opts[$j]=$(echo "${opts[$j]}" | xargs)
                done
                component_multiselect "$prompt" "$variable" "${opts[@]}"
            fi
            ;;
        radio_group)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/radio_group.sh"
            local default=$(yaml_get "${prefix}_${step_idx}_default")
            local options_count=$(yaml_get_step_options_count "$step_idx" "$prefix")
            if [ -n "$options_count" ] && [ "$options_count" -gt 0 ]; then
                local opts=()
                local opt_idx
                for ((opt_idx=0; opt_idx<options_count; opt_idx++)); do
                    opts+=("$(yaml_get "${prefix}_${step_idx}_options_${opt_idx}")")
                done
                component_radio_group "$prompt" "$variable" "$default" "${opts[@]}"
            else
                local options=$(yaml_get "${prefix}_${step_idx}_options")
                IFS=',' read -ra opts <<< "$options"
                local j
                for j in "${!opts[@]}"; do
                    opts[$j]=$(echo "${opts[$j]}" | xargs)
                done
                component_radio_group "$prompt" "$variable" "$default" "${opts[@]}"
            fi
            ;;
        confirm)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/confirm.sh"
            local default=$(yaml_get "${prefix}_${step_idx}_default" "false")
            component_confirm "$prompt" "$variable" "$default"
            ;;
        records)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/records.sh"

            local fields_count=$(yaml_get_step_fields_count "$step_idx" "$prefix")
            local field_args=()

            for ((field_idx = 0; field_idx < fields_count; field_idx++)); do
                local field_name=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_variable")
                if [ -z "$field_name" ]; then
                    print_error "Table field missing 'variable' key"
                    return 1
                fi
                local field_input=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_input")
                local field_display=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_display")
                local field_placeholder=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_placeholder")
                local field_min_length=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min_length")
                local field_min=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min")
                local field_max=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_max")
                local field_step=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_step")
                local field_default=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_default")
                local field_format=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_format")
                local field_options=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options")
                local field_active=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_active")
                local field_inactive=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_inactive")
                local field_validate_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_count")
                field_validate_count="${field_validate_count:-0}"
                local field_options_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_count")

                if [ -n "$field_options_count" ] && [ "$field_options_count" -gt 0 ]; then
                    local opts=()
                    local opt_idx
                    for ((opt_idx=0; opt_idx<field_options_count; opt_idx++)); do
                        opts+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_${opt_idx}")")
                    done
                    field_options=$(IFS=','; echo "${opts[*]}")
                fi

                field_args+=("$field_name" "$field_input" "$field_display" "$field_placeholder" "" "" "$field_min_length" "$field_min" "$field_max" "$field_step" "$field_default" "$field_format" "$field_options" "$field_active" "$field_inactive" "$field_validate_count")
                if [ "$field_validate_count" -gt 0 ]; then
                    local v_idx
                    for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_pattern")")
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_error")")
                    done
                fi
            done

            component_records "$prompt" "$variable" "$fields_count" "${field_args[@]}"
            ;;
        snippet)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/snippet.sh"

            local text=$(yaml_get "${prefix}_${step_idx}_text")
            if [ -z "$text" ]; then
                print_error "Snippet component missing 'text'"
                return 1
            fi

            local fields_count=$(yaml_get_step_fields_count "$step_idx" "$prefix")
            local field_args=()

            for ((field_idx = 0; field_idx < fields_count; field_idx++)); do
                local field_name=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_name")
                if [ -z "$field_name" ]; then
                    field_name=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_variable")
                fi
                if [ -z "$field_name" ]; then
                    print_error "Snippet field missing 'name'"
                    return 1
                fi
                local field_type=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_type")
                if [ -z "$field_type" ]; then
                    field_type=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_input")
                fi
                if [ -z "$field_type" ]; then
                    print_error "Snippet field missing 'type'"
                    return 1
                fi

                local field_placeholder=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_placeholder")
                local field_default=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_default")
                local field_min=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min")
                local field_max=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_max")
                local field_step=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_step")
                local field_format=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_format")
                local field_options=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options")
                local field_active=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_active")
                local field_inactive=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_inactive")
                local field_options_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_count")
                local field_validate_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_count" "0")

                if [ -n "$field_options_count" ] && [ "$field_options_count" -gt 0 ]; then
                    local opts=()
                    local opt_idx
                    for ((opt_idx=0; opt_idx<field_options_count; opt_idx++)); do
                        opts+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_${opt_idx}")")
                    done
                    field_options=$(IFS=','; echo "${opts[*]}")
                fi

                field_args+=("$field_name" "$field_type" "$field_placeholder" "$field_default" "$field_min" "$field_max" "$field_step" "$field_format" "$field_options" "$field_active" "$field_inactive" "$field_validate_count")
                if [ "$field_validate_count" -gt 0 ]; then
                    local v_idx
                    for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_pattern")")
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_error")")
                    done
                fi
            done

            component_snippet "$text" "$variable" "$fields_count" "${field_args[@]}"
            ;;
        table)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/table.sh"

            local fields_count=$(yaml_get_step_fields_count "$step_idx" "$prefix")
            local field_args=()

            for ((field_idx = 0; field_idx < fields_count; field_idx++)); do
                local field_name=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_variable")
                if [ -z "$field_name" ]; then
                    print_error "Table field missing 'variable' key"
                    return 1
                fi
                local field_input=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_input")
                local field_display=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_display")
                local field_placeholder=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_placeholder")
                local field_min_length=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min_length")
                local field_min=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_min")
                local field_max=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_max")
                local field_step=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_step")
                local field_default=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_default")
                local field_format=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_format")
                local field_options=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options")
                local field_active=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_active")
                local field_inactive=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_inactive")
                local field_options_count=$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_count")

                if [ -n "$field_options_count" ] && [ "$field_options_count" -gt 0 ]; then
                    local opts=()
                    local opt_idx
                    for ((opt_idx=0; opt_idx<field_options_count; opt_idx++)); do
                        opts+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_options_${opt_idx}")")
                    done
                    field_options=$(IFS=','; echo "${opts[*]}")
                fi

                field_args+=("$field_name" "$field_input" "$field_display" "$field_placeholder" "" "" "$field_min_length" "$field_min" "$field_max" "$field_step" "$field_default" "$field_format" "$field_options" "$field_active" "$field_inactive" "$field_validate_count")
                if [ "$field_validate_count" -gt 0 ]; then
                    local v_idx
                    for ((v_idx=0; v_idx<field_validate_count; v_idx++)); do
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_pattern")")
                        field_args+=("$(yaml_get "${prefix}_${step_idx}_fields_${field_idx}_validate_${v_idx}_error")")
                    done
                fi
            done

            component_table "$prompt" "$variable" "$fields_count" "${field_args[@]}"
            ;;
        file_browser)
            source "$(dirname "${BASH_SOURCE[0]}")/../components/file_browser.sh"

            local start_path=$(yaml_get "${prefix}_${step_idx}_start_path" ".")
            local filter=$(yaml_get "${prefix}_${step_idx}_filter" "")
            local browse_type=$(yaml_get "${prefix}_${step_idx}_mode" "file")
            local search_root=$(yaml_get "${prefix}_${step_idx}_search_root" ".")
            local max_depth=$(yaml_get "${prefix}_${step_idx}_max_depth" "4")

            component_file_browser "$prompt" "$variable" "$start_path" "$filter" "$browse_type" "$search_root" "$max_depth"
            ;;
        *)
            print_error "Unknown component type: $component_type"
            return 1
            ;;
    esac
}
