use std::ptr::copy_nonoverlapping;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::Lazy;
use secrets;
use secrets::traits::Zeroable;
use logger::{error, info};

//Convenient Secret stored in secure memory for temporary unlock of the db
pub static PLAINTEXT_DECRYPT_PASSWORD: Lazy<Arc<Mutex<secrets::SecretVec<u8>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(secrets::SecretVec::zero(64)))
});

pub fn is_wallet_unlocked() -> Result<bool, ()> {
    let plaintext = PLAINTEXT_DECRYPT_PASSWORD.clone();
    let ret_val = match plaintext.lock() {
        Ok(pass_guard) => {
            let zero_pass: [u8; 64] = [0u8; 64];
            let current_pass = pass_guard.borrow();
            if *current_pass != zero_pass {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        Err(_) => Err(())
    };
    ret_val
}

pub fn get_plaintext_password() -> Result<String, ()> {
    if !is_wallet_unlocked().unwrap() {
        return Err(());
    }
    match PLAINTEXT_DECRYPT_PASSWORD.clone().lock() {
        Ok(secret) => unsafe {
            let s = secret.borrow();
            let pass = std::str::from_utf8_unchecked(& *s);
            Ok(pass.to_string())                           
        },
        Err(_) => Err(())
    }
}
pub fn unlock_wallet(master_password: &str, password: &str, timeout_ms: Duration) -> Result<String, String> {
    match crypto::passwords::verify_password(password, master_password) {
        Ok(verified) => {
            if !verified {
                error!("Failed To Unlock Wallet. Invalid Password");
                Err("Failed To Unlock Wallet. Invalid Password".to_string())
            } else {
                let plaintext = PLAINTEXT_DECRYPT_PASSWORD.clone();
                match plaintext.lock() {
                    Ok(mut pass_guard) => {
                        let zero_pass: [u8; 64] = [0u8; 64];
                        let mut current_pass = pass_guard.borrow_mut();
                        if *current_pass != zero_pass {
                            error!("Failed To Unlock Wallet. Wallet Already Unlocked.");
                            return Err("Failed To Unlock Wallet. Wallet Already Unlocked".to_string());
                        }
                        unsafe {
                            current_pass.zero();
                            copy_nonoverlapping(password.as_ptr(), current_pass.as_mut().as_mut_ptr(), password.len());
                        }
                    },
                    Err(e) => {
                        error!("Failed to Unlock Wallet: {}", e);
                        return Err(format!("Failed to Unlock Wallet: {}", e));
                    }
                }
                std::thread::spawn(move || {
                    std::thread::sleep(timeout_ms);
                    let p = PLAINTEXT_DECRYPT_PASSWORD.clone();
                    match p.lock() {
                        Ok(mut pass) => {
                            let mut _pass = pass.borrow_mut();
                            _pass.zero();
                            info!("Wallet Locked.");
                        },
                        Err(e) => {
                            error!("Failed to Unlock Wallet: {}", e);
                        }
                    };
                });
                info!("Wallet Unlocked For {} ms", timeout_ms.as_millis());
                Ok("Wallet Unlocked".to_string())
            }
        },
        Err(_) => {
            error!("Failed To Unlock Wallet. Incorrect Password!");
            Err("Failed To Unlock Wallet. Incorrect Password!".to_string())
        }
    }
}