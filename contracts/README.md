# openPayroll Contract

## Description 

This is an Ink! contract written in the lib.rs file, which purpose is to manage a treasury, that can be spent by the parameters set by the owner at creation point. Those parameters can be changed over the time and more beneficiaries can be added or removed. The funds in the treasury can be withdrawn by the owner of the contract if needed. This could be helpful in the case of migrating to a new version of openPayroll, amending a mistake of sending too much funds, etc. 

The data contained on chain are the addresses of the beneficiaries, the owners address, the period, the base payment and the multipiers. This information is public and accessible though the blockchain explorer for every person. Each payee multipliers can be updated individually E.g. if a developer is promoted, his seniority multiplier will be changed so he will earn more money. The contract also will be pausable, so the owner can stop the payments if needed. We will provide some helper functions to let the payees know how much they can claim at any given time, and also a function that the owner can call to know how much will be paid in the next period with the current parameters.

## Contract Structures

Some comments may be found in the contract, but here you can take a look at an overview of the main structures used to repesent the domain:

``` RUST
type Multiplier = u128;
```

``` RUST
pub struct Beneficiary {
    account_id: AccountId,
    multipliers: Vec<Multiplier>,
    unclaimed_payments: Balance,
    last_claimed_period_block: BlockNumber,
}
```

``` RUST
pub struct OpenPayroll {
    owner: AccountId,
    beneficiaries: Mapping<AccountId, Beneficiary>,
    beneficiaries_accounts: Vec<AccountId>,
    periodicity: u32,
    base_payment: Balance,
    initial_block: u32,
    paused_block_at: Option<u32>,
    base_multipliers: Vec<String>,
}
```

## Contract functions

``` RUST
new(periodicity: u32, base_payment: Balance, base_multipliers: Vec<String>)
add_or_update_beneficiary(account_id: AccountId, multipliers: Vec<Multiplier>)
remove_beneficiary(account_id: AccountId) 
update_base_payment(base_payment: Balance)
update_periodicity(periodicity: u32)
update_storage_claim(account_id: AccountId) 
ensure_all_payments_uptodate()
get_amount_to_claim(account_id: AccountId) // returns pending balance to claim for a given account
claim_payment() // transfer pending balance to the sender account (if there's pending balance) 
calculate_outstanding_payments() // this function will calculate how much the contract remains unclaimed
get_beneficiary(account_id: AccountId) // given an address will return the beneficiary
```

## Design desicions: 

Here are some notes about some tech decisions we made during the development:

- In this version initial block will be set to the current block
- Owner is set to the account who called the constructor
- Beneficiaries are empty and need to be filled by the function provided
- Base multipliers can be empty, that means non multiplier will be applied and the beneficiary will have `base_payment * 1` on each period
- Multipliers multiply the base amount and add that amount to the period balance.
Eg: base_payment: 1000, Multipliers {seniority: 2 and experience_in_project: .5} that will produce the following math:
base_payment (1000) + seniority (2000) + experinece_in_project (500) = total for the period (3500)
- the function `ensure_all_payments_uptodate` is used to check if there's something remaining to claim before changing core parameters in order to avoid altering past periods amount. If something is missing to claim `update_base_payment` or `update_periodicity` won't be allowed by the contract.
- the contracts created will be pausable adding the pause/resume function, that can be called only by it's owner.

## Usage 
test:
`cargo test`

compile:
`cargo +nightly contract build --release`

