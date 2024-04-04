#!/bin/bash
# If you have ngrok setup, this script will run sailboat with it
set -euo pipefail

# Setup app cleanup
function cleanup {
  if [[ ! -z ${NGROK_PID+x} ]]; then
    echo "Exiting server"
    kill "$NGROK_PID"
  else
    echo "No running server found; exiting"
  fi
}
trap cleanup EXIT

# Start ngrok and hide the output
ngrok http 3000 --log=stdout > /dev/null &
NGROK_PID=$!
echo started ngrok
sleep .5

export SB_DOMAIN=$(curl localhost:4040/api/tunnels | sed -nr 's/.*"public_url":"https:\/\/([^"]*)".*/\1/p')
echo Ngrok running at $SB_DOMAIN
cargo watch -x run

# At the end of this script, the cleanup function will run
