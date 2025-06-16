use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref SQLITE_MUTEX: Mutex<i32> = Mutex::new(0i32); //Unlocks when goes out of scope due to fancy RAII
}

pub fn get_db_lock() -> &'static Mutex<i32> { &SQLITE_MUTEX }

pub mod create;
pub mod crud;
pub mod tick;
pub mod computors;
pub mod master_password;
pub mod peer;
pub mod identity;
pub mod transfer;
pub mod response_entity;
pub mod asset;