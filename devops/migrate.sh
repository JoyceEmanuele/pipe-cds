#!/bin/bash

# CREDS = postgres://postgres:uniquepassword@127.0.0.1:5432/centraltelemetrias
CREDS=$(jq -r '.POSTGRES_DATABASE_URL' ./configfile_example.json5)
if [[ -e "./configfile.json5" ]]; then
  CREDS=$(jq -r '.POSTGRES_DATABASE_URL' ./configfile.json5)
fi

diesel migration run --database-url $CREDS
