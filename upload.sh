#!/bin/sh
set -e

if [ $# -ne 2 ]; then
    echo "usage: $0 <file> <name>"
    exit 1
fi

FILE="$1"
NAME="$2"

if [ ! -f "$FILE" ]; then
    echo "error: '$FILE' not found"
    exit 1
fi

HASH=$(sha256sum "$FILE" | cut -d' ' -f1)
UPLOAD_DIR="uploads/$HASH"

mkdir -p "$UPLOAD_DIR"
cp "$FILE" "$UPLOAD_DIR/raw"

DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
printf '{"upload_date":"%s","original_name":"%s","uploader":"local"}\n' "$DATE" "$NAME" > "$UPLOAD_DIR/metadata.json"

echo "uploaded: /file/$HASH"
