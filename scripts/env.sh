# Source this before any cargo or near command: source scripts/env.sh
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
# On the Mac, use the sandbox binary already on disk. Elsewhere, near-workspaces downloads it.
if [ -x "$HOME/Documents/Projects/birthday-vault-gemma-test/tools/near-sandbox" ]; then
  export NEAR_SANDBOX_BIN_PATH="$HOME/Documents/Projects/birthday-vault-gemma-test/tools/near-sandbox"
fi
