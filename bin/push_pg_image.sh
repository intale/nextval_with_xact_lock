#!/bin/bash

PGVER="${PGVER:-18}"
TAG="${TAG:-idzyzenko/postresql-nextval_with_xact_lock:$PGVER}"

echo "Pushing $TAG"
docker push "$TAG"
