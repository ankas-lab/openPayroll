#![cfg_attr(not(feature = "std"), no_std)]
mod errors;

#[ink::contract]
mod open_payroll {
    use crate::errors::Error;
    use ink::prelude::collections::BTreeMap;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::traits::StorageLayout;
    use ink::storage::Mapping;

    // Define the types that will be used in the contract
    type Multiplier = u128;
    type MultiplierId = u32;
    // Establish the maximum number of beneficiaries and multipliers that can be added to the contract
    const MAX_BENEFICIARIES: usize = 100;
    const MAX_MULTIPLIERS: usize = 10;

    /// Emitted when a beneficiary claims their payment
    #[ink(event)]
    pub struct Claimed {
        #[ink(topic)]
        account_id: AccountId,
        amount: Balance,
        total_payment: Balance,
        claiming_period_block: BlockNumber,
    }

    /// Emitted when a multiplier is deactivated
    #[ink(event)]
    pub struct MultiplierDeactivated {
        #[ink(topic)]
        multiplier_id: MultiplierId,
        valid_until_block: BlockNumber,
    }

    /// Emitted when a multiplier is deleted
    #[ink(event)]
    pub struct MultiplierDeleted {
        #[ink(topic)]
        multiplier_id: MultiplierId,
        valid_until_block: BlockNumber,
    }

    /// Emiited when the ownership of the contract is transferred
    #[ink(event)]
    pub struct OwnershipProposed {
        #[ink(topic)]
        current_owner: AccountId,
        #[ink(topic)]
        proposed_owner: AccountId,
    }

    /// Emitted when the ownership of the contract is accepted
    #[ink(event)]
    pub struct OwnershipAccepted {
        #[ink(topic)]
        previous_owner: AccountId,
        #[ink(topic)]
        new_owner: AccountId,
    }

    /// Emitted when a beneficiary is added
    #[ink(event)]
    pub struct BeneficiaryAdded {
        #[ink(topic)]
        account_id: AccountId,
        multipliers_vec: Vec<(MultiplierId, Multiplier)>,
    }

    /// Emitted when a beneficiary is updated
    #[ink(event)]
    pub struct BeneficiaryUpdated {
        #[ink(topic)]
        account_id: AccountId,
        multipliers_vec: Vec<(MultiplierId, Multiplier)>,
    }

    /// Emitted when a beneficiary is removed
    #[ink(event)]
    pub struct BeneficiaryRemoved {
        #[ink(topic)]
        account_id: AccountId,
    }

    /// Emitted when a multiplier is added
    #[ink(event)]
    pub struct BaseMultiplierAdded {
        multiplier_id: MultiplierId,
        name: String,
    }

    /// Emitted when the preiodicity is updated
    #[ink(event)]
    pub struct PeriodicityUpdated {
        periodicity: u32,
    }

    /// Emitted when the contract is paused
    #[ink(event)]
    pub struct Paused {}

    /// Emitted when the contract is resumed
    #[ink(event)]
    pub struct Resumed {}

    /// Base multiplier structure containg a name and an option block number for being used when deactivating the multiplier
    #[derive(scale::Encode, scale::Decode, Eq, PartialEq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct BaseMultiplier {
        name: String,
        valid_until_block: Option<BlockNumber>,
    }
    impl BaseMultiplier {
        pub fn new(name: String) -> Self {
            Self {
                name,
                valid_until_block: None,
            }
        }
    }

    /// Beneficiary structure containing the account id, the multipliers, the unclaimed payments, and the last updated period block
    #[derive(scale::Encode, scale::Decode, Eq, PartialEq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout,))]
    pub struct Beneficiary {
        account_id: AccountId,
        multipliers: BTreeMap<MultiplierId, Multiplier>,
        unclaimed_payments: Balance,
        last_updated_period_block: BlockNumber,
    }

    /// Initial beneficiary structure containing the account id and the multipliers
    #[derive(scale::Encode, scale::Decode, Eq, PartialEq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct InitialBeneficiary {
        account_id: AccountId,
        // Vector rather than BTreeMap because its easier to buid from the frontend
        multipliers: Vec<(MultiplierId, Multiplier)>,
    }

    /// Claims in period structure containing the period and the total claims
    #[derive(scale::Encode, scale::Decode, Eq, PartialEq, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct ClaimsInPeriod {
        period: u32,
        total_claims: u32,
    }

    /// OpenPayroll contract structure containing the owner, the beneficiaries, the multipliers, the base payment, the periodicity,
    /// the initial block, the last updated block, the claims in period, the paused state, and the base multipliers
    // The presence of redundant information between the 'AccountsIds' in 'beneficiaries' and 'beneficiaries_accounts' is intentional.
    // Although they represent the same account, this redundancy is maintained in order to support efficient iteration over
    // 'beneficiaries_accounts' while fetching a beneficiary. By duplicating the account IDs, we achieve a constant time complexity of
    // O(1) when accessing beneficiary information directly from 'beneficiaries_accounts'.
    // Similarly, the redundancy in iterating over both 'MultiplierIds' in 'multipliers_list' and 'BaseMultipliers' is intentional for
    // improved access to the 'BaseMultiplier' field. Although 'MultiplierIds' could directly link to the corresponding 'BaseMultiplier',
    // maintaining both lists allows for efficient iteration over 'multipliers_list' while accessing the 'BaseMultiplier' values.
    // This design choice enables streamlined retrieval of relevant multiplier information without compromising performance.
    #[ink(storage)]
    pub struct OpenPayroll {
        /// The account to be transfered to, until the new owner accept it
        proposed_owner: Option<AccountId>,
        /// The accountId of the creator of the contract, who has 'priviliged' access to do administrative tasks
        owner: AccountId,
        /// Mapping from the accountId to the beneficiary information
        beneficiaries: Mapping<AccountId, Beneficiary>,
        /// Vector of Accounts
        beneficiaries_accounts: Vec<AccountId>,
        /// The payment periodicity in blocks
        periodicity: u32,
        /// The amount of each base payment
        base_payment: Balance,
        /// The initial block number
        initial_block: u32,
        /// The block number when the contract was paused
        paused_block_at: Option<u32>,
        /// The id of the next multiplier to be added
        next_multiplier_id: MultiplierId,
        /// The multipliers to apply to the base payment
        base_multipliers: Mapping<MultiplierId, BaseMultiplier>,
        /// A list of the multipliers_ids
        multipliers_list: Vec<MultiplierId>,
        /// Current claims in period
        claims_in_period: ClaimsInPeriod,
    }

    /// implementation of the OpenPayroll contract
    impl OpenPayroll {
        /// Constructor that initializes the owner, the base payment, the periodicity, the initial block, the base multipliers,
        /// and the initial beneficiaries
        #[ink(constructor, payable)]
        pub fn new(
            periodicity: u32,
            base_payment: Balance,
            initial_base_multipliers: Vec<String>,
            initial_beneficiaries: Vec<InitialBeneficiary>,
        ) -> Result<Self, Error> {
            let initial_block_number = Self::env().block_number();
            let proposed_owner = None;
            let owner = Self::env().caller();
            let mut next_multiplier_id = 0;

            // 0 payment or 0 periodicity make no sense
            if base_payment == 0 || periodicity == 0 {
                return Err(Error::InvalidParams);
            }

            // Check for duplicate beneficiaries
            check_no_duplicate_beneficiaries(
                &initial_beneficiaries.iter().map(|b| b.account_id).collect(),
            )?;

            // Check beneficiaries and multipliers limits
            if initial_beneficiaries.len() > MAX_BENEFICIARIES {
                return Err(Error::MaxBeneficiariesExceeded);
            }
            if initial_base_multipliers.len() > MAX_MULTIPLIERS {
                return Err(Error::MaxMultipliersExceeded);
            }

            let mut beneficiaries = Mapping::default();
            let mut accounts = Vec::new();
            let mut base_multipliers = Mapping::default();
            let mut multipliers_list = Vec::new();

            // Create the base multipliers
            for base_multiplier in initial_base_multipliers.iter() {
                base_multipliers.insert(
                    next_multiplier_id,
                    &BaseMultiplier::new(base_multiplier.clone()),
                );
                multipliers_list.push(next_multiplier_id);
                next_multiplier_id += 1;
            }

            // Create the initial beneficiaries
            for beneficiary_data in initial_beneficiaries.iter() {
                if beneficiary_data.multipliers.len() != multipliers_list.len() {
                    return Err(Error::InvalidMultipliersLength);
                }

                // Check for duplicate multipliers
                check_no_duplicate_multipliers(&beneficiary_data.multipliers)?;

                let multipliers = vec_to_btreemap(&beneficiary_data.multipliers);

                let beneficiary = Beneficiary {
                    account_id: beneficiary_data.account_id,
                    multipliers,
                    unclaimed_payments: 0,
                    last_updated_period_block: initial_block_number,
                };

                beneficiaries.insert(beneficiary_data.account_id, &beneficiary);
                accounts.push(beneficiary_data.account_id);
            }

            // Defines the claims in period
            let claims_in_period = ClaimsInPeriod {
                period: 0,
                total_claims: 0,
            };

            Ok(Self {
                proposed_owner,
                owner,
                beneficiaries,
                periodicity,
                base_payment,
                initial_block: initial_block_number,
                paused_block_at: None,
                beneficiaries_accounts: accounts,
                next_multiplier_id,
                base_multipliers,
                multipliers_list,
                claims_in_period,
            })
        }

        /// Claim payment for a single account id
        // If the amount is 0 no money is transferred. However, the "unclaimed_payments" field is set to the total
        // value that the beneficiary has yet to claim.
        #[ink(message)]
        pub fn claim_payment(
            &mut self,
            account_id: AccountId,
            amount: Balance,
        ) -> Result<(), Error> {
            self.ensure_is_not_paused()?;

            let beneficiary_res = self.beneficiaries.get(account_id);

            let mut beneficiary = match beneficiary_res {
                Some(b) => b,
                None => return Err(Error::AccountNotFound),
            };

            let current_block = self.env().block_number();

            // If there are deactivated multipliers, remove them from the beneficiary
            beneficiary.multipliers.retain(|&k, _| {
                let multiplier_block_validity =
                    self.base_multipliers.get(k).unwrap().valid_until_block;

                // We keep the multiplier if it is not deactivated
                // or if it is deactivated but the current block is before the deactivation block
                multiplier_block_validity.is_none()
                    || multiplier_block_validity.unwrap() > current_block
            });

            // gets the total amount that the beneficiary can claim and check the amount is not bigger than that
            let total_payment = self._get_amount_to_claim(account_id, true);
            if amount > total_payment {
                return Err(Error::ClaimedAmountIsBiggerThanAvailable);
            }

            // Check if the treasury has enough balance
            let treasury_balance = self.env().balance();
            if amount > treasury_balance {
                return Err(Error::NotEnoughBalanceInTreasury);
            }

            let claiming_period_block = self.get_current_period_initial_block();

            // If the beneficiary has not claimed anything in the current period
            if beneficiary.last_updated_period_block != claiming_period_block {
                self._update_claims_in_period(claiming_period_block);
            }

            // Update the beneficiary
            self.beneficiaries.insert(
                account_id,
                &Beneficiary {
                    account_id,
                    multipliers: beneficiary.multipliers,
                    unclaimed_payments: total_payment - amount,
                    last_updated_period_block: claiming_period_block,
                },
            );

            // Transfer the amount to the beneficiary if amount > 0
            if amount > 0 && self.env().transfer(account_id, amount).is_err() {
                return Err(Error::TransferFailed);
            }

            // Emit the Claimed event
            self.env().emit_event(Claimed {
                account_id,
                amount,
                total_payment,
                claiming_period_block,
            });

            Ok(())
        }

        /// Deactivate a multiplier
        /// It can be deleted one period after deactivation if every beneficiary has claimed the payment
        #[ink(message)]
        pub fn deactivate_multiplier(&mut self, multiplier_id: MultiplierId) -> Result<(), Error> {
            // Fetch the multiplier
            let mut multiplier = self
                .base_multipliers
                .get(multiplier_id)
                .ok_or(Error::MultiplierNotFound)?;
            // Check if the multiplier is already deactivated
            if multiplier.valid_until_block.is_some() {
                return Err(Error::MultiplierAlreadyDeactivated);
            }

            // Calculates deactivation on next period
            let valid_until_block = self.get_current_period_initial_block() + self.periodicity;

            // Set that value in the multiplier
            multiplier.valid_until_block = Some(valid_until_block);
            self.base_multipliers.insert(multiplier_id, &multiplier);

            // Emit the MultiplierDeactivated event
            self.env().emit_event(MultiplierDeactivated {
                multiplier_id,
                valid_until_block,
            });

            Ok(())
        }

        /// Delete a multiplier when conditions are met
        #[ink(message)]
        pub fn delete_unused_multiplier(
            &mut self,
            multiplier_id: MultiplierId,
        ) -> Result<(), Error> {
            let current_block = self.env().block_number();
            let multiplier = self
                .base_multipliers
                .get(multiplier_id)
                .ok_or(Error::MultiplierNotFound)?;

            // Check if the multiplier is already deactivated
            if multiplier.valid_until_block.is_none() {
                return Err(Error::MultiplierNotDeactivated);
            }

            // Check if the multiplier is expired
            if current_block > multiplier.valid_until_block.unwrap() {
                return Err(Error::MultiplierNotExpired);
            }

            // Check if all beneficiaries have claimed the payment
            self.ensure_all_claimed_in_period()?;

            // Remove multiplier from multipliers_list
            self.multipliers_list.retain(|x| *x != multiplier_id);

            // Remove multiplier from base_multipliers
            self.base_multipliers.remove(multiplier_id);

            // Emit the MultiplierDeleted event
            self.env().emit_event(MultiplierDeleted {
                multiplier_id,
                valid_until_block: multiplier.valid_until_block.unwrap(),
            });

            Ok(())
        }

        // Ensure_owner ensures that the caller is the owner of the contract
        fn ensure_owner(&self) -> Result<(), Error> {
            let account = self.env().caller();
            // Only owners can call this function
            if self.owner != account {
                return Err(Error::NotOwner);
            }
            Ok(())
        }

        /// Reads the paused state from the contract
        #[ink(message)]
        pub fn is_paused(&self) -> bool {
            self.paused_block_at.is_some()
        }

        // ensure_is_not_paused ensures that the contract is not paused
        fn ensure_is_not_paused(&self) -> Result<(), Error> {
            if self.is_paused() {
                return Err(Error::ContractIsPaused);
            }
            Ok(())
        }

        // Check multipliers are valid
        fn check_multipliers_are_valid(
            &self,
            multipliers: &[(MultiplierId, Multiplier)],
        ) -> Result<(), Error> {
            for (multiplier_id, _) in multipliers.iter() {
                if !self.base_multipliers.contains(multiplier_id) {
                    return Err(Error::MultiplierNotFound);
                }
                if self
                    .base_multipliers
                    .get(multiplier_id)
                    .unwrap()
                    .valid_until_block
                    .is_some()
                {
                    return Err(Error::MultiplierAlreadyDeactivated);
                }
            }
            Ok(())
        }

        /// Change ownership of the contract
        /// This is proposing a new owner that has to accept the ownership
        #[ink(message)]
        pub fn propose_transfer_ownership(&mut self, new_owner: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            self.proposed_owner = Some(new_owner);

            // Emit the OwnershipTransferred event
            self.env().emit_event(OwnershipProposed {
                current_owner: self.owner,
                proposed_owner: new_owner,
            });

            Ok(())
        }

        /// Accept ownership of the contract
        /// Once the ownership is proposed by transfer_ownsership function it needs to be accepted
        /// by the new owner. This prevents accidental ownership transfers.
        #[ink(message)]
        pub fn accept_ownership(&mut self) -> Result<(), Error> {
            let old_owner = self.owner;
            if self.proposed_owner == Some(self.env().caller()) {
                self.owner = self.proposed_owner.unwrap();
                self.proposed_owner = None;

                self.env().emit_event(OwnershipAccepted {
                    previous_owner: old_owner,
                    new_owner: self.owner,
                });

                Ok(())
            } else {
                Err(Error::NotOwner)
            }
        }

        // Function for doing the checking before adding a new beneficiary
        fn check_beneficiary_to_add(
            &self,
            account_id: AccountId,
            multipliers: &[(MultiplierId, Multiplier)],
        ) -> Result<(), Error> {
            self.ensure_owner()?;

            // Check that the beneficiary does not exist
            if self.beneficiaries.contains(account_id) {
                return Err(Error::AccountAlreadyExists);
            }

            // Check that the number of beneficiaries does not exceed the maximum
            if self.beneficiaries_accounts.len() + 1 > MAX_BENEFICIARIES {
                return Err(Error::MaxBeneficiariesExceeded);
            }

            // Check that the multipliers are valid
            self.check_multipliers_are_valid(multipliers)?;
            check_no_duplicate_multipliers(&std::vec::Vec::from(multipliers))?;

            Ok(())
        }

        /// Add a new beneficiary
        #[ink(message)]
        pub fn add_beneficiary(
            &mut self,
            account_id: AccountId,
            multipliers: Vec<(MultiplierId, Multiplier)>,
        ) -> Result<(), Error> {
            // Calls the function to do the checking
            self.check_beneficiary_to_add(account_id, &multipliers)?;

            let multipliers_vec = multipliers.clone();
            let multipliers = vec_to_btreemap(&multipliers);

            // insert the new beneficiary
            self.beneficiaries.insert(
                account_id,
                &Beneficiary {
                    account_id,
                    multipliers,
                    unclaimed_payments: 0,
                    last_updated_period_block: self.get_current_period_initial_block(),
                },
            );

            // Add the beneficiary to the list of beneficiaries
            self.beneficiaries_accounts.push(account_id);

            // Emit the BeneficiaryAdded event
            self.env().emit_event(BeneficiaryAdded {
                account_id,
                multipliers_vec,
            });

            Ok(())
        }

        /// Update an existing beneficiary
        #[ink(message)]
        pub fn update_beneficiary(
            &mut self,
            account_id: AccountId,
            multipliers: Vec<(MultiplierId, Multiplier)>,
        ) -> Result<(), Error> {
            self.ensure_owner()?;

            // Check that the beneficiary exists
            if !self.beneficiaries.contains(account_id) {
                return Err(Error::AccountNotFound);
            }

            // Check that the multipliers are valid
            self.check_multipliers_are_valid(&multipliers)?;
            check_no_duplicate_multipliers(&multipliers)?;

            let multipliers_vec = multipliers.clone();
            let multipliers = vec_to_btreemap(&multipliers);

            // calculate the amount to claim to be transferred to the uncleared payments
            let unclaimed_payments = self._get_amount_to_claim(account_id, false);

            // update de beneficiary with new multipliers and new unclaimed payments
            self.beneficiaries.insert(
                account_id,
                &Beneficiary {
                    account_id,
                    multipliers,
                    unclaimed_payments,
                    last_updated_period_block: self.get_current_period_initial_block(),
                },
            );

            // Emit the BeneficiaryUpdated event
            self.env().emit_event(BeneficiaryUpdated {
                account_id,
                multipliers_vec,
            });

            Ok(())
        }

        /// Remove a beneficiary
        #[ink(message)]
        pub fn remove_beneficiary(&mut self, account_id: AccountId) -> Result<(), Error> {
            self.ensure_owner()?;
            if !self.beneficiaries.contains(account_id) {
                return Err(Error::AccountNotFound);
            }
            self.beneficiaries.remove(account_id);

            // Remove the beneficiary from the list of beneficiaries
            self.beneficiaries_accounts.retain(|x| *x != account_id);

            // Emit the BeneficiaryRemoved event
            self.env().emit_event(BeneficiaryRemoved { account_id });

            Ok(())
        }

        /// Update the base_payment
        /// It makes sense once all the beneficiaries have claimed their payments
        #[ink(message)]
        pub fn update_base_payment(&mut self, base_payment: Balance) -> Result<(), Error> {
            self.ensure_owner()?;
            if base_payment == 0 {
                return Err(Error::InvalidParams);
            }

            //check if all payments are up to date
            self.ensure_all_claimed_in_period()?;
            self.base_payment = base_payment;

            Ok(())
        }

        /// Add a new base multiplier
        /// It's not checking for duplicates because it's just a string
        #[ink(message)]
        pub fn add_base_multiplier(&mut self, name: String) -> Result<(), Error> {
            self.ensure_owner()?;

            // Check that the number of multipliers does not exceed the maximum
            if self.multipliers_list.len() + 1 > MAX_MULTIPLIERS {
                return Err(Error::MaxMultipliersExceeded);
            }

            let base_multiplier = BaseMultiplier::new(name.clone());

            self.base_multipliers
                .insert(self.next_multiplier_id, &base_multiplier);

            self.multipliers_list.push(self.next_multiplier_id);

            // Increment the next_multiplier_id checking for overflow
            self.next_multiplier_id = match self.next_multiplier_id.checked_add(1) {
                Some(val) => val,
                None => return Err(Error::Overflow),
            };

            // Emit the BaseMultiplierAdded event
            self.env().emit_event(BaseMultiplierAdded {
                multiplier_id: self.next_multiplier_id - 1,
                name,
            });

            Ok(())
        }

        /// Update the periodicity of the payments
        /// All payments must be claimed before updating the periodicity
        #[ink(message)]
        pub fn update_periodicity(&mut self, periodicity: u32) -> Result<(), Error> {
            self.ensure_owner()?;
            if periodicity == 0 {
                return Err(Error::InvalidParams);
            }

            //check if all payments are up to date
            //self.ensure_all_payments_uptodate()?;
            self.ensure_all_claimed_in_period()?;
            self.periodicity = periodicity;

            // Emit the PeriodicityUpdated event
            self.env().emit_event(PeriodicityUpdated { periodicity });

            Ok(())
        }

        /// Check if all payments up to date or storage unclaiumed_payments is up-to-date
        #[ink(message)]
        pub fn ensure_all_payments_uptodate(&self) -> Result<(), Error> {
            let claimed_period_block = self.get_current_period_initial_block();

            // iterates over each account_id
            for account_id in self.beneficiaries_accounts.iter() {
                let beneficiary = self.beneficiaries.get(account_id).unwrap();

                if claimed_period_block > beneficiary.last_updated_period_block {
                    return Err(Error::PaymentsNotUpToDate);
                }
            }

            Ok(())
        }

        /// Get the amount of tokens that can be claimed by a beneficiary with specific block_numer
        fn _get_amount_to_claim_in_block(
            &self,
            account_id: AccountId,
            filtered_multipliers: bool,
            block: BlockNumber,
        ) -> Balance {
            // The check that beneficiary exists is done in the caller function
            let beneficiary = self.beneficiaries.get(account_id).unwrap();

            // Calculates the number of blocks that have elapsed since the last payment
            let blocks_since_last_payment = block - beneficiary.last_updated_period_block;

            // Calculates the number of periods that are due based on the elapsed blocks
            let unclaimed_periods: u128 = (blocks_since_last_payment / self.periodicity).into();

            // If there's no unclaimed periods, return the unclaimed payments
            // Otherwise, calculate the amount to claim and add the unclaimed payments
            if unclaimed_periods == 0 {
                beneficiary.unclaimed_payments
            } else {
                let payment_per_period =
                    self._get_amount_to_claim_for_one_period(&beneficiary, filtered_multipliers);

                payment_per_period * unclaimed_periods + beneficiary.unclaimed_payments
            }
        }

        /// check the amount to claim for one beneficiary in any period
        /// without unclaimed payments
        fn _get_amount_to_claim_for_one_period(
            &self,
            beneficiary: &Beneficiary,
            filtered_multipliers: bool,
        ) -> Balance {
            // E.g (M1 + M2) * B / 100
            // Sum all active multipliers
            let final_multiplier: u128 = if beneficiary.multipliers.is_empty() {
                1
            } else {
                match filtered_multipliers {
                    true => beneficiary.multipliers.values().sum(),
                    _ => beneficiary
                        .multipliers
                        .iter()
                        .filter(|(k, _)| {
                            self.base_multipliers
                                .get(k)
                                .unwrap()
                                .valid_until_block
                                .is_none()
                        })
                        .map(|(_, v)| v)
                        .sum(),
                }
            };

            final_multiplier * self.base_payment / 100
        }

        /// internal function to get the amount to claim
        /// filtered multipliers in true means that all multipliers are active
        fn _get_amount_to_claim(
            &self,
            account_id: AccountId,
            filtered_multipliers: bool,
        ) -> Balance {
            let current_block = self.env().block_number();

            self._get_amount_to_claim_in_block(account_id, filtered_multipliers, current_block)
        }

        /// Get amount in storage without transferring the funds
        /// Read Only function
        #[ink(message)]
        pub fn get_amount_to_claim(&self, account_id: AccountId) -> Result<Balance, Error> {
            if !self.beneficiaries.contains(account_id) {
                return Err(Error::AccountNotFound);
            }

            let result = self._get_amount_to_claim(account_id, false);

            Ok(result)
        }

        /// Updates the number of claims in a period
        /// If the period is the same, it increments the number of claims
        /// Otherwise, it resets the number of claims and set it to 1
        fn _update_claims_in_period(&mut self, claiming_period_block: BlockNumber) {
            if claiming_period_block == self.claims_in_period.period {
                // Updates current claims in period
                self.claims_in_period.total_claims += 1;
            } else {
                // Reset the claims in period
                self.claims_in_period.period = claiming_period_block;
                self.claims_in_period.total_claims = 1;
            }
        }

        /// check if all beneficiaries claimed in period
        fn ensure_all_claimed_in_period(&mut self) -> Result<(), Error> {
            let claiming_period_block = self.get_current_period_initial_block();

            let claims_in_period = self.claims_in_period.clone();

            if (claiming_period_block == claims_in_period.period
                && claims_in_period.total_claims == self.beneficiaries_accounts.len() as u32)
                || claiming_period_block == 0
            // initial period in intial block noone can claim
            {
                return Ok(());
            }

            Err(Error::NotAllClaimedInPeriod)
        }

        /// Pause the contract
        /// Pausing will only avoid to call the claim function
        #[ink(message)]
        pub fn pause(&mut self) -> Result<(), Error> {
            self.ensure_owner()?;
            if self.is_paused() {
                return Ok(());
            }
            self.paused_block_at = Some(self.env().block_number());
            self.env().emit_event(Paused {});
            Ok(())
        }

        /// Resume the contract
        /// Resuming will allow to call the claim function
        #[ink(message)]
        pub fn resume(&mut self) -> Result<(), Error> {
            self.ensure_owner()?;
            if !self.is_paused() {
                return Ok(());
            }
            self.paused_block_at = None;
            self.env().emit_event(Resumed {});
            Ok(())
        }

        /// Get beneficiary only read
        /// Read Only function
        #[ink(message)]
        pub fn get_beneficiary(&mut self, account_id: AccountId) -> Result<Beneficiary, Error> {
            if !self.beneficiaries.contains(account_id) {
                return Err(Error::AccountNotFound);
            }
            let beneficiary = self.beneficiaries.get(account_id).unwrap();
            Ok(beneficiary)
        }

        /// Get current block period
        /// Read Only function
        // The calculation current_block - ((current_block - self.initial_block) % self.periodicity) might be a bit tricky to understand at first glance.
        // Let's use an example to understand it. Assume self.initial_block to be 10, self.periodicity to be 20, and the current_block to be 65.
        // current_block - self.initial_block = 65 - 10 = 55 55 % self.periodicity = 55 % 20 = 15.
        // This gives us the number of blocks past the last "period start" in relation to initial_block and periodicity.  current_block - 15 = 65 - 15 = 50.
        // This is the block number where the current period started.
        #[ink(message)]
        pub fn get_current_period_initial_block(&self) -> BlockNumber {
            let current_block = self.env().block_number();
            current_block - ((current_block - self.initial_block) % self.periodicity)
        }

        /// Get next block period
        #[ink(message)]
        pub fn get_next_block_period(&self) -> BlockNumber {
            self.get_current_period_initial_block() + self.periodicity
        }

        /// Get all the debts up-to-date
        /// Read Only function
        #[ink(message)]
        pub fn get_total_debts(&self) -> Balance {
            let mut debts = 0;
            for account_id in self.beneficiaries_accounts.iter() {
                let beneficiary = self.beneficiaries.get(account_id).unwrap();
                debts += self._get_amount_to_claim(beneficiary.account_id, false);
            }

            debts
        }

        /// Get all the debts for the next period
        /// Read Only function
        #[ink(message)]
        pub fn get_total_debt_for_next_period(&self) -> Balance {
            let mut total = 0;
            for account_id in self.beneficiaries_accounts.iter() {
                let beneficiary = self.beneficiaries.get(account_id).unwrap();
                let amount = self._get_amount_to_claim_for_one_period(&beneficiary, false);
                total += amount;
            }

            total
        }

        /// Get all the debts including unclaimed for the next period
        /// Read Only function
        #[ink(message)]
        pub fn get_total_debt_with_unclaimed_for_next_period(&self) -> Balance {
            let block_next_period = self.get_next_block_period();

            let mut total = 0;
            for account_id in self.beneficiaries_accounts.iter() {
                let amount =
                    self._get_amount_to_claim_in_block(*account_id, false, block_next_period);
                total += amount;
            }

            total
        }

        /// Get all the beneficiaries
        /// Read Only function
        #[ink(message)]
        pub fn get_list_beneficiaries(&self) -> Vec<AccountId> {
            self.beneficiaries_accounts.clone()
        }

        /// Get contract balance
        /// Read Only function
        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        /// Get total balance after paying debts
        /// Read Only function
        #[ink(message)]
        pub fn get_balance_with_debts(&self) -> Balance {
            self.get_contract_balance() - self.get_total_debts()
        }

        /// Get list of unclaimed beneficiaries
        /// Read Only function
        #[ink(message)]
        pub fn get_unclaimed_beneficiaries(&self) -> Vec<AccountId> {
            let claiming_period_block = self.get_current_period_initial_block();

            let mut unclaimed_beneficiaries = Vec::new();
            // iterate over all beneficiaries
            // if last_updated_period_block < claiming_period_block
            // then add to unclaimed_beneficiaries
            for account_id in self.beneficiaries_accounts.iter() {
                let beneficiary = self.beneficiaries.get(account_id).unwrap();
                if beneficiary.last_updated_period_block < claiming_period_block {
                    unclaimed_beneficiaries.push(beneficiary.account_id);
                }
            }

            unclaimed_beneficiaries
        }

        /// Get count of unclaimed beneficiaries
        /// Read Only function
        #[ink(message)]
        pub fn get_count_of_unclaim_beneficiaries(&self) -> u8 {
            let claiming_period_block = self.get_current_period_initial_block();
            let mut total: u8 = 0;
            for account_id in self.beneficiaries_accounts.iter() {
                let beneficiary = self.beneficiaries.get(account_id).unwrap();
                if beneficiary.last_updated_period_block < claiming_period_block {
                    total += 1;
                }
            }

            total
        }

        /// Get the base amount to claim for each beneficiary
        #[ink(message)]
        pub fn get_base_payment(&self) -> Balance {
            self.base_payment
        }

        /// Get the periodicity of the payments
        #[ink(message)]
        pub fn get_periodicity(&self) -> BlockNumber {
            self.periodicity
        }

        /// Get the initial block of the contract
        #[ink(message)]
        pub fn get_initial_block(&self) -> BlockNumber {
            self.initial_block
        }

        /// Get the base multiplier
        #[ink(message)]
        pub fn get_multipliers_list(&self) -> Vec<MultiplierId> {
            self.multipliers_list.clone()
        }

        /// Get a base multiplier based on its id
        #[ink(message)]
        pub fn get_base_multiplier(&self, multiplier_id: MultiplierId) -> BaseMultiplier {
            self.base_multipliers.get(multiplier_id).unwrap()
        }
    }

    /// ---------------------------------------------------------------
    /// Pure functions
    /// ---------------------------------------------------------------

    /// Given a vector of (id, multiplier) pairs, return a BTreeMap of (id, multiplier) pairs
    fn vec_to_btreemap(vec: &[(MultiplierId, Multiplier)]) -> BTreeMap<MultiplierId, Multiplier> {
        let mut btree_map = BTreeMap::new();
        for (id, multiplier) in vec.iter() {
            btree_map.insert(*id, *multiplier);
        }
        btree_map
    }

    /// Given a list of beneficiaries it checks there are no duplicates
    #[allow(clippy::all)]
    fn check_no_duplicate_beneficiaries(beneficiaries: &Vec<AccountId>) -> Result<(), Error> {
        let mut sorted_beneficiaries = beneficiaries.clone();
        sorted_beneficiaries.sort_by_key(|&beneficiary| beneficiary);

        for i in 1..sorted_beneficiaries.len() {
            if sorted_beneficiaries[i - 1] == sorted_beneficiaries[i] {
                return Err(Error::DuplicatedBeneficiaries);
            }
        }

        Ok(())
    }

    /// Given a list of multipliers it checks there are no duplicates
    #[allow(clippy::all)]
    fn check_no_duplicate_multipliers(
        multipliers: &Vec<(MultiplierId, Multiplier)>,
    ) -> Result<(), Error> {
        let mut sorted_multipliers = multipliers.clone();
        sorted_multipliers.sort_by_key(|&(multiplier_id, _)| multiplier_id);

        for i in 1..sorted_multipliers.len() {
            if sorted_multipliers[i - 1].0 == sorted_multipliers[i].0 {
                return Err(Error::DuplicatedMultipliers);
            }
        }

        Ok(())
    }
    /// ---------------------------------------------------------------

    /// ---------------------------------------------------------------
    ///
    ///
    ///
    ///    Test Cases
    ///
    ///
    ///
    /// ---------------------------------------------------------------
    #[cfg(test)]
    mod tests {
        use ink::{
            env::{test::DefaultAccounts, DefaultEnvironment},
            primitives::AccountId,
        };

        use super::*;

        // UTILITY FUNCTIONS TO MAKE TESTING EASIER
        fn create_contract(
            initial_balance: Balance,
            accounts: &DefaultAccounts<DefaultEnvironment>,
        ) -> OpenPayroll {
            set_balance(contract_id(), initial_balance);
            let beneficiary_bob = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 100), (1, 3)],
            };
            let beneficiary_charlie = InitialBeneficiary {
                account_id: accounts.charlie,
                multipliers: vec![(0, 100), (1, 3)],
            };
            OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![beneficiary_bob, beneficiary_charlie],
            )
            .expect("Cannot create contract")
        }

        fn create_contract_with_no_beneficiaries(initial_balance: Balance) -> OpenPayroll {
            set_balance(contract_id(), initial_balance);
            OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![],
            )
            .expect("Cannot create contract")
        }

        fn create_contract_with_no_beneficiaries_periodicity(
            initial_balance: Balance,
            periodicity: u32,
        ) -> OpenPayroll {
            set_balance(contract_id(), initial_balance);
            OpenPayroll::new(
                periodicity,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![],
            )
            .expect("Cannot create contract")
        }

        fn create_accounts_and_contract(
            initial_balance: Balance,
        ) -> (
            ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment>,
            OpenPayroll,
        ) {
            let accounts = default_accounts();
            set_sender(accounts.alice);

            let contract = create_contract(initial_balance, &accounts);
            (accounts, contract)
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        fn advance_n_blocks(n: u32) {
            for _ in 0..n {
                ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
            }
        }

        fn get_current_block() -> u32 {
            ink::env::block_number::<ink::env::DefaultEnvironment>()
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }

        fn vec_to_btreemap(
            vec: &[(MultiplierId, Multiplier)],
        ) -> BTreeMap<MultiplierId, Multiplier> {
            let mut btree_map = BTreeMap::new();
            for (id, multiplier) in vec.iter() {
                btree_map.insert(*id, *multiplier);
            }
            btree_map
        }

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            create_contract(100_000_000u128, &accounts)
        }

        #[ink::test]
        fn create_contract_ok() {
            let accounts = default_accounts();
            let beneficiary_bob = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 100), (1, 3)],
            };
            let beneficiary_charlie = InitialBeneficiary {
                account_id: accounts.charlie,
                multipliers: vec![(0, 100), (1, 10)],
            };
            let res = OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![beneficiary_bob, beneficiary_charlie],
            );
            assert!(matches!(res, Ok(_)));
            let contract = res.unwrap();

            // check that base_multipliers are set correctly
            let data_0 = contract.base_multipliers.get(0).unwrap();
            let data_1 = contract.base_multipliers.get(1).unwrap();
            assert_eq!(
                data_0,
                BaseMultiplier {
                    name: "Seniority".to_string(),
                    valid_until_block: None,
                }
            );
            assert_eq!(
                data_1,
                BaseMultiplier {
                    name: "Performance".to_string(),
                    valid_until_block: None,
                }
            );

            // check that beneficiaries are set correctly
            let data_bob = contract.beneficiaries.get(accounts.bob).unwrap();
            let data_charlie = contract.beneficiaries.get(accounts.charlie).unwrap();
            assert_eq!(
                data_bob,
                Beneficiary {
                    account_id: accounts.bob,
                    multipliers: vec_to_btreemap(&[(0, 100), (1, 3)]),
                    unclaimed_payments: 0,
                    last_updated_period_block: 0,
                }
            );
            assert_eq!(
                data_charlie,
                Beneficiary {
                    account_id: accounts.charlie,
                    multipliers: vec_to_btreemap(&[(0, 100), (1, 10)]),
                    unclaimed_payments: 0,
                    last_updated_period_block: 0,
                }
            );

            // check accounts are set correctly
            assert_eq!(
                contract.beneficiaries_accounts.get(0).unwrap(),
                &accounts.bob
            );
            assert_eq!(
                contract.beneficiaries_accounts.get(1).unwrap(),
                &accounts.charlie
            );

            // check claims in period are set correctly
            assert_eq!(
                contract.claims_in_period,
                ClaimsInPeriod {
                    period: 0,
                    total_claims: 0,
                }
            );
        }

        #[ink::test]
        fn create_contract_with_invalid_amount_of_multipliers() {
            let accounts = default_accounts();
            let beneficiary_bob = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 100), (1, 3)],
            };
            let beneficiary_charlie = InitialBeneficiary {
                account_id: accounts.charlie,
                multipliers: vec![(0, 100)],
            };
            let res = OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![beneficiary_bob, beneficiary_charlie],
            );

            assert!(matches!(res, Err(Error::InvalidMultipliersLength)));

            let beneficiary_bob = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 100)],
            };
            let beneficiary_charlie = InitialBeneficiary {
                account_id: accounts.charlie,
                multipliers: vec![(0, 100)],
            };
            let res = OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![beneficiary_bob, beneficiary_charlie],
            );

            assert!(matches!(res, Err(Error::InvalidMultipliersLength)));

            let beneficiary_bob = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![],
            };
            let beneficiary_charlie = InitialBeneficiary {
                account_id: accounts.charlie,
                multipliers: vec![],
            };
            let res = OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![beneficiary_bob, beneficiary_charlie],
            );

            assert!(matches!(res, Err(Error::InvalidMultipliersLength)));

            let beneficiary_bob = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 10), (1, 3), (2, 3)],
            };
            let beneficiary_charlie = InitialBeneficiary {
                account_id: accounts.charlie,
                multipliers: vec![(0, 10), (1, 3)],
            };
            let res = OpenPayroll::new(
                2,
                1000,
                vec![
                    "Seniority".to_string(),
                    "Performance".to_string(),
                    "Years_at_company".to_string(),
                ],
                vec![beneficiary_bob, beneficiary_charlie],
            );

            assert!(matches!(res, Err(Error::InvalidMultipliersLength)));
        }

        #[ink::test]
        fn create_contract_with_duplicated_beneficiaries() {
            let accounts = default_accounts();
            let beneficiary_1 = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 100), (1, 3)],
            };
            let beneficiary_2 = InitialBeneficiary {
                account_id: accounts.bob,
                multipliers: vec![(0, 100), (1, 3)],
            };
            let res = OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                vec![beneficiary_1, beneficiary_2],
            );

            assert!(matches!(res, Err(Error::DuplicatedBeneficiaries)));
        }

        /// Add a new beneficiary and check that it is added
        #[ink::test]
        fn add_beneficiary() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 200), (1, 100)])
                .unwrap();
            assert_eq!(
                contract
                    .beneficiaries
                    .get(accounts.bob)
                    .unwrap()
                    .multipliers,
                vec_to_btreemap(&[(0, 200), (1, 100)])
            );
            contract
                .update_beneficiary(accounts.bob, vec![(0, 200), (1, 50)])
                .unwrap();
            assert_eq!(
                contract
                    .beneficiaries
                    .get(accounts.bob)
                    .unwrap()
                    .multipliers,
                vec_to_btreemap(&[(0, 200), (1, 50)])
            );

            // check if account was added to the vector
            assert_eq!(
                contract.beneficiaries_accounts.get(0).unwrap(),
                &accounts.bob
            );
        }

        /// Add a new beneficiary and fails because the sender is not the owner
        #[ink::test]
        fn add_beneficiary_without_access() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            set_sender(accounts.bob);
            assert!(matches!(
                contract.add_beneficiary(accounts.bob, vec![(0, 100), (1, 100)]),
                Err(Error::NotOwner)
            ));
            // check if account was NOT added to the vector
            assert_eq!(contract.beneficiaries_accounts.len(), 0);
        }

        /// Add a new beneficiary and fails because the multiplies is 0
        #[ink::test]
        fn add_beneficiary_with_no_multipliers() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            assert!(matches!(
                contract.add_beneficiary(accounts.bob, vec![]),
                Ok(_)
            ));
        }

        /// Remove a beneficiary and check that it is removed
        #[ink::test]
        fn remove_beneficiary() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            assert_eq!(contract.beneficiaries_accounts.len(), 1);
            assert_eq!(
                contract.beneficiaries_accounts.get(0).unwrap(),
                &accounts.bob
            );
            assert_eq!(
                contract
                    .beneficiaries
                    .get(accounts.bob)
                    .unwrap()
                    .multipliers,
                vec_to_btreemap(&[(0, 100), (1, 20)])
            );
            contract.remove_beneficiary(accounts.bob).unwrap();
            assert!(!contract.beneficiaries.contains(accounts.bob));
            // check if account was removed from the vector
            assert_eq!(contract.beneficiaries_accounts.len(), 0);
        }

        /// Remove a beneficiary and fails because the sender is not the owner
        #[ink::test]
        fn remove_beneficiary_without_access() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            set_sender(accounts.bob);
            assert!(matches!(
                contract.remove_beneficiary(accounts.bob),
                Err(Error::NotOwner)
            ));
            assert_eq!(contract.beneficiaries_accounts.len(), 1);
            assert_eq!(
                contract.beneficiaries_accounts.get(0).unwrap(),
                &accounts.bob
            );
        }

        /// Remove a beneficiary and fails because the beneficiary does not exist
        #[ink::test]
        fn remove_beneficiary_not_found() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            assert!(matches!(
                contract.remove_beneficiary(accounts.bob),
                Err(Error::AccountNotFound)
            ));
        }

        /// Update the base payment and check that it is updated
        #[ink::test]
        fn update_base_payment_in_initial_block() {
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);
            contract.update_base_payment(200_000_000u128).unwrap();
            assert_eq!(contract.base_payment, 200_000_000u128);
        }

        /// Update the base payment and check that it is updated
        #[ink::test]
        fn update_base_payment() {
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);

            advance_n_blocks(1);

            contract.update_base_payment(200_000_000u128).unwrap();
            assert_eq!(contract.base_payment, 200_000_000u128);
        }

        #[ink::test]
        fn update_base_payment_error() {
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);

            advance_n_blocks(3);

            assert!(matches!(
                contract.update_base_payment(200_000_000u128),
                Err(Error::NotAllClaimedInPeriod)
            ));
        }

        /// Update the base payment but fails because the sender is not the owner
        #[ink::test]
        fn update_base_payment_without_access() {
            let (accounts, mut contract) = create_accounts_and_contract(100_000_000u128);
            set_sender(accounts.bob);
            assert!(matches!(
                contract.update_base_payment(200_000_000u128),
                Err(Error::NotOwner)
            ));
        }

        /// Update the base payment but fails because the base payment is 0
        #[ink::test]
        fn update_base_payment_invalid_base_payment() {
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);
            assert!(matches!(
                contract.update_base_payment(0u128),
                Err(Error::InvalidParams)
            ));
        }

        /// Update the periodicity and check that it is updated
        #[ink::test]
        fn update_periodicity() {
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);
            contract.update_periodicity(100u32).unwrap();
            assert_eq!(contract.periodicity, 100u32);
        }

        /// Update the periodicity but fails because the sender is not the owner
        #[ink::test]
        fn update_periodicity_without_access() {
            let (accounts, mut contract) = create_accounts_and_contract(100_000_000u128);
            set_sender(accounts.bob);
            assert!(matches!(
                contract.update_periodicity(100u32),
                Err(Error::NotOwner)
            ));
        }

        /// Update the periodicity but fails because the periodicity is 0
        #[ink::test]
        fn update_periodicity_invalid_periodicity() {
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);

            assert!(matches!(
                contract.update_periodicity(0u32),
                Err(Error::InvalidParams)
            ));
        }

        /// Test pausing and unpausing the contract
        #[ink::test]
        fn pause_and_resume() {
            let starting_block = get_current_block();
            let (_, mut contract) = create_accounts_and_contract(100_000_000u128);

            contract.pause().unwrap();
            assert!(contract.is_paused());
            advance_n_blocks(1);
            contract.resume().unwrap();
            assert!(!contract.is_paused());
            // check for the starting block to be the same
            assert_eq!(contract.initial_block, starting_block);
        }

        /// Test pausing and resuming without access
        #[ink::test]
        fn pause_and_resume_without_access() {
            let (accounts, mut contract) = create_accounts_and_contract(100_000_000u128);
            set_sender(accounts.bob);
            assert!(matches!(contract.pause(), Err(Error::NotOwner)));
            assert!(matches!(contract.resume(), Err(Error::NotOwner)));
        }

        /// Test claiming a payment
        #[ink::test]
        fn claim_payment() {
            let (accounts, mut contract) = create_accounts_and_contract(100_000_000u128);
            contract
                .update_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            let contract_balance_before_payment = get_balance(contract.owner);
            let bob_balance_before_payment = get_balance(accounts.bob);
            set_sender(accounts.bob);

            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            contract
                .claim_payment(accounts.bob, amount_to_claim)
                .unwrap();
            assert!(get_balance(contract.owner) < contract_balance_before_payment);
            assert!(get_balance(accounts.bob) > bob_balance_before_payment);
        }

        /// Test claiming a payment
        #[ink::test]
        fn claim_parcial_payment() {
            let total_amount = 100_000_000u128;
            let total_not_claimed = 10;
            let (accounts, mut contract) = create_accounts_and_contract(total_amount);
            contract
                .update_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();

            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            let bob_balance_before_payment = get_balance(accounts.bob);
            set_sender(accounts.bob);

            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            contract
                .claim_payment(accounts.bob, amount_to_claim - total_not_claimed)
                .unwrap();
            assert!(
                get_balance(contract.owner) == total_amount - amount_to_claim + total_not_claimed
            );
            assert!(
                get_balance(accounts.bob)
                    == bob_balance_before_payment + amount_to_claim - total_not_claimed
            );
            assert!(
                contract
                    .beneficiaries
                    .get(accounts.bob)
                    .unwrap()
                    .unclaimed_payments
                    == total_not_claimed
            );
        }

        /// Test claiming a payment
        #[ink::test]
        fn claim_more_payment() {
            let total_amount = 100_000_000u128;
            let (accounts, mut contract) = create_accounts_and_contract(total_amount);
            contract
                .update_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();

            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            let bob_balance_before_payment = get_balance(accounts.bob);
            set_sender(accounts.bob);

            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            let res = contract.claim_payment(accounts.bob, amount_to_claim + 1);

            assert!(matches!(
                res,
                Err(Error::ClaimedAmountIsBiggerThanAvailable)
            ));
            assert!(get_balance(contract.owner) == total_amount);
            assert!(get_balance(accounts.bob) == bob_balance_before_payment);
        }

        /// Error when trying to update periodicity with some payments not claimed
        #[ink::test]
        fn update_periodicity_without_all_payments_updated() {
            let (accounts, mut contract) = create_accounts_and_contract(100_000_000u128);
            contract
                .update_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();

            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            let res = contract.update_periodicity(10u32);
            assert!(matches!(res, Err(Error::NotAllClaimedInPeriod)));
        }

        ///  update periodicity with all payments claimed with the param amount in 0 in the claim_payment
        #[ink::test]
        fn update_periodicity_with_all_payments_updated() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            // When you claim a payment with 0 amount, it will calculate the amount to claim an set it to unclaim payments.
            contract.claim_payment(accounts.bob, 0).unwrap();

            let res = contract.update_periodicity(10u32);

            assert!(matches!(res, Ok(())));
        }

        /// update periodicity with all payments claimed
        #[ink::test]
        fn update_periodicity_with_all_payments_claimed() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            set_sender(accounts.bob);

            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            contract
                .claim_payment(accounts.bob, amount_to_claim)
                .unwrap();

            set_sender(accounts.alice);
            let res = contract.update_periodicity(10u32);

            assert_eq!(res, Ok(()));
        }

        /// test if error when trying to update base payment with some payments not claimed
        #[ink::test]
        fn update_base_payment_without_all_payments_updated() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            let res = contract.update_base_payment(900);

            assert!(matches!(res, Err(Error::NotAllClaimedInPeriod)));
        }

        /// test if you can update a base payment with all payments claimed
        #[ink::test]
        fn update_base_payment_with_all_payments_claimed() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let mut contract = create_contract_with_no_beneficiaries(100_000_000u128);
            contract
                .add_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();
            // advance 3 blocks so a payment will be claimable
            advance_n_blocks(3);

            set_sender(accounts.bob);
            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            contract
                .claim_payment(accounts.bob, amount_to_claim)
                .unwrap();

            set_sender(accounts.alice);
            let res = contract.update_base_payment(900);

            assert_eq!(res, Ok(()));
        }

        // test if beneficiaries are ok in the contract
        #[ink::test]
        fn create_contract_with_beneficiaries_ok() {
            let (accounts, contract) = create_accounts_and_contract(100_000_000u128);

            assert_eq!(contract.beneficiaries_accounts.len(), 2);
            assert!(contract.beneficiaries.contains(accounts.bob));
            assert!(contract.beneficiaries.contains(accounts.charlie));
        }

        // check for beneficiaries after updating it
        #[ink::test]
        fn update_benefiaries_created_in_create_contract() {
            let total_balance = 100_000_000u128;
            let (accounts, mut contract) = create_accounts_and_contract(total_balance);

            contract
                .update_beneficiary(accounts.bob, vec![(0, 100), (1, 20)])
                .unwrap();

            //check if multipliers are ok
            assert_eq!(
                contract
                    .beneficiaries
                    .get(accounts.bob)
                    .unwrap()
                    .multipliers,
                vec_to_btreemap(&[(0, 100), (1, 20)])
            );
            assert_eq!(
                contract
                    .beneficiaries
                    .get(accounts.charlie)
                    .unwrap()
                    .multipliers,
                vec_to_btreemap(&[(0, 100), (1, 3)])
            );
        }

        // Delete a multiplier
        #[ink::test]
        fn check_deactivate_multiplier() {
            let total_balance = 100_000_000u128;
            let (_, mut contract) = create_accounts_and_contract(total_balance);

            advance_n_blocks(6);

            let res = contract.deactivate_multiplier(1);

            advance_n_blocks(5);

            assert_eq!(res, Ok(()));

            let multiplier_0 = contract.base_multipliers.get(0).unwrap();
            let multiplier_1 = contract.base_multipliers.get(1).unwrap();
            assert_eq!(multiplier_1.valid_until_block.unwrap(), 8);
            assert_eq!(multiplier_0.valid_until_block, None);
        }

        // Check current block period
        #[ink::test]
        fn check_current_start_period_block() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let contract = create_contract_with_no_beneficiaries_periodicity(100_000_000u128, 3);

            advance_n_blocks(6);
            let current_block_period = contract.get_current_period_initial_block();
            assert_eq!(current_block_period, 6);

            advance_n_blocks(1);
            let current_block_period = contract.get_current_period_initial_block();
            assert_eq!(current_block_period, 6);

            advance_n_blocks(1);
            let current_block_period = contract.get_current_period_initial_block();
            assert_eq!(current_block_period, 6);

            advance_n_blocks(1);
            let current_block_period = contract.get_current_period_initial_block();
            assert_eq!(current_block_period, 9);
        }

        // Check the fn next_block_period
        #[ink::test]
        fn check_next_block_period() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let contract = create_contract_with_no_beneficiaries_periodicity(100_000_000u128, 3);

            let next_block_period = contract.get_next_block_period();
            assert_eq!(next_block_period, 3);

            advance_n_blocks(4);
            let next_block_period = contract.get_next_block_period();
            assert_eq!(next_block_period, 6);
        }

        /// check for the fn get_list_payees
        #[ink::test]
        fn check_list_beneficiaries() {
            let total_balance = 100_000_000u128;
            let (accounts, contract) = create_accounts_and_contract(total_balance);

            let list_beneficiaries = contract.get_list_beneficiaries();
            assert_eq!(list_beneficiaries, vec![accounts.bob, accounts.charlie]);

            let contract = create_contract_with_no_beneficiaries_periodicity(total_balance, 3);
            let list_beneficiaries = contract.get_list_beneficiaries();
            assert_eq!(list_beneficiaries, vec![]);
        }

        // check for get_amount_to_claim and get_contract_balance
        #[ink::test]
        fn check_contract_balance() {
            let total_balance = 100_000_001u128;
            let (accounts, mut contract) = create_accounts_and_contract(total_balance);

            assert_eq!(contract.get_contract_balance(), total_balance);

            advance_n_blocks(3);

            // bob claims
            set_sender(accounts.bob);
            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            contract
                .claim_payment(accounts.bob, amount_to_claim)
                .unwrap();

            // check final amount
            assert_eq!(contract.get_contract_balance(), 99998971u128);
        }

        // check for get_unclaimed_beneficiaries and get_count_of_unclaim_beneficiaries in diffent blocks
        #[ink::test]
        fn check_unclaimed_beneficiaries() {
            let total_balance = 100_000_001u128;
            let (accounts, mut contract) = create_accounts_and_contract(total_balance);

            let unclaimed_beneficiaries = contract.get_unclaimed_beneficiaries();
            let count_of_unclaim_beneficiaries = contract.get_count_of_unclaim_beneficiaries();

            assert_eq!(unclaimed_beneficiaries, vec![]);
            assert_eq!(count_of_unclaim_beneficiaries, 0);

            advance_n_blocks(1);
            let unclaimed_beneficiaries = contract.get_unclaimed_beneficiaries();
            let count_of_unclaim_beneficiaries = contract.get_count_of_unclaim_beneficiaries();

            // should be the same because we are in the same period
            assert_eq!(unclaimed_beneficiaries, vec![]);
            assert_eq!(count_of_unclaim_beneficiaries, 0);

            // in total 2 blocks to have beneficiaries that not claimed
            advance_n_blocks(1);
            let unclaimed_beneficiaries = contract.get_unclaimed_beneficiaries();
            let count_of_unclaim_beneficiaries = contract.get_count_of_unclaim_beneficiaries();
            assert_eq!(
                unclaimed_beneficiaries,
                vec![accounts.bob, accounts.charlie]
            );
            assert_eq!(count_of_unclaim_beneficiaries, 2);

            // claim bob and check the amount of unclaim beneficiaries
            set_sender(accounts.bob);
            let amount_to_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            contract
                .claim_payment(accounts.bob, amount_to_claim)
                .unwrap();

            let unclaimed_beneficiaries = contract.get_unclaimed_beneficiaries();
            let count_of_unclaim_beneficiaries = contract.get_count_of_unclaim_beneficiaries();
            assert_eq!(unclaimed_beneficiaries, vec![accounts.charlie]);
            assert_eq!(count_of_unclaim_beneficiaries, 1);
        }

        /// Test get_balance_with_debts and get_total_debts readonly function when debts is 0
        #[ink::test]
        fn check_total_balance_and_debts_on_init() {
            let total_balance = 100_000_001u128;
            let (_, contract) = create_accounts_and_contract(100_000_001u128);
            let total_debts = contract.get_total_debts();
            assert_eq!(total_debts, 0);
            assert_eq!(contract.get_balance_with_debts(), total_balance);
        }

        /// Test 2 readonly function related with total debts and balance
        /// fn: get_total_debts and get_balance_with_debts
        ///
        /// workaround: create a contract, advance 2 blocks for next period & check debts with individual debts
        #[ink::test]
        fn check_total_debts_with_individual_debts() {
            let total_balance = 100_000_001u128;
            let (accounts, contract) = create_accounts_and_contract(total_balance);

            // goto next period so can beneficiaries can claim
            advance_n_blocks(2);
            let bob_amount_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            let charlie_amount_claim = contract.get_amount_to_claim(accounts.charlie).unwrap();
            let total_debts = contract.get_total_debts();

            // check the specifi value and the sum of both individual debts
            assert_eq!(total_debts, 2060);
            assert_eq!(total_debts, bob_amount_claim + charlie_amount_claim);

            // check if the balance with debts is correct (total_balance - total_debts)
            assert_eq!(
                contract.get_balance_with_debts(),
                total_balance - (bob_amount_claim + charlie_amount_claim)
            );
        }

        /// Test get_total_debts readonly function after all claims
        ///
        /// workaround: create a contract, advance 2 blocks for next period, claim all and check debts
        #[ink::test]
        fn check_is_total_debts_is_zero_after_all_claims() {
            let total_balance = 100_000_001u128;
            let (accounts, mut contract) = create_accounts_and_contract(total_balance);

            // goto next period so can beneficiaries can claim
            advance_n_blocks(2);
            let bob_amount_claim = contract.get_amount_to_claim(accounts.bob).unwrap();
            let charlie_amount_claim = contract.get_amount_to_claim(accounts.charlie).unwrap();

            // claim bob and charlie, then check if debt is 0
            set_sender(accounts.bob);
            contract
                .claim_payment(accounts.bob, bob_amount_claim)
                .unwrap();
            set_sender(accounts.charlie);
            contract
                .claim_payment(accounts.charlie, charlie_amount_claim)
                .unwrap();

            assert_eq!(contract.get_total_debts(), 0);
        }

        #[ink::test]
        fn check_total_debt_with_unclaimed_for_next_period_on_init() {
            let (_, contract) = create_accounts_and_contract(100_000_001u128);

            let total_debts = contract.get_total_debt_with_unclaimed_for_next_period();
            assert_eq!(total_debts, 2060);
        }

        /// Test 2 readonly function related with total debts for next period
        /// fn: get_total_debt_with_unclaimed_for_next_period and get_total_debt_for_next_period
        #[ink::test]
        fn check_total_debt_with_unclaimed_for_next_period_advancing_a_period() {
            let (_, contract) = create_accounts_and_contract(100_000_001u128);

            advance_n_blocks(2);

            let total_debts_with_unclaimed =
                contract.get_total_debt_with_unclaimed_for_next_period();
            let total_debts_next_period = contract.get_total_debt_for_next_period();

            assert_eq!(total_debts_with_unclaimed, 4120);
            assert_eq!(total_debts_next_period, 2060);
        }

        // Check if dispatch error when adding more thatn beneficiaries allowed
        #[ink::test]
        fn check_max_beneficiaries() {
            let mut contract = create_contract_with_no_beneficiaries(100_000_001u128);
            let max_beneficiaries = 100u8;

            for u8_number in 0..max_beneficiaries {
                let arr_of_32: [u8; 32] = [u8_number; 32];
                contract
                    .add_beneficiary(AccountId::from(arr_of_32), vec![])
                    .unwrap();
            }

            let contract_beneficiaries = contract.beneficiaries_accounts.len() as u8;

            assert_eq!(contract_beneficiaries, max_beneficiaries);

            // try to add one more beneficiary
            let res = contract.add_beneficiary(AccountId::from([255u8; 32]), vec![]);

            assert!(matches!(res, Err(Error::MaxBeneficiariesExceeded)));
        }

        // Test failing when try to claim not transfered ownership
        #[ink::test]
        fn failing_not_transfered_ownership() {
            let (_, mut contract) = create_accounts_and_contract(100_000_001u128);

            // try to accept ownership
            let accept_ownsership_result = contract.accept_ownership();
            assert!(matches!(accept_ownsership_result, Err(Error::NotOwner)));
        }

        // Test change ownership
        #[ink::test]
        fn check_transfer_ownership() {
            let (accounts, mut contract) = create_accounts_and_contract(100_000_001u128);

            // check no transfered ownership was called yet
            assert_eq!(contract.proposed_owner, None);
            // check if owner is alice
            assert_eq!(contract.owner, accounts.alice);

            // change owner to bob
            set_sender(accounts.alice);
            let transfer_ownership_result = contract.propose_transfer_ownership(accounts.bob);
            assert!(transfer_ownership_result.is_ok());

            // check if owner is bob
            assert_eq!(contract.proposed_owner, Some(accounts.bob));

            // accept ownership
            set_sender(accounts.bob);
            let accept_ownsership_result = contract.accept_ownership();
            assert!(accept_ownsership_result.is_ok());

            assert_eq!(contract.owner, accounts.bob);
            assert_eq!(contract.proposed_owner, None);
        }

        // Check if dispatch error when adding more beneficiaries allowed from creation
        #[ink::test]
        fn check_max_beneficiaries_from_creation() {
            set_balance(contract_id(), 100u128);

            let max_beneficiaries = 100u8;
            let mut beneficiaries = Vec::new();
            for u8_number in 0..max_beneficiaries + 1 {
                let arr_of_32: [u8; 32] = [u8_number; 32];
                let beneficiary = InitialBeneficiary {
                    account_id: AccountId::from(arr_of_32),
                    multipliers: vec![],
                };
                beneficiaries.push(beneficiary);
            }

            let res = OpenPayroll::new(
                2,
                1000,
                vec!["Seniority".to_string(), "Performance".to_string()],
                beneficiaries,
            );

            assert!(matches!(res, Err(Error::MaxBeneficiariesExceeded)));
        }

        // Check if dispatch error when adding more thatn multipliers allowed from creation
        #[ink::test]
        fn check_max_multipliers_from_creation() {
            set_balance(contract_id(), 100u128);

            let max_multipliers = 10u8;
            let mut multipliers = Vec::new();
            for num in 0..max_multipliers + 1 {
                multipliers.push(num.to_string());
            }

            let beneficiary = InitialBeneficiary {
                account_id: AccountId::from([1; 32]),
                multipliers: vec![],
            };

            let res = OpenPayroll::new(2, 1000, multipliers, vec![beneficiary]);

            assert!(matches!(res, Err(Error::MaxMultipliersExceeded)));
        }

        // Check if dispatch error when adding more thatn multipliers allowed from creation
        #[ink::test]
        fn check_max_multipliers() {
            let mut contract = create_contract_with_no_beneficiaries(100_000_001u128);
            let max_multipliers = 10u8;

            for u8_number in 2..max_multipliers {
                contract.add_base_multiplier(u8_number.to_string()).unwrap();
            }

            assert_eq!(contract.multipliers_list.len(), max_multipliers.into());

            // try to add one more beneficiary
            let res = contract.add_base_multiplier("max+1".to_string());

            assert!(matches!(res, Err(Error::MaxMultipliersExceeded)));
        }
    }
}
