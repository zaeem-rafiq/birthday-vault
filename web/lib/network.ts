// The network rule from PLAN.md section 5.2, the same as the collection contract's
// `is_mainnet_id`: an id is mainnet only if it ends in `.near` and not in `.test.near`.
export function isMainnetId(id: string): boolean {
  return id.endsWith(".near") && !id.endsWith(".test.near");
}

/** A full vault account id belongs to the configured demo collection. */
export function isDemoVaultId(id: string, collectionId: string): boolean {
  if (!collectionId || isMainnetId(id)) return false;
  const suffix = `.${collectionId}`;
  if (!id.endsWith(suffix)) return false;
  const token = id.slice(0, -suffix.length);
  return /^bv-[0-9]+$/.test(token);
}
