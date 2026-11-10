"""Print one new ed25519 key pair in NEAR form: "<public key> <secret key>".

Uses openssl, so no Python package is needed. scripts/setup-hub.sh calls it for the demo
hub's key pool and writes the secret keys straight to a gitignored file, never to the screen.
"""
import subprocess

ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58(data: bytes) -> str:
    n = int.from_bytes(data, "big")
    out = ""
    while n:
        n, r = divmod(n, 58)
        out = ALPHABET[r] + out
    return "1" * (len(data) - len(data.lstrip(b"\0"))) + out


der = subprocess.run(["openssl", "genpkey", "-algorithm", "ed25519", "-outform", "DER"], capture_output=True, check=True).stdout
pub_der = subprocess.run(["openssl", "pkey", "-inform", "DER", "-pubout", "-outform", "DER"], input=der, capture_output=True, check=True).stdout
seed, pub = der[-32:], pub_der[-32:]
print(f"ed25519:{b58(pub)} ed25519:{b58(seed + pub)}")
