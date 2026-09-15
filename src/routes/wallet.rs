use rocket::{get, post};
use rocket::serde::{Deserialize, json::Json};
use logger::{debug, error, info};
use store;
use crate::routes::MINPASSWORDLEN;

/// JSON body carrying just the master password (never in the URL).
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PasswordRequest {
    #[serde(default)]
    pub password: String,
}

/// Body of `POST /wallet/unlock`.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UnlockRequest {
    pub password: String,
    pub timeout_ms: u64,
}

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

// POST /wallet/unlock  {"password": "...", "timeout_ms": 60000}
#[post("/wallet/unlock", format = "json", data = "<body>")]
pub fn unlock(body: Json<UnlockRequest>) -> String {
    let UnlockRequest { password, timeout_ms } = body.into_inner();
    if password.len() < MINPASSWORDLEN {
        return "Password Too Short!".to_string();
    } else if password.len() > 64 {
        return "Password Too Long!".to_string();
    }
    if timeout_ms > 99999 {
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

// POST /wallet/set_master_password  {"password": "..."}
#[post("/wallet/set_master_password", format = "json", data = "<body>")]
pub fn set_master_password(body: Json<PasswordRequest>) -> String {
    let password: &str = body.password.as_str();
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
                            logger::info("Master Password Set!");
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


// POST /wallet/reset  {"password": "..."}
// Replaces the master password: forgets any current unlock, removes every
// identity encrypted under the old password (its seeds cannot be recovered
// without it) and the old password itself, then sets the new one. Unencrypted
// identities are kept. Works on a fresh database too (nothing to remove). The
// "Import DB From CSV" wizard calls this, then adds the CSV rows one by one.
#[post("/wallet/reset", format = "json", data = "<body>")]
pub fn reset_wallet(body: Json<PasswordRequest>) -> String {
    let password: &str = body.password.as_str();
    if password.len() < MINPASSWORDLEN {
        return format!("Password Too Short!");
    }
    let path = store::get_db_path();
    // An unlock under the old password must not outlive it: identities added
    // while "unlocked" are encrypted with the unlocked password. Zeroing the
    // buffer is what the unlock timer does when it fires.
    if let Ok(mut unlocked) = protocol::wallet_unlock::PLAINTEXT_DECRYPT_PASSWORD.lock() {
        unlocked.fill(0);
    }
    if let Err(err) = store::sqlite::identity::delete_encrypted_identities(path.as_str()) {
        logger::error(format!("Failed To Reset Wallet; Could Not Remove Encrypted Identities: {}", err).as_str());
        return format!("Failed To Reset Wallet: {}", err);
    }
    if let Err(err) = store::sqlite::master_password::delete_master_password(path.as_str()) {
        logger::error(format!("Failed To Reset Wallet; Could Not Remove Master Password: {}", err).as_str());
        return format!("Failed To Reset Wallet: {}", err);
    }
    match crypto::passwords::hash_password(password) {
        Ok(hashed) => {
            match store::sqlite::master_password::set_master_password(path.as_str(), hashed.as_str()) {
                Ok(_) => {
                    logger::info("Wallet Reset; Master Password Set!");
                    format!("Master Password Set!")
                },
                Err(err) => format!("{}", err),
            }
        },
        Err(err) => format!("{}", err),
    }
}


// POST /wallet/encrypt  {"password": "..."}
#[post("/wallet/encrypt", format = "json", data = "<body>")]
pub fn encrypt_wallet(body: Json<PasswordRequest>) -> String {
    let password: &str = body.password.as_str();
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
                                                    Ok(_) => {
                                                        info(format!("Updating Database, Identity.({}) Encrypted.", &encrypted.identity).as_str());
                                                        println!("Updating Database, Identity.({}) Encrypted.", &encrypted.identity)
                                                    },
                                                    Err(err) => error!("Failed To Encrypt Identity.({}) : <{}>", &encrypted.identity, err)
                                                }
                                            },
                                            Err(err) => {
                                                return format!("{}", err);
                                            }
                                        }
                                    }
                                }
                                logger::info("Wallet Encrypted!");
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

// POST /wallet/download  {"password": "..."}  — an empty/short password dumps seeds still encrypted
#[post("/wallet/download", format = "json", data = "<body>")]
pub fn download_wallet(body: Json<PasswordRequest>) -> String {
    let password: &str = body.password.as_str();
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
