#!/usr/bin/env bash
# conditions.sh - Conditional expression evaluator

# Source dependencies
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/state.sh"

# Evaluate a condition expression
# Args: $1 - condition string (e.g., "choice == 'option1'")
# Returns: 0 if true, 1 if false
# Exits with error if condition is malformed or references undefined variables
evaluate_condition() {
    local condition="$1"

    if [[ -z "$condition" ]]; then
        echo "Error: Empty condition" >&2
        exit 1
    fi

    # Remove leading/trailing whitespace
    condition="${condition#"${condition%%[![:space:]]*}"}"
    condition="${condition%"${condition##*[![:space:]]}"}"

    # Detect operator (order matters - check multi-char operators first)
    local operator=""
    local left=""
    local right=""

    if [[ "$condition" =~ ^(.+)[[:space:]]*(==)[[:space:]]*(.+)$ ]]; then
        operator="=="
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    elif [[ "$condition" =~ ^(.+)[[:space:]]*(!=)[[:space:]]*(.+)$ ]]; then
        operator="!="
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    elif [[ "$condition" =~ ^(.+)[[:space:]]*(=~)[[:space:]]*(.+)$ ]]; then
        operator="=~"
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    elif [[ "$condition" =~ ^(.+)[[:space:]]*(>=)[[:space:]]*(.+)$ ]]; then
        operator=">="
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    elif [[ "$condition" =~ ^(.+)[[:space:]]*(<=)[[:space:]]*(.+)$ ]]; then
        operator="<="
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    elif [[ "$condition" =~ ^(.+)[[:space:]]*(\>)[[:space:]]*(.+)$ ]]; then
        operator=">"
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    elif [[ "$condition" =~ ^(.+)[[:space:]]*(\<)[[:space:]]*(.+)$ ]]; then
        operator="<"
        left="${BASH_REMATCH[1]}"
        right="${BASH_REMATCH[3]}"
    else
        echo "Error: Invalid condition syntax: $condition" >&2
        echo "Supported operators: ==, !=, =~, >, <, >=, <=" >&2
        exit 1
    fi

    # Trim whitespace from operands
    left="${left#"${left%%[![:space:]]*}"}"
    left="${left%"${left##*[![:space:]]}"}"
    right="${right#"${right%%[![:space:]]*}"}"
    right="${right%"${right##*[![:space:]]}"}"

    # Resolve left operand (variable name or var.field)
    local left_value

    # Check if left operand uses dot notation (loop context)
    if [[ "$left" =~ ^([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)$ ]]; then
        local var_name="${BASH_REMATCH[1]}"
        local field_name="${BASH_REMATCH[2]}"

        # Get loop context
        local loop_var=$(state_get "__current_loop_var")
        local loop_parent=$(state_get "__current_loop_parent")
        local loop_idx=$(state_get "__current_loop_idx")

        if [ "$var_name" = "$loop_var" ] && [ -n "$loop_parent" ] && [ -n "$loop_idx" ]; then
            # Resolve from loop context
            left_value=$(state_get "${loop_parent}_${loop_idx}_${field_name}")
        else
            # Try direct variable_field format
            left_value=$(state_get "${var_name}_${field_name}")
        fi
    else
        # Simple variable name
        left_value=$(state_get "$left")
    fi

    if [[ -z "$left_value" ]]; then
        echo "Error: Variable '$left' is not defined in state" >&2
        exit 1
    fi

    # Resolve right operand (can be string literal or variable)
    local right_value

    # Check if right is a quoted string literal
    if [[ "$right" =~ ^[\'\"](.*)[\'\"]$ ]]; then
        # String literal - extract content
        right_value="${BASH_REMATCH[1]}"
    else
        # Check if right operand uses dot notation
        if [[ "$right" =~ ^([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)$ ]]; then
            local var_name="${BASH_REMATCH[1]}"
            local field_name="${BASH_REMATCH[2]}"

            # Get loop context
            local loop_var=$(state_get "__current_loop_var")
            local loop_parent=$(state_get "__current_loop_parent")
            local loop_idx=$(state_get "__current_loop_idx")

            if [ "$var_name" = "$loop_var" ] && [ -n "$loop_parent" ] && [ -n "$loop_idx" ]; then
                # Resolve from loop context
                right_value=$(state_get "${loop_parent}_${loop_idx}_${field_name}")
            else
                # Try direct variable_field format
                right_value=$(state_get "${var_name}_${field_name}")
            fi
        else
            # Try to get from state (variable reference)
            right_value=$(state_get "$right")
        fi

        # If not in state, treat as literal value (for numbers)
        if [[ -z "$right_value" ]]; then
            right_value="$right"
        fi
    fi

    # Evaluate based on operator
    case "$operator" in
        "==")
            [[ "$left_value" == "$right_value" ]]
            return $?
            ;;
        "!=")
            [[ "$left_value" != "$right_value" ]]
            return $?
            ;;
        "=~")
            # Regex match - right_value is the pattern
            [[ "$left_value" =~ $right_value ]]
            return $?
            ;;
        ">")
            # Numeric comparison
            if ! [[ "$left_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Left operand '$left_value' is not a number for operator >" >&2
                exit 1
            fi
            if ! [[ "$right_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Right operand '$right_value' is not a number for operator >" >&2
                exit 1
            fi
            awk -v l="$left_value" -v r="$right_value" 'BEGIN { exit !(l > r) }' </dev/null
            return $?
            ;;
        "<")
            # Numeric comparison
            if ! [[ "$left_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Left operand '$left_value' is not a number for operator <" >&2
                exit 1
            fi
            if ! [[ "$right_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Right operand '$right_value' is not a number for operator <" >&2
                exit 1
            fi
            awk -v l="$left_value" -v r="$right_value" 'BEGIN { exit !(l < r) }' </dev/null
            return $?
            ;;
        ">=")
            # Numeric comparison
            if ! [[ "$left_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Left operand '$left_value' is not a number for operator >=" >&2
                exit 1
            fi
            if ! [[ "$right_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Right operand '$right_value' is not a number for operator >=" >&2
                exit 1
            fi
            awk -v l="$left_value" -v r="$right_value" 'BEGIN { exit !(l >= r) }' </dev/null
            return $?
            ;;
        "<=")
            # Numeric comparison
            if ! [[ "$left_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Left operand '$left_value' is not a number for operator <=" >&2
                exit 1
            fi
            if ! [[ "$right_value" =~ ^-?[0-9]+(\.[0-9]+)?$ ]]; then
                echo "Error: Right operand '$right_value' is not a number for operator <=" >&2
                exit 1
            fi
            awk -v l="$left_value" -v r="$right_value" 'BEGIN { exit !(l <= r) }' </dev/null
            return $?
            ;;
        *)
            echo "Error: Unknown operator: $operator" >&2
            exit 1
            ;;
    esac
}
