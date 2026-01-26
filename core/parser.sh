#!/bin/bash

[[ -n "${_PARSER_SH_LOADED:-}" ]] && return
_PARSER_SH_LOADED=1

declare -A YAML_DATA
YAML_STEP_COUNT=0

# Parse YAML file and populate YAML_DATA
yaml_parse_file() {
    local file="$1"

    if [ ! -f "$file" ]; then
        echo "YAML file not found: $file" >&2
        return 1
    fi

    YAML_DATA=()
    YAML_STEP_COUNT=0

    local -a lines=()
    while IFS= read -r line; do
        lines+=("$line")
    done < "$file"

    _parse_steps_block 0 ${#lines[@]} "steps"
}

# Get indentation level of a line
_get_indent() {
    local line="$1"
    local indent=0
    while [[ "${line:$indent:1}" == " " ]]; do
        ((indent++))
    done
    echo "$indent"
}

# Parse a steps block recursively
# Args: start_line end_line prefix
_parse_steps_block() {
    local start_line=$1
    local end_line=$2
    local prefix=$3

    local step_idx=-1
    local i=$start_line

    while ((i < end_line)); do
        local line="${lines[$i]}"

        # Skip comments and empty lines
        if [[ "$line" =~ ^[[:space:]]*# ]] || [[ -z "${line// }" ]]; then
            ((i++))
            continue
        fi

        local indent=$(_get_indent "$line")
        local trimmed="${line#"${line%%[![:space:]]*}"}"

        # Skip "steps:" marker
        if [[ "$trimmed" == "steps:" ]]; then
            ((i++))
            continue
        fi

        # Detect new step (starts with "- ")
        if [[ "$trimmed" =~ ^-[[:space:]]+(.*) ]]; then
            local after_dash="${BASH_REMATCH[1]}"
            ((step_idx++))

            # Check if this is a condition block
            if [[ "$after_dash" =~ ^condition:[[:space:]]*$ ]]; then
                YAML_DATA["${prefix}_${step_idx}_type"]="condition"

                # Parse the condition block
                local cond_start=$((i + 1))
                local cond_end=$cond_start
                local base_indent=$indent

                # Find the end of this condition block
                for ((j = cond_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${check_line// }" ]] && continue

                    local check_indent=$(_get_indent "$check_line")
                    local check_trimmed="${check_line#"${check_line%%[![:space:]]*}"}"

                    # If we hit another step at same or lower indent, we're done
                    if [[ "$check_trimmed" =~ ^- ]] && ((check_indent <= base_indent)); then
                        break
                    fi
                    cond_end=$j
                done
                cond_end=$((cond_end + 1))

                # Parse condition content (if and nested steps)
                _parse_condition_block "$cond_start" "$cond_end" "${prefix}_${step_idx}"

                i=$((cond_end - 1))
                ((i++))
                continue
            fi

            # Check if this is a loop block
            if [[ "$after_dash" =~ ^loop:[[:space:]]*$ ]]; then
                YAML_DATA["${prefix}_${step_idx}_type"]="loop"

                # Parse the loop block
                local loop_start=$((i + 1))
                local loop_end=$loop_start
                local base_indent=$indent

                # Find the end of this loop block
                for ((j = loop_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${check_line// }" ]] && continue

                    local check_indent=$(_get_indent "$check_line")
                    local check_trimmed="${check_line#"${check_line%%[![:space:]]*}"}"

                    # If we hit another step at same or lower indent, we're done
                    if [[ "$check_trimmed" =~ ^- ]] && ((check_indent <= base_indent)); then
                        break
                    fi
                    loop_end=$j
                done
                loop_end=$((loop_end + 1))

                # Parse loop content (items, variable, and nested steps)
                _parse_loop_block "$loop_start" "$loop_end" "${prefix}_${step_idx}"

                i=$((loop_end - 1))
                ((i++))
                continue
            fi

            # Check if this is a pipeline block
            if [[ "$after_dash" =~ ^pipeline:[[:space:]]*$ ]]; then
                YAML_DATA["${prefix}_${step_idx}_type"]="pipeline"

                # Parse the pipeline block
                local pipe_start=$((i + 1))
                local pipe_end=$pipe_start
                local base_indent=$indent

                # Find the end of this pipeline block
                for ((j = pipe_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${check_line// }" ]] && continue

                    local check_indent=$(_get_indent "$check_line")
                    local check_trimmed="${check_line#"${check_line%%[![:space:]]*}"}"

                    # If we hit another step at same or lower indent, we're done
                    if [[ "$check_trimmed" =~ ^- ]] && ((check_indent <= base_indent)); then
                        break
                    fi
                    pipe_end=$j
                done
                pipe_end=$((pipe_end + 1))

                # Parse pipeline content (message, show_output, commands)
                _parse_pipeline_block "$pipe_start" "$pipe_end" "${prefix}_${step_idx}"

                i=$((pipe_end - 1))
                ((i++))
                continue
            fi

            # Regular step with key:value on same line
            if [[ "$after_dash" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
                local key="${BASH_REMATCH[1]}"
                local value="${BASH_REMATCH[2]}"
                value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                YAML_DATA["${prefix}_${step_idx}_${key}"]="$value"
            fi

            ((i++))
            continue
        fi

        # Parse properties of current step
        if ((step_idx >= 0)) && [[ "$trimmed" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local value="${BASH_REMATCH[2]}"

            # Check for fields or options array
            if [[ "$key" == "fields" ]] && [[ -z "$value" ]]; then
                local fields_start=$((i + 1))
                local fields_end=$fields_start

                # Find end of fields block
                for ((j = fields_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${check_line// }" ]] && continue

                    local check_indent=$(_get_indent "$check_line")
                    if ((check_indent <= indent)); then
                        break
                    fi
                    fields_end=$j
                done
                fields_end=$((fields_end + 1))

                _parse_fields_block "$fields_start" "$fields_end" "${prefix}_${step_idx}"
                i=$fields_end
                continue
            fi

            if [[ "$key" == "text" ]] && { [[ -z "$value" ]] || [[ "$value" == "|" ]] || [[ "$value" == ">" ]]; }; then
                local text_start=$((i + 1))
                local text_end=$text_start

                for ((j = text_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    local check_indent=$(_get_indent "$check_line")
                    if ((check_indent <= indent)) && [[ -n "${check_line// }" ]]; then
                        break
                    fi
                    text_end=$j
                done
                text_end=$((text_end + 1))

                _parse_text_block "$text_start" "$text_end" "${prefix}_${step_idx}_text"
                i=$text_end
                continue
            fi

            if [[ "$key" == "validate" ]] && [[ -z "$value" ]]; then
                local val_start=$((i + 1))
                local val_end=$val_start

                for ((j = val_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${check_line// }" ]] && continue

                    local check_indent=$(_get_indent "$check_line")
                    if ((check_indent <= indent)); then
                        break
                    fi
                    val_end=$j
                done
                val_end=$((val_end + 1))

                _parse_validate_block "$val_start" "$val_end" "${prefix}_${step_idx}"
                i=$val_end
                continue
            fi

            if [[ "$key" == "options" ]] && [[ -z "$value" ]]; then
                local opts_start=$((i + 1))
                local opts_end=$opts_start
                local opt_idx=0

                # Parse options array
                for ((j = opts_start; j < end_line; j++)); do
                    local opt_line="${lines[$j]}"
                    [[ "$opt_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${opt_line// }" ]] && continue

                    local opt_indent=$(_get_indent "$opt_line")
                    local opt_trimmed="${opt_line#"${opt_line%%[![:space:]]*}"}"

                    if ((opt_indent <= indent)); then
                        break
                    fi

                    if [[ "$opt_trimmed" =~ ^-[[:space:]]+(.*) ]]; then
                        local opt_value="${BASH_REMATCH[1]}"
                        opt_value=$(echo "$opt_value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                        YAML_DATA["${prefix}_${step_idx}_options_${opt_idx}"]="$opt_value"
                        ((opt_idx++))
                    fi
                    opts_end=$j
                done
                YAML_DATA["${prefix}_${step_idx}_options_count"]=$opt_idx
                i=$((opts_end + 1))
                continue
            fi

            # Regular key-value
            value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
            YAML_DATA["${prefix}_${step_idx}_${key}"]="$value"
        fi

        ((i++))
    done

    # Set step count
    local total_steps=$((step_idx + 1))
    if [[ "$prefix" == "steps" ]]; then
        YAML_STEP_COUNT=$total_steps
    else
        # For nested steps (in conditions), store the count
        YAML_DATA["${prefix}_count"]=$total_steps
    fi
}

# Parse a condition block (if + nested steps)
# Args: start_line end_line prefix
_parse_condition_block() {
    local start_line=$1
    local end_line=$2
    local prefix=$3

    local i=$start_line
    local found_if=false
    local found_steps=false

    while ((i < end_line)); do
        local line="${lines[$i]}"

        # Skip comments and empty lines
        if [[ "$line" =~ ^[[:space:]]*# ]] || [[ -z "${line// }" ]]; then
            ((i++))
            continue
        fi

        local indent=$(_get_indent "$line")
        local trimmed="${line#"${line%%[![:space:]]*}"}"

        # Parse "if:" condition
        if [[ "$trimmed" =~ ^if:[[:space:]]*(.*)$ ]]; then
            local condition="${BASH_REMATCH[1]}"
            condition=$(echo "$condition" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
            YAML_DATA["${prefix}_if"]="$condition"
            found_if=true
            ((i++))
            continue
        fi

        # Parse nested "steps:"
        if [[ "$trimmed" =~ ^steps:[[:space:]]*$ ]]; then
            found_steps=true
            local nested_start=$((i + 1))

            # Find the actual end of the nested steps block
            local nested_end=$nested_start
            local base_indent=$indent

            for ((j = nested_start; j < end_line; j++)); do
                local nest_line="${lines[$j]}"
                [[ "$nest_line" =~ ^[[:space:]]*# ]] && continue
                [[ -z "${nest_line// }" ]] && continue

                local nest_indent=$(_get_indent "$nest_line")
                local nest_trimmed="${nest_line#"${nest_line%%[![:space:]]*}"}"

                # Check if we've exited the nested steps (back to lower indent level)
                # The nested steps are indented more than the "steps:" keyword
                if [[ "$nest_trimmed" =~ ^- ]] && ((nest_indent <= base_indent)); then
                    break
                fi
                nested_end=$j
            done
            nested_end=$((nested_end + 1))

            # Parse the nested steps block
            _parse_steps_block "$nested_start" "$nested_end" "${prefix}_steps"

            # The count is set by _parse_steps_block in YAML_DATA
            local nested_count="${YAML_DATA[${prefix}_steps_count]:-0}"
            YAML_DATA["${prefix}_steps_count"]="$nested_count"

            # We're done with this condition block
            break
        fi

        ((i++))
    done
}

# Parse a loop block (items, variable, and nested steps)
# Args: start_line end_line prefix
_parse_loop_block() {
    local start_line=$1
    local end_line=$2
    local prefix=$3

    local i=$start_line
    local found_items=false
    local found_variable=false
    local found_steps=false

    while ((i < end_line)); do
        local line="${lines[$i]}"

        # Skip comments and empty lines
        if [[ "$line" =~ ^[[:space:]]*# ]] || [[ -z "${line// }" ]]; then
            ((i++))
            continue
        fi

        local indent=$(_get_indent "$line")
        local trimmed="${line#"${line%%[![:space:]]*}"}"

        # Parse "items:" (can be array, list, or variable name)
        if [[ "$trimmed" =~ ^items:[[:space:]]*(.*)$ ]]; then
            local items_value="${BASH_REMATCH[1]}"

            # Check if it's an array [a, b, c]
            if [[ "$items_value" =~ ^\[.*\]$ ]]; then
                # Parse inline array
                items_value="${items_value#\[}"
                items_value="${items_value%\]}"

                IFS=',' read -ra items_arr <<< "$items_value"
                local item_idx=0
                for item in "${items_arr[@]}"; do
                    item=$(echo "$item" | xargs)  # Trim whitespace
                    item=$(echo "$item" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')  # Remove quotes
                    YAML_DATA["${prefix}_items_${item_idx}"]="$item"
                    ((item_idx++))
                done
                YAML_DATA["${prefix}_items_count"]="$item_idx"
            elif [[ -z "$items_value" ]]; then
                # Multiline list under items:
                local list_start=$((i + 1))
                local list_end=$list_start
                local list_idx=0
                local base_indent=$indent

                for ((j = list_start; j < end_line; j++)); do
                    local list_line="${lines[$j]}"
                    [[ "$list_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${list_line// }" ]] && continue

                    local list_indent=$(_get_indent "$list_line")
                    local list_trimmed="${list_line#"${list_line%%[![:space:]]*}"}"

                    if ((list_indent <= base_indent)); then
                        break
                    fi

                    if [[ "$list_trimmed" =~ ^-[[:space:]]+(.*) ]]; then
                        local item_value="${BASH_REMATCH[1]}"
                        item_value=$(echo "$item_value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                        YAML_DATA["${prefix}_items_${list_idx}"]="$item_value"
                        ((list_idx++))
                    fi
                    list_end=$j
                done
                YAML_DATA["${prefix}_items_count"]="$list_idx"
                i=$list_end
            else
                # It's a variable name
                YAML_DATA["${prefix}_items"]="$items_value"
            fi

            found_items=true
            ((i++))
            continue
        fi

        # Parse "variable:" - the loop variable name
        if [[ "$trimmed" =~ ^variable:[[:space:]]*(.*)$ ]]; then
            local variable="${BASH_REMATCH[1]}"
            variable=$(echo "$variable" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
            YAML_DATA["${prefix}_variable"]="$variable"
            found_variable=true
            ((i++))
            continue
        fi

        # Parse "separator:" (optional)
        if [[ "$trimmed" =~ ^separator:[[:space:]]*(.*)$ ]]; then
            local separator="${BASH_REMATCH[1]}"
            separator=$(echo "$separator" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
            YAML_DATA["${prefix}_separator"]="$separator"
            ((i++))
            continue
        fi

        # Parse nested "steps:"
        if [[ "$trimmed" =~ ^steps:[[:space:]]*$ ]]; then
            found_steps=true
            local nested_start=$((i + 1))

            # Find the actual end of the nested steps block
            local nested_end=$nested_start
            local base_indent=$indent

            for ((j = nested_start; j < end_line; j++)); do
                local nest_line="${lines[$j]}"
                [[ "$nest_line" =~ ^[[:space:]]*# ]] && continue
                [[ -z "${nest_line// }" ]] && continue

                local nest_indent=$(_get_indent "$nest_line")
                local nest_trimmed="${nest_line#"${nest_line%%[![:space:]]*}"}"

                # Check if we've exited the nested steps (back to lower indent level)
                if [[ "$nest_trimmed" =~ ^- ]] && ((nest_indent <= base_indent)); then
                    break
                fi
                nested_end=$j
            done
            nested_end=$((nested_end + 1))

            # Parse the nested steps block
            _parse_steps_block "$nested_start" "$nested_end" "${prefix}_steps"

            # The count is set by _parse_steps_block in YAML_DATA
            local nested_count="${YAML_DATA[${prefix}_steps_count]:-0}"
            YAML_DATA["${prefix}_steps_count"]="$nested_count"

            # We're done with this loop block
            break
        fi

        ((i++))
    done
}

# Parse a pipeline block (message, show_output, commands list)
# Args: start_line end_line prefix
_parse_pipeline_block() {
    local start_line=$1
    local end_line=$2
    local prefix=$3

    local i=$start_line
    local cmd_idx=-1

    while ((i < end_line)); do
        local line="${lines[$i]}"

        # Skip comments and empty lines
        if [[ "$line" =~ ^[[:space:]]*# ]] || [[ -z "${line// }" ]]; then
            ((i++))
            continue
        fi

        local indent=$(_get_indent "$line")
        local trimmed="${line#"${line%%[![:space:]]*}"}"

        # Parse commands list
        if [[ "$trimmed" =~ ^commands:[[:space:]]*$ ]]; then
            local commands_indent=$indent
            ((i++))

            while ((i < end_line)); do
                local cmd_line="${lines[$i]}"
                if [[ "$cmd_line" =~ ^[[:space:]]*# ]] || [[ -z "${cmd_line// }" ]]; then
                    ((i++))
                    continue
                fi

                local cmd_indent=$(_get_indent "$cmd_line")
                local cmd_trimmed="${cmd_line#"${cmd_line%%[![:space:]]*}"}"

                if ((cmd_indent <= commands_indent)); then
                    break
                fi

                if [[ "$cmd_trimmed" =~ ^-[[:space:]]+(.*) ]]; then
                    ((cmd_idx++))
                    local after_dash="${BASH_REMATCH[1]}"
                    local item_indent=$cmd_indent

                    if [[ "$after_dash" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
                        local key="${BASH_REMATCH[1]}"
                        local value="${BASH_REMATCH[2]}"
                        value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                        YAML_DATA["${prefix}_commands_${cmd_idx}_${key}"]="$value"
                    fi

                    ((i++))
                    while ((i < end_line)); do
                        local prop_line="${lines[$i]}"
                        if [[ "$prop_line" =~ ^[[:space:]]*# ]] || [[ -z "${prop_line// }" ]]; then
                            ((i++))
                            continue
                        fi

                        local prop_indent=$(_get_indent "$prop_line")
                        local prop_trimmed="${prop_line#"${prop_line%%[![:space:]]*}"}"

                        if ((prop_indent <= item_indent)); then
                            break
                        fi

                        if [[ "$prop_trimmed" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
                            local key="${BASH_REMATCH[1]}"
                            local value="${BASH_REMATCH[2]}"
                            value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                            YAML_DATA["${prefix}_commands_${cmd_idx}_${key}"]="$value"
                        fi

                        ((i++))
                    done
                    continue
                fi

                ((i++))
            done

            continue
        fi

        # Pipeline-level properties
        if [[ "$trimmed" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local value="${BASH_REMATCH[2]}"
            value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
            YAML_DATA["${prefix}_${key}"]="$value"
        fi

        ((i++))
    done

    YAML_DATA["${prefix}_commands_count"]=$((cmd_idx + 1))
}

# Parse validate block (list of pattern/error rules)
# Args: start_line end_line prefix
_parse_validate_block() {
    local start_line=$1
    local end_line=$2
    local prefix=$3

    local rule_idx=-1
    local i=$start_line

    while ((i < end_line)); do
        local line="${lines[$i]}"

        if [[ "$line" =~ ^[[:space:]]*# ]] || [[ -z "${line// }" ]]; then
            ((i++))
            continue
        fi

        local indent=$(_get_indent "$line")
        local trimmed="${line#"${line%%[![:space:]]*}"}"

        if [[ "$trimmed" =~ ^-[[:space:]]+(.*) ]]; then
            ((rule_idx++))
            local after_dash="${BASH_REMATCH[1]}"
            local item_indent=$indent

            if [[ "$after_dash" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
                local key="${BASH_REMATCH[1]}"
                local value="${BASH_REMATCH[2]}"
                value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                YAML_DATA["${prefix}_validate_${rule_idx}_${key}"]="$value"
            fi

            ((i++))
            while ((i < end_line)); do
                local prop_line="${lines[$i]}"
                if [[ "$prop_line" =~ ^[[:space:]]*# ]] || [[ -z "${prop_line// }" ]]; then
                    ((i++))
                    continue
                fi

                local prop_indent=$(_get_indent "$prop_line")
                local prop_trimmed="${prop_line#"${prop_line%%[![:space:]]*}"}"

                if ((prop_indent <= item_indent)); then
                    break
                fi

                if [[ "$prop_trimmed" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
                    local key="${BASH_REMATCH[1]}"
                    local value="${BASH_REMATCH[2]}"
                    value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                    YAML_DATA["${prefix}_validate_${rule_idx}_${key}"]="$value"
                fi

                ((i++))
            done
            continue
        fi

        ((i++))
    done

    YAML_DATA["${prefix}_validate_count"]=$((rule_idx + 1))
}

# Parse multiline text block
# Args: start_line end_line key
_parse_text_block() {
    local start_line=$1
    local end_line=$2
    local key=$3

    local i
    local min_indent=9999

    for ((i = start_line; i < end_line; i++)); do
        local line="${lines[$i]}"
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        if [[ -z "${line// }" ]]; then
            continue
        fi
        local indent=$(_get_indent "$line")
        if ((indent < min_indent)); then
            min_indent=$indent
        fi
    done

    if ((min_indent == 9999)); then
        YAML_DATA["$key"]=""
        return
    fi

    local out=""
    for ((i = start_line; i < end_line; i++)); do
        local line="${lines[$i]}"
        if [[ "$line" =~ ^[[:space:]]*# ]]; then
            continue
        fi
        if [ ${#line} -ge $min_indent ]; then
            line="${line:$min_indent}"
        fi
        out+="$line"
        if [ $i -lt $((end_line - 1)) ]; then
            out+=$'\n'
        fi
    done

    YAML_DATA["$key"]="$out"
}

# Parse fields block for object component
# Args: start_line end_line prefix
_parse_fields_block() {
    local start_line=$1
    local end_line=$2
    local prefix=$3

    local field_idx=-1
    local i=$start_line

    while ((i < end_line)); do
        local line="${lines[$i]}"

        # Skip comments and empty lines
        if [[ "$line" =~ ^[[:space:]]*# ]] || [[ -z "${line// }" ]]; then
            ((i++))
            continue
        fi

        local trimmed="${line#"${line%%[![:space:]]*}"}"

        # New field starts with "- "
        if [[ "$trimmed" =~ ^-[[:space:]]+(.*) ]]; then
            ((field_idx++))
            local after_dash="${BASH_REMATCH[1]}"

            if [[ "$after_dash" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
                local key="${BASH_REMATCH[1]}"
                local value="${BASH_REMATCH[2]}"
                value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                YAML_DATA["${prefix}_fields_${field_idx}_${key}"]="$value"
            fi

            ((i++))
            continue
        fi

        # Field properties
        if ((field_idx >= 0)) && [[ "$trimmed" =~ ^([a-zA-Z0-9_]+):[[:space:]]*(.*)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local value="${BASH_REMATCH[2]}"

            if [[ "$key" == "validate" ]] && [[ -z "$value" ]]; then
                local val_start=$((i + 1))
                local val_end=$val_start
                local base_indent=$(_get_indent "$line")

                for ((j = val_start; j < end_line; j++)); do
                    local check_line="${lines[$j]}"
                    [[ "$check_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${check_line// }" ]] && continue

                    local check_indent=$(_get_indent "$check_line")
                    if ((check_indent <= base_indent)); then
                        break
                    fi
                    val_end=$j
                done
                val_end=$((val_end + 1))

                _parse_validate_block "$val_start" "$val_end" "${prefix}_fields_${field_idx}"
                i=$val_end
                continue
            fi

            if [[ "$key" == "options" ]] && [[ -z "$value" ]]; then
                local opts_start=$((i + 1))
                local opts_end=$opts_start
                local opt_idx=0
                local base_indent=$(_get_indent "$line")

                for ((j = opts_start; j < end_line; j++)); do
                    local opt_line="${lines[$j]}"
                    [[ "$opt_line" =~ ^[[:space:]]*# ]] && continue
                    [[ -z "${opt_line// }" ]] && continue

                    local opt_indent=$(_get_indent "$opt_line")
                    local opt_trimmed="${opt_line#"${opt_line%%[![:space:]]*}"}"

                    if ((opt_indent <= base_indent)); then
                        break
                    fi

                    if [[ "$opt_trimmed" =~ ^-[[:space:]]+(.*) ]]; then
                        local opt_value="${BASH_REMATCH[1]}"
                        opt_value=$(echo "$opt_value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
                        YAML_DATA["${prefix}_fields_${field_idx}_options_${opt_idx}"]="$opt_value"
                        ((opt_idx++))
                    fi
                    opts_end=$j
                done

                YAML_DATA["${prefix}_fields_${field_idx}_options_count"]=$opt_idx
                i=$((opts_end + 1))
                continue
            fi

            value=$(echo "$value" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
            YAML_DATA["${prefix}_fields_${field_idx}_${key}"]="$value"
        fi

        ((i++))
    done

    YAML_DATA["${prefix}_fields_count"]=$((field_idx + 1))
}

yaml_get() {
    local key="$1"
    local default="${2:-}"
    echo "${YAML_DATA[$key]:-$default}"
}

yaml_get_step_type() {
    local step_idx="$1"
    local prefix="${2:-steps}"

    # Check for explicit type (condition)
    if [[ -n "${YAML_DATA[${prefix}_${step_idx}_type]:-}" ]]; then
        echo "${YAML_DATA[${prefix}_${step_idx}_type]}"
        return
    fi

    # Otherwise determine by which key is present
    if [[ -n "${YAML_DATA[${prefix}_${step_idx}_input]:-}" ]]; then
        echo "input"
    elif [[ -n "${YAML_DATA[${prefix}_${step_idx}_output]:-}" ]]; then
        echo "output"
    elif [[ -n "${YAML_DATA[${prefix}_${step_idx}_component]:-}" ]]; then
        echo "component"
    elif [[ -n "${YAML_DATA[${prefix}_${step_idx}_command]:-}" ]]; then
        echo "command"
    elif [[ -n "${YAML_DATA[${prefix}_${step_idx}_script]:-}" ]]; then
        echo "script"
    else
        echo "unknown"
    fi
}

yaml_get_step_subtype() {
    local step_idx="$1"
    local prefix="${2:-steps}"
    local step_type=$(yaml_get_step_type "$step_idx" "$prefix")

    case "$step_type" in
        input)
            echo "${YAML_DATA[${prefix}_${step_idx}_input]}"
            ;;
        output)
            echo "${YAML_DATA[${prefix}_${step_idx}_output]}"
            ;;
        component)
            echo "${YAML_DATA[${prefix}_${step_idx}_component]}"
            ;;
        command)
            echo "command"
            ;;
        script)
            echo "script"
            ;;
        pipeline)
            echo "pipeline"
            ;;
        condition)
            echo "condition"
            ;;
        loop)
            echo "loop"
            ;;
        *)
            echo ""
            ;;
    esac
}

yaml_get_step_count() {
    echo "$YAML_STEP_COUNT"
}

yaml_get_step_fields_count() {
    local step_idx="$1"
    local prefix="${2:-steps}"
    echo "${YAML_DATA[${prefix}_${step_idx}_fields_count]:-0}"
}

yaml_get_step_options_count() {
    local step_idx="$1"
    local prefix="${2:-steps}"
    echo "${YAML_DATA[${prefix}_${step_idx}_options_count]:-0}"
}
