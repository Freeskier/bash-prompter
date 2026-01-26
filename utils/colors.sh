#!/bin/bash

[[ -n "${_COLORS_SH_LOADED:-}" ]] && return
_COLORS_SH_LOADED=1

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly YELLOW='\033[1;33m'
readonly CYAN='\033[0;36m'
readonly MAGENTA='\033[0;35m'
readonly BOLD='\033[1m'
readonly UNDERLINE='\033[4m'
readonly DIM='\033[2m'
readonly NC='\033[0m'
