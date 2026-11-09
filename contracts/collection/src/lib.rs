#![allow(unused_imports)]
use near_sdk::near_bindgen;
use near_sdk::borsh::{BorshSerialize, BorshDeserialize};
use near_sdk::state::ContractState;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Collection {
    pub value: u64,
}

impl ContractState for Collection {}

#[near_bindgen]
impl Collection {
    #[init]
    pub fn new() -> Self {
        Self { value: 0 }
    }
}
