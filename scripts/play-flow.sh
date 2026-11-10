#!/usr/bin/env bash
# Runs the whole play-money flow on testnet, the way a visitor and a family would. Plan Phase 2.
#   scripts/play-flow.sh testnet <collection-id>
# A pool key starts a demo on the hub, which creates and funds three throwaway accounts. The
# guardian mints a vault with a short lock and sends a play gift. An early withdrawal must fail.
# After the unlock the guardian withdraws part, the child claims the NFT, and the child withdraws
# the rest. The throwaway keys live only in a private temp folder and are never printed.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck disable=SC1091
source scripts/env.sh
[ "${1:-}" = testnet ] || { echo "Refused: the play flow runs on testnet only." >&2; exit 1; }
COLL=${2:?usage: play-flow.sh testnet <collection-id>}
set -a
# shellcheck disable=SC1091
source .env.render.local
set +a
HUB=$NEXT_PUBLIC_HUB_ID
USDC=$NEXT_PUBLIC_USDC_ID
POOL_SK=${NEXT_PUBLIC_HUB_KEYS%%,*}
TMP=$(mktemp -d)
chmod 700 "$TMP"
trap 'rm -rf "$TMP"' EXIT
LOG="$TMP/near.log"

for who in guardian child buyer; do python3 -I scripts/ed25519_pair.py > "$TMP/$who"; done
pk() { cut -d' ' -f1 "$TMP/$1"; }
sk() { cut -d' ' -f2 "$TMP/$1"; }
implicit() {
  python3 -I -c "
import sys
a='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
s=sys.argv[1].split(':')[1]; n=0
for c in s: n=n*58+a.index(c)
print(n.to_bytes(32,'big').hex())" "$(pk "$1")"
}
# call <signer-id> <signer-key> <contract> <method> <json> <deposit> <gas>
call() {
  near contract call-function as-transaction "$3" "$4" json-args "$5" prepaid-gas "$7" attached-deposit "$6" \
    sign-as "$1" network-config testnet sign-with-plaintext-private-key "$2" send >>"$LOG" 2>&1
}
view() { near contract call-function as-read-only "$1" "$2" json-args "$3" network-config testnet now 2>/dev/null; }
balance() { view "$USDC" ft_balance_of "{\"account_id\":\"$1\"}" | tr -d '"[:space:]'; }
fail() { echo "FAIL: $1" >&2; grep -v -e plaintext-private-key -e "Signature:" "$LOG" | tail -20 >&2; exit 1; }
expect_balance() {
  for _ in $(seq 1 20); do [ "$(balance "$1")" = "$2" ] && { echo "ok: $3 = $2"; return; }; sleep 2; done
  fail "$3 is $(balance "$1"), expected $2"
}

G=$(implicit guardian); C=$(implicit child); B=$(implicit buyer)
echo "1. A pool key starts a demo on $HUB"
call "$HUB" "$POOL_SK" "$HUB" start_demo \
  "{\"guardian_pk\":\"$(pk guardian)\",\"child_pk\":\"$(pk child)\",\"buyer_pk\":\"$(pk buyer)\"}" '0 NEAR' '100 Tgas' \
  || fail "start_demo"
expect_balance "$G" 5000000 "guardian play USDC"

SERIAL=$(view "$COLL" contract_info '{}' | python3 -I -c "import json,sys; print(json.load(sys.stdin)['next_serial'])")
TOKEN="bv-$SERIAL"
VAULT="$TOKEN.$COLL"
UNLOCK=$(python3 -I -c "import time; print(time.time_ns() + 60_000_000_000)")
echo "2. The guardian mints $TOKEN for the child, locked for 60 seconds"
call "$G" "$(sk guardian)" "$COLL" mint_vault \
  "{\"child_id\":\"$C\",\"unlock_ns\":\"$UNLOCK\",\"title\":\"Play flow check\"}" '0.025 NEAR' '100 Tgas' || fail "mint_vault"
view "$COLL" vault_info "{\"token_id\":\"$TOKEN\"}" | grep -q "\"$G\"" || fail "vault_info does not name the guardian"

echo "3. The guardian sends a play gift of 1,000,000 units"
call "$G" "$(sk guardian)" "$USDC" ft_transfer "{\"receiver_id\":\"$VAULT\",\"amount\":\"1000000\"}" '1 yoctoNEAR' '30 Tgas' \
  || fail "ft_transfer to the vault"
expect_balance "$VAULT" 1000000 "vault"

echo "4. An early withdrawal must fail"
if call "$G" "$(sk guardian)" "$COLL" withdraw "{\"token_id\":\"$TOKEN\",\"ft_contract\":\"$USDC\",\"amount\":\"250000\"}" '1 yoctoNEAR' '100 Tgas'; then
  fail "the early withdrawal succeeded"
fi
grep -q "locked" "$LOG" || fail "the early withdrawal failed for another reason"
echo "ok: refused with 'locked'"
expect_balance "$VAULT" 1000000 "vault after the refused withdrawal"

echo "5. Waiting for the unlock"
while [ "$(python3 -I -c 'import time; print(time.time_ns())')" -lt "$((UNLOCK + 5000000000))" ]; do sleep 3; done

echo "6. The guardian withdraws 250,000"
call "$G" "$(sk guardian)" "$COLL" withdraw "{\"token_id\":\"$TOKEN\",\"ft_contract\":\"$USDC\",\"amount\":\"250000\"}" '1 yoctoNEAR' '100 Tgas' \
  || fail "the guardian's withdrawal"
expect_balance "$G" 4250000 "guardian play USDC"

echo "7. The child claims the NFT"
call "$C" "$(sk child)" "$COLL" claim_as_child "{\"token_id\":\"$TOKEN\"}" '1 yoctoNEAR' '30 Tgas' || fail "claim_as_child"
view "$COLL" vault_info "{\"token_id\":\"$TOKEN\"}" | grep -q "\"owner_id\": \"$C\"" || fail "the child does not own the vault"

echo "8. The child withdraws the remaining 750,000"
call "$C" "$(sk child)" "$COLL" withdraw "{\"token_id\":\"$TOKEN\",\"ft_contract\":\"$USDC\",\"amount\":\"750000\"}" '1 yoctoNEAR' '100 Tgas' \
  || fail "the child's withdrawal"
expect_balance "$C" 750000 "child play USDC"
expect_balance "$VAULT" 0 "vault"
echo "PLAY FLOW PASS: $TOKEN on $COLL. Guardian $G, child $C, buyer $B."
