#!/bin/bash
# optimize_wifi_plugin.sh - Helper script to optimize the WiFi plugin

set -e  # Exit immediately on any error

echo "Optimizing WiFi Plugin Code"
echo "=========================="

# Path to the WiFi plugin source file
WIFI_SRC_PATH="plugins/plugin_wifi/src/wifi_manager_cp.rs"

# Check if the file exists
if [ ! -f "$WIFI_SRC_PATH" ]; then
    echo "Error: WiFi plugin source file not found at $WIFI_SRC_PATH"
    exit 1
fi

# Make a backup of the file
cp "$WIFI_SRC_PATH" "${WIFI_SRC_PATH}.bak"
echo "Created backup at ${WIFI_SRC_PATH}.bak"

# We'll add #[allow(dead_code)] attributes to the unused functions
# This is a cleaner approach than removing them, as they might be needed later

# Process the file and add attributes where needed
awk '
BEGIN {
    functions_to_mark = "run_scan|security_type_to_string|channel_to_frequency|process_scan_results|parse_channel|parse_signal_level"
    in_function = 0
    current_function = ""
}

{
    # Check if this line is a function declaration we want to mark
    if ($0 ~ /^fn +[a-zA-Z0-9_]+\(/ && $0 !~ /\).*\{/ && $0 !~ /^#\[allow\(dead_code\)\]/) {
        # Extract the function name for multi-line function declarations
        match($0, /^fn +([a-zA-Z0-9_]+)\(/, arr)
        if (length(arr) > 1) {
            current_function = arr[1]
            if (current_function ~ functions_to_mark) {
                in_function = 1
                print "#[allow(dead_code)]"
                print $0
                next
            }
        }
    }
    # Check if this is a complete function declaration in a single line
    else if ($0 ~ /^fn +[a-zA-Z0-9_]+.*\{/ && $0 !~ /^#\[allow\(dead_code\)\]/) {
        match($0, /^fn +([a-zA-Z0-9_]+)/, arr)
        if (length(arr) > 1) {
            if (arr[1] ~ functions_to_mark) {
                print "#[allow(dead_code)]"
                print $0
                next
            }
        }
    }
    
    # Default: print the line unchanged
    print $0
}
' "${WIFI_SRC_PATH}.bak" > "$WIFI_SRC_PATH"

echo "Successfully added #[allow(dead_code)] attributes to unused functions in $WIFI_SRC_PATH"
echo ""
echo "Note: These functions are kept for cross-platform compatibility"
echo "but marked to prevent compiler warnings."
echo ""
echo "To test the changes, rebuild the project with:"
echo "./b_ios.sh [--release]"
echo ""
