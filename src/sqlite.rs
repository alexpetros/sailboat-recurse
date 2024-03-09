use rusqlite::{Connection, Error};


pub fn get_conn(path: &str) -> Result<Connection, Error> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

