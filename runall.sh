#!/bin/bash

# Paths to Godot versions
GODOT_PATH_REGULAR="../Godot_v4.4.1"
GODOT_PATH_DEBUG="../Godot_debug_v4.4.1"

# Default to regular Godot
GODOT_PATH="$GODOT_PATH_REGULAR"

# Default number of clients
NUM_CLIENTS=1

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
        --clients)
            NUM_CLIENTS="$2"
            shift 2
            ;;
        -c)
            NUM_CLIENTS="$2"
            shift 2
            ;;
        -*)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug] [--clients <number>]"
            exit 1
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug] [--clients <number>]"
            exit 1
            ;;
    esac
done

# Validate number of clients
if ! [[ "$NUM_CLIENTS" =~ ^[0-9]+$ ]] || [ "$NUM_CLIENTS" -le 0 ]; then
    echo "Error: Number of clients must be a positive integer"
    exit 1
fi

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

# Start specified number of client instances
for ((i=1; i<=NUM_CLIENTS; i++)); do
    if [ $NUM_CLIENTS -eq 1 ]; then
        # If only one client, use the default title
        gnome-terminal --disable-factory --title "Client" -- "$GODOT_PATH" --path "./fgclient/" &
    else
        # If multiple clients, add a number to the title
        gnome-terminal --disable-factory --title "Client-$i" -- "$GODOT_PATH" --path "./fgclient/" &
    fi
    echo "Registering $!"
    PIDS+=($!)
done

# Busy wait
echo "All Godot instances started. Press Ctrl+C to terminate."
while true; do
    sleep 1
done
