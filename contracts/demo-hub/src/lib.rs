use near_sdk::json_types::U64;
use near_sdk::store::LookupSet;
use near_sdk::{
    env, near, require, AccountId, BorshStorageKey, CurveType, Gas, NearToken, PanicOnDefault,
    Promise, PublicKey,
};

const GUARDIAN_NEAR: NearToken = NearToken::from_millinear(100);
const CHILD_NEAR: NearToken = NearToken::from_millinear(20);
const BUYER_NEAR: NearToken = NearToken::from_millinear(30);
const USDC_CALL_GAS: Gas = Gas::from_tgas(10);

#[near]
#[derive(BorshStorageKey)]
enum StorageKey {
    Used,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct DemoHub {
    usdc: AccountId,
    hour_ns: u64,
    day_ns: u64,
    hourly_limit: u32,
    daily_limit: u32,
    starts: Vec<u64>,
    used: LookupSet<AccountId>,
}

#[near]
impl DemoHub {
    fn implicit_id(pk: &PublicKey) -> AccountId {
        let bytes = pk.as_bytes();
        // pk.as_bytes()[0] is the curve byte. We want bytes from index 1 onwards.
        let hex: String = bytes[1..].iter().map(|b| format!("{b:02x}")).collect();
        hex.parse().unwrap()
    }

    #[init]
    pub fn new(
        usdc: AccountId,
        hour_ns: U64,
        day_ns: U64,
        hourly_limit: u32,
        daily_limit: u32,
    ) -> Self {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "only the hub"
        );
        require!(
            hour_ns.0 > 0
                && day_ns.0 >= hour_ns.0
                && hourly_limit > 0
                && hourly_limit <= daily_limit,
            "bad limits"
        );

        Self {
            usdc,
            hour_ns: hour_ns.0,
            day_ns: day_ns.0,
            hourly_limit,
            daily_limit,
            starts: Vec::new(),
            used: LookupSet::new(StorageKey::Used),
        }
    }

    pub fn start_demo(
        &mut self,
        guardian_pk: PublicKey,
        child_pk: PublicKey,
        buyer_pk: PublicKey,
    ) -> near_sdk::serde_json::Value {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "only the hub"
        );

        require!(
            guardian_pk.curve_type() == CurveType::ED25519
                && child_pk.curve_type() == CurveType::ED25519
                && buyer_pk.curve_type() == CurveType::ED25519,
            "only ed25519 keys"
        );

        require!(
            guardian_pk != child_pk && guardian_pk != buyer_pk && child_pk != buyer_pk,
            "keys must differ"
        );

        let guardian = Self::implicit_id(&guardian_pk);
        let child = Self::implicit_id(&child_pk);
        let buyer = Self::implicit_id(&buyer_pk);

        require!(!self.used.contains(&guardian), "this demo already started");

        let now = env::block_timestamp();

        // Remove entries older than day_ns
        while let Some(&t) = self.starts.first() {
            if now.saturating_sub(t) >= self.day_ns {
                self.starts.remove(0);
            } else {
                break;
            }
        }

        let hourly_count = self
            .starts
            .iter()
            .filter(|&&t| now.saturating_sub(t) < self.hour_ns)
            .count() as u32;

        require!(
            hourly_count < self.hourly_limit,
            "hourly limit reached, try again later"
        );
        let daily_count = self.starts.len() as u32;
        require!(
            daily_count < self.daily_limit,
            "daily limit reached, try again tomorrow"
        );

        self.starts.push(now);
        self.used.insert(guardian.clone());

        Promise::new(guardian.clone())
            .transfer(GUARDIAN_NEAR)
            .detach();
        Promise::new(child.clone()).transfer(CHILD_NEAR).detach();
        Promise::new(buyer.clone()).transfer(BUYER_NEAR).detach();

        Promise::new(self.usdc.clone())
            .function_call(
                "faucet".to_string(),
                near_sdk::serde_json::to_vec(
                    &near_sdk::serde_json::json!({ "account_id": guardian }),
                )
                .unwrap(),
                NearToken::from_yoctonear(0),
                USDC_CALL_GAS,
            )
            .function_call(
                "register".to_string(),
                near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({ "account_id": child }))
                    .unwrap(),
                NearToken::from_yoctonear(0),
                USDC_CALL_GAS,
            )
            .function_call(
                "register".to_string(),
                near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({ "account_id": buyer }))
                    .unwrap(),
                NearToken::from_yoctonear(0),
                USDC_CALL_GAS,
            )
            .detach();

        near_sdk::serde_json::json!({
            "guardian": guardian,
            "child": child,
            "buyer": buyer
        })
    }

    pub fn hub_info(&self) -> near_sdk::serde_json::Value {
        let now = env::block_timestamp();
        let starts_last_hour = self
            .starts
            .iter()
            .filter(|&&t| now.saturating_sub(t) < self.hour_ns)
            .count();
        let starts_last_day = self
            .starts
            .iter()
            .filter(|&&t| now.saturating_sub(t) < self.day_ns)
            .count();

        near_sdk::serde_json::json!({
            "usdc": self.usdc,
            "hourly_limit": self.hourly_limit,
            "daily_limit": self.daily_limit,
            "starts_last_hour": starts_last_hour,
            "starts_last_day": starts_last_day,
        })
    }

    pub fn has_started(&self, guardian_id: AccountId) -> bool {
        self.used.contains(&guardian_id)
    }
}
