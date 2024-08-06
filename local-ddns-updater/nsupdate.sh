#!/bin/sh -e
# This is just an example.
DOMAIN="server.example.com"
KEYFILE="HMAC-SHA256:XXXXX"

addr=$1

nsupdate -y ${KEYFILE} <<EOF
update delete ${DOMAIN} AAAA
update add ${DOMAIN} 300 AAAA $addr
send
EOF
