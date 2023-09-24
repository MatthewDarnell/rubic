pub fn open_database(path: &str, create: bool) -> Result<sqlite::Connection, String> {
    let query = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS peer (
      id TEXT UNIQUE NOT NULL PRIMARY KEY,
      ip TEXT UNIQUE NOT NULL,
      nick TEXT,
      whitelisted INTEGER,
      ping UNSIGNED INTEGER,
      last_responded UNSIGNED INTEGER,
      CREATED  datetime DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS account (
        name TEXT UNIQUE NOT NULL PRIMARY KEY,
        seed TEXT,
        salt TEXT,
        hash TEXT,
        is_encrypted INTEGER,
        created DATETIME DEFAULT CURRENT_TIMESTAMP
    );
    CREATE TABLE IF NOT EXISTS identities (
        account TEXT NOT NULL,
        identity_index INTEGER,
        identity TEXT UNIQUE,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(account) REFERENCES account(name)
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
        Err(err) => Err(String::from("Failed To Create Db!"))
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
