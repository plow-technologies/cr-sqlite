#include "changes-vtab-common.h"
#include "consts.h"
#include <string.h>
#include "util.h"

char *crsql_extractWhereList(
    crsql_ColumnInfo *zColumnInfos,
    int columnInfosLen,
    const char *quoteConcatedVals)
{
  char **zzParts = 0;
  if (columnInfosLen == 1)
  {
    zzParts = sqlite3_malloc(1 * sizeof(char *));
    zzParts[0] = strdup(quoteConcatedVals);
  }
  else
  {
    // zzParts will not be greater or less than columnInfosLen.
    // TODO: we need something other than `crsql_split` here. `crsql_quoteConcatSplit`
    zzParts = crsql_split(quoteConcatedVals, PK_DELIM_DEPRECATED, columnInfosLen);
  }

  if (zzParts == 0)
  {
    return 0;
  }

  for (int i = 0; i < columnInfosLen; ++i)
  {
    // this is safe since pks are extracted as `quote` in the prior queries
    // %z will de-allocate pksArr[i] so we can re-allocate it in the assignment
    // TODO: we currently invoke this in a non safe case
    // where pksArr is receive from a network socket rather than the
    // local db. In the apply patches path.
    zzParts[i] = sqlite3_mprintf("\"%s\" = %z", zColumnInfos[i].name, zzParts[i]);
  }

  // join2 will free the contents of zzParts given identity is a pass-thru
  char *ret = crsql_join2((char *(*)(const char *)) & crsql_identity, zzParts, columnInfosLen, " AND ");
  sqlite3_free(zzParts);
  return ret;
}