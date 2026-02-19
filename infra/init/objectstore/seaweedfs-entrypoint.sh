#!/bin/sh
set -eu

ACCESS_KEY_ESCAPED=$(printf '%s' "$OBJECTSTORE_ACCESS_KEY" | sed 's/[&/]/\\&/g')
SECRET_KEY_ESCAPED=$(printf '%s' "$OBJECTSTORE_SECRET_KEY" | sed 's/[&/]/\\&/g')

sed \
  -e "s/__OBJECTSTORE_ACCESS_KEY__/${ACCESS_KEY_ESCAPED}/g" \
  -e "s/__OBJECTSTORE_SECRET_KEY__/${SECRET_KEY_ESCAPED}/g" \
  /etc/seaweedfs/s3-config.tmpl.json > /tmp/s3-config.json

exec weed server \
  -dir=/data \
  -ip.bind=0.0.0.0 \
  -volume.max=0 \
  -s3 \
  -s3.config=/tmp/s3-config.json
