import * as jspb from 'google-protobuf'



export class JournalRecord extends jspb.Message {
  getProducerId(): string;
  setProducerId(value: string): JournalRecord;

  getPkid(): number;
  setPkid(value: number): JournalRecord;

  getKey(): string;
  setKey(value: string): JournalRecord;

  getContent(): Uint8Array | string;
  getContent_asU8(): Uint8Array;
  getContent_asB64(): string;
  setContent(value: Uint8Array | string): JournalRecord;

  getCreateTime(): number;
  setCreateTime(value: number): JournalRecord;

  getTagsList(): Array<string>;
  setTagsList(value: Array<string>): JournalRecord;
  clearTagsList(): JournalRecord;
  addTags(value: string, index?: number): JournalRecord;

  getOffset(): number;
  setOffset(value: number): JournalRecord;

  getNamespace(): string;
  setNamespace(value: string): JournalRecord;

  getShardName(): string;
  setShardName(value: string): JournalRecord;

  getSegment(): number;
  setSegment(value: number): JournalRecord;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): JournalRecord.AsObject;
  static toObject(includeInstance: boolean, msg: JournalRecord): JournalRecord.AsObject;
  static serializeBinaryToWriter(message: JournalRecord, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): JournalRecord;
  static deserializeBinaryFromReader(message: JournalRecord, reader: jspb.BinaryReader): JournalRecord;
}

export namespace JournalRecord {
  export type AsObject = {
    producerId: string,
    pkid: number,
    key: string,
    content: Uint8Array | string,
    createTime: number,
    tagsList: Array<string>,
    offset: number,
    namespace: string,
    shardName: string,
    segment: number,
  }
}

