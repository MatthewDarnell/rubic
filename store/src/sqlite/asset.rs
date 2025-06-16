use sqlite::State;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;


/*
    CREATE TABLE IF NOT EXISTS asset (
        identity TEXT NOT NULL,
        asset TEXT NOT NULL,
        issuance TEXT,
        ownership TEXT,
        tick INTEGER NOT NULL,
        universe_index INTEGER NOT NULL,
        siblings TEXT NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        peer TEXT,
        FOREIGN KEY(source_identity) REFERENCES identities(identity) ON DELETE CASCADE
    );
    */
pub fn create_asset(path: &str, peer: &str, identity: &str, issuance: &str, ownership: Option<&str>, possession: Option<&str>, tick: u32, universe_index: u32, siblings: &str) -> Result<(), String> {
    let _lock = get_db_lock().lock().unwrap();
    let ownership_query = ownership.unwrap_or_else(|| "");
    let possession_query = possession.unwrap_or_else(|| "");
    let prep_query = "INSERT INTO asset (peer, identity, issuance, ownership, possession, tick, universe_index, siblings) VALUES (
    :peer, :identity, :issuance, :ownership, :possession, :tick, :universe_index, :siblings);";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":peer", peer),
                        (":identity", identity),
                        (":issuance", issuance),
                        (":ownership", ownership_query),
                        (":possession", possession_query),
                        (":tick", tick.to_string().as_str()),
                        (":universe_index", universe_index.to_string().as_str()),
                        (":siblings", siblings)
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => { Ok(()) },
                                Err(error) => { Err(error.to_string()) },
                                _ => { Err("Weird!".to_string()) }
                            }
                        },
                        Err(err) => { Err(err.to_string()) }
                    }
                },
                Err(err) => {
                    error(format!("Failed To Prepare Statement! {}", err.to_string()).as_str());
                    Err(err.to_string())
                }
            }
        },
        Err(err) => {
            error(format!("Failed To Open Database! {}", err.to_string()).as_str());
            Err(err.to_string())
        }
    }
}