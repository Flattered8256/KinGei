#!/bin/sh
set -e

SECRET_FILE="${JWT_SECRET_FILE:-/kingei-data/jwt_secret}"

if [ -z "$JWT_SECRET" ]; then
  if [ -f "$SECRET_FILE" ] && [ -s "$SECRET_FILE" ]; then
    JWT_SECRET=$(cat "$SECRET_FILE")
  else
    mkdir -p "$(dirname "$SECRET_FILE")"
    JWT_SECRET=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
    umask 077
    printf '%s' "$JWT_SECRET" > "$SECRET_FILE"
  fi
  export JWT_SECRET
fi

exec "$@"
