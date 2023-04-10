# openPayroll

## Overview

The objective of Open Payroll is to meet the needs of organizations that wish to make transparent payments during a given period.
The objective is to create a contract that enables anyone to configure and generate their own payroll system.

The payroll contract is owned entirely by its creator. This creator could be a DAO address, a multisig or a single person. The contract manages a treasury from where all the payments are deducted. There is a base amount and a set of multipliers associated to the addresses of the payees.

E.g. We create a payroll contract for paying developers salaries. We will have a base amount and only one multiplier which is the employee's seniority.
Alice is a junior employee and Bob is a senior employee. Alice's multiplier is 1 and Bob's multiplier is 2. The base amount is 1000. The payroll contract will allow Alice to claim 1000 and Bob 2000 every period.

The payroll smart contract transparently displays the addresses of all participants, along with the multipliers being utilized, allowing complete visibility to everyone. The initial rollout of this project is a super opinionated and geared towards an open payroll system, but this notion will allow us to build on various scenarios, such as any kind of recurring payments, subscriptions, etc.

## About the contract

Build an Ink! contract, which purpose is to manage a treasury, that can be spent by the parameters set by the owner at creation point. Those parameters can be changed over the time and more beneficiaries can be added or removed. The funds in the treasury can be withdrawn by the owner of the contract if needed. This could be helpful in the case of migrating to a new version of openPayroll, amending a mistake of sending too much funds, etc. 

Further information about the contract can be found [here](./contracts/README.md)
## About the interactions in the FE

All the posible interactions with the contract, including:

- Creation parameters needed to create a new payroll contract:

  - Base amount
  - Multipliers
  - Period
  - Beneficiaries

- Contract interactions from the owner's perspective:

  - Change the current parameters in the contract.
  - Add or remove beneficiaries.
  - Withdraw funds from the treasury.
  - Pause the contract.
  - Change the owner of the contract.
  - Calculate the amount that will be paid in the next period.

- Contract interactions from the payees' perspective:
  - Calculate the amount that they can claim.
  - Calculate the amount that they can claim in the next period with the current parameters.
  - Claim the payments that are already available.

Further information about the Front-end can be found [here](./website/README.md)