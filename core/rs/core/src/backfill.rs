use sqlite_nostd::{sqlite3, Connection, Destructor, ManagedStmt, ResultCode};
extern crate alloc;
use alloc::format;
use alloc::{vec, vec::Vec};

/**
 * Backfills rows in a table with clock values.
 */
pub fn backfill_table(
    db: *mut sqlite3,
    table: &str,
    pk_cols: Vec<&str>,
    non_pk_cols: Vec<&str>,
) -> Result<ResultCode, ResultCode> {
    db.exec_safe("SAVEPOINT backfill")?;

    let sql = format!(
      "SELECT {pk_cols} FROM \"{table}\" as t1
        LEFT JOIN \"{table}__crsql_clock\" as t2 ON {pk_on_conditions} WHERE t2.\"{first_pk}\" IS NULL",
      table = crate::escape_ident(table),
      pk_cols = pk_cols
          .iter()
          .map(|f| format!("t1.\"{}\"", crate::escape_ident(f)))
          .collect::<Vec<_>>()
          .join(", "),
      pk_on_conditions = pk_cols
          .iter()
          .map(|f| format!("t1.\"{}\" = t2.\"{}\"", crate::escape_ident(f), crate::escape_ident(f)))
          .collect::<Vec<_>>()
          .join(" AND "),
      first_pk = crate::escape_ident(pk_cols[0]),
    );
    let stmt = db.prepare_v2(&sql);

    let result = match stmt {
        Ok(stmt) => create_clock_rows_from_stmt(stmt, db, table, &pk_cols, &non_pk_cols),
        Err(e) => Err(e),
    };

    if let Err(e) = result {
        db.exec_safe("ROLLBACK TO backfill")?;
        return Err(e);
    }

    if let Err(e) = backfill_missing_columns(db, table, &pk_cols, &non_pk_cols) {
        db.exec_safe("ROLLBACK TO backfill")?;
        return Err(e);
    }

    db.exec_safe("RELEASE backfill")
}

/**
* Given a statement that returns rows in the source table not present
* in the clock table, create those rows in the clock table.
*/
fn create_clock_rows_from_stmt(
    read_stmt: ManagedStmt,
    db: *mut sqlite3,
    table: &str,
    pk_cols: &Vec<&str>,
    non_pk_cols: &Vec<&str>,
) -> Result<ResultCode, ResultCode> {
    let sql = format!(
        "INSERT INTO \"{table}__crsql_clock\"
          ({pk_cols}, __crsql_col_name, __crsql_col_version, __crsql_db_version) VALUES
          ({pk_values}, ?, 1, crsql_nextdbversion())",
        table = crate::escape_ident(table),
        pk_cols = pk_cols
            .iter()
            .map(|f| format!("\"{}\"", crate::escape_ident(f)))
            .collect::<Vec<_>>()
            .join(", "),
        pk_values = pk_cols.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
    );
    let write_stmt = db.prepare_v2(&sql)?;

    while read_stmt.step()? == ResultCode::ROW {
        // bind primary key values
        for (i, _name) in pk_cols.iter().enumerate() {
            let value = read_stmt.column_value(i as i32)?;
            write_stmt.bind_value(i as i32 + 1, value)?;
        }

        // TODO: handle the case here where the _are no_ non_pk_cols!!!
        // just insert the pk only sentinel.
        for col in non_pk_cols.iter() {
            // We even backfill default values since we can't differentiate between an explicit
            // reset to a default vs an implicit set to default on create.
            write_stmt.bind_text(pk_cols.len() as i32 + 1, col, Destructor::STATIC)?;
            write_stmt.step()?;
            write_stmt.reset()?;
        }
        if (non_pk_cols.len() == 0) {
            write_stmt.bind_text(pk_cols.len() as i32 + 1, "__crsql_pko", Destructor::STATIC)?;
            write_stmt.step()?;
            write_stmt.reset()?;
        }
    }

    Ok(ResultCode::OK)
}

/**
* For each column, make sure there was a clock table entry.
* If not, fill the data in for it for each row.
*
* Can we optimize and skip cases where it is equivalent to the default value?
* E.g., adding a new column set to default values should not require a backfill...
*/
fn backfill_missing_columns(
    db: *mut sqlite3,
    table: &str,
    pk_cols: &Vec<&str>,
    non_pk_cols: &Vec<&str>,
) -> Result<ResultCode, ResultCode> {
    for non_pk_col in non_pk_cols {
        fill_column(db, table, &pk_cols, non_pk_col)?;
    }

    Ok(ResultCode::OK)
}

/**
*
*/
fn fill_column(
    db: *mut sqlite3,
    table: &str,
    pk_cols: &Vec<&str>,
    non_pk_col: &str,
) -> Result<ResultCode, ResultCode> {
    // Only return rows for which
    // - a row does not exist for that pk combo _and_ cid in the clock table
    // an optimization would be to filter out rows which are set to the default value.
    let sql = format!(
        "SELECT {pk_cols} FROM {table} as t1
          LEFT JOIN \"{table}__crsql_clock\" as t2 ON {pk_on_conditions} AND t2.__crsql_col_name = ?
          WHERE t2.\"{first_pk}\" IS NULL",
        table = crate::escape_ident(table),
        pk_cols = pk_cols
            .iter()
            .map(|f| format!("t1.\"{}\"", crate::escape_ident(f)))
            .collect::<Vec<_>>()
            .join(", "),
        pk_on_conditions = pk_cols
            .iter()
            .map(|f| format!(
                "t1.\"{}\" = t2.\"{}\"",
                crate::escape_ident(f),
                crate::escape_ident(f)
            ))
            .collect::<Vec<_>>()
            .join(" AND "),
        first_pk = crate::escape_ident(pk_cols[0])
    );
    let read_stmt = db.prepare_v2(&sql)?;
    read_stmt.bind_text(1, non_pk_col, Destructor::STATIC)?;

    let non_pk_cols = vec![non_pk_col];
    create_clock_rows_from_stmt(read_stmt, db, table, pk_cols, &non_pk_cols)
}
