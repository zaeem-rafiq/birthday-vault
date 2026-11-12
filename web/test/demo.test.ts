import { describe, expect, it } from "vitest";
import { validNickname, refusalText } from "@/lib/demo";
import { base58Decode, implicitId } from "@/lib/near";
import { KeyPair } from "near-api-js";

describe("demo rules", () => {
  it("accepts nicknames of letters, digits and spaces up to 24", () => {
    expect(validNickname("Maya")).toBe(true);
    expect(validNickname("Maya 2")).toBe(true);
    expect(validNickname("a".repeat(24))).toBe(true);
    expect(validNickname("a".repeat(25))).toBe(false);
    expect(validNickname("   ")).toBe(false);
    expect(validNickname("<script>")).toBe(false);
  });

  it("derives the 64-character implicit id from the key bytes", () => {
    const kp = KeyPair.fromRandom("ed25519");
    const pk = kp.getPublicKey();
    const id = implicitId(pk.toString());
    expect(id).toMatch(/^[0-9a-f]{64}$/);
    expect(id).toBe(Buffer.from(pk.data).toString("hex"));
    expect(base58Decode("1".repeat(32)).every((b) => b === 0)).toBe(true);
  });

  it("explains contract refusals in plain words", () => {
    expect(refusalText("locked")).toMatch(/still locked/);
    expect(refusalText("only the owner")).toMatch(/current owner/);
  });
});

describe("error reading", () => {
  it("reads a failover error that holds BigInt fields without throwing", async () => {
    const { errorText } = await import("@/lib/near");
    const inner = Object.assign(new Error("Not enough balance for abc (balance: 1, cost: 2)"), {
      balance: BigInt(1),
      cost: BigInt(2),
    });
    const outer = Object.assign(new Error("Exceeded 1 providers to execute request"), { errors: [inner] });
    expect(() => errorText(outer)).not.toThrow();
    expect(errorText(outer)).toBe("Not enough balance");
  });

  it("finds the contract's own panic message", async () => {
    const { errorText } = await import("@/lib/near");
    const e = Object.assign(new Error("Transaction failed"), {
      context: { big: BigInt(5) },
      kind: "Smart contract panicked: locked",
    });
    expect(errorText(e)).toBe("locked");
  });
});

describe("review fixes", () => {
  it("unwraps a failover error to the provider's own message", async () => {
    const { errorText } = await import("@/lib/near");
    const inner = new Error("Transaction nonce 7 must be larger than nonce of the used access key 9");
    const outer = Object.assign(new Error("Exceeded 1 providers to execute request"), { errors: [inner] });
    expect(errorText(outer)).toMatch(/nonce/);
  });

  it("keeps the demo in memory when the browser refuses storage", async () => {
    const { loadDemo, newDemo, saveDemo } = await import("@/lib/demo");
    // The test runs in Node, where localStorage does not exist, like a browser that refuses it.
    const d = newDemo("Mia");
    saveDemo(d);
    expect(loadDemo()?.keys.guardian.id).toBe(d.keys.guardian.id);
    saveDemo(null);
    expect(loadDemo()).toBeNull();
  });
});
