#!/usr/bin/env sh

set -xe

openssl req -nodes \
          -x509 \
          -days 3650 \
          -newkey rsa:4096 \
          -keyout ca.key \
          -out ca.cert \
          -sha256 \
          -batch \
          -subj "/CN=ponytown RSA CA"

openssl req -nodes \
          -newkey rsa:3072 \
          -keyout inter.key \
          -out inter.req \
          -sha256 \
          -batch \
          -subj "/CN=ponytown RSA level 2 intermediate"

openssl req -nodes \
          -newkey rsa:2048 \
          -keyout server.key \
          -out server.req \
          -sha256 \
          -batch \
          -subj "/CN=testserver.com"

openssl rsa \
          -in server.key \
          -out server.rsa

openssl req -nodes \
          -newkey rsa:2048 \
          -keyout client.key \
          -out client.req \
          -sha256 \
          -batch \
          -subj "/CN=ponytown client"

openssl rsa \
          -in client.key \
          -out client.rsa

# ----------------------------------------------

openssl x509 -req \
          -in inter.req \
          -out inter.cert \
          -CA ca.cert \
          -CAkey ca.key \
          -sha256 \
          -days 3650 \
          -set_serial 123 \
          -extensions v3_inter -extfile openssl.cnf

openssl x509 -req \
          -in server.req \
          -out server.cert \
          -CA inter.cert \
          -CAkey inter.key \
          -sha256 \
          -days 2000 \
          -set_serial 456 \
          -extensions v3_end -extfile openssl.cnf

openssl x509 -req \
          -in client.req \
          -out client.cert \
          -CA inter.cert \
          -CAkey inter.key \
          -sha256 \
          -days 2000 \
          -set_serial 789 \
          -extensions v3_client -extfile openssl.cnf

cat inter.cert ca.cert > server.chain
cat server.cert inter.cert ca.cert > server.fullchain

cat inter.cert ca.cert > client.chain
cat client.cert inter.cert ca.cert > client.fullchain

openssl asn1parse -in ca.cert -out ca.der > /dev/null
