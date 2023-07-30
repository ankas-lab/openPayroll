# openPayroll

## Overview

The goal of Open Payroll is to address the needs of organizations seeking to establish transparent payment processes within a specific timeframe. Its primary objective is to develop a contract that empowers individuals to configure and generate their own payroll system.

The ownership of the payroll contract resides entirely with its creator, who can be a DAO address, a multisig, or an individual. This contract operates by managing a treasury, from which all payments are deducted. The system comprises a base amount and a series of multipliers associated with the payees' addresses.

For example, let's consider a payroll contract designed to pay developers' salaries. It includes a base amount and a single multiplier based on the employee's seniority level. Alice is a junior employee with a multiplier of 1, while Bob is a senior employee with a multiplier of 2. With a base amount of 1000, the payroll contract allows Alice to claim 1000 units, and Bob to claim 2000 units during each payment period.

The payroll smart contract provides transparent visibility by displaying the addresses of all participants, along with the multipliers employed. This openness ensures complete transparency for everyone involved. Initially, the project focuses on an open payroll system with a specific approach, but it lays the foundation for further development, enabling scenarios like recurring payments, subscriptions, and more.

## Interactions and Functionality of the Payroll Contract

O - Creation Parameters for creating a new payroll contract:

- Periodicity
- Base Payment
- Initial Base Multipliers
- Initial Beneficiaries

O - Contract Interactions from the Owner's Perspective:

- Modify the existing parameters in the contract.
- Add, update or remove beneficiaries.
- Add and withdraw funds from the treasury.
- Pause the contract, temporarily suspending the claim process,  halting any further payment disbursements.
- Resume the contract, restoring its functionality.
- Change the owner of the contract.

O - Contract Interactions from the Payees' Perspective:

- Claim the payments that are already available for them.

## Design decisions:

Here are some key technical decisions we made during the development:

- Initial Block Set to Current Block: In this version, the initial block is set to the current block. This ensures that the blockchain starts recording data from the present time.

- Owner Assignment: The owner of the contract is set to the account that called the constructor. This establishes the initial ownership of the contract.

- Base Multipliers Flexibility: The base multipliers can be left empty, indicating that no multiplier will be applied. In such cases, the beneficiary will receive just the base payment during each payment period.

- Multiplier Calculation: Multipliers are used to calculate the corresponding payment. For example, if the base payment is 1000 and there are multipliers such as seniority (2) and experience in the project (0.5), the calculation would be as follows: base payment (1000) * (seniority (2) + experience in the project (0.5)) = total for the period (2500).

- Ensuring Payment Completeness: The function ensure_all_payments_uptodate serves the purpose of checking if there are any remaining amounts to be claimed before changing core parameters. This check ensures that past periods' payment amounts are not altered.

- Pausable Contracts: The created contracts are equipped with the ability to be paused using the pause/resume function. This functionality can only be invoked by the contract's owner, providing control over the contract's operation.


## 🚀 Compile and test the contract

- Clone the repository with the following command and enter the project folder:

    ```bash
    git clone https://github.com/polkadrys/openPayroll.git && cd openPayroll
    ```

### Docker

- ⚠️ Requirements:
  - docker >= 20

1. Make sure your daemon `docker` is running in your system.

2. Build the docker image:

    ```bash
    docker build -t open-payroll:0.1.0 .
    ```

    #### Compile the contract

    ```bash
    docker run -v "$(pwd)/src:/src" open-payroll:0.1.0 cargo contract build --release
    ```
    
    > 🔍 You will find the contract artifacts in the `src/target/ink` folder. 

    #### Run the tests
 
    ```bash
    docker run -v "$(pwd)/src:/src" open-payroll:0.1.0 cargo test
    ```