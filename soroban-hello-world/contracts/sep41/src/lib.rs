#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};
use soroban_token_sdk::{metadata::TokenMetadata, TokenUtils};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Allowance(Address, Address),
    Balance(Address),
    Admin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
#[repr(u32)]
pub enum ContractError {
    InternalError = 1,
    AlreadyInitializedError = 3,
    UnauthorizedError = 4,
    NegativeAmountError = 8,
    BalanceError = 10,
    OverflowError = 12,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount");
    }
}

pub trait TokenTrait {
    fn __constructor(e: Env, admin: Address, decimal: u32, name: String, symbol: String);

    fn allowance(e: Env, from: Address, spender: Address) -> i128;

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);

    fn balance(e: Env, id: Address) -> i128;

    fn transfer(e: Env, from: Address, to: Address, amount: i128);

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn burn(e: Env, from: Address, amount: i128);

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> String;

    fn symbol(e: Env) -> String;

    fn mint(e: Env, to: Address, amount: i128);

    fn set_admin(e: Env, new_admin: Address);

    fn admin(e: Env) -> Address;
}

#[contract]
pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn __constructor(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if e.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        // Set admin
        e.storage().instance().set(&DataKey::Admin, &admin);

        // Set metadata
        let metadata = TokenMetadata {
            decimal,
            name,
            symbol,
        };
        e.storage()
            .instance()
            .set(&symbol_short!("METADATA"), &metadata);
    }

    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        check_nonnegative_amount(amount);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        check_nonnegative_amount(amount);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        check_nonnegative_amount(amount);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    fn decimals(e: Env) -> u32 {
        let metadata = e
            .storage()
            .instance()
            .get::<Symbol, TokenMetadata>(&symbol_short!("METADATA"))
            .unwrap();
        metadata.decimal
    }

    fn name(e: Env) -> String {
        let metadata = e
            .storage()
            .instance()
            .get::<Symbol, TokenMetadata>(&symbol_short!("METADATA"))
            .unwrap();
        metadata.name
    }

    fn symbol(e: Env) -> String {
        let metadata = e
            .storage()
            .instance()
            .get::<Symbol, TokenMetadata>(&symbol_short!("METADATA"))
            .unwrap();
        metadata.symbol
    }

    fn mint(e: Env, to: Address, amount: i128) {
        let admin = e
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        check_nonnegative_amount(amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = e
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .unwrap();
        admin.require_auth();

        e.storage().instance().set(&DataKey::Admin, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }

    fn admin(e: Env) -> Address {
        e.storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .unwrap()
    }
}

// Helper functions
fn write_allowance(e: &Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    let key = DataKey::Allowance(from, spender);
    let allowance = AllowanceValue {
        amount,
        expiration_ledger,
    };
    e.storage().persistent().set(&key, &allowance);
}

fn read_allowance(e: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(from, spender);
    if let Some(allowance) = e
        .storage()
        .persistent()
        .get::<DataKey, AllowanceValue>(&key)
    {
        if allowance.expiration_ledger < e.ledger().sequence() {
            AllowanceValue {
                amount: 0,
                expiration_ledger: allowance.expiration_ledger,
            }
        } else {
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
}

fn spend_allowance(e: &Env, from: Address, spender: Address, amount: i128) {
    let allowance = read_allowance(e, from.clone(), spender.clone());
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    write_allowance(
        e,
        from,
        spender,
        allowance.amount - amount,
        allowance.expiration_ledger,
    );
}

fn read_balance(e: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    e.storage().persistent().get(&key).unwrap_or(0)
}

fn receive_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    let key = DataKey::Balance(addr);
    e.storage().persistent().set(&key, &(balance + amount));
}

fn spend_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    if balance < amount {
        panic!("insufficient balance");
    }
    let key = DataKey::Balance(addr);
    e.storage().persistent().set(&key, &(balance - amount));
}
