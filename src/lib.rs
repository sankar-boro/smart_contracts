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

    #[ink(storage)]
    pub struct ReserveBank {
        reserved_balance: Lazy<Balance>,
        balances: StorageHashMap<AccountId, Balance>,
        // _a: StorageHashMap<AccountId, Vec<(AccountId, Balance)>>,
        _b: StorageHashMap<AccountId, Vec<(AccountId, Balance)>>
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct DocumentTransfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Borrow {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl ReserveBank {
        #[ink(constructor)]
        pub fn new(reserved_balance: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, reserved_balance);
            let instance = Self {
                reserved_balance: Lazy::new(reserved_balance),
                balances,
                // _a: StorageHashMap::new(),
                _b: StorageHashMap::new(),
            };
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: reserved_balance,
            });
            instance
        }

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
        pub fn borrowed_balance_of(&self, user_id: AccountId) -> Balance {
            let a: Option<&Vec<(AccountId, Balance)>> = self._b.get(&user_id);
            match a {
                Some(_a) => {
                    let mut t: Balance = 0;
                    _a.into_iter().for_each(|(_, b)| { 
                        t += b;
                    });
                    t
                }
                None => 0,
            }
        }

        #[ink(message)]
        pub fn borrow(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            self.decrease_balance(from, value);
            self.increase_borrow_balance(from, to, value);
            self.env().emit_event(Borrow {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }

        fn deduce(&mut self, user_id: AccountId, ori: Balance, value: Balance) {
            self.balances.insert(user_id, ori - value);
        }

        fn increase(&mut self, user_id: AccountId, ori: Balance, value: Balance) {
            self.balances.insert(user_id, ori + value);
        }

        fn decrease_balance(&mut self, from: AccountId, value: Balance) {
            let b: Balance = self.balance_of(from);
            self.deduce(from, b, value);
        }

        fn increase_balance(&mut self, user_id: AccountId, value: Balance) {
            let b: Balance = self.balance_of(user_id);
            self.increase(user_id, b, value);
        }


        fn increase_borrow_balance(&mut self, from: AccountId, to: AccountId, value: Balance) {
            let x: Option<&Vec<(AccountId, Balance)>> = self._b.get(&to);
            match x {
                Some(_x) => {
                    let a: Vec<(AccountId, Balance)> = _x.clone();

                    let mut tem: Vec<(AccountId, Balance)> = a.clone();
                    let mut found = false;
                    let b: Vec<(AccountId, Balance)> = a.into_iter().map(|(aa, bb)| {
                        if aa == from {
                            found = true;
                            return (aa, bb + value);
                        }
                        return (aa, value);
                    }).collect();

                    if !found {
                        tem.push((from, value));
                        self._b.insert(to, tem);
                    } else {
                        self._b.insert(to, b);
                    }

                }
                None => {
                    let mut x: Vec<(AccountId, Balance)> = Vec::new();
                    x.push((from, value));
                    self._b.insert(to, x);
                }
            }
        }
        
        #[ink(message)]
        pub fn transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            self.transfer_from_to(from, to, value)
        }

        #[ink(message)]
        pub fn send_documents(&mut self, _self: AccountId, to: AccountId, value: Balance, file: Vec<u8>) -> Result<()> {
            let x: Option<&Vec<(AccountId, Balance)>> = self._b.get(&_self);
            match x {
                Some(borrows) => {
                    let mut dbalance = value;
                    let _b:Vec<(AccountId, Balance)> = borrows.clone();
                    let _x: Vec<(AccountId, Balance)> = _b.into_iter().map(|(y, z)| {
                        if dbalance == z && dbalance != 0 {
                            self.increase_balance(y, dbalance);
                            dbalance = 0;
                            return (y, 0);
                        } else if dbalance > z {
                            self.increase_balance(y, z);
                            dbalance = dbalance - z;
                            return (y, 0);
                        } else if dbalance < z {
                            self.increase_balance(y, dbalance);
                            return (y, z - dbalance);
                        } else {
                            return (y, z);
                        }
                    })
                    .filter(|(_f1, _f2)| { *_f2 != 0 })
                    .collect();
                    self._b.insert(_self, _x);
                }
                None => {
                    self.decrease_balance(to, value);
                    self.increase_balance(_self, value);
                }
            }
            self.env().emit_event(DocumentTransfer {
                from: Some(_self),
                to: Some(to),
                value,
            });
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

    }
}
