#!/bin/bash
# usage: ./tests/setup.sh [config_path]

set -e

CONFIG_PATH="${1:-data/config.toml}"

# Extract sqlite database path from config
DB_PATH=$(grep -A1 '^\[database\]' "$CONFIG_PATH" | grep 'path' | sed 's/.*= *"\(.*\)"/\1/')

if [ -z "$DB_PATH" ]; then
    echo "Error: Could not find database path in $CONFIG_PATH"
    exit 1
fi

echo "Setting up test database at: $DB_PATH"

SERVER_SECRET="server_secret"
sqlite3 "$DB_PATH" <<EOF
INSERT OR REPLACE INTO settings (key, value)
VALUES ('server_secret', '$SERVER_SECRET');
EOF

# Hashed value of "secret" using argon2
HASHED_SECRET='$argon2id$v=19$m=19456,t=2,p=1$91LG/a4lfzNcoXmXacA7TQ$rrxv/GKqKW9NiORIqs5A+BZxYdqPTkMUlBnxT9CSM2M'

sqlite3 "$DB_PATH" <<EOF
INSERT OR REPLACE INTO keys (access, secret, description)
VALUES ('access', '$HASHED_SECRET', 'test credentials');
EOF

echo "You can now run tests with: hurl --test tests/*.hurl"
