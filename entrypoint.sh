#!/bin/sh
mkdir -p $XDG_CONFIG_HOME/sampic
CONFIG_LOCATION="$XDG_CONFIG_HOME/sampic/sampic.toml"

cat >$CONFIG_LOCATION <<EOF
storage = 'Local'
api_key = '$API_KEY'
api_secret_key = '$SECRET_KEY'
region = 'fr-par'
endpoint = 'https://s3.fr-par.scw.cloud'
bucket = 'sampic-store'
local_path = '/tmp/'
sampic_endpoint = 'example.com'
EOF
cat $CONFIG_LOCATION
./sampic server
