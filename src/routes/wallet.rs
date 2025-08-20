use rocket::get;
use logger::{debug, error};
use store;
use crate::routes::MINPASSWORDLEN;

#[get("/wallet/is_encrypted")]
pub fn is_wallet_encrypted() -> String {
    match store::sqlite::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(pass) => {
            if pass.len() > 0 {
                format!("true")
            } else {
                format!("false")
            }
        },
        Err(err) => format!("{:?}", err)
    }
}

#[get("/wallet/unlocked")]
pub fn is_unlocked() -> String {
    match protocol::wallet_unlock::is_wallet_unlocked() {
        Ok(pass) => format!("{}", pass),
        Err(err) => format!("{:?}", err)
    }
}

#[get("/wallet/unlock/<password>/<timeout_ms>")]
pub fn unlock(password: String, timeout_ms: u64) -> String {
    if password.len() < MINPASSWORDLEN {
        return "Password Too Short!".to_string();
    } else if password.len() > 64 {
        return "Password Too Long!".to_string();
    }
    if timeout_ms > 9999 {
        return "Wallet Unlock Timeout Period Too Long!".to_string();
    }
    let timeout_ms = std::time::Duration::from_millis(timeout_ms);
    match store::sqlite::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(master_password) => {
            protocol::wallet_unlock::unlock_wallet(master_password[1].as_str(), password.as_str(), timeout_ms).unwrap_or_else(|err| err)       
        },
        Err(_) => "Wallet Cannot Be Unlocked. Not Already Encrypted!".to_string()
    }
}

#[get("/wallet/set_master_password/<password>")]
pub fn set_master_password(password: &str) -> String {
    if password.len() < MINPASSWORDLEN {
        return format!("Password Too Short!");
    }
    match store::sqlite::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(_) => format!("Wallet Password Already Set!"),
        Err(_) => {
            match crypto::passwords::hash_password(password) {
                Ok(hashed) => {
                    match store::sqlite::master_password::set_master_password(store::get_db_path().as_str(), hashed.as_str()) {
                        Ok(_) => {
                            return format!("Master Password Set!");
                        },
                        Err(err) => {
                            return format!("{}", err);
                        }
                    }
                },
                Err(err) => {
                    return format!("{}", err);
                }
            }
        }
    }
}

#[get("/wallet/encrypt/<password>")]
pub fn encrypt_wallet(password: &str) -> String {
    match store::sqlite::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(pass) => {
            if pass.len() == 0 {
                return format!("You Must Set A Master Password First!");
            } else {
                match crypto::passwords::verify_password(password, pass[1].as_str()) {
                    Ok(verified) => {
                        if !verified {
                            return format!("Invalid Password!");
                        }
                        match store::sqlite::identity::fetch_all_identities_full(store::get_db_path().as_str()) {
                            Ok(identities) => {
                                for mut id in identities {
                                    if !(&id.encrypted) {
                                        match id.encrypt_identity(password) {
                                            Ok(encrypted) => {
                                                match store::sqlite::identity::update_identity_encrypted(store::get_db_path().as_str(), &encrypted) {
                                                    Ok(_) => println!("Updating Database, Identity.({}) Encrypted.", &encrypted.identity),
                                                    Err(err) => error!("Failed To Encrypt Identity.({}) : <{}>", &encrypted.identity, err)
                                                }
                                            },
                                            Err(err) => {
                                                return format!("{}", err);
                                            }
                                        }
                                    }
                                }
                                return format!("Wallet Encrypted!");
                            },
                            Err(err) => {return format!("{}", err);}
                        }
                    },
                    Err(_) => {
                        return format!("Invalid Password!");
                    }
                }
            }
        },
        Err(err) => {
            return format!("{:?}", err);
        }
    }
}

#[get("/wallet/download/<password>")]
pub fn download_wallet(password: &str) -> String {
    let mut ret_val: String = String::from("");
    match store::sqlite::identity::fetch_all_identities_full(store::get_db_path().as_str()) {
        Ok(mut identities) => {
            if password.len() < MINPASSWORDLEN {
                //invalid master password, don't decrypt wallet
                debug!("Dumping Wallet, Leaving Encrypted");
            }
            let mut is_valid = false;

            for identity in &mut identities {
                let id: String = identity.identity.clone();
                ret_val += &id.clone();
                ret_val += ",";
                let encrypted: bool = identity.encrypted;
                //ret_val +=
                if password.len() < MINPASSWORDLEN {
                    is_valid = true;
                    ret_val += &identity.seed.clone();
                    ret_val += ",";

                    ret_val += ",";
                    ret_val += "\n";
                } else {
                    if encrypted {
                        debug!("Decrypting {}", &id);
                        match identity.decrypt_identity(password) {
                            Ok(decrypted) => {
                                is_valid = true;
                                ret_val += &decrypted.seed.clone();
                                ret_val += ",";

                                ret_val += &decrypted.salt.clone();
                                ret_val += ",";

                                ret_val += &decrypted.hash.clone();
                                ret_val += "\n";
                            },
                            Err(_) => {
                                ret_val += ",,\n";
                            }
                        }
                    } else {
                        is_valid = true;
                        ret_val += &identity.seed.clone();
                        ret_val += ",";

                        ret_val += &identity.salt.clone();
                        ret_val += ",";

                        ret_val += &identity.hash.clone();
                        ret_val += "\n";
                    }
                }
            }
            if is_valid {
                format!("{}", ret_val)
            } else {
                format!("Invalid Password!")
            }
        },
        Err(err) => format!("{}", err)
    }
}
