use near_sdk::{env, Promise, NearToken, Gas, AccountId, near};
use near_sdk::json_types::U128;

#[near(contract_state)]
#[derive(Default)]
pub struct Vault {}

#[near]
impl Vault {
    fn check_parent(&self) {
        let current_id = env::current_account_id();
        let parent = current_id.to_string().split('.').skip(1).collect::<Vec<&str>>().join(".");
        assert_eq!(
            env::predecessor_account_id(),
            AccountId::try_from(parent).expect("Invalid parent account id"),
            "Parent only"
        );
    }

    #[allow(unused_variables)]
    pub fn ft_send(&mut self, token: AccountId, receiver: AccountId, amount: U128) -> Promise {
        self.check_parent();

        Promise::new(token).function_call(
            "ft_transfer",
            near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({
                "receiver_id": receiver,
                "amount": amount
            })).expect("Failed to serialize arguments"),
            NearToken::from_yoctonear(1),
            Gas::from_tgas(30),
        )
    }

    #[allow(unused_variables)]
    pub fn near_send(&mut self, receiver: AccountId, amount: U128) -> Promise {
        self.check_parent();

        let amount_u128: u128 = amount.0;
        Promise::new(receiver).transfer(NearToken::from_yoctonear(amount_u128))
    }

    #[allow(unused_variables)]
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        U128(0)
    }

    #[allow(unused_variables)]
    pub fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        msg: String,
    ) -> bool {
        true
    }
}
