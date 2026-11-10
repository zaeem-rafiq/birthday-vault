use near_contract_standards::fungible_token::metadata::{
    FT_METADATA_SPEC, FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    AccountId, BorshStorageKey, env, near, require, PanicOnDefault, PromiseOrValue, NearToken,
};

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    hub: AccountId,
}

const FAUCET_AMOUNT: u128 = 5_000_000;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near]
impl Contract {
    #[init]
    pub fn new(hub_id: AccountId) -> Self {
        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Test USDC (play money)".to_string(),
            symbol: "tUSDC".to_string(),
            decimals: 6,
            icon: None,
            reference: None,
            reference_hash: None,
        };

        Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            hub: hub_id,
        }
    }

    pub fn faucet(&mut self, account_id: AccountId) {
        require!(
            env::predecessor_account_id() == self.hub,
            "only the hub"
        );

        if !self.token.accounts.contains_key(&account_id) {
            self.token.internal_register_account(&account_id);
        }

        self.token.internal_deposit(&account_id, FAUCET_AMOUNT);

        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &account_id,
            amount: FAUCET_AMOUNT.into(),
            memo: Some("faucet"),
        }
        .emit();
    }

    pub fn register(&mut self, account_id: AccountId) {
        require!(
            env::predecessor_account_id() == self.hub,
            "only the hub"
        );

        if !self.token.accounts.contains_key(&account_id) {
            self.token.internal_register_account(&account_id);
        }
    }
}

#[near]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            near_sdk::log!(
                "Account @{} burned {}",
                sender_id,
                burned_amount
            );
        }
        used_amount.into()
    }
}

#[near]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        let _ = force;
        env::panic_str("unregister is disabled: the hub pays for storage")
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().expect("Metadata not initialized")
    }
}
