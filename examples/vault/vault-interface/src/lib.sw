library;

abi Vault {
    #[storage(read, write)]
    fn create();

    #[payable]
    #[storage(read, write)]
    fn deposit();

    #[storage(read, write)]
    fn withdraw(amount: u64);
}
