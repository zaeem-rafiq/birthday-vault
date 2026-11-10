use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, NonFungibleTokenMetadataProvider, NFT_METADATA_SPEC, TokenMetadata};
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::events::NftMint;
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{U64, U128};
use near_sdk::{env, near, AccountId, BorshStorageKey, CryptoHash, NearToken, PanicOnDefault, Promise, Gas, PromiseError, require};
use near_sdk::serde;

const ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 8 8'%3E%3Crect width='8' height='8' fill='%23c58b3a'/%3E%3C/svg%3E";
const USDC_REGISTRATION: u128 = 1_250_000_000_000_000_000_000;
const VAULT_FUNDING: u128 = 2_500_000_000_000_000_000_000;
const MINT_COST: u128 = 25_000_000_000_000_000_000_000;
const REG_GAS: Gas = Gas::from_tgas(10);
const MINT_CALLBACK_GAS: Gas = Gas::from_tgas(30);
const VAULT_MEDIA: &str = "https://birthday-vault.onrender.com/vault.svg";

#[near(event_json(standard = "nep171"))]
pub enum CollectionEvents {
    #[event_version("1.0.0")]
    Mint { owner_id: AccountId, token_ids: Vec<TokenId>, memo: Option<String> },
}

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

        let free_balance = env::account_balance().as_yoctonear() - (env::storage_usage() as u128 * env::storage_byte_cost().as_yoctonear());
        let required = (vault_code.len() as u128 * 100_000_000_000_000_000_000u128) + 100_000_000_000_000_000_000_000u128;
        if free_balance < required {
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

    fn default_metadata(&self) -> NFTContractMetadata {
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

    #[payable]
    pub fn mint_vault(&mut self, child_id: AccountId, unlock_ns: U64, title: Option<String>) -> Promise {
        let deposit = env::attached_deposit();
        require!(deposit.as_yoctonear() >= MINT_COST, "deposit below mint cost");
        require!(self.vault_code_hash.is_some(), "vault code not set");

        let guardian = env::predecessor_account_id();
        require!(guardian != child_id, "guardian and child must differ");

        let coll_id = env::current_account_id().to_string();
        let is_invalid = |id: &AccountId| {
            let s = id.to_string();
            s == coll_id || s.ends_with(&format!(".{}", coll_id))
        };
        require!(!is_invalid(&guardian) && !is_invalid(&child_id), "invalid account");

        let now = env::block_timestamp();
        let min_unlock = now.checked_add(self.min_lock_ns).expect("overflow");
        let max_unlock = now.checked_add(self.max_lock_ns).expect("overflow");
        require!(unlock_ns.0 >= min_unlock && unlock_ns.0 <= max_unlock, "bad unlock time");

        if let Some(ref t) = title {
            require!(t.len() <= 64, "title too long");
        }

        let serial = self.next_serial;
        self.next_serial += 1;
        let token_id = format!("bv-{}", serial);
        let vault_id = format!("{}.{}", token_id, coll_id);
        require!(vault_id.len() <= 64, "vault id too long");

        let hash = self.vault_code_hash.unwrap();
        let reg = Promise::new(self.usdc.clone()).function_call(
            "storage_deposit".to_string(),
            near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({"account_id": vault_id, "registration_only": true})).unwrap(),
            NearToken::from_yoctonear(USDC_REGISTRATION),
            REG_GAS,
        );
        let mk = Promise::new(vault_id.clone().parse::<AccountId>().unwrap()).create_account().use_global_contract(hash).transfer(NearToken::from_yoctonear(VAULT_FUNDING));

        mk.and(reg).then(
            Self::ext(env::current_account_id())
                .with_static_gas(MINT_CALLBACK_GAS)
                .on_vault_created(token_id, guardian.clone(), child_id, unlock_ns, title, guardian, deposit.as_yoctonear()),
        )
    }

    #[private]
    pub fn on_vault_created(&mut self, token_id: TokenId, guardian: AccountId, child: AccountId, unlock_ns: U64, title: Option<String>, payer: AccountId, deposit: u128) -> bool {
        let created = !matches!(env::promise_result_checked(0, 0), Err(PromiseError::Failed));
        let registered = !matches!(env::promise_result_checked(1, 0), Err(PromiseError::Failed));

        if created && registered {
            let before = env::storage_usage();
            let metadata = TokenMetadata {
                title: Some(title.clone().unwrap_or_else(|| "Birthday Vault".to_string())),
                description: Some("A birthday gift that stays locked until the child's birthday.".to_string()),
                media: Some(VAULT_MEDIA.to_string()),
                media_hash: None,
                copies: Some(1),
                issued_at: Some(env::block_timestamp_ms().to_string()),
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            };
            self.nft.internal_mint_with_refund(token_id.clone(), guardian.clone(), Some(metadata), None);
            env::log_str(&format!("Minted token {}", token_id));
            CollectionEvents::Mint {
                owner_id: guardian.clone(),
                token_ids: vec![token_id.clone()],
                memo: None,
            }.emit();
            self.vaults.insert(&token_id, &VaultState {
                guardian,
                child,
                unlock_ns: unlock_ns.0,
                created_ns: env::block_timestamp(),
                pending: 0,
            });
            let byte_cost = env::storage_byte_cost().as_yoctonear();
            let storage_cost = (env::storage_usage() - before) as u128 * byte_cost;
            let refund = deposit - VAULT_FUNDING - USDC_REGISTRATION - storage_cost;
            if refund > 1 {
                Promise::new(payer).transfer(NearToken::from_yoctonear(refund));
            }
            true
        } else {
            let mut refund = deposit;
            if created {
                refund -= VAULT_FUNDING;
            }
            if registered {
                refund -= USDC_REGISTRATION;
            }
            if refund > 1 {
                Promise::new(payer).transfer(NearToken::from_yoctonear(refund));
            }
            false
        }
    }

    pub fn vault_info(&self, token_id: TokenId) -> Option<near_sdk::serde_json::Value> {
        let state = self.vaults.get(&token_id)?;
        let now = env::block_timestamp();
        Some(near_sdk::serde_json::json!({
            "vault_id": format!("{}.{}", token_id, env::current_account_id()),
            "owner_id": self.nft.owner_by_id.get(&token_id),
            "guardian": state.guardian,
            "child": state.child,
            "unlock_ns": state.unlock_ns.to_string(),
            "locked": now < state.unlock_ns,
            "pending": state.pending,
            "mode": format!("{:?}", self.mode),
            "usdc": self.usdc,
        }))
    }

    pub fn vault_info_by_account(&self, vault_id: AccountId) -> near_sdk::serde_json::Value {
        let coll_id = env::current_account_id().to_string();
        if !vault_id.to_string().ends_with(&format!(".{}", coll_id)) {
            return near_sdk::serde_json::Value::Null;
        }
        let token_id = vault_id.to_string().split('.').next().unwrap_or_default().to_string();
        self.vault_info(token_id).unwrap_or(near_sdk::serde_json::Value::Null)
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.nft.nft_token(token_id)
    }
}

#[near]
impl NonFungibleTokenEnumeration for Collection {
    fn nft_total_supply(&self) -> U128 {
        self.nft.nft_total_supply()
    }

    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        self.nft.nft_tokens(from_index, limit)
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        self.nft.nft_supply_for_owner(account_id)
    }

    fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        self.nft.nft_tokens_for_owner(account_id, from_index, limit)
    }
}

#[near]
impl NonFungibleTokenMetadataProvider for Collection {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.default_metadata()
    }
}
