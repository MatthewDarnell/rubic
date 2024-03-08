use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Duration;
use rocket::get;
use spmc::Receiver;
use uuid::Uuid;
use store;


//transfer/${sourceIdentity}/${destinationIdentity}/${amountToSend}/${expirationTick}/${password}
#[get("/transfer/<source>/<dest>/<amount>/<expiration>/<password>")]
pub fn transfer(source: &str, dest: &str, amount: &str, expiration: &str, password: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);

    let string_amount: String = amount.to_string();

    let string_expiration: String = expiration.to_string();

    let source_identity: String = source.to_string();
    let dest_identity: String = dest.to_string();
    let password_string: String = password.to_string();

    if source_identity.len() != 60 {
        return format!("Invalid Source Identity!");
    }

    if dest_identity.len() != 60 {
        return format!("Invalid Destination Identity!");
    }

    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "transfer".to_string());
    map.insert("source".to_string(), source_identity);
    map.insert("dest".to_string(), dest_identity);
    map.insert("amount".to_string(), string_amount);
    map.insert("expiration".to_string(), string_expiration);

    if password.len() > 1 {
        map.insert("password".to_string(), password_string);
    }
    let request_id: String = Uuid::new_v4().to_string();
    map.insert("message_id".to_string(), request_id.clone());
    tx.send(map).unwrap();
    let mut index = 0;
    loop {
        index = index + 1;
        if index > 75 {
            return format!("Timed Out")
        }
        std::thread::sleep(Duration::from_millis(250));
        match rx.try_recv() {
            Ok(response) => {
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(_) => {
                //error!("Failed To Receive From Api");
            }
        }
    }
}