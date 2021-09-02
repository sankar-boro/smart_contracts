#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod reserve_bank {
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
        },
        Lazy,
    };

    /// A simple ERC-20 contract.
    #[ink(storage)]
    pub struct ReserveBank {
        /// Total token supply.
        reserved_balance: Lazy<Balance>,
        /// Mapping from owner to number of owned token.
        balances: StorageHashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        borrowers: StorageHashMap<AccountId, (AccountId, Balance)>,
        _b: StorageHashMap<AccountId, Vec<(AccountId, Balance)>>
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Borrow {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl ReserveBank {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(reserved_balance: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, reserved_balance);
            let instance = Self {
                reserved_balance: Lazy::new(reserved_balance),
                balances,
                borrowers: StorageHashMap::new(),
                _b: StorageHashMap::new(),
            };
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: reserved_balance,
            });
            instance
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn reserved_balance(&self) -> Balance {
            *self.reserved_balance
        }

        #[ink(message)]
        pub fn my_balance(&self) -> Balance {
            let _self = self.env().caller();
            self.balances.get(&_self).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn my_borrowed_balance(&self) -> (AccountId, Balance) {
            let _self = self.env().caller();
            self.borrowers.get(&_self).copied().unwrap()
        }

        #[ink(message)]
        pub fn borrowed_balance_of(&self, owner: AccountId) -> (AccountId, Balance) {
            self.borrowers.get(&owner).copied().unwrap()
        }

        #[ink(message)]
        pub fn borrow(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            self.borrow_from_to(from, to, value)
        }
        
        #[ink(message)]
        pub fn transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            self.transfer_from_to(from, to, value)
        }

        #[ink(message)]
        pub fn send_documents(&mut self, to: AccountId, value: Balance, file: Vec<u8>) -> Result<()> {
            let _self = self.env().caller();
            let a = self.borrowers.get(&_self).copied().unwrap();
            let b: AccountId = a.0;
            let m = self.balances.get(&b).copied().unwrap();
            self.balances.insert(b, m + value);
            self.borrowers.insert(_self, (b, 0));
            Ok(())
        }

        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }

        fn borrow_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }
            self.balances.insert(from, from_balance - value);
            
            self.borrowers.insert(to, (from, value));
            self.env().emit_event(Borrow {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }

    }
}
