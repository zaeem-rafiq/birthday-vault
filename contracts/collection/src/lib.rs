use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider, NFT_METADATA_SPEC};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{U64, U128};
use near_sdk::{env, near, AccountId, BorshStorageKey, CryptoHash, NearToken, PanicOnDefault, Promise};
use near_sdk::serde;

const ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8'%3E%3Crect width='8' height='8' fill='%23c58b3a'/%3E%3C/svg%3E";

#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Demo,
    Production,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct VaultState {
    pub guardian: AccountId,
    pub child: AccountId,
    pub unlock_ns: u64,
    pub created_ns: u64,
    pub pending: u32,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Collection {
    nft: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    vault_code_hash: Option<CryptoHash>,
    usdc: AccountId,
    mode: Mode,
    min_lock_ns: u64,
    max_lock_ns: u64,
    next_serial: u64,
    vaults: LookupMap<TokenId, VaultState>,
}

#[near]
#[derive(BorshStorageKey)]
pub enum StorageKey {
    Owners,
    TokenMetadata,
    Enumeration,
    ContractMetadata,
    Vaults,
}

#[near]
impl Collection {
    fn parent_of(id: &AccountId) -> String {
        id.to_string()
            .split_once('.')
            .map(|(_, rest)| rest.to_string())
            .unwrap_or_default()
    }

    fn assert_self_or_parent() {
        let predecessor = env::predecessor_account_id();
        let current = env::current_account_id();
        if predecessor != current && predecessor.to_string() != Self::parent_of(&current) {
            panic!("only self or parent");
        }
    }

    pub fn is_mainnet_id(&self, account_id: String) -> bool {
        Self::is_mainnet_static(&account_id)
    }

    fn is_mainnet_static(account_id: &str) -> bool {
        account_id.ends_with(".near") && !account_id.ends_with(".test.near")
    }

    #[init]
    pub fn new(usdc: AccountId, mode: Mode, min_lock_ns: U64, max_lock_ns: U64) -> Self {
        Self::assert_self_or_parent();

        let current_id = env::current_account_id().to_string();
        let is_demo_net = current_id.ends_with(".testnet") || current_id.ends_with(".sandbox") || current_id.ends_with(".test.near");

        if mode == Mode::Demo {
            if !is_demo_net {
                panic!("mode does not match network");
            }
            if !(0 < min_lock_ns.0 && min_lock_ns.0 <= max_lock_ns.0) {
                panic!("bad lock limits");
            }
        } else if mode == Mode::Production {
            if !Self::is_mainnet_static(&current_id) {
                panic!("mode does not match network");
            }
            if !(min_lock_ns.0 >= 86_400_000_000_000 && max_lock_ns.0 <= 1_262_304_000_000_000_000) {
                panic!("bad lock limits");
            }
        }

        Self {
            nft: NonFungibleToken::new(
                StorageKey::Owners,
                env::current_account_id(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                None::<StorageKey>,
            ),
            metadata: LazyOption::new(StorageKey::ContractMetadata, None),
            vault_code_hash: None,
            usdc,
            mode,
            min_lock_ns: min_lock_ns.0,
            max_lock_ns: max_lock_ns.0,
            next_serial: 1,
            vaults: LookupMap::new(StorageKey::Vaults),
        }
    }

    pub fn set_vault_code(&mut self, #[serializer(borsh)] vault_code: Vec<u8>) {
        Self::assert_self_or_parent();
        if self.vault_code_hash.is_some() {
            panic!("vault code already set");
        }

        let required_balance = NearToken::from_yoctonear((vault_code.len() as u128 * 100_000_000_000_000_000_000) + 100_000_000_000_000_000_000_000);
        if env::account_balance() < required_balance {
            panic!("not enough balance");
        }

        self.vault_code_hash = Some(env::sha256(&vault_code).try_into().expect("sha256 must return 32 bytes"));
        Promise::new(env::current_account_id())
            .deploy_global_contract(vault_code)
            .detach();
    }

    pub fn contract_info(&self) -> near_sdk::serde_json::Value {
        near_sdk::serde_json::json!({
            "mode": format!("{:?}", self.mode),
            "usdc": self.usdc,
            "min_lock_ns": self.min_lock_ns.to_string(),
            "max_lock_ns": self.max_lock_ns.to_string(),
            "vault_code_hash": self.vault_code_hash.as_ref().map(|h| h.iter().map(|b| format!("{:02x}", b)).collect::<String>()).or(None),
            "next_serial": self.next_serial,
        })
    }

    pub fn nft_total_supply(&self) -> U128 {
        U128(self.nft.owner_by_id.len() as u128)
    }

    pub fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap_or_else(|| {
            let name = match self.mode {
                Mode::Demo => "Birthday Vault (DEMO, play money)",
                Mode::Production => "Birthday Vault",
            };
            NFTContractMetadata {
                name: name.to_string(),
                symbol: "VAULT".to_string(),
                spec: NFT_METADATA_SPEC.to_string(),
                icon: Some(ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            }
        })
    }
}

impl NonFungibleTokenMetadataProvider for Collection {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.nft_metadata()
    }
}
