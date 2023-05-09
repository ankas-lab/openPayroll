#[derive(scale::Encode, scale::Decode, Eq, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    // The caller is not the owner of the contract
    NotOwner,
    // The contract is paused
    ContractIsPaused,
    // The params are invalid
    InvalidParams,
    // The account is not found
    AccountNotFound,
    // The contract does not have enough balance to pay
    NotEnoughBalanceInTreasury,
    // The transfer failed
    TransferFailed,
    // The beneficiary has no unclaimed payments
    NoUnclaimedPayments,
    // Some of the beneficiaries have unclaimed payments
    PaymentsNotUpToDate,
    // Not all the payments are claimed in the last period
    NotAllClaimedInPeriod,
    // The amount to claim is bigger than the available amount
    ClaimedAmountIsBiggerThanAvailable,
    // The amount of multipliers per Beneficiary is not equal to the amount of periods
    InvalidMultipliersLength,
    // The multiplier id does not exist
    MultiplierNotFound,
    // The multiplier is already deactivated
    MultiplierAlreadyDeactivated,
    // The multiplier is not deactivated
    MultiplierNotDeactivated,
    // There are duplicated multipliers
    DuplicatedMultipliers,
    // There are duplicated beneficiaries
    DuplicatedBeneficiaries,
    // The multiplier is not expired yet
    MultiplierNotExpired,
}
