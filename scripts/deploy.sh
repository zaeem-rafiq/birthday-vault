#!/usr/bin/env bash
# Deploys one Birthday Vault collection. Plan section 9, Phase 2.
#   scripts/deploy.sh <testnet|mainnet> <funder> <collection-name> [--dry-run]
# Uses only the near CLI, so signing keys stay in the macOS keychain. --dry-run builds,
# prints sizes and costs, and signs nothing. Mainnet is refused unless scripts/mainnet.sh
# runs it for the owner with OWNER_RUN=1. On testnet, scripts/setup-hub.sh must run first,
# because the collection uses tusdc.<funder> as its USDC.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck disable=SC1091
source scripts/env.sh

NET=${1:?usage: deploy.sh <testnet|mainnet> <funder> <collection-name> [--dry-run]}
FUNDER=${2:?funder account}
NAME=${3:?collection name, for example g1}
DRY=${4:-}
case "$NET" in
  testnet)
    MODE=Demo; MIN_NS=30000000000; MAX_NS=3600000000000; USDC="tusdc.$FUNDER" ;;
  mainnet)
    MODE=Production; MIN_NS=86400000000000; MAX_NS=1262304000000000000
    USDC=17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1
    if [ "$DRY" != "--dry-run" ] && [ "${OWNER_RUN:-}" != 1 ]; then
      echo "Refused: a mainnet deploy runs only through scripts/mainnet.sh, by the owner." >&2
      exit 1
    fi ;;
  *) echo "The network must be testnet or mainnet." >&2; exit 1 ;;
esac
COLL="$NAME.$FUNDER"
send() { near "$@" network-config "$NET" sign-with-keychain send; }
view() { near contract call-function as-read-only "$COLL" "$1" json-args "$2" network-config "$NET" now; }

# 1. Build every contract the collection needs, and print sizes and costs.
cargo build --locked -q -p vault --target wasm32-unknown-unknown --release
mkdir -p target/near/vault
cp target/wasm32-unknown-unknown/release/vault.wasm target/near/vault/vault.wasm
cargo near build non-reproducible-wasm --locked --no-abi --manifest-path contracts/collection/Cargo.toml >/dev/null 2>&1
CW=target/near/collection/collection.wasm
VW=target/near/vault/vault.wasm
CB=$(wc -c < "$CW" | tr -d ' ')
VB=$(wc -c < "$VW" | tr -d ' ')
FUND=$(python3 -I -c "print(f'{$CB * 0.00001 + $VB * 0.0001 + 0.5:.5f}')")
echo "Collection $COLL on $NET, mode $MODE, USDC $USDC"
echo "collection.wasm $CB bytes, vault.wasm $VB bytes"
echo "Funding the collection: $FUND NEAR ($CB x 0.00001 + $VB x 0.0001 + 0.5). Test mint: 0.025 NEAR."
if [ "$DRY" = "--dry-run" ]; then echo "Dry run: nothing signed."; exit 0; fi

# 2. Create the collection account, funded by the funder.
send account create-account fund-myself "$COLL" "$FUND NEAR" autogenerate-new-keypair save-to-keychain sign-as "$FUNDER"

# 3. Deploy and initialize in one transaction.
send contract deploy "$COLL" use-file "$CW" with-init-call new \
  json-args "{\"usdc\":\"$USDC\",\"mode\":\"$MODE\",\"min_lock_ns\":\"$MIN_NS\",\"max_lock_ns\":\"$MAX_NS\"}" \
  prepaid-gas '100 Tgas' attached-deposit '0 NEAR'

# 4. Borsh arguments for set_vault_code: a little-endian u32 length, then the vault bytes.
ARGS=$(mktemp -t vault-args)
python3 -I -c "import struct,sys; d=open(sys.argv[1],'rb').read(); open(sys.argv[2],'wb').write(struct.pack('<I',len(d))+d)" "$VW" "$ARGS"

# 5. Publish the vault code as a global contract, by hash.
send contract call-function as-transaction "$COLL" set_vault_code file-args "$ARGS" \
  prepaid-gas '300 Tgas' attached-deposit '0 NEAR' sign-as "$COLL"
rm -f "$ARGS"

# 6. Let the global code reach every shard.
echo "Waiting 35 seconds for global code distribution..."
sleep 35

# 7. Mint a test vault and verify it.
UNLOCK=$(python3 -I -c "import time; print(time.time_ns() + $MIN_NS + 120_000_000_000)")
send contract call-function as-transaction "$COLL" mint_vault \
  json-args "{\"child_id\":\"deploy-check-child.$FUNDER\",\"unlock_ns\":\"$UNLOCK\",\"title\":\"Deploy check\"}" \
  prepaid-gas '100 Tgas' attached-deposit '0.025 NEAR' sign-as "$FUNDER"
view contract_info '{}'
INFO=$(view vault_info '{"token_id":"bv-1"}' 2>&1)
echo "$INFO"
echo "$INFO" | grep -q "\"vault_id\"" || { echo "FAIL: the test vault has no vault_info." >&2; exit 1; }
KEYS=$(near account list-keys "bv-1.$COLL" network-config "$NET" now 2>&1)
echo "$KEYS"
if echo "$KEYS" | grep -qi "ed25519:"; then echo "FAIL: the test vault has an access key." >&2; exit 1; fi
echo "DEPLOY CHECK PASS: $COLL is live on $NET with test vault bv-1.$COLL and no keys on the vault."
echo "The collection still has its access key. Lock it with scripts/lock.sh $NET $COLL once the play flow passes (scripts/play-flow.sh)."
