import { describe, expect, it } from "vitest";
import { isDemoVaultId, isMainnetId } from "@/lib/network";

describe("network rule", () => {
  it("classifies ids the same way as the contract", () => {
    expect(isMainnetId("bv-1.g1.alice.near")).toBe(true);
    expect(isMainnetId("bv.test.near")).toBe(false);
    expect(isMainnetId("bv-1.g1.birthday-vault-zr1109.testnet")).toBe(false);
    expect(isMainnetId("x.sandbox")).toBe(false);
  });

  it("accepts only bv-<n> vaults of the configured collection", () => {
    expect(isDemoVaultId("bv-12.bv.test.near", "bv.test.near")).toBe(true);
    expect(isDemoVaultId("bv-x.bv.test.near", "bv.test.near")).toBe(false);
    expect(isDemoVaultId("bv-1.other.testnet", "bv.test.near")).toBe(false);
    expect(isDemoVaultId("bv-1.g1.alice.near", "g1.alice.near")).toBe(false);
  });
});
