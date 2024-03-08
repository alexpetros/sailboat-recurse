use rusqlite::{Connection, Error};


pub fn get_conn(path: &str) -> Result<Connection, Error> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

    // let mut query = conn.prepare("SELECT author_name, author_handle, content FROM posts").unwrap();
    // let rows = query.query_map((), |row| {
    //     let post = Post {
    //         author_name: row.get(0)?,
    //         author_handle: row.get(1)?,
    //         content: row.get(2)?
    //     };
    //     Ok(post)
    // }).unwrap();

    // let mut posts = Vec::new();
    // for post in rows {
    //     posts.push(post.unwrap())
    // }


