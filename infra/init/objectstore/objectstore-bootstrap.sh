#!/bin/sh
set -eu

IFS=','
for bucket in $OBJECTSTORE_BUCKETS; do
  aws --endpoint-url "$OBJECTSTORE_ENDPOINT" s3api head-bucket --bucket "$bucket" >/dev/null 2>&1 \
    || aws --endpoint-url "$OBJECTSTORE_ENDPOINT" s3api create-bucket --bucket "$bucket" >/dev/null
done

echo "Object storage buckets initialized."
