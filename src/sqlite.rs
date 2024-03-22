use rusqlite::{Connection, Error};

pub fn initliaze_db(path: &str) -> Result<(), Error> {
    let conn = get_conn(path)?;
    let sql = include_str!("./db/migrations/0-init.sql");
    conn.execute_batch(sql)?;
    conn.close().map_err(|e| { e.1 })?;
    Ok(())
}

pub fn get_conn(path: &str) -> Result<Connection, Error> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

