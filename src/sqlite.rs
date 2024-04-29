use rusqlite::{Connection, Error};

pub fn initliaze_db(path: &str) -> Result<(), Error> {
    let conn = get_conn(path)?;
    let sql = include_str!("./db/migrations/0-init.sql");
    conn.execute_batch(sql)?;
    conn.close().map_err(|e| e.1)?;
    Ok(())
}

pub fn get_conn(path: &str) -> Result<Connection, Error> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

#[macro_export]
macro_rules! query_row_custom {
    (
        $db:expr,
        $struct_name:ident { $( $field_name:ident : $type:ty ),+ },
        $query:literal,
        $params:expr
    ) => {{
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct $struct_name {
            $( $field_name : $type),+
        }
        let mut query = $db.prepare($query)?;
        let row = query.query_row($params, |row| {
            let item = $crate::make_struct!(row, $struct_name, $( $field_name ),+);
            Ok(item)
        });
        row
    }}
}

#[macro_export]
macro_rules! query_row {
    (
        $db:expr,
        $struct_name:ident { $( $field_name:ident : $type:ty ),+ },
        $query:literal,
        $params:expr
    ) => {{
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct $struct_name {
            $( $field_name : $type),+
        }
        let query_str = concat!("SELECT ", $crate::join_with_commas!($( $field_name ),+), " ", $query);
        let mut query = $db.prepare(query_str)?;
        let row = query.query_row($params, |row| {
            let item = $crate::make_struct!(row, $struct_name, $( $field_name ),+);
            Ok(item)
        });
        row
    }}
}

#[macro_export]
macro_rules! query_map {
    (
        $db:expr,
        $struct_name:ident { $( $field_name:ident : $type:ty ),+ },
        $query:literal,
        $params:expr
    ) => {{
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct $struct_name {
            $( $field_name : $type),+
        }
        let query_str = concat!("SELECT ", $crate::join_with_commas!($( $field_name ),+), " ", $query);
        let mut query = $db.prepare(query_str)?;
        let rows = query.query_map($params, |row| {
            let item = $crate::make_struct!(row, $struct_name, $( $field_name ),+);
            Ok(item)
        })?;

        let rows: Vec<$struct_name> = rows.collect::<Result<_, _>>()?;
        rows
    }}
}

/// Create a struct recursively
/// Rust does now allow helper macros to populate the body of a struct definition
/// So instead we turn each row.get(x) into a token, recursively, and then populate the struct
/// with those tokens.
#[macro_export]
macro_rules! make_struct {
    (@ $row:expr, $_count:expr, $struct_name:ident, { } [ $($result:tt)* ]) => {
        $struct_name {
            $($result)*
        }
    };

    (@
     $row:expr,
     $count:expr,
     $struct_name:ident,
     { $first_field_name:ident $($field_name:ident)* }
     [ $($result:tt)* ]) => {
        $crate::make_struct!(
            @
            $row,
            $count + 1,
            $struct_name,
            { $($field_name)* }
            [$($result)* $first_field_name: $row.get($count)?, ])
    };

    ($row:expr, $struct_name:ident, $( $field_name:ident ),+ )  => {
        $crate::make_struct!(@ $row, 0, $struct_name, { $($field_name)* } [])
    };
}

/// Stringify and join a series of identifiers with commas
#[macro_export]
macro_rules! join_with_commas {
    ( $name:ident ) => {
        stringify!($name)
    };
    ( $first:ident, $( $name:ident ),+ ) => {
        concat!(stringify!($first), ", ", $crate::join_with_commas!( $( $name ),+ ))
    };
}


