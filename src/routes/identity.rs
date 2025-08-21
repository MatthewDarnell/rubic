use rocket::get;
use store::get_db_path;
use store::sqlite::identity::insert_new_identity;
use store::sqlite::master_password::get_master_password;
use crypto::passwords::verify_password;
use protocol::identity;
use crate::routes::MINPASSWORDLEN;

#[get("/balance/<address>")]
pub fn balance(address: &str) -> String {
    match store::sqlite::identity::fetch_balance_by_identity(store::get_db_path().as_str(), address) {
        Ok(value) => { format!("{:?}", value) },
        Err(error) => format!("{}", error)
    }
}

#[get("/identities")]
pub fn get_identities() -> String {
    match store::sqlite::identity::fetch_all_identities_full(store::get_db_path().as_str()) {
        Ok(v) => {
            let mut response: Vec<String> = vec![];
            for identity in &v {
                let encrypted: String = match identity.encrypted {
                    true => "true".to_string(),
                    _ => "false".to_string()
                };
                response.push(identity.identity.clone());
                response.push(encrypted);
            }
            format!("{:?}", response)
        },
        Err(err) => format!("{}", err)
    }
}

#[get("/identity/from_seed/<seed>")]
pub fn get_identity_from_seed(seed: &str) -> String {
    let i: identity::Identity = identity::Identity::new(seed);
    format!("{}", i.identity.as_str())
}


#[get("/identity/new/<password>")]
pub fn create_random_identity(password: &str) -> String {
    let mut seed_string: String = String::from("");
    while seed_string.len() < 55 {
        let temp_seed: Vec<u8> = crypto::random::random_bytes(32);
        for val in temp_seed {
            if val >= 97 && val <= 122 {
                seed_string += char::from(val).to_string().as_str();
                if seed_string.len() >= 55 {
                    break;
                }
            }
        }
    }
    let mut id: identity::Identity = identity::Identity::new(seed_string.as_str());
    
    let unlocked = protocol::wallet_unlock::is_wallet_unlocked();
    if unlocked.is_ok() {
        let is_unlocked = unlocked.unwrap();
        if is_unlocked {
            return match protocol::wallet_unlock::get_plaintext_password() {
                Ok(password) => {
                    id = id.encrypt_identity(password.as_str()).expect("Failed To Encrypt Identity With Unlocked Password!");
                    let response = match insert_new_identity(get_db_path().as_str(), &id) {
                        Ok(_) => "200",
                        Err(_) => "Failed To Insert Identity!"
                    };
                    format!("{}", response)
                },
                Err(_) => "Failed To Retrieve Unlocked Password".to_string()
            }
        }
    }
    
    
    if password.len() >= MINPASSWORDLEN { //Minimum length
        let master_password = get_master_password(get_db_path().as_str())
                                            .expect("Failed To Fetch Master Password!");
        match verify_password(password, master_password[1].as_str()) {
            Ok(verified) => {
                if !verified {
                    return format!("Invalid Password!");
                }
                id = id.encrypt_identity(password).expect("Failed To Encrypt Identity!");
            },
            Err(error) => { return format!("Failed to Verify Master Password!: <{}>", error); }
        }
    }

    let response = match insert_new_identity(get_db_path().as_str(), &id) {
        Ok(_) => "200",
        Err(_) => "Failed To Insert Identity!"
    };
    return format!("{}", response);
}

#[get("/identity/add/<seed>")]
pub fn add_identity(seed: &str) -> String {
    if seed.len() != 55 {
        return format!("Invalid Seed! Must be Exactly 55 characters in length!");
    }
    for i in seed.as_bytes() {
        if *i < b'a' || *i > b'z' {
            return format!("Invalid Seed! Must be a-z lowercase!");
        }
    }
    let mut id: identity::Identity = identity::Identity::new(seed);

    let unlocked = protocol::wallet_unlock::is_wallet_unlocked();
    if unlocked.is_ok() {
        let is_unlocked = unlocked.unwrap();
        if is_unlocked {
            return match protocol::wallet_unlock::get_plaintext_password() {
                Ok(password) => {
                    id = id.encrypt_identity(password.as_str()).expect("Failed To Encrypt Identity With Unlocked Password!");
                    let response = match insert_new_identity(get_db_path().as_str(), &id) {
                        Ok(_) => "200",
                        Err(_) => "Failed To Insert Identity!"
                    };
                    format!("{}", response)
                },
                Err(_) => "Failed To Retrieve Unlocked Password".to_string()
            }
        }
    }
    
    let response = match insert_new_identity(get_db_path().as_str(), &id) {
        Ok(_) => "200",
        Err(_) => "Failed To Insert Identity!"
    };
    return format!("{}", response);
}

#[get("/identity/add/<seed>/<password>")]
pub fn add_identity_with_password(seed: &str, password: &str) -> String {
    if seed.len() != 55 {
        return format!("Invalid Seed! Must be Exactly 55 characters in length!");
    }
    for i in seed.as_bytes() {
        if *i < b'a' || *i > b'z' {
            return format!("Invalid Seed! Must be a-z lowercase!");
        }
    }
    let mut id: identity::Identity = identity::Identity::new(seed);

    let unlocked = protocol::wallet_unlock::is_wallet_unlocked();
    if unlocked.is_ok() {
        let is_unlocked = unlocked.unwrap();
        if is_unlocked {
            return match protocol::wallet_unlock::get_plaintext_password() {
                Ok(password) => {
                    id = id.encrypt_identity(password.as_str()).expect("Failed To Encrypt Identity With Unlocked Password!");
                    let response = match insert_new_identity(get_db_path().as_str(), &id) {
                        Ok(_) => "200",
                        Err(_) => "Failed To Insert Identity!"
                    };
                    format!("{}", response)
                },
                Err(_) => "Failed To Retrieve Unlocked Password".to_string()
            }
        }
    }
    
    
    if password.len() >= MINPASSWORDLEN { //Minimum length
        let master_password = get_master_password(get_db_path().as_str())
            .expect("Failed To Fetch Master Password!");
        match verify_password(password, master_password[1].as_str()) {
            Ok(verified) => {
                if !verified {
                    return format!("Invalid Password!");
                }
                id = id.encrypt_identity(password).expect("Failed To Encrypt Identity!");
            },
            Err(error) => { return format!("Failed to Verify Master Password!: <{}>", error); }
        }
    }
    let response = match insert_new_identity(get_db_path().as_str(), &id) {
        Ok(_) => "200",
        Err(_) => "Failed To Insert Identity!"
    };
    return format!("{}", response);
}

#[get("/identity/delete/<identity>/<password>")]
pub fn delete_identity(identity: &str, password: &str) -> String {
    match store::sqlite::identity::fetch_identity(get_db_path().as_str(), identity) {
        Ok(mut id) => {
            let unlocked = protocol::wallet_unlock::is_wallet_unlocked();
            if unlocked.is_ok() {
                if unlocked.unwrap() {
                    let response = match store::sqlite::identity::delete_identity(get_db_path().as_str(), identity) {
                        Ok(_) => "200",
                        Err(_) => "Failed To Delete Identity!"
                    };
                    return format!("{}", response);   
                }
            }
            
            if id.encrypted && password.len() < MINPASSWORDLEN {
                return "Must Supply Password To Delete Encrypted Identity".to_string();
            }
            if id.encrypted {
                match id.decrypt_identity(password) {
                    Ok(_) => {},
                    Err(_) => {
                        return "Invalid Password!".to_string()
                    }
                }
            }
            let response = match store::sqlite::identity::delete_identity(get_db_path().as_str(), identity) {
                Ok(_) => "200",
                Err(_) => "Failed To Delete Identity!"
            };
            format!("{}", response)
        },
        Err(_) => {
            "Identity Not Found!".to_string()
        }
    }
}
