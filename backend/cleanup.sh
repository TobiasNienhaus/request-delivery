#!/bin/sh

validate_cli() {
    if [ -z $2 ]; then
        echo "$1"
        exit 1
    fi
}

validate_cli "Missing token file path" $1

token=$(cat $1)
validate_cli "Missing token" $token

validate_cli "Missing backend base URI" $2

echo "$2/admin/cleanup"

if [ $(curl -X "DELETE" -L -s -w "%{http_code}" "$2/admin/cleanup" -H "X-Auth: $token") -eq 202 ]; then
    echo "Cleared successfully!"
else
    echo "Failed to clear Tokens!"
fi
