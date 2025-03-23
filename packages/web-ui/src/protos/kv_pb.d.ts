import * as jspb from 'google-protobuf'



export class SetRequest extends jspb.Message {
  getKey(): string;
  setKey(value: string): SetRequest;

  getValue(): string;
  setValue(value: string): SetRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetRequest): SetRequest.AsObject;
  static serializeBinaryToWriter(message: SetRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetRequest;
  static deserializeBinaryFromReader(message: SetRequest, reader: jspb.BinaryReader): SetRequest;
}

export namespace SetRequest {
  export type AsObject = {
    key: string,
    value: string,
  }
}

export class SetReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetReply): SetReply.AsObject;
  static serializeBinaryToWriter(message: SetReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetReply;
  static deserializeBinaryFromReader(message: SetReply, reader: jspb.BinaryReader): SetReply;
}

export namespace SetReply {
  export type AsObject = {
  }
}

export class GetRequest extends jspb.Message {
  getKey(): string;
  setKey(value: string): GetRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetRequest): GetRequest.AsObject;
  static serializeBinaryToWriter(message: GetRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetRequest;
  static deserializeBinaryFromReader(message: GetRequest, reader: jspb.BinaryReader): GetRequest;
}

export namespace GetRequest {
  export type AsObject = {
    key: string,
  }
}

export class GetReply extends jspb.Message {
  getValue(): string;
  setValue(value: string): GetReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetReply): GetReply.AsObject;
  static serializeBinaryToWriter(message: GetReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetReply;
  static deserializeBinaryFromReader(message: GetReply, reader: jspb.BinaryReader): GetReply;
}

export namespace GetReply {
  export type AsObject = {
    value: string,
  }
}

export class DeleteRequest extends jspb.Message {
  getKey(): string;
  setKey(value: string): DeleteRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteRequest): DeleteRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteRequest;
  static deserializeBinaryFromReader(message: DeleteRequest, reader: jspb.BinaryReader): DeleteRequest;
}

export namespace DeleteRequest {
  export type AsObject = {
    key: string,
  }
}

export class DeleteReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteReply): DeleteReply.AsObject;
  static serializeBinaryToWriter(message: DeleteReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteReply;
  static deserializeBinaryFromReader(message: DeleteReply, reader: jspb.BinaryReader): DeleteReply;
}

export namespace DeleteReply {
  export type AsObject = {
  }
}

export class ExistsRequest extends jspb.Message {
  getKey(): string;
  setKey(value: string): ExistsRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ExistsRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ExistsRequest): ExistsRequest.AsObject;
  static serializeBinaryToWriter(message: ExistsRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ExistsRequest;
  static deserializeBinaryFromReader(message: ExistsRequest, reader: jspb.BinaryReader): ExistsRequest;
}

export namespace ExistsRequest {
  export type AsObject = {
    key: string,
  }
}

export class ExistsReply extends jspb.Message {
  getFlag(): boolean;
  setFlag(value: boolean): ExistsReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ExistsReply.AsObject;
  static toObject(includeInstance: boolean, msg: ExistsReply): ExistsReply.AsObject;
  static serializeBinaryToWriter(message: ExistsReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ExistsReply;
  static deserializeBinaryFromReader(message: ExistsReply, reader: jspb.BinaryReader): ExistsReply;
}

export namespace ExistsReply {
  export type AsObject = {
    flag: boolean,
  }
}

export class ListShardRequest extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): ListShardRequest;

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
  }
}

export class ListShardReply extends jspb.Message {
  getShardsInfoList(): Array<string>;
  setShardsInfoList(value: Array<string>): ListShardReply;
  clearShardsInfoList(): ListShardReply;
  addShardsInfo(value: string, index?: number): ListShardReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListShardReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListShardReply): ListShardReply.AsObject;
  static serializeBinaryToWriter(message: ListShardReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListShardReply;
  static deserializeBinaryFromReader(message: ListShardReply, reader: jspb.BinaryReader): ListShardReply;
}

export namespace ListShardReply {
  export type AsObject = {
    shardsInfoList: Array<string>,
  }
}

export class GetPrefixRequest extends jspb.Message {
  getPrefix(): string;
  setPrefix(value: string): GetPrefixRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetPrefixRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetPrefixRequest): GetPrefixRequest.AsObject;
  static serializeBinaryToWriter(message: GetPrefixRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetPrefixRequest;
  static deserializeBinaryFromReader(message: GetPrefixRequest, reader: jspb.BinaryReader): GetPrefixRequest;
}

export namespace GetPrefixRequest {
  export type AsObject = {
    prefix: string,
  }
}

export class GetPrefixReply extends jspb.Message {
  getValuesList(): Array<string>;
  setValuesList(value: Array<string>): GetPrefixReply;
  clearValuesList(): GetPrefixReply;
  addValues(value: string, index?: number): GetPrefixReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetPrefixReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetPrefixReply): GetPrefixReply.AsObject;
  static serializeBinaryToWriter(message: GetPrefixReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetPrefixReply;
  static deserializeBinaryFromReader(message: GetPrefixReply, reader: jspb.BinaryReader): GetPrefixReply;
}

export namespace GetPrefixReply {
  export type AsObject = {
    valuesList: Array<string>,
  }
}

