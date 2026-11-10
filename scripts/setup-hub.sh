#!/usr/bin/env bash
# Deploys the testnet demo hub and its play USDC. Plan section 9, Phase 2. Testnet only.
#   scripts/setup-hub.sh <funder.testnet> [--dry-run | --redeploy-code]
# Creates tusdc.<funder> and hub.<funder>, adds a pool of function-call keys that may call only
# start_demo on the hub, and writes the hub settings and the pool to .env.render.local, which
# git ignores. The pool keys are public by design: the testnet page ships them, and they can
# only start capped demos paid from the hub's testnet balance.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck disable=SC1091
source scripts/env.sh

FUNDER=${1:?usage: setup-hub.sh <funder.testnet> [--dry-run | --redeploy-code]}
DRY=${2:-}
case "$FUNDER" in *.testnet) ;; *) echo "Refused: the demo hub is testnet only." >&2; exit 1 ;; esac
NET=testnet
TUSDC="tusdc.$FUNDER"
HUB="hub.$FUNDER"
DEMO_FUNDS=1.5
POOL=8
send() { near "$@" network-config "$NET" sign-with-keychain send; }

for c in tusdc demo-hub; do
  cargo near build non-reproducible-wasm --locked --no-abi --manifest-path "contracts/$c/Cargo.toml" >/dev/null 2>&1
done
TW=target/near/tusdc/tusdc.wasm
HW=target/near/demo_hub/demo_hub.wasm
TB=$(wc -c < "$TW" | tr -d ' ')
HB=$(wc -c < "$HW" | tr -d ' ')
TF=$(python3 -I -c "print(f'{$TB * 0.00001 + 0.3:.5f}')")
HF=$(python3 -I -c "print(f'{$HB * 0.00001 + 0.3 + $DEMO_FUNDS:.5f}')")
echo "tUSDC $TUSDC: $TB bytes, funded with $TF NEAR"
echo "Hub $HUB: $HB bytes, funded with $HF NEAR, of which $DEMO_FUNDS NEAR pays for about $(python3 -I -c "print(int($DEMO_FUNDS / 0.39))") demos at 0.39 NEAR each"
echo "Key pool: $POOL keys, each limited to start_demo with a 1 NEAR gas allowance"
if [ "$DRY" = "--dry-run" ]; then echo "Dry run: nothing signed."; exit 0; fi
if [ "$DRY" = "--redeploy-code" ]; then
  # Replace the hub's code and keep its state, for a constant change. tUSDC is untouched.
  send contract deploy "$HUB" use-file "$HW" without-init-call
  echo "HUB REDEPLOY PASS: new code on $HUB, state kept."
  exit 0
fi

send account create-account fund-myself "$TUSDC" "$TF NEAR" autogenerate-new-keypair save-to-keychain sign-as "$FUNDER"
send contract deploy "$TUSDC" use-file "$TW" with-init-call new json-args "{\"hub_id\":\"$HUB\"}" \
  prepaid-gas '100 Tgas' attached-deposit '0 NEAR'
send account create-account fund-myself "$HUB" "$HF NEAR" autogenerate-new-keypair save-to-keychain sign-as "$FUNDER"
send contract deploy "$HUB" use-file "$HW" with-init-call new \
  json-args "{\"usdc\":\"$TUSDC\",\"hour_ns\":\"3600000000000\",\"day_ns\":\"86400000000000\",\"hourly_limit\":20,\"daily_limit\":100}" \
  prepaid-gas '100 Tgas' attached-deposit '0 NEAR'

SECRETS=""
for _ in $(seq 1 "$POOL"); do
  PAIR=$(python3 -I scripts/ed25519_pair.py)
  send account add-key "$HUB" grant-function-call-access --allowance '1 NEAR' \
    --contract-account-id "$HUB" --function-names start_demo use-manually-provided-public-key "${PAIR%% *}"
  SECRETS="${SECRETS:+$SECRETS,}${PAIR#* }"
done
umask 077
{
  echo "NEXT_PUBLIC_NETWORK=testnet"
  echo "NEXT_PUBLIC_HUB_ID=$HUB"
  echo "NEXT_PUBLIC_USDC_ID=$TUSDC"
  echo "NEXT_PUBLIC_HUB_KEYS=$SECRETS"
} > .env.render.local
echo "HUB SETUP PASS: $HUB and $TUSDC are live on testnet. The pool of $POOL keys is in .env.render.local."
