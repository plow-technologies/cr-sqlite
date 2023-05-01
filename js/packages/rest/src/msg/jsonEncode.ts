// bigint to string conversions
// given a msg types, encodes it to a json string.
// - converts uint8array to hex string
// - converts bigint to string

import { Change, Msg, Tag, tags } from "../Types.js";
import util from "../private/util.js";

export default function encode(msg: Msg): string {
  // implement encode
  switch (msg._tag) {
    case tags.applyChanges:
      return JSON.stringify({
        _tag: tags.applyChanges,
        toDbid: util.bytesToHex(msg.toDbid),
        fromDbid: util.bytesToHex(msg.fromDbid),
        schemaName: msg.schemaVersion,
        seqStart: [msg.seqStart[0].toString(), msg.seqStart[1]],
        seqEnd: [msg.seqEnd[0].toString(), msg.seqEnd[1]],
        changes: encodeChanges(msg.changes),
      });
    case tags.getChanges:
      return JSON.stringify({
        _tag: tags.getChanges,
        dbid: util.bytesToHex(msg.dbid),
        requestorDbid: util.bytesToHex(msg.requestorDbid),
        schemaName: msg.schemaVersion,
        since: [msg.since[0].toString(), msg.since[1]],
      });
    case tags.establishOutboundStream:
      return JSON.stringify({
        _tag: tags.establishOutboundStream,
        localDbid: util.bytesToHex(msg.localDbid),
        remoteDbid: util.bytesToHex(msg.remoteDbid),
        schemaName: msg.schemaVersion,
        seqStart: [msg.seqStart[0].toString(), msg.seqStart[1]],
      });
    case tags.getLastSeen:
      return JSON.stringify({
        _tag: tags.getLastSeen,
        fromDbid: util.bytesToHex(msg.fromDbid),
      });
    case tags.getLastSeenResponse:
      return JSON.stringify({
        _tag: tags.getLastSeenResponse,
        seq: [msg.seq[0].toString(), msg.seq[1]],
      });
    case tags.getChangesResponse:
      return JSON.stringify({
        _tag: tags.getChangesResponse,
        seqStart: [msg.seqStart[0].toString(), msg.seqStart[1]],
        seqEnd: [msg.seqEnd[0].toString(), msg.seqEnd[1]],
        changes: encodeChanges(msg.changes),
      });
    case tags.createOrMigrate:
      return JSON.stringify({
        _tag: tags.createOrMigrate,
        dbid: util.bytesToHex(msg.dbid),
        schemaName: msg.schemaName,
        schemaVersion: msg.schemaVersion,
      });
    case tags.createOrMigrateResponse:
      return JSON.stringify({
        _tag: tags.createOrMigrateResponse,
        status: msg.status,
      });
    case tags.ackChanges:
      return JSON.stringify({
        _tag: tags.ackChanges,
        seqEnd: [msg.seqEnd[0].toString(), msg.seqEnd[1]],
      });
    case tags.streamingChanges:
      return JSON.stringify({
        _tag: tags.streamingChanges,
        seqStart: [msg.seqStart[0].toString(), msg.seqStart[1]],
        seqEnd: [msg.seqEnd[0].toString(), msg.seqEnd[1]],
        changes: encodeChanges(msg.changes),
      });
    case tags.applyChangesResponse:
      return JSON.stringify({
        _tag: tags.applyChangesResponse,
        status: msg.status,
        seqEnd:
          msg.seqEnd != null
            ? [msg.seqEnd[0].toString(), msg.seqEnd[1]]
            : undefined,
        msg: msg.msg,
      });
  }
}

function encodeChanges(changes: readonly Change[]): readonly any[] {
  return changes.map((c) => {
    return [c[0], c[1], c[2], c[3], c[4].toString(), c[5].toString()];
  });
}
