#!/usr/bin/env bash
# Locks a deployed collection by deleting its last access key. Plan section 9, Phase 2, steps 8 and 9.
#   scripts/lock.sh testnet <collection-id> [--dry-run]
# Testnet only: mainnet is refused. Before it deletes anything it checks that the collection has
# its vault code set, that the test vault bv-1 exists, and that bv-1 has no access keys. It then
# deletes the collection's only key with the near CLI (signed from the macOS keychain), confirms
# that `near account list-keys` is empty, and prints the code hash. --dry-run runs the checks
# and signs nothing. Running it on a collection that is already locked changes nothing.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck disable=SC1091
source scripts/env.sh

NET=${1:?usage: lock.sh testnet <collection-id> [--dry-run]}
COLL=${2:?usage: lock.sh testnet <collection-id> [--dry-run]}
DRY=${3:-}
case "$NET" in
  testnet) ;;
  mainnet) echo "Refused: lock.sh does not run on mainnet." >&2; exit 1 ;;
  *) echo "The network must be testnet." >&2; exit 1 ;;
esac
case "$COLL" in *.testnet) ;; *) echo "Refused: $COLL is not a .testnet account." >&2; exit 1 ;; esac
case "$DRY" in ""|--dry-run) ;; *) echo "Unknown option: $DRY" >&2; exit 1 ;; esac

keys_of() { near account list-keys "$1" network-config "$NET" now 2>&1; }
pubkeys() { grep -o 'ed25519:[1-9A-HJ-NP-Za-km-z]*' || true; }
code_hash() {
  near account view-account-summary "$COLL" network-config "$NET" now 2>&1 | grep "Local Contract" | awk '{print $NF}'
}
fail() { echo "FAIL: $1" >&2; exit 1; }

# 1. The collection has its code, its vault code is published, and the test vault is sound.
INFO=$(near contract call-function as-read-only "$COLL" contract_info json-args '{}' network-config "$NET" now 2>&1) \
  || fail "contract_info did not answer on $COLL."
echo "$INFO"
echo "$INFO" | grep -Eq '"vault_code_hash": "[0-9a-f]{64}"' || fail "$COLL has no vault code hash. Run deploy.sh first."
VINFO=$(near contract call-function as-read-only "$COLL" vault_info json-args '{"token_id":"bv-1"}' network-config "$NET" now 2>&1) \
  || fail "vault_info did not answer."
echo "$VINFO" | grep -q '"vault_id"' || fail "the test vault bv-1 does not exist on $COLL. Run deploy.sh first."
VKEYS=$(keys_of "bv-1.$COLL")
echo "$VKEYS" | grep -q '^+---' || fail "could not read the key list of bv-1.$COLL."
[ -z "$(echo "$VKEYS" | pubkeys)" ] || fail "the test vault bv-1.$COLL has an access key."
echo "ok: test vault bv-1.$COLL exists and has 0 access keys"

# 2. The collection's own keys.
KEYS=$(keys_of "$COLL")
echo "$KEYS" | grep -q '^+---' || fail "could not read the key list of $COLL."
COUNT=$(echo "$KEYS" | pubkeys | wc -l | tr -d ' ')
if [ "$COUNT" = 0 ]; then
  echo "$COLL is already locked: 0 access keys."
  echo "Code hash: $(code_hash)"
  exit 0
fi
[ "$COUNT" = 1 ] || fail "$COLL has $COUNT access keys, expected exactly 1. Delete nothing; check by hand."
KEY=$(echo "$KEYS" | pubkeys)
echo "$COLL has one access key: $KEY"
if [ "$DRY" = "--dry-run" ]; then echo "Dry run: nothing signed."; exit 0; fi

# 3. Delete the last key, then confirm.
near account delete-keys "$COLL" public-keys "$KEY" network-config "$NET" sign-with-keychain send
AFTER=""
for _ in $(seq 1 15); do
  AFTER=$(keys_of "$COLL")
  if echo "$AFTER" | grep -q '^+---' && [ -z "$(echo "$AFTER" | pubkeys)" ]; then break; fi
  sleep 2
done
echo "$AFTER"
[ -z "$(echo "$AFTER" | pubkeys)" ] || fail "$COLL still has an access key."
HASH=$(code_hash)
[ -n "$HASH" ] || fail "could not read the code hash."
echo "LOCK PASS: $COLL has 0 access keys on $NET. Code hash $HASH."
