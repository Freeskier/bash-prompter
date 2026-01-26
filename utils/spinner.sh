#!/bin/bash

[[ -n "${_SPINNER_SH_LOADED:-}" ]] && return
_SPINNER_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/colors.sh"

SPINNER_CHARS=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')

_run_spinner() {
    local message="$1"
    local command="$2"
    local color="${3:-$YELLOW}"
    local show_output="${4:-false}"
    local log_lines="${5:-3}"
    local output_file="${6:-}"
    local final_status="${7:-true}"

    local created_output=false
    if [ -z "$output_file" ]; then
        output_file="$(mktemp)"
        created_output=true
    fi

    : > "$output_file"

    # Run command in background, capture output
    bash -c "$command" >> "$output_file" 2>&1 &
    local pid=$!

    tput civis
    local i=0

    # Initial draw
    printf "${color}${SPINNER_CHARS[$i]}${NC} ${BOLD}${message}${NC}\n"
    if [ "$show_output" = "true" ]; then
        for ((j=0; j<log_lines; j++)); do
            printf "${DIM}  │${NC}\n"
        done
    fi

    while kill -0 "$pid" 2>/dev/null; do
        if [ "$show_output" = "true" ]; then
            tput cuu $((log_lines + 1))
        else
            tput cuu 1
        fi

        tput el
        printf "${color}${SPINNER_CHARS[$i]}${NC} ${BOLD}${message}${NC}\n"

        if [ "$show_output" = "true" ]; then
            if [ -s "$output_file" ]; then
                local tail_lines
                tail_lines=$(tail -n "$log_lines" "$output_file" 2>/dev/null)
                local actual_lines
                actual_lines=$(printf '%s\n' "$tail_lines" | wc -l | tr -d ' ')
                while IFS= read -r line; do
                    tput el
                    printf "${DIM}  │ ${line:0:80}${NC}\n"
                done <<< "$tail_lines"
                for ((j=actual_lines; j<log_lines; j++)); do
                    tput el
                    printf "${DIM}  │${NC}\n"
                done
            else
                for ((j=0; j<log_lines; j++)); do
                    tput el
                    printf "${DIM}  │${NC}\n"
                done
            fi
        fi

        i=$(( (i + 1) % ${#SPINNER_CHARS[@]} ))
        sleep 0.1
    done

    wait "$pid"
    local exit_code=$?

    # Clear block
    if [ "$show_output" = "true" ]; then
        tput cuu $((log_lines + 1))
        for ((j=0; j<log_lines+1; j++)); do
            tput el
            echo
        done
        tput cuu $((log_lines + 1))
    else
        tput cuu 1
        tput el
    fi

    if [ "$final_status" = "true" ]; then
        if [ $exit_code -eq 0 ]; then
            printf "${GREEN}✓${NC} ${message}\n"
        else
            printf "${RED}✗${NC} ${message} ${RED}(failed)${NC}\n"
            if [ -s "$output_file" ]; then
                echo -e "${DIM}Last output:${NC}"
                tail -n 5 "$output_file" 2>/dev/null | sed 's/^/  /'
            fi
        fi
    fi

    tput cnorm

    if [ "$created_output" = "true" ]; then
        rm -f "$output_file"
    fi

    return $exit_code
}

run_with_spinner() {
    _run_spinner "$@" "true"
}

run_with_spinner_silent() {
    _run_spinner "$@" "false"
}
