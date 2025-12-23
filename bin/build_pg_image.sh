#!/bin/bash

PGVER="${PGVER:-18}"
TAG="${TAG:-idzyzenko/postresql-nextval_with_xact_lock:$PGVER}"

echo "Building $TAG image for PostgreSQL v$PGVER"
docker build . -t "$TAG" --build-arg PGVER="$PGVER"
