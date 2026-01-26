#!/bin/bash

[[ -n "${_PRINT_SH_LOADED:-}" ]] && return
_PRINT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/colors.sh"

print_step() {
    echo -e "\n${BLUE}▶${NC} ${BOLD}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_dim() {
    echo -e "${DIM}$1${NC}"
}

print_prompt() {
    local prompt="$1"
    local placeholder="$2"

    echo -e "${BOLD}$prompt${NC}"
    if [ -n "$placeholder" ]; then
        echo -e "${DIM}  ($placeholder)${NC}"
    fi
    echo -n "  > "
}
