#!/bin/bash

set -euo pipefail

APP_PATH=./target/debug/sailboat

# Setup app cleanup
function cleanup {
  if [[ ! -z ${APP_PID+x} ]]; then
    echo "Exiting server"
    kill "$APP_PID"
  else
    echo "No running server found; exiting"
  fi
}
trap cleanup EXIT

# Start the app and get the PID
"$APP_PATH" &
APP_PID=$!

sleep 1

node --test

# At the end of this script, the cleanup function will run
