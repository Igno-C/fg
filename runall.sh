#!/bin/bash

# Paths to Godot versions
GODOT_PATH_REGULAR="../Godot_v4.4.1"
GODOT_PATH_DEBUG="../Godot_debug_v4.4.1"

# Default to regular Godot
GODOT_PATH="$GODOT_PATH_REGULAR"

# Array to store PIDs of started processes
PIDS=()

# Function to handle script termination
cleanup() {
    echo -e "\nTerminating all Godot processes..."
    for pid in "${PIDS[@]}"; do
        echo "Terminating $pid"
        kill "$pid"
    done
    exit 0
}

# Trap signals to ensure cleanup
trap cleanup SIGINT SIGTERM

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)
            echo "Using debug build of Godot"
            GODOT_PATH="$GODOT_PATH_DEBUG"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug]"
            exit 1
            ;;
    esac
done

# Function to start a Godot instance and track its PID
start_godot() {
    local path="$1"
    gnome-terminal --disable-factory --title "$path" -- "$GODOT_PATH" --path "$path" --headless &
    echo "Registering $!"
    PIDS+=($!)
}

# Start Godot instances
start_godot "./fggateway/"
start_godot "./fgauth/"
start_godot "./fgdatabase/"
start_godot "./fgserver/"
#start_godot "./fgserver/"

# Start client (not headless)
gnome-terminal --disable-factory --title "Client" -- "$GODOT_PATH" --path "./fgclient/" &
echo "Registering $!"
PIDS+=($!)

# Busy wait
echo "All Godot instances started. Press Ctrl+C to terminate."
while true; do
    sleep 1
done

