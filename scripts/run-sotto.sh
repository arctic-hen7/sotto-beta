#!/bin/bash
# Executes Sotto on Linux and MacOS. This expects to be in the same directory as the `sotto-app` executable.

# Move into the directory this script is in
cd $(dirname "$0")
# And execute the app, making sure it knows about our Python virtual environment
PATH="venv/bin:$PATH" ./sotto-app
