//! Test double: records the last ft_transfer call so tests can inspect it.
use near_sdk::json_types::U128;
use near_sdk::{env, near, AccountId};

#[near(contract_state)]
#[derive(Default)]
pub struct Token {
    predecessor: String,
    receiver: String,
    amount: String,
    deposit: String,
    calls: u32,
}

#[near]
impl Token {
    #[payable]
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128) {
        self.predecessor = env::predecessor_account_id().to_string();
        self.receiver = receiver_id.to_string();
        self.amount = amount.0.to_string();
        self.deposit = env::attached_deposit().as_yoctonear().to_string();
        self.calls += 1;
    }

    pub fn last_call(&self) -> (String, String, String, String, u32) {
        (self.predecessor.clone(), self.receiver.clone(), self.amount.clone(), self.deposit.clone(), self.calls)
    }
}
