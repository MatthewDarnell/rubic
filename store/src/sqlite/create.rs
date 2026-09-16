
pub fn open_database(path: &str, create: bool) -> Result<sqlite::Connection, String> {
    let query = "
    PRAGMA foreign_keys = ON;
    PRAGMA journal_mode = WAL;
    PRAGMA busy_timeout = 1000;
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
    CREATE TABLE IF NOT EXISTS tick (
      tick INTEGER UNIQUE,
      peer TEXT NOT NULL,
      valid BOOLEAN DEFAULT false,
      transaction_digests_hash TEXT NOT NULL DEFAULT '',
      transaction_digests TEXT NOT NULL DEFAULT '',
      created DATETIME DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY(peer) REFERENCES peer(id)
    );
    CREATE TABLE IF NOT EXISTS master_password (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      ct TEXT UNIQUE NOT NULL,
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
    CREATE TABLE IF NOT EXISTS transfer (
        source_identity TEXT NOT NULL,
        destination_identity TEXT NOT NULL,
        amount UNSIGNED INTEGER NOT NULL,
        tick UNSIGNED INTEGER NOT NULL,
        signature TEXT NOT NULL,
        txid TEXT DEFAULT NULL UNIQUE,
        broadcast BOOLEAN DEFAULT FALSE,
        last_broadcast_tick INTEGER DEFAULT 0,
        status INTEGER DEFAULT -1,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY(source_identity) REFERENCES identities(identity) ON DELETE CASCADE
    );
    CREATE TABLE IF NOT EXISTS computors (
        epoch INTEGER NOT NULL UNIQUE,
        pub_keys TEXT NOT NULL,
        signature TEXT NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        peer TEXT
    );
    CREATE TABLE IF NOT EXISTS asset_issuance (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        pub_key TEXT NOT NULL,
        type INTEGER NOT NULL,
        name TEXT NOT NULL,
        num_decimal INTEGER,
        unit_measure TEXT,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        peer TEXT,
        UNIQUE (pub_key, type, name, num_decimal, unit_measure) ON CONFLICT IGNORE
    );
    CREATE TABLE IF NOT EXISTS asset_record (
        asset_id INTEGER NOT NULL,
        record_type TEXT CHECK( record_type IN ('O','P') ) NOT NULL,
        identity TEXT NOT NULL,
        managing_contract INTEGER,
        issuance_index INTEGER,
        num_shares INTEGER,
        tick INTEGER NOT NULL,
        FOREIGN KEY(asset_id) REFERENCES asset_issuance(id) ON DELETE CASCADE,
        FOREIGN KEY(identity) REFERENCES identities(identity) ON DELETE CASCADE,
        UNIQUE (asset_id, identity, record_type, managing_contract, issuance_index, tick) ON CONFLICT REPLACE
    );
    CREATE TABLE IF NOT EXISTS asset_transfer (
        txid TEXT NOT NULL,
        issuer TEXT NOT NULL,
        new_owner_and_possessor TEXT NOT NULL,
        name TEXT NOT NULL,
        num_shares INTEGER NOT NULL,
        input_type INTEGER NOT NULL,
        input_size INTEGER NOT NULL,
        FOREIGN KEY(txid) REFERENCES transfer(txid) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS qx_order (
        txid TEXT NOT NULL,
        issuer TEXT NOT NULL,
        name TEXT NOT NULL,
        price INTEGER NOT NULL,
        num_shares INTEGER NOT NULL,
        input_type INTEGER NOT NULL,
        input_size INTEGER NOT NULL,
        FOREIGN KEY(txid) REFERENCES transfer(txid) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS qx_entity_order (
        identity TEXT NOT NULL,
        side TEXT CHECK( side IN ('A','B') ) NOT NULL,
        issuer TEXT NOT NULL,
        asset TEXT NOT NULL,
        price INTEGER NOT NULL,
        num_shares INTEGER NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY(identity, side, issuer, asset, price, num_shares)
    );

    CREATE TABLE IF NOT EXISTS qx_orderbook (
        asset TEXT NOT NULL,
        entity TEXT NOT NULL,
        price INTEGER NOT NULL,
        stale INTEGER DEFAULT 0,
        num_shares INTEGER NOT NULL,
        offset_at_price INTEGER NOT NULL,
        side TEXT CHECK( side IN ('A','B') ) NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY(asset, side, entity, price, num_shares)
    );

    DROP TABLE IF EXISTS random_round;
    CREATE TABLE IF NOT EXISTS random_session (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        identity TEXT NOT NULL,
        tier INTEGER NOT NULL,
        first_tick INTEGER NOT NULL,
        status INTEGER DEFAULT 0,
        broadcast_through INTEGER DEFAULT -1,
        leave_step INTEGER DEFAULT -1,
        last_accepted_tick INTEGER DEFAULT 0,
        reason TEXT DEFAULT '',
        auto_restart INTEGER DEFAULT 0,
        fail_streak INTEGER DEFAULT 0,
        continued_by INTEGER DEFAULT 0,
        failed_tick INTEGER DEFAULT 0,
        total_steps INTEGER DEFAULT 1000,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (identity) REFERENCES identities(identity) ON DELETE CASCADE
    );
    CREATE TABLE IF NOT EXISTS random_provider_report (
        identity TEXT NOT NULL,
        peer TEXT NOT NULL,
        checked_tick INTEGER NOT NULL,
        slots TEXT NOT NULL,
        updated DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (identity, peer)
    );
    CREATE TABLE IF NOT EXISTS random_step (
        session_id INTEGER NOT NULL,
        step INTEGER NOT NULL,
        tick INTEGER NOT NULL,
        reveal_secret TEXT NOT NULL,
        commit_secret TEXT NOT NULL,
        stay_txid TEXT,
        stay_sig TEXT,
        leave_txid TEXT,
        leave_sig TEXT,
        PRIMARY KEY (session_id, step),
        FOREIGN KEY (session_id) REFERENCES random_session(id) ON DELETE CASCADE
    );
    CREATE INDEX IF NOT EXISTS random_step_stay_txid ON random_step(stay_txid);
    CREATE INDEX IF NOT EXISTS random_step_leave_txid ON random_step(leave_txid);

";
    match sqlite::open(path) {
        Ok(mut connection) => {
            match connection.set_busy_timeout(1000) {
                Ok(_) => {},
                Err(error) => logger::error(format!("Failed To Set DB Busy Handler {}", error.to_string()).as_str())
            }
            match create {
                true => {
                    match connection.execute(query) {
                        Ok(_) => {
                            migrate(&connection);
                            Ok(connection)
                        },
                        Err(_err) => {
                            eprintln!("Error {}", _err);
                            Err(String::from(_err.to_string()))
                        }
                    }
                },
                false => {
                    Ok(connection)
                }
            }
        },
        Err(_err) => {
            eprintln!("Error opening database: {}", _err);
            Err(String::from("Failed To Create Db!"))
        }
    }
}


/// Columns added after a table first shipped. `CREATE TABLE IF NOT EXISTS` does
/// not touch existing tables, so each is applied as an idempotent ALTER: a
/// "duplicate column" error just means the database already has it.
fn migrate(connection: &sqlite::Connection) {
    const ADDED_COLUMNS: [&str; 8] = [
        "ALTER TABLE transfer ADD COLUMN last_broadcast_tick INTEGER DEFAULT 0;",
        "ALTER TABLE random_session ADD COLUMN last_accepted_tick INTEGER DEFAULT 0;",
        "ALTER TABLE random_session ADD COLUMN reason TEXT DEFAULT '';",
        "ALTER TABLE random_session ADD COLUMN auto_restart INTEGER DEFAULT 0;",
        "ALTER TABLE random_session ADD COLUMN fail_streak INTEGER DEFAULT 0;",
        "ALTER TABLE random_session ADD COLUMN continued_by INTEGER DEFAULT 0;",
        "ALTER TABLE random_session ADD COLUMN failed_tick INTEGER DEFAULT 0;",
        "ALTER TABLE random_session ADD COLUMN total_steps INTEGER DEFAULT 1000;",
    ];
    for statement in ADDED_COLUMNS {
        if let Err(err) = connection.execute(statement) {
            let text = err.to_string();
            if !text.contains("duplicate column") {
                logger::error(format!("Database migration failed ({}): {}", statement, text).as_str());
            }
        }
    }
    // Numbered one-off repairs, tracked in the file's user_version.
    const REPAIRS: [&str; 2] = [
        // 1: RANDOM steps marked failed because the contract did not accept them.
        //    Only the tick's transaction list can say whether a step was included.
        "UPDATE transfer SET status = -1 WHERE status = 1 AND destination_identity = 'DAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANMIG';",
        // 2: provider status is now kept per answering peer (random_provider_report).
        "DROP TABLE IF EXISTS random_provider;",
    ];
    let mut version: i64 = 0;
    let _ = connection.iterate("PRAGMA user_version;", |row| {
        version = row.first().and_then(|(_, v)| v.and_then(|v| v.parse().ok())).unwrap_or(0);
        true
    });
    for (index, statement) in REPAIRS.iter().enumerate() {
        let number = index as i64 + 1;
        if version >= number {
            continue;
        }
        match connection.execute(statement).and_then(|_| connection.execute(format!("PRAGMA user_version = {};", number))) {
            Ok(_) => version = number,
            Err(err) => {
                logger::error(format!("Database repair {} failed: {}", number, err).as_str());
                break;
            }
        }
    }
}

#[cfg(test)]
mod store_tests {
    use crate::sqlite::create::open_database;
    use serial_test::serial;

    # [test]
    # [serial]
    fn create_new_db_in_memory() {
        match open_database(":memory:", true) {
            Ok(_) =>{ 
                //println!("db created in memory"); 
            },
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
