#!/bin/bash

[[ -n "${_STATE_SH_LOADED:-}" ]] && return
_STATE_SH_LOADED=1

declare -A STATE

state_set() {
    local key="$1"
    local value="$2"
    if [ -z "$key" ]; then
        return 1
    fi
    STATE["$key"]="$value"
}

state_get() {
    local key="$1"
    local default="${2:-}"
    if [ -z "$key" ]; then
        echo "$default"
        return 0
    fi
    echo "${STATE[$key]:-$default}"
}

state_has() {
    local key="$1"
    if [ -z "$key" ]; then
        return 1
    fi
    [[ -n "${STATE[$key]+x}" ]]
}

state_delete() {
    local key="$1"
    if [ -z "$key" ]; then
        return 1
    fi
    unset STATE["$key"]
}

state_clear() {
    STATE=()
}

state_keys() {
    echo "${!STATE[@]}"
}

interpolate() {
    local text="$1"
    local result="$text"

    # Get loop context (if in loop)
    local loop_var=$(state_get "__current_loop_var")
    local loop_parent=$(state_get "__current_loop_parent")
    local loop_idx=$(state_get "__current_loop_idx")

    # First, handle {{loop_var.field}} syntax (loop context)
    if [ -n "$loop_var" ] && [ -n "$loop_parent" ] && [ -n "$loop_idx" ]; then
        while [[ "$result" =~ \{\{[[:space:]]*${loop_var}\.([a-zA-Z0-9_]+)[[:space:]]*\}\} ]]; do
            local field="${BASH_REMATCH[1]}"
            local var_value=$(state_get "${loop_parent}_${loop_idx}_${field}")
            local matched="${BASH_REMATCH[0]}"
            result="${result//$matched/$var_value}"
        done
    fi

    # Handle {{var.field.subfield}} syntax (convert dots to underscores)
    # This allows users to write {{servers.count}} instead of {{servers_count}}
    while [[ "$result" =~ \{\{[[:space:]]*([a-zA-Z0-9_]+)\.([a-zA-Z0-9_.]+)[[:space:]]*\}\} ]]; do
        local var_path="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}"
        local var_name="${var_path//\./_}"  # Convert dots to underscores
        local var_value=$(state_get "$var_name")
        local matched="${BASH_REMATCH[0]}"
        result="${result//$matched/$var_value}"
    done

    # Finally handle simple {{var}} syntax
    while [[ "$result" =~ \{\{[[:space:]]*([a-zA-Z0-9_]+)[[:space:]]*\}\} ]]; do
        local var_name="${BASH_REMATCH[1]}"
        local var_value=$(state_get "$var_name")
        local matched="${BASH_REMATCH[0]}"
        result="${result//$matched/$var_value}"
    done

    echo "$result"
}
