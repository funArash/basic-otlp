#!/usr/bin/env sh

set -xe

# Create a new "root" CA (Certificate of Authority)
openssl req -nodes \
          -x509 \
          -days 3650 \
          -newkey rsa:4096 \
          -keyout ca.key \
          -out ca.cert \
          -sha256 \
          -batch \
          -subj "/CN=ponytown RSA CA"

# Create a new level 2 intermediate Cert and Key
openssl req -nodes \
          -newkey rsa:3072 \
          -keyout inter.key \
          -out inter.req \
          -sha256 \
          -batch \
          -subj "/CN=ponytown RSA level 2 intermediate"

# Create a new Server Cert and Key
openssl req -nodes \
          -newkey rsa:2048 \
          -keyout server.key \
          -out server.req \
          -sha256 \
          -batch \
          -subj "/CN=testserver.com"

chmod +r server.key

# openssl rsa \
#           -in server.key \
#           -out server.rsa

# Create a new Client Cert and Key
openssl req -nodes \
          -newkey rsa:2048 \
          -keyout client.key \
          -out client.req \
          -sha256 \
          -batch \
          -subj "/CN=ponytown client"

# openssl rsa \
#           -in client.key \
#           -out client.rsa

# ------------- Signing of the Certs and the level 2 intermediate CA -----------------

# Signing of the level 2 intermediate CA with the new Root CA
openssl x509 -req \
          -in inter.req \
          -out inter.cert \
          -CA ca.cert \
          -CAkey ca.key \
          -sha256 \
          -days 3650 \
          -set_serial 123 \
          -extensions v3_inter -extfile openssl.cnf

# Signing of Server Cert with the new level 2 intermediate CA
openssl x509 -req \
          -in server.req \
          -out server.cert \
          -CA inter.cert \
          -CAkey inter.key \
          -sha256 \
          -days 2000 \
          -set_serial 456 \
          -extensions v3_end -extfile openssl.cnf

# Signing of Client Cert with the new level 2 intermediate CA
openssl x509 -req \
          -in client.req \
          -out client.cert \
          -CA inter.cert \
          -CAkey inter.key \
          -sha256 \
          -days 2000 \
          -set_serial 789 \
          -extensions v3_client -extfile openssl.cnf

# # Packaging..
# cat inter.cert ca.cert > server.chain
# cat server.cert inter.cert ca.cert > server.fullchain

# cat inter.cert ca.cert > client.chain
# cat client.cert inter.cert ca.cert > client.fullchain

# openssl asn1parse -in ca.cert -out ca.der > /dev/null
