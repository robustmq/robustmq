import * as jspb from 'google-protobuf'



export class ListShardRequest extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): ListShardRequest;

  getShardName(): string;
  setShardName(value: string): ListShardRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListShardRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListShardRequest): ListShardRequest.AsObject;
  static serializeBinaryToWriter(message: ListShardRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListShardRequest;
  static deserializeBinaryFromReader(message: ListShardRequest, reader: jspb.BinaryReader): ListShardRequest;
}

export namespace ListShardRequest {
  export type AsObject = {
    namespace: string,
    shardName: string,
  }
}

export class ListShardReply extends jspb.Message {
  getShardsList(): Array<string>;
  setShardsList(value: Array<string>): ListShardReply;
  clearShardsList(): ListShardReply;
  addShards(value: string, index?: number): ListShardReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListShardReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListShardReply): ListShardReply.AsObject;
  static serializeBinaryToWriter(message: ListShardReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListShardReply;
  static deserializeBinaryFromReader(message: ListShardReply, reader: jspb.BinaryReader): ListShardReply;
}

export namespace ListShardReply {
  export type AsObject = {
    shardsList: Array<string>,
  }
}

export class ListSegmentRequest extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): ListSegmentRequest;

  getShardName(): string;
  setShardName(value: string): ListSegmentRequest;

  getSegmentNo(): number;
  setSegmentNo(value: number): ListSegmentRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSegmentRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListSegmentRequest): ListSegmentRequest.AsObject;
  static serializeBinaryToWriter(message: ListSegmentRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSegmentRequest;
  static deserializeBinaryFromReader(message: ListSegmentRequest, reader: jspb.BinaryReader): ListSegmentRequest;
}

export namespace ListSegmentRequest {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segmentNo: number,
  }
}

export class ListSegmentReply extends jspb.Message {
  getSegmentsList(): Array<string>;
  setSegmentsList(value: Array<string>): ListSegmentReply;
  clearSegmentsList(): ListSegmentReply;
  addSegments(value: string, index?: number): ListSegmentReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSegmentReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListSegmentReply): ListSegmentReply.AsObject;
  static serializeBinaryToWriter(message: ListSegmentReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSegmentReply;
  static deserializeBinaryFromReader(message: ListSegmentReply, reader: jspb.BinaryReader): ListSegmentReply;
}

export namespace ListSegmentReply {
  export type AsObject = {
    segmentsList: Array<string>,
  }
}

