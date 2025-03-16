#!/bin/bash
# Script to generate self-signed TLS certificates for development

# Create directory for certificates if it doesn't exist
mkdir -p keys/tls

# Generate a private key
openssl genrsa -out keys/tls/key.pem 2048

# Generate a certificate signing request (CSR)
openssl req -new -key keys/tls/key.pem -out keys/tls/csr.pem -subj "/CN=localhost/O=Wordle Development/C=US"

# Generate a self-signed certificate valid for 365 days
openssl x509 -req -days 365 -in keys/tls/csr.pem -signkey keys/tls/key.pem -out keys/tls/certificate.pem

# Clean up the CSR
rm keys/tls/csr.pem

# Set appropriate permissions
chmod 600 keys/tls/key.pem
chmod 644 keys/tls/certificate.pem

echo "Self-signed TLS certificates have been generated:"
echo "- Private key: keys/tls/key.pem"
echo "- Certificate: keys/tls/certificate.pem"
echo ""
echo "Note: These certificates are self-signed and intended for development only."
echo "For production, use certificates from a trusted certificate authority."
