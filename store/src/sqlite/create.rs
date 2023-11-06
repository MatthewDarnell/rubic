pub fn open_database(path: &str, create: bool) -> Result<sqlite::Connection, String> {
    let query = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS peer (
      id TEXT UNIQUE NOT NULL PRIMARY KEY,
      ip TEXT UNIQUE NOT NULL,
      nick TEXT,
      whitelisted INTEGER,
      ping INTEGER,
      last_responded INTEGER,
      created DATETIME DEFAULT CURRENT_TIMESTAMP,
      connected BOOLEAN DEFAULT false
    );
    CREATE TABLE IF NOT EXISTS master_password (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      seed TEXT UNIQUE NOT NULL,
      salt TEXT NOT NULL,
      hash TEXT NOT NULL,
      created DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS identities (
        seed TEXT,
        salt TEXT,
        hash TEXT,
        is_encrypted INTEGER,
        identity TEXT UNIQUE,
        created DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS response (
        peer TEXT NOT NULL,
        header TEXT NOT NULL,
        type INTEGER NOT NULL,
        data TEXT NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS response_entity (
        peer TEXT NOT NULL,
        identity TEXT NOT NULL,
        incoming INTEGER NOT NULL,
        outgoing INTEGER NOT NULL,
        balance INTEGER NOT NULL,
        num_in_txs INTEGER NOT NULL,
        num_out_txs INTEGER NOT NULL,
        latest_in_tick INTEGER NOT NULL,
        latest_out_tick INTEGER NOT NULL,
        tick INTEGER NOT NULL,
        spectrum_index INTEGER NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP
    );
";
    //        FOREIGN KEY(identity) REFERENCES identities(identity)
    match sqlite::open(path) {
        Ok(connection) => {
            match create {
                true => {
                    match connection.execute(query) {
                        Ok(_) => Ok(connection),
                        Err(err) => Err(String::from(err.to_string()))
                    }
                },
                false => {
                    Ok(connection)
                }
            }
        },
        Err(_) => Err(String::from("Failed To Create Db!"))
    }
}


#[cfg(test)]
mod store_tests {
    use identity::Identity;
    use crate::sqlite::create::open_database;
    use serial_test::serial;
    use std::fs;

    # [test]
    # [serial]
    fn create_new_db_in_memory() {
        match open_database(":memory:", true) {
            Ok(_) =>{ println!("db created in memory"); },
            Err(err) => {
                println!("{}", err);
                assert_eq!(1, 2);
            }
        }
    }


    #[test]
    #[serial]
    fn create_new_db_in_disk() {
        use std::fs;
        {
            match open_database("test.sqlite", true) {
                Ok(_) => {},
                Err(err) => {
                    println!("{}", err);
                    assert_eq!(1, 2);
                }
            }
        }
        fs::remove_file("test.sqlite").unwrap();

    }
}
