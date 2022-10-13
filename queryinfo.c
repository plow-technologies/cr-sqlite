/**
 * Given a query:
 * - get its type
 * - get its unquoted table name
 *
 */
#include "cfsqlite-consts.h"
#include "cfsqlite-util.h"
#include "sqlite3ext.h"
SQLITE_EXTENSION_INIT3
#include "queryinfo.h"
#include <assert.h>

/**
 * Given a query passed to cfsqlite, determine what kind of schema modification
 * query it is.
 *
 * We need to know given each schema modification type
 * requires unique handling in the crr layer.
 *
 * The provided query must be a normalized query.
 * 
 * We don't need a full fledge parser yet
 * so we're just scanning through the nomralized query.
 */
static int determineQueryType(const char *query, char **err)
{
  int rc = SQLITE_OK;

  // https://www.sqlite.org/lang_createtable.html
  if (strncmp("create temp", query, 12) == 0 || strncmp("create table", query, 13) == 0)
  {
    return CREATE_TABLE;
  }
  // https://www.sqlite.org/lang_createindex.html
  if (strncmp("create unique", query, 14) == 0 || strncmp("create index", query, CREATE_INDEX_LEN) == 0)
  {
    return CREATE_INDEX;
  }
  // https://www.sqlite.org/lang_altertable.html
  if (strncmp("alter table", query, ALTER_TABLE_LEN) == 0)
  {
    return ALTER_TABLE;
  }
  // https://www2.sqlite.org/lang_dropindex.html
  if (strncmp("drop index", query, DROP_INDEX_LEN) == 0)
  {
    return DROP_INDEX;
  }
  // https://www.sqlite.org/lang_droptable.html
  if (strncmp("drop table", query, DROP_TABLE_LEN) == 0)
  {
    return DROP_TABLE;
  }

  *err = sqlite3_mprintf("Unknown schema modification statement provided: %s", query);

  return SQLITE_MISUSE;
}

cfsql_QueryInfo *queryInfoForCreateTable(char *normalized, char **err)
{
  cfsql_QueryInfo *ret = newQueryInfo();

  // chunk into tokens
  char *newStart = normalized + 7;

  if (strncmp(newStart, "temporary", 9) == 0) {
    ret->isTemp = 1;
    newStart += 10;
  } else if (strncmp(newStart, "temp", 4) == 0) {
    ret->isTemp = 1;
    newStart += 5;
  }

  // skip past "table "
  newStart += 6;

  if (strncmp(newStart, "if", 2) == 0) {
    ret->ifExists = 1;
  }

  // skip past "if not exist "
  newStart += 14;

  if (cfsql_isIdentifierOpenQuote(*newStart)) {

  } else {
    // scan till word
  }

  ret->type = CREATE_TABLE;
  ret->reformedQuery = normalized;
  return ret;
}

cfsql_QueryInfo *queryInfoForDropTable(char *normalized, char **err)
{
  cfsql_QueryInfo *ret = 0;
}

cfsql_QueryInfo *queryInfoForAlterTable(char *normalized, char **err)
{
  cfsql_QueryInfo *ret = 0;
}

cfsql_QueryInfo *queryInfoForCreateIndex(char *normalized, char **err)
{
  cfsql_QueryInfo *ret = 0;
}

cfsql_QueryInfo *queryInfoForDropIndex(char *normalized, char **err)
{
  cfsql_QueryInfo *ret = 0;
}

cfsql_QueryInfo *cfsql_queryInfo(const char *query, char **err)
{
  // 1. determine query type
  // 2. find table name start
  // 3. determine quoting
  // 4. scan till end of quoting
  //    this scans until we hit an unescaped quote character.

  char *normalized = cfsql_normalize(query);
  cfsql_QueryInfo *ret = 0;

  if (normalized == 0)
  {
    *err = strdup("Failed to normalized the provided query");
    return 0;
  }

  cfsql_QueryType queryType = determineQueryType(normalized, err);
  if (queryType == SQLITE_MISUSE)
  {
    sqlite3_free(normalized);
    return 0;
  }

  switch (queryType)
  {
  case CREATE_TABLE:
    ret = queryInfoForCreateTable(normalized, err);
  case DROP_TABLE:
    ret = queryInfoForDropTable(normalized, err);
  case ALTER_TABLE:
    ret = queryInfoForAlterTable(normalized, err);
  case CREATE_INDEX:
    ret = queryInfoForCreateIndex(normalized, err);
  case DROP_INDEX:
    ret = queryInfoForDropIndex(normalized, err);
  default:
    assert("impossible");
  }

  return ret;
}

void cfsql_freeQueryInfo(cfsql_QueryInfo *queryInfo)
{
  sqlite3_free(queryInfo->reformedQuery);
  sqlite3_free(queryInfo->schemaName);
  sqlite3_free(queryInfo->tblName);
  sqlite3_free(queryInfo);
}

static newQueryInfo()
{
  cfsql_QueryInfo *ret = malloc(sizeof *ret);

  ret->ifExists = 0;
  ret->ifNotExists = 0;
  ret->reformedQuery = 0;
  ret->schemaName = 0;
  ret->tblName = 0;
  ret->type = 0;
  ret->isTemp = 0;

  return ret;
}