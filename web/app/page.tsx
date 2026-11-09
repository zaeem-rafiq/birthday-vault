"use client";

import { useEffect, useState } from "react";
import { KeyPair } from "near-api-js";

export default function Home() {
  const [publicKey, setPublicKey] = useState("");

  useEffect(() => {
    setPublicKey(KeyPair.fromRandom("ed25519").getPublicKey().toString());
  }, []);

  return (
    <main style={{ fontFamily: "system-ui, sans-serif", padding: 32 }}>
      <h1>Birthday Vault</h1>
      <p>Toolchain check: near-api-js runs in this page.</p>
      <p data-testid="public-key">{publicKey || "Generating a key..."}</p>
    </main>
  );
}
