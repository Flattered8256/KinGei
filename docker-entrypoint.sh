#!/bin/sh
set -e

mkdir -p /kingei-data/database/sqlite /kingei-data/files

if [ -z "$DATABASE_URL" ]; then
  if [ "$KINGEI_DB_BACKEND" = "postgres" ]; then
    echo "缺少 DATABASE_URL" >&2
    exit 1
  fi
  DATABASE_URL="sqlite:///kingei-data/database/sqlite/kinGei.db"
  export DATABASE_URL
fi

if [ -z "$STORAGE_DRIVER" ]; then
  STORAGE_DRIVER=local
  export STORAGE_DRIVER
fi

if [ -z "$STORAGE_LOCAL_DIR" ]; then
  STORAGE_LOCAL_DIR=/kingei-data/files
  export STORAGE_LOCAL_DIR
fi

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
