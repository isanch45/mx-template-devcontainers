#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait CrowdfundingContract {
    #[init]
    fn init(
        &self,
        target: BigUint,
        deadline: u64,
        min_deposit: BigUint,
        max_per_wallet: BigUint,
        admin2: ManagedAddress,
    ) {
        require!(!target.is_zero(), "Target cannot be zero");
        self.target().set(&target);
        self.max_total().set(&(target.clone() + target.clone() / 10u32));
        self.deadline().set(&deadline);
        self.min_deposit().set(&min_deposit);
        self.max_per_wallet().set(&max_per_wallet);
        self.admin1().set(&self.blockchain().get_caller());
        self.admin2().set(&admin2);
        self.total_funds().set(BigUint::zero());
        self.terminated().set(false);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn fund(&self) {
        let caller = self.blockchain().get_caller();
        let value = self.call_value().egld_value();
        let now = self.blockchain().get_block_timestamp();

        require!(now <= self.deadline().get(), "Deadline passed");
        require!(value >= self.min_deposit().get(), "Below minimum deposit");

        let user_total = self.user_deposits(&caller).get() + value.clone();
        require!(user_total <= self.max_per_wallet().get(), "Above max per wallet");

        let new_total = self.total_funds().get() + value.clone();
        require!(new_total <= self.max_total().get(), "Above total max");

        self.user_deposits(&caller).set(&user_total);
        self.total_funds().set(&new_total);
    }

    #[endpoint]
    fn claimFunds(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.is_admin(&caller), "Not admin");
        require!(!self.terminated().get(), "Campaign terminated");

        let total = self.total_funds().get();
        require!(total >= self.target().get(), "Target not reached");

        self.send().direct_egld(&caller, &total);
        self.total_funds().set(BigUint::zero());
    }

    #[endpoint]
    fn claimRefund(&self) {
        let caller = self.blockchain().get_caller();
        let now = self.blockchain().get_block_timestamp();

        let should_refund = now > self.deadline().get() || self.terminated().get();
        require!(should_refund, "Refund not allowed");

        let deposit = self.user_deposits(&caller).get();
        require!(!deposit.is_zero(), "No funds to refund");

        self.send().direct_egld(&caller, &deposit);
        self.user_deposits(&caller).clear();
    }

    #[endpoint]
    fn terminateCampaign(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.is_admin(&caller), "Not admin");
        self.terminated().set(true);
    }

    #[endpoint]
    fn uploadInvoiceHash(&self, hash: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        require!(self.is_admin(&caller), "Not admin");
        self.invoice_hash().set(&hash);
        self.invoice_event(&caller, &hash);
    }

    #[view(getContractAddress)]
    fn get_contract_address(&self) -> ManagedAddress {
        self.blockchain().get_sc_address()
    }

    #[endpoint]
    fn upgradeParams(&self, new_min: BigUint, new_max: BigUint) {
        let caller = self.blockchain().get_caller();
        require!(self.is_admin(&caller), "Not admin");
        self.min_deposit().set(&new_min);
        self.max_per_wallet().set(&new_max);
    }

    #[view(isAdmin)]
    fn is_admin(&self, addr: &ManagedAddress) -> bool {
        addr == &self.admin1().get() || addr == &self.admin2().get()
    }

    #[storage_mapper("target")]
    fn target(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("max_total")]
    fn max_total(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("min_deposit")]
    fn min_deposit(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("max_per_wallet")]
    fn max_per_wallet(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("total_funds")]
    fn total_funds(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("admin1")]
    fn admin1(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("admin2")]
    fn admin2(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("terminated")]
    fn terminated(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("invoice_hash")]
    fn invoice_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("user_deposits")]
    fn user_deposits(&self, user: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[event("invoice_uploaded")]
    fn invoice_event(&self, #[indexed] admin: &ManagedAddress, hash: &ManagedBuffer);
}
