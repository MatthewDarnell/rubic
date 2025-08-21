use std::ptr::copy_nonoverlapping;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::Lazy;
//use secrets;
//use secrets::traits::Zeroable;
use logger::{error, info};

//Convenient Secret stored in secure memory for temporary unlock of the db
/*pub static PLAINTEXT_DECRYPT_PASSWORD: Lazy<Arc<Mutex<secrets::SecretVec<u8>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(secrets::SecretVec::zero(64)))
});*/
pub static PLAINTEXT_DECRYPT_PASSWORD: Lazy<Arc<Mutex<Vec<u8>>>> = Lazy::new(|| {
    Arc::new(Mutex::new([0u8; 64].to_vec()))
});

fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

pub fn is_wallet_unlocked() -> Result<bool, ()> {
    let plaintext = PLAINTEXT_DECRYPT_PASSWORD.clone();
    let ret_val = match plaintext.lock() {
        Ok(pass_guard) => {
            let zero_pass: [u8; 64] = [0u8; 64];
            if do_vecs_match(&pass_guard, &zero_pass.to_vec()) {
                Ok(true)
            } else {
                Ok(false)
            }
            /*
            let current_pass = pass_guard.borrow();
            if *current_pass != zero_pass {
                Ok(true)
            } else {
                Ok(false)
            }

             */
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
            //let s = secret.borrow();
            let s = secret;
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
                    Ok(pass_guard) => {
                        let zero_pass: [u8; 64] = [0u8; 64];
                        //let mut current_pass = pass_guard.borrow_mut();
                        let mut current_pass = pass_guard;
                        if *current_pass != zero_pass {
                            error!("Failed To Unlock Wallet. Wallet Already Unlocked.");
                            return Err("Failed To Unlock Wallet. Wallet Already Unlocked".to_string());
                        }
                        unsafe {
                            //current_pass.zero();
                            copy_nonoverlapping(zero_pass.as_ptr(), current_pass.as_mut_ptr(), 64);
                            //copy_nonoverlapping(password.as_ptr(), current_pass.as_mut().as_mut_ptr(), password.len());
                            copy_nonoverlapping(password.as_ptr(), current_pass.as_mut_ptr(), password.len());
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
                        Ok(pass) => {
                            //let mut _pass = pass.borrow_mut();
                            let mut _pass = pass;
                            let zero_pass: [u8; 64] = [0u8; 64];
                            //_pass.zero();
                            unsafe {
                                copy_nonoverlapping(zero_pass.as_ptr(), _pass.as_mut_ptr(), 64);
                            }
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