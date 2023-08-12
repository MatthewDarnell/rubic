pub fn open_database(path: &str, create: bool) -> Result<sqlite::Connection, String> {
    let query = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS account (
        name TEXT UNIQUE NOT NULL PRIMARY KEY,
        seed TEXT UNIQUE,
        salt TEXT UNIQUE,
        hash TEXT UNIQUE,
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
";
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
