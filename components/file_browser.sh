#!/bin/bash

[[ -n "${_COMPONENT_FILE_BROWSER_SH_LOADED:-}" ]] && return
_COMPONENT_FILE_BROWSER_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/cursor.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/colors.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_file_browser() {
    local prompt="$1"
    local variable="$2"
    local start_path="${3:-.}"
    local filter="${4:-}"
    local browse_type="${5:-file}"
    local search_root="${6:-.}"
    local max_depth="${7:-4}"

    prompt=$(interpolate "$prompt")
    start_path=$(interpolate "$start_path")
    filter=$(interpolate "$filter")

    print_step "$prompt"
    echo -e "${DIM}  (↑↓ navigate, ← back, → enter, Enter select, type to search, type '/' '.' '~' for path, Ctrl+W delete word)${NC}"

    # Expand ~ to home directory
    start_path="${start_path/#\~/$HOME}"
    local current_path=$(realpath "$start_path" 2>/dev/null || echo "$start_path")
    local base_cwd="$current_path"
    local selected_index=0
    local scroll_offset=0
    local typed_path=""
    local search_query=""
    local search_root_mode="fixed"
    local resolved_search_root=""
    local cached_search_query=""
    local cached_search_root=""
    local cached_search_depth=""
    local cached_browse_path=""
    local cached_browse_filter=""
    search_root=$(interpolate "$search_root")
    max_depth=$(interpolate "$max_depth")
    search_root="${search_root/#\~/$HOME}"
    if [ -z "$search_root" ] || [ "$search_root" = "." ]; then
        search_root_mode="fixed"
        resolved_search_root="$current_path"
    elif [[ "$search_root" != /* ]]; then
        resolved_search_root="$(realpath "$current_path/$search_root" 2>/dev/null || echo "$current_path/$search_root")"
    else
        resolved_search_root=$(realpath "$search_root" 2>/dev/null || echo "$search_root")
    fi

    # Join path without double slashes
    join_path() {
        local base="$1"
        local name="$2"
        if [ "$base" = "/" ]; then
            echo "/$name"
        else
            echo "$base/$name"
        fi
    }

    # Normalize path for display
    display_path() {
        local path="$1"
        if [[ "$path" == "$base_cwd" ]]; then
            echo "."
        elif [[ "$path" == "$base_cwd/"* ]]; then
            echo ".${path#$base_cwd}"
        elif [[ "$path" == "$HOME"* ]]; then
            echo "~${path#$HOME}"
        else
            echo "$path"
        fi
    }

    # Resolve typed path to absolute
    resolve_typed_path() {
        local p="$1"
        if [[ "$p" == "~"* ]]; then
            p="$HOME${p#\~}"
        fi
        if [[ "$p" != /* ]]; then
            p="$(join_path "$current_path" "$p")"
        fi
        echo "$p"
    }

    highlight_match() {
        local text="$1"
        local pattern="$2"
        if [ -z "$pattern" ]; then
            echo "$text"
            return
        fi
        local pat="${pattern// /}"
        local lower_text="${text,,}"
        local lower_pat="${pat,,}"
        local tlen=${#text}
        local plen=${#pat}
        local i=0
        local j=0
        local out=""
        while [ $i -lt $tlen ]; do
            local ch="${text:$i:1}"
            if [ $j -lt $plen ]; then
                local target="${lower_pat:$j:1}"
                if [ "${lower_text:$i:1}" = "$target" ]; then
                    out+="${GREEN}${BOLD}${ch}${NC}"
                    j=$((j + 1))
                    i=$((i + 1))
                    continue
                fi
            fi
            out+="$ch"
            i=$((i + 1))
        done
        echo "$out"
    }

    # Get list of files (names only)
    get_files_list() {
        local path="$1"
        if [ ! -d "$path" ]; then
            return 1
        fi
        ls -A "$path" 2>/dev/null
    }

    # Lowercase helper
    to_lower() {
        echo "${1,,}"
    }

    # Fuzzy match (subsequence)
    fuzzy_match() {
        local text="$1"
        local pattern="$2"
        local lower_text="${text,,}"
        local lower_pattern="${pattern,,}"
        local tlen=${#lower_text}
        local plen=${#lower_pattern}
        local i=0
        local j=0
        while [ $i -lt $tlen ] && [ $j -lt $plen ]; do
            if [ "${lower_text:$i:1}" = "${lower_pattern:$j:1}" ]; then
                j=$((j + 1))
            fi
            i=$((i + 1))
        done
        [ $j -eq $plen ]
    }

    # Build files array
    build_files_array() {
        files_array=()
        while IFS= read -r line; do
            local fname="$line"
            local fpath
            fpath="$(join_path "$current_path" "$fname")"

            # Always show directories, filter only files
            if [ -n "$search_query" ]; then
                if ! fuzzy_match "$fname" "$search_query"; then
                    continue
                fi
            fi
            if [ -d "$fpath" ]; then
                files_array+=("$fname")
            elif [ -n "$filter" ]; then
                if [[ "$fname" == *"$filter"* ]]; then
                    files_array+=("$fname")
                fi
            else
                files_array+=("$fname")
            fi
        done < <(get_files_list "$current_path")

        cached_browse_path="$current_path"
        cached_browse_filter="$filter"
        cached_browse_array=("${files_array[@]}")
    }

    build_search_array() {
        files_array=()
        local query="$search_query"
        if [ -z "$query" ]; then
            return
        fi
        local depth="$max_depth"
        if [[ -z "$depth" || ! "$depth" =~ ^[0-9]+$ ]]; then
            depth=4
        fi

        local -a tokens=()
        local token
        while IFS= read -r token; do
            token="$(echo "$token" | xargs)"
            if [ -n "$token" ]; then
                tokens+=("$token")
            fi
        done <<< "${query// /$'\n'}"

        local max_results=200
        local count=0

        while IFS= read -r fpath; do
            local ok=true
            local t
            for t in "${tokens[@]}"; do
                if ! fuzzy_match "$fpath" "$t"; then
                    ok=false
                    break
                fi
            done

            if [ "$ok" = true ]; then
                files_array+=("$fpath")
                count=$((count + 1))
                if [ $count -ge $max_results ]; then
                    break
                fi
            fi
        done < <(find "$resolved_search_root" -maxdepth "$depth" \
            \( -path /proc -o -path /sys -o -path /dev -o -path /run -o -path /tmp \) -prune -o \
            -type f -o -type d 2>/dev/null)
    }

    ensure_search_results() {
        local depth="$max_depth"
        if [[ -z "$depth" || ! "$depth" =~ ^[0-9]+$ ]]; then
            depth=4
        fi
        if [ "$search_query" = "$cached_search_query" ] \
            && [ "$resolved_search_root" = "$cached_search_root" ] \
            && [ "$depth" = "$cached_search_depth" ]; then
            return
        fi
        build_search_array
        cached_search_query="$search_query"
        cached_search_root="$resolved_search_root"
        cached_search_depth="$depth"
    }

    reset_search_cache() {
        cached_search_query=""
        cached_search_root=""
        cached_search_depth=""
    }

    ensure_browse_results() {
        if [ -n "$search_query" ]; then
            return
        fi
        if [ "$current_path" = "$cached_browse_path" ] \
            && [ "$filter" = "$cached_browse_filter" ] \
            && [ ${#cached_browse_array[@]} -gt 0 ]; then
            files_array=("${cached_browse_array[@]}")
            return
        fi
        build_files_array
    }

    delete_word_from_query() {
        local q="$1"
        q="${q% }"
        q="${q%$'\t'}"
        while [ -n "$q" ] && [[ "${q: -1}" != " " ]]; do
            q="${q%?}"
        done
        q="${q% }"
        echo "$q"
    }

    delete_segment_from_path() {
        local p="$1"
        if [[ "$p" == */* ]]; then
            p="${p%/}"
            p="${p%/*}/"
            if [ "$p" = "/" ]; then
                echo "/"
            else
                echo "$p"
            fi
        else
            echo ""
        fi
    }

    # Draw screen
    draw_screen() {
        local idx
        ensure_browse_results

        # Line 1: Path/Search
        tput el
        if [ -n "$typed_path" ]; then
            local typed_display="$typed_path"
            if [[ "$typed_path" == /* ]]; then
                typed_display="$(display_path "$typed_path")"
            fi
            echo -e "  ${YELLOW}${typed_display}█${NC} ${DIM}(TAB to autocomplete)${NC}"
        elif [ -n "$search_query" ]; then
            echo -e "  ${YELLOW}Search:${NC} ${search_query} ${DIM}($(display_path "$resolved_search_root"), depth ${max_depth})${NC}"
        else
            echo -e "  ${DIM}Search...${NC}"
        fi

        # Lines 2-9: results
        local max_display=8
        local total=${#files_array[@]}
        if [ ${#files_array[@]} -eq 0 ]; then
            # Empty directory
            tput el
            echo -e "  ${DIM}(empty)${NC}"
            for ((idx=1; idx<max_display; idx++)); do
                tput el
                echo ""
            done
        else
            for ((idx=0; idx<max_display; idx++)); do
                tput el
                local file_idx=$((scroll_offset + idx))
                if [ $file_idx -lt ${#files_array[@]} ]; then
                    local fname="${files_array[$file_idx]}"
                    local fpath
                    local display_name=""
                    if [ -n "$search_query" ]; then
                        fpath="$fname"
                        local short_path
                        short_path="$(display_path "$fpath")"
                        display_name="$(highlight_match "$short_path" "$search_query")"
                    else
                        fpath="$(join_path "$current_path" "$fname")"
                        display_name="$fname"
                    fi

                    if [ $file_idx -eq $selected_index ]; then
                        if [ -d "$fpath" ]; then
                            echo -e "  ${YELLOW}>${NC} ${BLUE}${BOLD}${display_name}/${NC}"
                        else
                            echo -e "  ${YELLOW}>${NC} ${BOLD}${display_name}${NC}"
                        fi
                    else
                        if [ -d "$fpath" ]; then
                            echo -e "    ${BLUE}${display_name}/${NC}"
                        else
                            echo -e "    ${display_name}"
                        fi
                    fi
                else
                    echo ""
                fi
            done
        fi

        # Line 10: Scroll indicator at the bottom
        tput el
        if [ $total -gt 8 ]; then
            echo -e "  ${DIM}[${scroll_offset}-$((scroll_offset + 8)) of ${total}]${NC}"
        else
            echo ""
        fi
    }

    # Hide cursor
    cursor_hide
    trap 'cursor_show; exit 130' INT

    # Build initial list
    declare -a files_array
    declare -a cached_browse_array
    build_files_array

    # Draw initial screen
    draw_screen

    # Main loop
    read_char() {
        local ch=""
        IFS= read -rsn1 ch
        if [[ "$ch" == $'\x1b' ]]; then
            # Read full escape sequence if present
            local k1=""
            local k2=""
            IFS= read -rsn1 -t 0.001 k1 || true
            IFS= read -rsn1 -t 0.001 k2 || true
            echo "${ch}${k1}${k2}"
            return
        fi
        while IFS= read -rsn1 -t 0.001 extra; do
            ch+="$extra"
        done
        echo "$ch"
    }

    while true; do
        local key
        key="$(read_char)"

        local needs_redraw=0

        if [[ $key == $'\x1b'* ]]; then
            local k1="${key:1:1}"
            local k2="${key:2:1}"
            if [[ $k1 == '[' ]]; then
                case "$k2" in
                    'A') # Up
                        if [ $selected_index -gt 0 ]; then
                            selected_index=$((selected_index - 1))
                            # Adjust scroll offset if needed
                            if [ $selected_index -lt $scroll_offset ]; then
                                scroll_offset=$selected_index
                            fi
                            needs_redraw=1
                        fi
                        ;;
                    'B') # Down
                        local max=$((${#files_array[@]} - 1))
                        if [ $max -lt 0 ]; then max=0; fi
                        if [ $selected_index -lt $max ]; then
                            selected_index=$((selected_index + 1))
                            # Adjust scroll offset if needed
                            if [ $selected_index -ge $((scroll_offset + 8)) ]; then
                                scroll_offset=$((selected_index - 7))
                            fi
                            needs_redraw=1
                        fi
                        ;;
                    'C') # Right - enter directory
                        if [ ${#files_array[@]} -gt 0 ] && [ $selected_index -lt ${#files_array[@]} ]; then
                            local sel="${files_array[$selected_index]}"
                            local sel_path
                            if [ -n "$search_query" ]; then
                                sel_path="$sel"
                            else
                                sel_path="$(join_path "$current_path" "$sel")"
                            fi
                            if [ -d "$sel_path" ]; then
                                current_path=$(realpath "$sel_path")
                                selected_index=0
                                scroll_offset=0
                                typed_path=""
                                search_query=""
                                reset_search_cache
                                build_files_array
                                needs_redraw=1
                            fi
                        fi
                        ;;
                    'D') # Left - go up
                        local parent=$(dirname "$current_path")
                        if [ "$parent" != "$current_path" ]; then
                            current_path="$parent"
                            selected_index=0
                            scroll_offset=0
                            typed_path=""
                            search_query=""
                            reset_search_cache
                            build_files_array
                            needs_redraw=1
                        fi
                        ;;
                esac
            fi
        elif [[ $key == $'\t' ]] || [[ $key == $'\x09' ]]; then
            # TAB - autocomplete
            if [ -n "$typed_path" ]; then
                # Expand to absolute
                local expanded_path
                expanded_path="$(resolve_typed_path "$typed_path")"

                # Determine directory to search and prefix to match
                local search_dir=""
                local prefix=""

                if [ -d "$expanded_path" ]; then
                    # If it's already a directory, search in it
                    search_dir="$expanded_path"
                    prefix=""
                else
                    # Otherwise, search in parent directory
                    search_dir=$(dirname "$expanded_path")
                    prefix=$(basename "$expanded_path")
                fi

                # Find matches
                local matches=()
                if [ -d "$search_dir" ]; then
                    while IFS= read -r file; do
                        # Skip . and ..
                        if [ "$file" = "." ] || [ "$file" = ".." ]; then
                            continue
                        fi
                        if [ -z "$prefix" ] || [[ "$file" == "$prefix"* ]]; then
                            matches+=("$file")
                        fi
                    done < <(ls -A "$search_dir" 2>/dev/null)

                    if [ ${#matches[@]} -eq 1 ]; then
                        # Single match - complete it
                        local completed
                        completed="$(join_path "$search_dir" "${matches[0]}")"
                        if [ -d "$completed" ]; then
                            typed_path="${completed%/}/"
                            current_path=$(realpath "$completed")
                            selected_index=0
                            scroll_offset=0
                            build_files_array
                        else
                            typed_path="$completed"
                        fi
                        needs_redraw=1
                    elif [ ${#matches[@]} -gt 1 ]; then
                        # Multiple matches - complete common prefix
                        local common="${matches[0]}"
                        for match in "${matches[@]:1}"; do
                            local match_idx=0
                            while [ $match_idx -lt ${#common} ] && [ $match_idx -lt ${#match} ] && [ "${common:$match_idx:1}" = "${match:$match_idx:1}" ]; do
                                match_idx=$((match_idx + 1))
                            done
                            common="${common:0:$match_idx}"
                        done

                        if [ ${#common} -gt ${#prefix} ]; then
                            typed_path="$(join_path "$search_dir" "$common")"
                            local check_path
                            check_path="$(resolve_typed_path "$typed_path")"
                            if [ -d "$check_path" ]; then
                                current_path=$(realpath "$check_path")
                                selected_index=0
                                scroll_offset=0
                                build_files_array
                            fi
                            needs_redraw=1
                        fi
                        # No progress -> fallback to selected list item
                        if [ ${#common} -le ${#prefix} ]; then
                            if [ ${#files_array[@]} -gt 0 ] && [ $selected_index -lt ${#files_array[@]} ]; then
                                local sel="${files_array[$selected_index]}"
                                local sel_path="$(join_path "$current_path" "$sel")"
                                if [ -d "$sel_path" ]; then
                                    typed_path="${sel_path%/}/"
                                    current_path=$(realpath "$sel_path")
                                    selected_index=0
                                    scroll_offset=0
                                    build_files_array
                                    needs_redraw=1
                                else
                                    typed_path="$sel_path"
                                    needs_redraw=1
                                fi
                            fi
                        fi
                    else
                        # No matches - fallback to selected list item
                        if [ ${#files_array[@]} -gt 0 ] && [ $selected_index -lt ${#files_array[@]} ]; then
                            local sel="${files_array[$selected_index]}"
                            local sel_path="$(join_path "$current_path" "$sel")"
                            if [ -d "$sel_path" ]; then
                                typed_path="${sel_path%/}/"
                                current_path=$(realpath "$sel_path")
                                selected_index=0
                                scroll_offset=0
                                build_files_array
                                needs_redraw=1
                            else
                                typed_path="$sel_path"
                                needs_redraw=1
                            fi
                        fi
                    fi
                fi
            else
                # No typed path - autocomplete from current directory
                if [ "$current_path" = "/" ]; then
                    typed_path="/"
                else
                    typed_path="${current_path%/}/"
                fi
                needs_redraw=1
            fi
        elif [[ $key == $'\177' ]] || [[ $key == $'\b' ]]; then
            # Backspace
            if [ ${#typed_path} -gt 0 ]; then
                typed_path="${typed_path%?}"
                if [ -n "$typed_path" ]; then
                    # Expand ~ for checking
                    local check_path
                    check_path="$(resolve_typed_path "$typed_path")"
                    if [ -d "$check_path" ]; then
                        current_path=$(realpath "$check_path")
                        selected_index=0
                        scroll_offset=0
                        build_files_array
                    fi
                fi
                needs_redraw=1
            elif [ ${#search_query} -gt 0 ]; then
                search_query="${search_query%?}"
                selected_index=0
                scroll_offset=0
                if [ -n "$search_query" ]; then
                    ensure_search_results
                else
                    reset_search_cache
                    build_files_array
                fi
                needs_redraw=1
            fi
        elif [[ $key == $'\x17' ]]; then
            # Ctrl+W - delete word
            if [ ${#typed_path} -gt 0 ]; then
                typed_path="$(delete_segment_from_path "$typed_path")"
                local check_path
                check_path="$(resolve_typed_path "$typed_path")"
                if [ -d "$check_path" ]; then
                    current_path=$(realpath "$check_path")
                    selected_index=0
                    scroll_offset=0
                    build_files_array
                fi
                needs_redraw=1
            elif [ ${#search_query} -gt 0 ]; then
                search_query="$(delete_word_from_query "$search_query")"
                selected_index=0
                scroll_offset=0
                if [ -n "$search_query" ]; then
                    ensure_search_results
                else
                    reset_search_cache
                    build_files_array
                fi
                needs_redraw=1
            fi
        elif [[ $key == "" ]]; then
            # Enter
            if [ -n "$typed_path" ]; then
                # Expand to absolute
                local expanded_path
                expanded_path="$(resolve_typed_path "$typed_path")"
                if [ -d "$expanded_path" ]; then
                    current_path=$(realpath "$expanded_path")
                    typed_path=""
                    search_query=""
                    reset_search_cache
                    selected_index=0
                    scroll_offset=0
                    build_files_array
                    needs_redraw=1
                    if [ "$browse_type" = "directory" ]; then
                        break
                    fi
                elif [ -f "$expanded_path" ] && [ "$browse_type" = "file" ]; then
                    current_path="$expanded_path"
                    break
                fi
            else
                if [ ${#files_array[@]} -gt 0 ] && [ $selected_index -lt ${#files_array[@]} ]; then
                    local sel="${files_array[$selected_index]}"
                    local sel_path=""
                    if [ -n "$search_query" ]; then
                        sel_path="$sel"
                    else
                        sel_path="$(join_path "$current_path" "$sel")"
                    fi
                    if [ -d "$sel_path" ]; then
                        current_path=$(realpath "$sel_path")
                        selected_index=0
                        scroll_offset=0
                        typed_path=""
                        search_query=""
                        reset_search_cache
                        build_files_array
                        needs_redraw=1
                    elif [ -f "$sel_path" ] && [ "$browse_type" = "file" ]; then
                        current_path="$sel_path"
                        break
                    fi
                elif [ "$browse_type" = "directory" ]; then
                    break
                fi
            fi
        else
            # Regular key - add to typed path or fuzzy query (process bursts)
            local idx=0
            while [ $idx -lt ${#key} ]; do
                local ch="${key:$idx:1}"
                if [ -z "$typed_path" ] && [ -z "$search_query" ] && ([[ "$ch" == "/" ]] || [[ "$ch" == "." ]] || [[ "$ch" == "~" ]]); then
                    typed_path="$ch"
                    needs_redraw=1
                    idx=$((idx + 1))
                    continue
                fi
                if [ -n "$typed_path" ] || [[ "$ch" == "/" ]] || [[ "$ch" == "." ]] || [[ "$ch" == "~" ]]; then
                    if [ -n "$search_query" ]; then
                        search_query=""
                        reset_search_cache
                    fi
                    if [ -z "$typed_path" ] && [ "$ch" = "." ]; then
                        typed_path="."
                    else
                        typed_path+="$ch"
                    fi
                    local check_path
                    check_path="$(resolve_typed_path "$typed_path")"
                    if [ -d "$check_path" ]; then
                        current_path=$(realpath "$check_path")
                        selected_index=0
                        scroll_offset=0
                        build_files_array
                    fi
                    needs_redraw=1
                else
                    search_query+="$ch"
                    selected_index=0
                    scroll_offset=0
                    ensure_search_results
                    needs_redraw=1
                fi
                idx=$((idx + 1))
            done
        fi

        if [ $needs_redraw -eq 1 ]; then
            tput cuu 10
            draw_screen
        fi
    done

    # Cleanup
    trap - INT
    cursor_show

    # Save to state
    state_set "$variable" "$current_path"

    print_success "Selected: $current_path"
}