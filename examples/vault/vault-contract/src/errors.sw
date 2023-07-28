library;

pub enum AssetError {
    IncorrectAsset: (),
    InsufficientBalance: (),
}

pub enum VaultError {
    AlreadyExists: (),
    DoesNotExist: (),
}
