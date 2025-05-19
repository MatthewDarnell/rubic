use rocket::get;
use logger::{debug, error, info};
use store::get_db_path;
use store::sqlite::crud;

//transfer/${sourceIdentity}/${destinationIdentity}/${amountToSend}/${expirationTick}/${password}
#[get("/transfer/<source>/<dest>/<amount>/<expiration>/<password>")]
pub fn transfer(source: &str, dest: &str, amount: &str, expiration: &str, password: &str) -> String {
    let source_identity: String = source.to_string();
    let dest_identity: String = dest.to_string();

    if source_identity.len() != 60 {
        return format!("Invalid Source Identity!");
    }

    if dest_identity.len() != 60 {
        return format!("Invalid Destination Identity!");
    }


    let mut source_identity = match crud::fetch_identity(get_db_path().as_str(), source) {
        Ok(identity) => identity,
        Err(_) => {
            error!("Failed To Make Transfer, Unknown Identity {}", source);
            return "Unknown Identity".to_string();
        }
    };

    if source_identity.encrypted {
        if password.len() > 1 {
            source_identity = match crud::master_password::get_master_password(get_db_path().as_str()) {
                Ok(master_password) => {
                    match crypto::passwords::verify_password(password, master_password[1].as_str()) {
                        Ok(verified) => {
                            if !verified {
                                error!("Failed To Create Transfer; Invalid Password");
                                return "Invalid Password".to_string();
                            } else {
                                match source_identity.decrypt_identity(password) {
                                    Ok(identity) => identity,
                                    Err(_) => {
                                        error!("Failed To Create Transfer; Invalid Password For This Identity");
                                        return "Invalid Password For This Identity!".to_string();
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            error!("Failed To Verify Master Password Vs Supplied Password");
                            return "Failed To Verify Master Password Vs Supplied Password!".to_string();
                        }
                    }
                },
                Err(_) => {
                    error!("Identity Is Encrypted, Yet No Master Password Set! Weird");
                    return "Identity Is Encrypted, Yet No Master Password Set! Weird!".to_string();
                }
            };
        } else {
            error!("Failed To Decrypt Password For Transfer; No Password Supplied");
            return "Must Enter A Password!".to_string();
        }
    } else {
        debug!("Creating Transfer, Wallet Is Not Encrypted!");
    }
    let amt: u64 = amount.parse().unwrap();
    let tck: u32 = expiration.parse().unwrap();

    info!("Creating Transfer: {} .({}) ---> {} (Expires At Tick.<{}>)", &source_identity.identity.as_str(), amt.to_string().as_str(), dest, tck.to_string().as_str());
    let transfer_tx = api::transfer::TransferTransaction::from_vars(&source_identity, &dest, amt, tck);
    let txid = transfer_tx.txid();

    let sig = transfer_tx._signature;
    let sig_str = hex::encode(sig);

    match crud::create_transfer(
        get_db_path().as_str(),
        source_identity.identity.as_str(),
        dest_identity.as_str(),
        amt,
        tck,
        sig_str.as_str(),
        txid.as_str()
    ) {
        Ok(_) => {
            txid
        },
        Err(err) => {
            println!("Error Inserting Tx into Db: {}", err);
            "Error Creating Transfer".to_string()
        }
    }
}