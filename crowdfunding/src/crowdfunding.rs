#![no_std]

use multiversx_sc::derive_imports::*;
#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Clone, Copy)]
pub enum Status {
    FundingPeriod,
    Successful,
    Failed,
}

#[multiversx_sc::contract]
pub trait CrowdfundingSc {
    #[init]
    fn init(&self, target: BigUint, deadline: u64) {
        require!(target > 0, "Target must be more than 0");
        self.target().set(target);

        require!(
            deadline > self.get_current_time(),
            "Deadline can't be in the past"
        );
        self.deadline().set(deadline);

        self.max_total_per_wallet().set(BigUint::zero());
        self.min_deposit_per_tx().set(BigUint::zero());
        self.max_total_project().set(BigUint::zero());
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(setMaxTotalPerWallet)]
    fn set_max_total_per_wallet(&self, max: BigUint) {
        self.max_total_per_wallet().set(max);
    }

    #[only_owner]
    #[endpoint(setMinDepositPerTx)]
    fn set_min_deposit_per_tx(&self, min: BigUint) {
        self.min_deposit_per_tx().set(min);
    }

    #[only_owner]
    #[endpoint(setMaxTotalProject)]
    fn set_max_total_project(&self, max: BigUint) {
        self.max_total_project().set(max);
    }

    #[endpoint]
    #[payable("EGLD")]
    fn fund(&self) {
        let payment = self.call_value().egld_value(); // ManagedRef<BigUint>
        let amount = payment.clone_value(); // Convertim a BigUint
        let current_time = self.blockchain().get_block_timestamp();

        require!(
            current_time < self.deadline().get(),
            "cannot fund after deadline"
        );

        let min = self.min_deposit_per_tx().get();
        if min > 0u32 {
            require!(
                amount >= min.clone(),
                "Deposit is below minimum per transaction"
            );
        }

        let caller = self.blockchain().get_caller();
        let deposited_amount = self.deposit(&caller).get();

        let max_wallet = self.max_total_per_wallet().get();
        if max_wallet > 0u32 {
            require!(
                deposited_amount.clone() + amount.clone() <= max_wallet.clone(),
                "Deposit exceeds max total per wallet"
            );
        }

        let current_total = self.get_current_funds();
        let max_project = self.max_total_project().get();
        if max_project > 0u32 {
            require!(
                current_total + amount.clone() <= max_project.clone(),
                "Deposit exceeds project total cap"
            );
        }

        self.deposit(&caller).set(deposited_amount + amount);
    }

    #[endpoint]
    fn claim(&self) {
        match self.status() {
            Status::FundingPeriod => sc_panic!("cannot claim before deadline"),
            Status::Successful => {
                let caller = self.blockchain().get_caller();
                require!(
                    caller == self.blockchain().get_owner_address(),
                    "only owner can claim successful funding"
                );

                let sc_balance = self.get_current_funds();
                self.send().direct_egld(&caller, &sc_balance);
            }
            Status::Failed => {
                let caller = self.blockchain().get_caller();
                let deposit = self.deposit(&caller).get();

                if deposit > 0u32 {
                    self.deposit(&caller).clear();
                    self.send().direct_egld(&caller, &deposit);
                }
            }
        }
    }

    #[view]
    fn status(&self) -> Status {
        if self.get_current_time() <= self.deadline().get() {
            Status::FundingPeriod
        } else if self.get_current_funds() >= self.target().get() {
            Status::Successful
        } else {
            Status::Failed
        }
    }

    #[view(getCurrentFunds)]
    fn get_current_funds(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0)
    }

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    // storage

    #[view(getTarget)]
    #[storage_mapper("target")]
    fn target(&self) -> SingleValueMapper<BigUint>;

    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<u64>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getMaxTotalPerWallet)]
    #[storage_mapper("max_total_per_wallet")]
    fn max_total_per_wallet(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinDepositPerTx)]
    #[storage_mapper("min_deposit_per_tx")]
    fn min_deposit_per_tx(&self) -> SingleValueMapper<BigUint>;

    #[view(getMaxTotalProject)]
    #[storage_mapper("max_total_project")]
    fn max_total_project(&self) -> SingleValueMapper<BigUint>;
}
