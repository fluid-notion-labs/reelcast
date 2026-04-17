#!/usr/bin/env bash
set -euo pipefail

# Reelcast — self-signed TLS cert setup
# Run on the Ubuntu server once. Generates cert + key, prints Mac trust instructions.

CERT_DIR="${REELCAST_CERT_DIR:-$HOME/.config/reelcast}"
CERT="$CERT_DIR/cert.pem"
KEY="$CERT_DIR/key.pem"
DAYS=3650

echo "==> Reelcast TLS setup"

# Auto-detect LAN IP
LAN_IP=$(ip route get 1.1.1.1 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") print $(i+1)}' | head -1)
if [[ -z "$LAN_IP" ]]; then
    LAN_IP=$(hostname -I | awk '{print $1}')
fi

echo "    Detected LAN IP: $LAN_IP"
read -rp "    Use this IP? [Y/n] (or enter a different one): " INPUT
if [[ -n "$INPUT" && "$INPUT" != "y" && "$INPUT" != "Y" ]]; then
    LAN_IP="$INPUT"
fi

echo "    Cert dir: $CERT_DIR"
mkdir -p "$CERT_DIR"
chmod 700 "$CERT_DIR"

echo "==> Generating self-signed cert (${DAYS} days)..."
openssl req -x509 \
    -newkey rsa:4096 \
    -keyout "$KEY" \
    -out "$CERT" \
    -days "$DAYS" \
    -nodes \
    -subj "/CN=reelcast/O=reelcast/C=AU" \
    -addext "subjectAltName=IP:${LAN_IP},IP:127.0.0.1,DNS:localhost"

chmod 600 "$KEY"
chmod 644 "$CERT"

echo ""
echo "==> Done."
echo ""
echo "    cert: $CERT"
echo "    key:  $KEY"
echo ""
echo "==> Start reelcast with TLS:"
echo ""
echo "    reelcast -l /path/to/movies --cert $CERT --key $KEY"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  macOS: trust the cert (one time only)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  1. Copy cert to your Mac:"
echo "     scp ${USER}@${LAN_IP}:${CERT} ~/Downloads/reelcast.pem"
echo ""
echo "  2. Double-click ~/Downloads/reelcast.pem"
echo "     → Keychain Access opens, add to System keychain"
echo ""
echo "  3. Find 'reelcast' in Keychain → Get Info → Trust → Always Trust"
echo ""
echo "  4. Open https://${LAN_IP}:3000"
echo ""
