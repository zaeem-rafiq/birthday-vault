use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, near_bindgen, AccountId, Gas, Promise};
use near_sdk::json_types::U128;
use near_sdk::serde_json::{self as serde_json, Value};

#[near(contract_state)]
#[derive(Deserialize, Serialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct HostileToken {
    mode: String,
    fail_registration: bool,
    transfers: u32,
}

#[near]
impl HostileToken {
    pub fn set_behavior(&mut self, mode: String, fail_registration: bool) {
        self.mode = mode;
        self.fail_registration = fail_registration;
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> Value {
        if self.fail_registration {
            env::panic_str("registration refused");
        }
        env::log_str("storage_deposit success");
        serde_json::json!({
            "total": "1250000000000000000000",
            "available": "0"
        })
    }

    #[payable]
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        match self.mode.as_str() {
            "" => {
                self.transfers += 1;
            }
            "fail" => {
                env::panic_str("transfer refused");
            }
            "huge" => {
                self.transfers += 1;
                let huge_value = "x".repeat(100_000);
                env::value_return(&huge_value);
            }
            "slow" => {
                self.transfers += 1;
                let _ = Promise::new(env::current_account_id())
                    .function_call(
                        "slow_done".to_string(),
                        vec![],
                        near_sdk::NearToken::from_yoctonear(0),
                        Gas::from_tgas(5),
                    )
                    .as_return();
            }
            _ => {
                self.transfers += 1;
            }
        }
    }

    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        U128(1_000_000)
    }

    pub fn transfers(&self) -> u32 {
        self.transfers
    }

    #[allow(unused_variables)]
    pub fn slow_done(&self) {
        // does nothing
    }
}
