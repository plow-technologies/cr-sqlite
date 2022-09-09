use sqlite3_parser::ast::{
  AlterTableBody, ColumnDefinition, CreateTableBody, Name, NamedTableConstraint, QualifiedName,
};

use crate::{
  ast::{to_string, ColumnDefinitionExt, QualifiedNameExt},
  sql_bits::{ifne_str, table_opts_str, temp_str},
};

/**
 * Takes in a non crr table definition and returns the crr table definition for that table.
 *
 * Conversion to a crr definition involves adding
 * - a version column for each column
 * - a single causal length column
 * - a single update source column
 */
pub fn create_crr_tbl_stmt(
  temporary: &bool,
  if_not_exists: &bool,
  tbl_name: &QualifiedName,
  body: &CreateTableBody,
) -> String {
  format!(
    "CREATE {temporary} TABLE {if_not_exists} {tbl_name} {body_def}",
    temporary = temp_str(temporary),
    if_not_exists = ifne_str(if_not_exists),
    tbl_name = tbl_name.to_crr_table_ident(),
    body_def = create_body_def(body).unwrap()
  )
}

/**
 * Takes in the body definition of a non-crr table.
 *
 * From this, it creates the crr definition of that table by add:
 * - version columns for each column
 * - a single causal length column
 * - a single update source column
 *
 * Version column for LWW support.
 * Causal length to track deletes.
 * Update source to track where an update came in from.
 *
 * Future -- we could extend this method to work on either body def.
 * If it receive a crr body def it just returns that body def unmodified.
 */
fn create_body_def(body: &CreateTableBody) -> Result<String, &'static str> {
  match body {
    CreateTableBody::ColumnsAndConstraints {
      columns,
      constraints,
      options,
    } => Ok(format!(
      "({column_and_constraint_list}) {table_options}",
      // add version columns, causal length and update src
      // validate constraints (e.g., too many unique keys)
      column_and_constraint_list = to_crr_column_idents(columns)
        .into_iter()
        .chain(to_crr_constraint_idents(constraints).into_iter())
        .collect::<Vec<_>>()
        .join(",\n"),
      table_options = table_opts_str(options)
    )),
    CreateTableBody::AsSelect(_) => {
      Err("table creation from select statements is not yet supported for crrs")
    }
  }
}

/**
 * Takes a list of column definitions and returns those column definitions + the crr column definitions.
 *
 * This adds version columns, causal length and update source columns.
 *
 * We might want to narrow the scope of this function down to only adding version columns and using
 * other functions to append causal length and update source.
 */
fn to_crr_column_idents(columns: &Vec<ColumnDefinition>) -> Vec<String> {
  let mut ret: Vec<String> = vec![];

  // TODO: primary keys do not get versions.
  // Test that primary key is known regardless of constraint specification on the table. I.e., inline on col or after.
  for c in columns {
    // just copy over the definition
    ret.push(to_string(c));
    if c.is_primary_key() {
      continue;
    }

    // create the version col
    ret.push(to_string(ColumnDefinition {
      col_name: Name(format!("{}__cfsql_v", c.col_name.0)),
      col_type: None,
      constraints: vec![],
    }));
  }

  ret.push(to_string(ColumnDefinition {
    col_name: Name("cfsql_cl".to_string()),
    col_type: None,
    constraints: vec![],
  }));

  ret.push(to_string(ColumnDefinition {
    col_name: Name("cfsql_src".to_string()),
    col_type: None,
    constraints: vec![],
  }));

  ret
}

/**
 * Takes an optional list of table constraints and stringifies them
 */
fn to_crr_constraint_idents(constraints: &Option<Vec<NamedTableConstraint>>) -> Vec<String> {
  match constraints {
    None => vec![],
    Some(constraints) => constraints.iter().map(|x| to_string(x)).collect(),
  }
}

/**
 * For each row in a crr table we must track the logical time at which that row was created/updated/deleted.
 *
 * The crr_clock_table holds this information for all the rows in a crr table.
 *
 * These clocks are used to compute deltas between two peers. Peer A sends its clock to peer B and, using only that clock,
 * peer B can tell peer A what updates it is missing.
 */
pub fn create_crr_clock_tbl_stmt(
  temporary: &bool,
  if_not_exists: &bool,
  tbl_name: &QualifiedName,
) -> String {
  format!(
    "CREATE {temporary} TABLE {ifne} {tbl_name} (
    \"id\" integer NOT NULL,
    \"siteId\" integer NOT NULL,
    \"version\" integer NOT NULL,
    PRIMARY KEY (\"siteId\", \"id\")
  )",
    temporary = temp_str(temporary),
    ifne = ifne_str(if_not_exists),
    tbl_name = tbl_name.to_crr_clock_table_ident(),
  )
}

pub fn create_alter_crr_tbl_stmt(body: &AlterTableBody) -> String {
  // branches:
  // rename table
  // rename col
  // add col
  // drop col
  // --
  // rename table -> rename crr
  // rename col -> std but crr tbl + version col
  // add col -> std but crr tbl + version col
  // drop col -> std but crr tbl + drop version col
  format!("ALTER TABLE")
}