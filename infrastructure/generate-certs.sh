#!/bin/bash
# Generate self-signed TLS certificates for OTEL Collector development
# Usage: ./generate-certs.sh
# Output: infrastructure/certs/ directory with CA, server, and client certs

set -euo pipefail

CERTS_DIR="$(dirname "$0")/certs"
mkdir -p "$CERTS_DIR"

cd "$CERTS_DIR"

echo "==> Generating CA certificate..."
openssl req -new -x509 -days 3650 -nodes \
  -subj "/CN=HoneyLink Root CA/O=HoneyLink Dev/C=US" \
  -keyout ca.key \
  -out ca.crt

echo "==> Generating server certificate..."
openssl req -new -nodes \
  -subj "/CN=otel-collector/O=HoneyLink Dev/C=US" \
  -keyout server.key \
  -out server.csr

# Server certificate SAN (Subject Alternative Names) for localhost and Docker
cat > server-ext.cnf <<EOF
subjectAltName = DNS:otel-collector,DNS:localhost,IP:127.0.0.1,IP:172.28.0.2
extendedKeyUsage = serverAuth
EOF

openssl x509 -req -in server.csr -days 365 \
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out server.crt \
  -extfile server-ext.cnf

echo "==> Generating client certificate..."
openssl req -new -nodes \
  -subj "/CN=honeylink-telemetry/O=HoneyLink Dev/C=US" \
  -keyout client.key \
  -out client.csr

cat > client-ext.cnf <<EOF
extendedKeyUsage = clientAuth
EOF

openssl x509 -req -in client.csr -days 365 \
  -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out client.crt \
  -extfile client-ext.cnf

# Set secure permissions
chmod 600 *.key
chmod 644 *.crt

# Cleanup temporary files
rm -f *.csr *.cnf *.srl

echo "==> Certificates generated successfully:"
echo "    CA Certificate:     $CERTS_DIR/ca.crt"
echo "    Server Certificate: $CERTS_DIR/server.crt"
echo "    Server Key:         $CERTS_DIR/server.key"
echo "    Client Certificate: $CERTS_DIR/client.crt"
echo "    Client Key:         $CERTS_DIR/client.key"
echo ""
echo "==> Certificate details:"
openssl x509 -in server.crt -noout -text | grep -A 2 "Subject Alternative Name"
echo ""
echo "==> Next steps:"
echo "    1. Copy ca.crt to honeylink-telemetry crate config"
echo "    2. Run: docker-compose -f infrastructure/docker-compose.observability.yml up -d"
echo "    3. Verify OTLP endpoint: curl -k https://localhost:4317"
