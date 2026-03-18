#!/bin/bash
set -e

# Make plugins available
mkdir -p /app/Plugins
if [ -d "/config/Plugins" ]; then
  cp -r /config/Plugins/* /app/Plugins/
fi

# Link config
if [ -f "/config/config.json" ]; then
  rm -f /app/config.json
  ln -s /config/config.json /app/config.json
fi

exec dotnet neo-cli.dll "$@"
