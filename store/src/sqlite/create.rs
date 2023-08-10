pub fn open_database(path: &str, create: bool) -> Result<sqlite::Connection, String> {
    let query = "
    CREATE TABLE IF NOT EXISTS identities (identity_index INTEGER PRIMARY KEY, seed TEXT, seed_ct TEXT, salt TEXT, hash TEXT, identity TEXT, created DATETIME DEFAULT CURRENT_TIMESTAMP);
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

#[test]
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
