# Setup-Sonda v2 - Implementation Guide

## Overview

Setup-Sonda v2 is a complete rewrite focused on clean architecture, DRY principles, and extensibility. The system provides an interactive CLI framework for building configuration wizards with support for various input types, validation, components, and state management.

## Architecture

### Core Principles

1. **Single Responsibility** - Each module has one clear purpose
2. **DRY (Don't Repeat Yourself)** - No code duplication, universal input collector
3. **Open/Closed Principle** - Open for extension (new inputs), closed for modification
4. **Dispatcher Pattern** - Central validation and input routing
5. **State Management** - Centralized bash associative array with nested keys

### Directory Structure

```
v2/
├── core/
│   ├── state.sh          # State management & variable interpolation
│   ├── parser.sh         # YAML parser with array support
│   ├── validator.sh      # Universal validation dispatcher
│   └── runner.sh         # Main execution loop
├── inputs/
│   ├── text.sh           # Base text input with validate rules
│   ├── email.sh          # Email wrapper (reuses text.sh)
│   ├── password.sh       # Masked password input
│   ├── url.sh            # URL wrapper with validation
│   ├── ip.sh             # IP address wrapper
│   ├── slider.sh         # Interactive slider with live preview
│   └── date.sh           # Interactive date/time picker
├── outputs/
│   └── info.sh           # Information display with interpolation
├── components/
│   ├── select.sh         # Single choice from list (arrow keys)
│   ├── multiselect.sh    # Multiple choice with checkboxes
│   ├── radio_group.sh    # Radio buttons (single choice)
│   └── object.sh         # Form with multiple fields
├── utils/
│   ├── colors.sh         # Terminal color constants
│   ├── print.sh          # Print utilities (success, error, info, etc.)
│   ├── input_reader.sh   # Universal char-by-char input reading
│   └── input_collector.sh # Universal input collection & validation
└── main.sh               # Entry point

```

## Key Features

### 1. State Management

**File:** `core/state.sh`

State is stored in a global bash associative array `STATE` with support for nested keys using dot notation.

```bash
# Set values
state_set "user.name" "John"
state_set "user.email" "john@example.com"
state_set "port" "8080"

# Get values
state_get "user.name"  # Returns: John
state_get "port"       # Returns: 8080

# Interpolation (works everywhere in YAML)
interpolate "Hello {{user.name}}, your email is {{user.email}}"
# Returns: "Hello John, your email is john@example.com"
```

**Nested objects from components:**
```yaml
- component: object
  variable: "server"
  fields:
    - variable: hostname
      input: text
    - variable: port
      input: slider

# Creates: server.hostname, server.port
```

### 2. YAML Parser

**File:** `core/parser.sh`

Supports:
- Steps with different types (input, output, command, component)
- Arrays for options (YAML arrays converted to bash arrays)
- Nested fields for components
- Automatic type detection

**YAML Structure:**
```yaml
steps:
  - input: text           # Input step
    prompt: "Name"
    variable: "name"
    
  - output: info          # Output step
    value: "Hello {{name}}"
    
  - command: "date"       # Command execution
    variable: "today"
    
  - component: select     # Component step
    prompt: "Choose"
    options:
      - Option 1
      - Option 2
    variable: "choice"
```

### 3. Input System

#### Base Input: Text

**File:** `inputs/text.sh`

All simple text-based inputs inherit from this:

```yaml
- input: text
  prompt: "Enter name"
  placeholder: "John Doe"
  validate:
    - pattern: "^[A-Z].*"
      error: "Must start with capital letter"
  default: "John"
  variable: "name"
```

Features:
- Character-by-character reading
- Placeholder support (disappears on first char)
- Multi-rule validation via `validate` list
- Default value support
- Live preview while typing

#### Wrapper Inputs

**Concept:** New inputs can wrap existing ones by providing preset patterns.

**Example: Email (wraps text.sh)**
```bash
input_email() {
    local pattern="^.+@.+\..+$"
    local on_error="Invalid email format"
    input_text "$prompt" "$variable" "$placeholder" "$default" "$pattern" "$on_error"
}
```

**Example: IP (wraps text.sh)**
```bash
input_ip() {
    local pattern="^([0-9]{1,3}\.){3}[0-9]{1,3}$"
    local on_error="Invalid IP format (xxx.xxx.xxx.xxx)"
    input_text "$prompt" "$variable" "$placeholder" "$default" "$pattern" "$on_error"
}
```

**Adding new input types:**
1. Create `inputs/newtype.sh`
2. Implement validation in `core/validator.sh`
3. Add case in `core/runner.sh` (for standalone use)
4. Done! Automatically works in components via `input_collector.sh`

#### Interactive Inputs

**Slider:**
```yaml
- input: slider
  prompt: "Choose value"
  min: 0
  max: 100
  step: 5
  default: 50
  variable: "value"
```

Features:
- Single-line UI with `\r` (carriage return) for redraw
- Arrow keys `←→` to adjust value
- Direct number input with live preview
- Visual bar: `[━━━━◉━━━━] 50`
- Shows "...typing" indicator while entering numbers
- Works both standalone and in object component

**Date:**
```yaml
- input: date
  prompt: "Select date"
  format: "YYYY-MM-DD HH:mm"
  variable: "datetime"
```

Features:
- Single-line UI: `[2026-01-24 15:30]`
- `←→` navigate between fields
- `↑↓` increment/decrement values
- Direct number input with underscore preview: `2___` → `20__` → `202_` → `2026`
- Cyan brackets for styling
- Active field is yellow/bold, others are gray
- Final redraw makes all fields gray after confirm

### 4. Validation System

**File:** `core/validator.sh`

**Dispatcher Pattern:**
```bash
validate() {
    local input_type="$1"
    local value="$2"
    shift 2
    
    case "$input_type" in
        text) _validate_text "$value" "$@" ;;
        email) _validate_email "$value" "$@" ;;
        password) _validate_password "$value" "$@" ;;
        url) _validate_url "$value" "$@" ;;
        ip) _validate_ip "$value" "$@" ;;
        slider) _validate_slider "$value" "$@" ;;
        *) echo "Unknown input type"; return 1 ;;
    esac
}
```

Each validator returns:
- Exit code 0 = valid
- Exit code 1 = invalid, with error message via `echo`

### 5. Universal Input Collector

**File:** `utils/input_collector.sh`

**Problem:** Without this, every component (object, records, form, etc.) would need to duplicate input handling logic for text, password, slider, date, etc.

**Solution:** Single universal function that handles ALL input types.

```bash
collect_input "$label" "$input_type" \
    --placeholder "$placeholder" \
    --validate "$pattern" "$error" \
    --min "$min" \
    --max "$max" \
    --step "$step" \
    --format "$format"

# Returns value in INPUT_VALUE global variable
```

**Benefits:**
- Add new input type → automatically works in ALL components
- No code duplication
- Consistent behavior everywhere
- Easy to maintain

### 6. Components

#### Select (Single Choice)

```yaml
- component: select
  prompt: "Choose option"
  options:
    - Option 1
    - Option 2
    - Option 3
  variable: "choice"
```

- `↑↓` navigate
- `Enter` confirm
- Yellow `>` shows selection

#### Multiselect (Multiple Choice)

```yaml
- component: multiselect
  prompt: "Select technologies"
  options:
    - JavaScript
    - Python
    - Go
  variable: "tech"
```

- `↑↓` navigate
- `Space` toggle selection
- `Enter` confirm
- Checkboxes: `[ ]` unchecked, `[✓]` checked
- Returns comma-separated string

#### Radio Group (Single Choice)

```yaml
- component: radio_group
  prompt: "Choose size"
  options:
    - Small
    - Medium
    - Large
  default: "Medium"
  variable: "size"
```

- `↑↓` navigate
- `Enter` select
- Radio buttons: `○` unselected, `◉` selected (green)
- Difference from select: stays visible, arrow-driven

#### Object (Form)

**The killer feature** - collects multiple fields as a structured object.

```yaml
- component: object
  prompt: "Configure server"
  fields:
    - variable: hostname      # Key in state
      display: "Hostname"     # Label shown to user
      input: text
      placeholder: "server.example.com"
      validate:
        - pattern: "^[a-z0-9.-]+$"
          error: "Invalid hostname"
      
    - variable: port
      display: "Port"
      input: slider           # Interactive slider inline!
      min: 1000
      max: 9999
      step: 1
      default: 8080
      
    - variable: start_time
      display: "Start time"
      input: date             # Interactive date inline!
      format: "YYYY-MM-DD HH:mm"
      
    - variable: admin_email
      display: "Admin email"
      input: email
      
  variable: "server"
```

Creates nested state:
- `server.hostname`
- `server.port`
- `server.start_time`
- `server.admin_email`

**Visual appearance:**
```
▶ Configure server
  Hostname: server.example.com
  Port: [━━━━━━◉━━━━] 8080
  Start time: [2026-01-24 15:30]
  Admin email: admin@example.com
```

**Key points:**
- Uses `input_collector.sh` - supports ALL input types automatically
- Each field validates independently
- Errors show inline
- No empty lines between fields

## YAML API Reference

### Input Types

**Text:**
```yaml
- input: text
  prompt: "Question"
  placeholder: "hint"      # Optional, shows as gray background
  validate:                # Optional, list of validation rules
    - pattern: "regex"
      error: "Error msg"
  default: "value"         # Optional, used when Enter on empty
  variable: "var_name"
```

**Select:**
```yaml
- input: select
  prompt: "Choose"
  options:
    - Option 1
    - Option 2
  variable: "choice"
```

**Email:**
```yaml
- input: email
  prompt: "Email address"
  variable: "email"
  # Automatically validates email format
```

**Password:**
```yaml
- input: password
  prompt: "Enter password"
  placeholder: "min 8 chars"
  variable: "password"
  # Shows asterisks instead of characters
```

**URL:**
```yaml
- input: url
  prompt: "Website URL"
  variable: "website"
  # Validates http:// or https:// format
```

**IP:**
```yaml
- input: ip
  prompt: "Server IP"
  variable: "server_ip"
  # Validates xxx.xxx.xxx.xxx format
```

**Slider:**
```yaml
- input: slider
  prompt: "Select value"
  min: 0
  max: 100
  step: 5
  default: 50
  variable: "value"
```

**Date:**
```yaml
- input: date
  prompt: "Select date"
  format: "YYYY-MM-DD HH:mm"  # Supports: YYYY, MM, DD, HH, mm, SS
  variable: "datetime"
```

**Color (HEX):**
```yaml
- input: color
  prompt: "Primary color"
  default: "#FF5500"
  variable: "primary_color"
```

**List:**
```yaml
- input: list
  prompt: "Tags"
  separator: ","        # Optional, default ","
  default: "a,b,c"
  variable: "tags"
```

**Toggle:**
```yaml
- input: toggle
  prompt: "Can you confirm?"
  active: "yes"
  inactive: "no"
  default: "false"      # Optional
  variable: "confirm"
```

### Output Types

**Info:**
```yaml
- output: info
  value: "Message with {{variable}} interpolation"
```

### Commands

```yaml
- command: "shell command here"
  variable: "output"        # Stores stdout in variable
```

Exit code is checked - non-zero stops execution.

**Optional UI flags:**
```yaml
- command: "git clone https://repo"
  message: "Cloning repo"
  spinner: true
  show_output: true
  log_lines: 3
```

### Pipelines (Multiple Commands)

```yaml
- pipeline:
    message: "Provisioning"
    show_output: true
    log_lines: 3
    commands:
      - name: "Identifying..."
        run: "git ls-remote https://repo"
      - name: "Cloning..."
        run: "git clone https://repo"
      - name: "Fetching tags..."
        run: "git fetch --tags"
```

Each command runs with a spinner and optional live output (last N lines).

### Scripts (Source Mode)

```yaml
- script: "./scripts/setup.sh"
  args: "--user \"{{name}}\" --email \"{{email}}\""
  variable: "script_output"  # Optional: store stdout
```

Scripts run via `source` in the current shell, so any variable assignments inside
the script can become available in later steps. To explicitly push values back
into the state, use `export` in the script (e.g. `export db_user=...`). If you
want the script's stdout in later steps, assign `variable` and reference it with
`{{script_output}}`.

### Component Types

**Multiselect:**
```yaml
- component: multiselect
  prompt: "Select multiple"
  options:
    - Item 1
    - Item 2
  variable: "selected"      # Returns: "Item 1,Item 2"
```

**Radio Group:**
```yaml
- component: radio_group
  prompt: "Choose one"
  options:
    - Yes
    - No
    - Maybe
  default: "Yes"            # Optional
  variable: "answer"
```

**Object:**
```yaml
- component: object
  prompt: "Form title"
  fields:
    - variable: field_key   # Used in state (variable.field_key)
      display: "Label"      # Shown to user (optional, uses name if missing)
      input: <input_type>   # Any input type: text, email, password, slider, date, etc.
      # ... input-specific parameters
  variable: "object_name"
```

## Implementation Patterns

### Adding a New Input Type

1. **Create input file** (`inputs/myinput.sh`):
```bash
#!/bin/bash

[[ -n "${_INPUT_MYINPUT_SH_LOADED:-}" ]] && return
_INPUT_MYINPUT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/text.sh"  # If wrapping text

input_myinput() {
    local prompt="$1"
    local variable="$2"
    # ... parameters
    
    # Option A: Wrap existing input
    input_text "$prompt" "$variable" "$placeholder" "$default" "$pattern" "$on_error"
    
    # Option B: Custom implementation
    # ... your logic here
}

# For use in components (inline version)
input_myinput_inline() {
    local label="$1"
    # ... inline version logic
    # Must set INPUT_VALUE at the end
}
```

2. **Add validator** (`core/validator.sh`):
```bash
validate() {
    case "$input_type" in
        # ... existing cases
        myinput)
            _validate_myinput "$value" "$@"
            return $?
            ;;
    esac
}

_validate_myinput() {
    local value="$1"
    # Validation logic
    if [[ ! valid ]]; then
        echo "Error message"
        return 1
    fi
    return 0
}
```

3. **Add to runner** (`core/runner.sh`):
```bash
case "$input_type" in
    # ... existing cases
    myinput)
        source "$(dirname "${BASH_SOURCE[0]}")/../inputs/myinput.sh"
        input_myinput "$prompt" "$variable" # ... params
        ;;
esac
```

4. **Update input_collector** (`utils/input_collector.sh`):
```bash
collect_input() {
    case "$input_type" in
        # ... existing cases
        myinput)
            source "$(dirname "${BASH_SOURCE[0]}")/../inputs/myinput.sh"
            input_myinput_inline "$label" # ... params from params array
            ;;
    esac
}
```

**Done!** Your input now works:
- Standalone: `- input: myinput`
- In object: `input: myinput`
- In any future component automatically

### Adding a New Component

1. **Create component file** (`components/mycomponent.sh`):
```bash
#!/bin/bash

[[ -n "${_COMPONENT_MYCOMPONENT_SH_LOADED:-}" ]] && return
_COMPONENT_MYCOMPONENT_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/../utils/print.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../utils/input_collector.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../core/state.sh"

component_mycomponent() {
    local prompt="$1"
    local variable="$2"
    # ... more parameters
    
    prompt=$(interpolate "$prompt")
    print_step "$prompt"
    
    # Use input_collector for fields
    collect_input "$label" "$input_type" --placeholder "$placeholder" # ...
    local value="$INPUT_VALUE"
    
    # Validate if needed
    if validate_collected_input "$input_type" "$value" # ... params; then
        state_set "$variable" "$value"
    fi
}
```

2. **Add to runner** (`core/runner.sh`):
```bash
case "$component_type" in
    # ... existing cases
    mycomponent)
        source "$(dirname "${BASH_SOURCE[0]}")/../components/mycomponent.sh"
        # ... prepare parameters from YAML
        component_mycomponent "$prompt" "$variable" # ... params
        ;;
esac
```

**That's it!** All inputs work automatically thanks to `input_collector.sh`.

## Technical Details

### Variable Collision Fix

**Critical:** Never use variable `i` in any function that might be called from runner's main loop.

```bash
# BAD:
for i in "${!array[@]}"; do
    # This overwrites runner's loop variable!
done

# GOOD:
local j
for j in "${!array[@]}"; do
    # Uses local variable
done
```

### Include Guards

All files use include guards to prevent multiple sourcing:

```bash
[[ -n "${_FILENAME_SH_LOADED:-}" ]] && return
_FILENAME_SH_LOADED=1
```

### Exit Code Handling

Removed `set -e` from main.sh because validation functions intentionally return non-zero exit codes.

Always capture exit codes explicitly:
```bash
local error_msg
error_msg=$(validate "text" "$value")
local exit_code=$?  # Capture before any other command!
```

### Global Variables

- `STATE` - main state storage (associative array)
- `INPUT_VALUE` - return value from input functions
- `YAML_DATA` - parsed YAML data (associative array)
- `YAML_STEP_COUNT` - number of steps parsed

### Signal Handling

Ctrl+C handler in main.sh:
```bash
trap 'echo -e "\n\nInterrupted by user"; exit 130' INT
```

All interactive inputs also trap INT to restore cursor:
```bash
tput civis  # Hide cursor
trap 'tput cnorm; exit 130' INT
# ... input logic
trap - INT  # Remove trap
tput cnorm  # Show cursor
```

## Best Practices

### 1. Always Interpolate User-Facing Strings

```bash
prompt=$(interpolate "$prompt")
placeholder=$(interpolate "$placeholder")
```

### 2. Use Locals for Loop Variables

```bash
local idx
for idx in "${!array[@]}"; do
    # Safe from variable collision
done
```

### 3. Validate at the Right Level

- Simple inputs (text, email, etc.): Always validate
- Interactive inputs (slider, date): Skip validation (already constrained)
- Components: Use `validate_collected_input` for consistency

### 4. Single-Line Interactive Inputs

For inputs used in components (like object):
- Use `\r` (carriage return) to redraw same line
- Never use `tput cuu` (cursor up) - breaks layout
- Provide both standalone and inline versions

### 5. Consistent Styling

- Prompts: `${BLUE}▶${NC} ${BOLD}text${NC}`
- Labels in forms: `${CYAN}Label:${NC}`
- Errors: `${RED}✗${NC} message`
- Success: `${GREEN}✓${NC} message`
- Info: `${BLUE}ℹ${NC} message`
- Placeholders: `${DIM}text${NC}`

## Testing

### Manual Testing

```bash
./main.sh test-components.yml        # Selection components
./main.sh test-interactive-inputs.yml # Slider & date in object
./main.sh test-slider-in-object.yml  # Focused slider test
./main.sh test-advanced.yml          # Standalone slider & date
```

### Debug Parser

```bash
bash debug.sh yourconfig.yml
```

Shows all parsed YAML keys and values.

### Syntax Check

```bash
for f in core/*.sh inputs/*.sh outputs/*.sh components/*.sh utils/*.sh; do
    bash -n "$f" || echo "ERROR in $f"
done
```

## Future Enhancements

### Potential New Inputs

- **rating**: `★★★☆☆` (1-5 stars)
- **toggle**: `● ON  ○ OFF` (boolean switch)
- **color**: Color picker with preview
- **time_duration**: `[HH:MM:SS]` (like date but for durations)
- **phone**: Auto-formatted phone numbers
- **currency**: With thousand separators
- **ip_interactive**: Like date but for IP segments

### Potential New Components

- **records**: Spreadsheet-like multi-row data entry
- **wizard**: Multi-page form with navigation
- **tree**: Hierarchical selection
- **autocomplete**: Type-ahead suggestions

## Migration from v1

v2 is a complete rewrite. Not compatible with v1 YAML format.

**Key changes:**
- `step: text` → `input: text`
- `component: single_select` → `component: select`
- `component: multi_select` → `component: multiselect`
- Options must be YAML arrays (not comma-separated strings)
- State structure changed to nested keys with dots

## Troubleshooting

### "Unknown input type: X"

- Check `core/validator.sh` has case for X
- Check `core/runner.sh` has case for X
- Check `utils/input_collector.sh` has case for X

### "Value out of range" for slider

- Slider doesn't need validation
- Add to skip list in component validation logic

### Inputs not appearing in object

- Check `input_collector.sh` handles the input type
- Verify all parameters are passed with `--param` syntax

### Variable collision / infinite loop

- Check for loop variables named `i`
- Use local variables with different names

### Parser not detecting fields

- Check YAML indentation (must be consistent)
- Arrays need `-` prefix
- Check `fields_count` is being set in parser

## Architecture Decisions

### Why Bash?

- Zero dependencies
- Runs everywhere
- Perfect for system configuration tasks
- Native command execution

### Why Single-Line Interactive Inputs?

- Works in any context (standalone, forms, records)
- No layout breaking with `tput cuu`
- Simpler mental model
- Better for nested components

### Why Input Collector?

- **DRY**: One place for all input handling
- **Extensibility**: New inputs work everywhere automatically
- **Consistency**: Same behavior in all components
- **Maintainability**: Fix once, fixed everywhere

### Why Nested State Keys?

- Cleaner than flat keys (`server_hostname` vs `server.hostname`)
- Natural object representation
- Easy to group related data
- Future-proof for JSON/YAML export

## Credits

Built with focus on clean code, extensibility, and developer experience.

**Version:** 2.0  
**Status:** Production Ready  
**License:** MIT

---

## Conditional Branching (TODO - Implementation Guide)

### Overview

Conditional branching allows executing steps only when certain conditions are met, enabling dynamic workflows based on user choices.

### YAML Syntax

```yaml
steps:
  - input: select
    prompt: "Install database?"
    options: [Yes, No]
    variable: "install_db"

  # Conditional block - executes nested steps only if condition is true
  - condition:
      if: "{{install_db}} == Yes"
      steps:
        - input: select
          prompt: "Choose database"
          options: [MySQL, PostgreSQL]
          variable: "db_type"
        
        - input: text
          prompt: "Database host"
          variable: "db_host"
        
        - input: slider
          prompt: "Port"
          min: 3000
          max: 5432
          variable: "db_port"

  - output: info
    value: "Configuration complete"
```

### Supported Operators

**Comparison:**
- `==` - equals
- `!=` - not equals
- `>` - greater than
- `<` - less than
- `>=` - greater or equal
- `<=` - less or equal

**String operations:**
- `contains` - string contains substring
- `!contains` - string does not contain substring
- `startsWith` - string starts with
- `endsWith` - string ends with

**Boolean:**
- `empty` - variable is empty
- `!empty` - variable is not empty

**Examples:**
```yaml
if: "{{name}} == John"
if: "{{port}} > 1000"
if: "{{install_db}} != No"
if: "{{options}} contains MySQL"
if: "{{email}} !empty"
```

### Implementation Steps

#### 1. Parser Changes (core/parser.sh)

**Add condition block detection:**

```bash
# In yaml_parse_file() function, detect condition blocks
if [[ "$trimmed" == "condition:" ]]; then
    in_condition=true
    condition_step_idx=$step_idx
    continue
fi

# Parse 'if' field in condition
if $in_condition && [[ "$trimmed" =~ ^if:[[:space:]]*(.*)$ ]]; then
    local condition="${BASH_REMATCH[1]}"
    condition=$(echo "$condition" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')
    YAML_DATA["steps_${condition_step_idx}_condition_if"]="$condition"
    continue
fi

# Parse nested 'steps' array in condition
if $in_condition && [[ "$trimmed" == "steps:" ]]; then
    in_condition_steps=true
    condition_nested_idx=-1
    continue
fi
```

**Store nested steps with special key format:**
```bash
# Nested steps stored as:
# steps_X_condition_steps_Y_field
# where X is parent step index, Y is nested step index
```

#### 2. Condition Evaluator (core/conditions.sh)

**Create new file:**

```bash
#!/bin/bash

[[ -n "${_CONDITIONS_SH_LOADED:-}" ]] && return
_CONDITIONS_SH_LOADED=1

source "$(dirname "${BASH_SOURCE[0]}")/state.sh"

evaluate_condition() {
    local condition="$1"
    
    # Interpolate variables first
    condition=$(interpolate "$condition")
    
    # Parse condition: "left operator right"
    # Handle operators: ==, !=, >, <, >=, <=, contains, !contains, empty, !empty
    
    # Special case: empty/!empty (unary)
    if [[ "$condition" =~ ^(.+)[[:space:]]+empty$ ]]; then
        local value="${BASH_REMATCH[1]}"
        [ -z "$value" ] && return 0 || return 1
    fi
    
    if [[ "$condition" =~ ^(.+)[[:space:]]+!empty$ ]]; then
        local value="${BASH_REMATCH[1]}"
        [ -n "$value" ] && return 0 || return 1
    fi
    
    # Binary operators
    if [[ "$condition" =~ ^(.+)[[:space:]]+(==|!=|>|<|>=|<=|contains|!contains|startsWith|endsWith)[[:space:]]+(.+)$ ]]; then
        local left="${BASH_REMATCH[1]}"
        local op="${BASH_REMATCH[2]}"
        local right="${BASH_REMATCH[3]}"
        
        case "$op" in
            "==")
                [ "$left" = "$right" ] && return 0 || return 1
                ;;
            "!=")
                [ "$left" != "$right" ] && return 0 || return 1
                ;;
            ">")
                (( left > right )) && return 0 || return 1
                ;;
            "<")
                (( left < right )) && return 0 || return 1
                ;;
            ">=")
                (( left >= right )) && return 0 || return 1
                ;;
            "<=")
                (( left <= right )) && return 0 || return 1
                ;;
            "contains")
                [[ "$left" == *"$right"* ]] && return 0 || return 1
                ;;
            "!contains")
                [[ "$left" != *"$right"* ]] && return 0 || return 1
                ;;
            "startsWith")
                [[ "$left" == "$right"* ]] && return 0 || return 1
                ;;
            "endsWith")
                [[ "$left" == *"$right" ]] && return 0 || return 1
                ;;
        esac
    fi
    
    # Invalid condition format
    echo "Invalid condition: $condition" >&2
    return 1
}
```

#### 3. Runner Changes (core/runner.sh)

**Add condition step type detection:**

```bash
yaml_get_step_type() {
    local step_idx="$1"
    
    if [[ -n "${YAML_DATA[steps_${step_idx}_condition_if]:-}" ]]; then
        echo "condition"
    elif [[ -n "${YAML_DATA[steps_${step_idx}_input]:-}" ]]; then
        echo "input"
    # ... rest of existing checks
}
```

**Add condition execution:**

```bash
source "$(dirname "${BASH_SOURCE[0]}")/conditions.sh"

case "$step_type" in
    condition)
        _run_condition_step "$i"
        ;;
    input)
        _run_input_step "$i" "$step_subtype"
        ;;
    # ... rest of cases
esac

_run_condition_step() {
    local step_idx="$1"
    local condition=$(yaml_get "steps_${step_idx}_condition_if")
    
    # Evaluate condition
    if evaluate_condition "$condition"; then
        # Get nested steps count
        local nested_count=$(yaml_get "steps_${step_idx}_condition_steps_count")
        
        # Execute nested steps
        for ((nested_idx=0; nested_idx<nested_count; nested_idx++)); do
            # Get nested step type
            local nested_type=$(yaml_get "steps_${step_idx}_condition_steps_${nested_idx}_type")
            
            # Execute nested step (similar to main loop)
            # This may need refactoring to support recursion
            _execute_step "$step_idx" "$nested_idx"
        done
    fi
    # If condition false, skip nested steps
}
```

#### 4. Testing

**Create test file (test-branching.yml):**

```yaml
steps:
  - input: select
    prompt: "Choose environment"
    options:
      - Development
      - Production
    variable: "env"

  - condition:
      if: "{{env}} == Development"
      steps:
        - output: info
          value: "Development mode - skipping security"
        
        - input: text
          prompt: "Dev server"
          default: "localhost"
          variable: "server"

  - condition:
      if: "{{env}} == Production"
      steps:
        - output: info
          value: "Production mode - extra security required"
        
        - input: text
          prompt: "Production server"
          validate:
            - pattern: "^[a-z0-9.-]+$"
              error: "Must be valid hostname"
          variable: "server"
        
        - input: password
          prompt: "Admin password"
          variable: "admin_pass"

  - output: info
    value: "Server configured: {{server}}"
```

### Implementation Complexity

**Estimated effort:** Medium

**Files to modify:**
1. `core/parser.sh` - Add condition block parsing (~100 lines)
2. `core/conditions.sh` - New file, condition evaluator (~150 lines)
3. `core/runner.sh` - Add condition execution (~80 lines)

**Challenges:**
- Parser needs to handle nested steps (steps within steps)
- May need recursive step execution
- Need to maintain step indices correctly
- Condition evaluation needs to be robust

**Alternative: Simple inline conditions first**

If nested steps parsing is too complex, start with simpler inline conditions:

```yaml
- input: text
  prompt: "Database host"
  if: "{{install_db}} == Yes"
  variable: "db_host"
```

This requires minimal parser changes - just check for `if` field on each step and evaluate before execution.

### Future Enhancements

**Logical operators:**
```yaml
if: "{{env}} == Production && {{region}} == EU"
if: "{{install_db}} == Yes || {{install_cache}} == Yes"
if: "!({{skip}} == true)"
```

**Switch/Case (separate feature):**
```yaml
- switch: "{{db_type}}"
  cases:
    MySQL:
      - output: info
        value: "MySQL selected"
    PostgreSQL:
      - output: info
        value: "PostgreSQL selected"
```
