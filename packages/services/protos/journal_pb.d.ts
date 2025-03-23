import * as jspb from 'google-protobuf'



export class ListShardRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListShardRequest;

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
    clusterName: string,
    namespace: string,
    shardName: string,
  }
}

export class ListShardReply extends jspb.Message {
  getShards(): Uint8Array | string;
  getShards_asU8(): Uint8Array;
  getShards_asB64(): string;
  setShards(value: Uint8Array | string): ListShardReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListShardReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListShardReply): ListShardReply.AsObject;
  static serializeBinaryToWriter(message: ListShardReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListShardReply;
  static deserializeBinaryFromReader(message: ListShardReply, reader: jspb.BinaryReader): ListShardReply;
}

export namespace ListShardReply {
  export type AsObject = {
    shards: Uint8Array | string,
  }
}

export class CreateShardRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateShardRequest;

  getNamespace(): string;
  setNamespace(value: string): CreateShardRequest;

  getShardName(): string;
  setShardName(value: string): CreateShardRequest;

  getShardConfig(): Uint8Array | string;
  getShardConfig_asU8(): Uint8Array;
  getShardConfig_asB64(): string;
  setShardConfig(value: Uint8Array | string): CreateShardRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateShardRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateShardRequest): CreateShardRequest.AsObject;
  static serializeBinaryToWriter(message: CreateShardRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateShardRequest;
  static deserializeBinaryFromReader(message: CreateShardRequest, reader: jspb.BinaryReader): CreateShardRequest;
}

export namespace CreateShardRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
    shardConfig: Uint8Array | string,
  }
}

export class CreateShardReply extends jspb.Message {
  getSegmentNo(): number;
  setSegmentNo(value: number): CreateShardReply;

  getReplicaList(): Array<number>;
  setReplicaList(value: Array<number>): CreateShardReply;
  clearReplicaList(): CreateShardReply;
  addReplica(value: number, index?: number): CreateShardReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateShardReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateShardReply): CreateShardReply.AsObject;
  static serializeBinaryToWriter(message: CreateShardReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateShardReply;
  static deserializeBinaryFromReader(message: CreateShardReply, reader: jspb.BinaryReader): CreateShardReply;
}

export namespace CreateShardReply {
  export type AsObject = {
    segmentNo: number,
    replicaList: Array<number>,
  }
}

export class DeleteShardRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteShardRequest;

  getNamespace(): string;
  setNamespace(value: string): DeleteShardRequest;

  getShardName(): string;
  setShardName(value: string): DeleteShardRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteShardRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteShardRequest): DeleteShardRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteShardRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteShardRequest;
  static deserializeBinaryFromReader(message: DeleteShardRequest, reader: jspb.BinaryReader): DeleteShardRequest;
}

export namespace DeleteShardRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
  }
}

export class DeleteShardReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteShardReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteShardReply): DeleteShardReply.AsObject;
  static serializeBinaryToWriter(message: DeleteShardReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteShardReply;
  static deserializeBinaryFromReader(message: DeleteShardReply, reader: jspb.BinaryReader): DeleteShardReply;
}

export namespace DeleteShardReply {
  export type AsObject = {
  }
}

export class ListSegmentRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListSegmentRequest;

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
    clusterName: string,
    namespace: string,
    shardName: string,
    segmentNo: number,
  }
}

export class ListSegmentReply extends jspb.Message {
  getSegments(): Uint8Array | string;
  getSegments_asU8(): Uint8Array;
  getSegments_asB64(): string;
  setSegments(value: Uint8Array | string): ListSegmentReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSegmentReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListSegmentReply): ListSegmentReply.AsObject;
  static serializeBinaryToWriter(message: ListSegmentReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSegmentReply;
  static deserializeBinaryFromReader(message: ListSegmentReply, reader: jspb.BinaryReader): ListSegmentReply;
}

export namespace ListSegmentReply {
  export type AsObject = {
    segments: Uint8Array | string,
  }
}

export class CreateNextSegmentRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateNextSegmentRequest;

  getNamespace(): string;
  setNamespace(value: string): CreateNextSegmentRequest;

  getShardName(): string;
  setShardName(value: string): CreateNextSegmentRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateNextSegmentRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateNextSegmentRequest): CreateNextSegmentRequest.AsObject;
  static serializeBinaryToWriter(message: CreateNextSegmentRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateNextSegmentRequest;
  static deserializeBinaryFromReader(message: CreateNextSegmentRequest, reader: jspb.BinaryReader): CreateNextSegmentRequest;
}

export namespace CreateNextSegmentRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
  }
}

export class CreateNextSegmentReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateNextSegmentReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateNextSegmentReply): CreateNextSegmentReply.AsObject;
  static serializeBinaryToWriter(message: CreateNextSegmentReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateNextSegmentReply;
  static deserializeBinaryFromReader(message: CreateNextSegmentReply, reader: jspb.BinaryReader): CreateNextSegmentReply;
}

export namespace CreateNextSegmentReply {
  export type AsObject = {
  }
}

export class DeleteSegmentRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteSegmentRequest;

  getNamespace(): string;
  setNamespace(value: string): DeleteSegmentRequest;

  getShardName(): string;
  setShardName(value: string): DeleteSegmentRequest;

  getSegmentSeq(): number;
  setSegmentSeq(value: number): DeleteSegmentRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSegmentRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSegmentRequest): DeleteSegmentRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteSegmentRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSegmentRequest;
  static deserializeBinaryFromReader(message: DeleteSegmentRequest, reader: jspb.BinaryReader): DeleteSegmentRequest;
}

export namespace DeleteSegmentRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
    segmentSeq: number,
  }
}

export class DeleteSegmentReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSegmentReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSegmentReply): DeleteSegmentReply.AsObject;
  static serializeBinaryToWriter(message: DeleteSegmentReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSegmentReply;
  static deserializeBinaryFromReader(message: DeleteSegmentReply, reader: jspb.BinaryReader): DeleteSegmentReply;
}

export namespace DeleteSegmentReply {
  export type AsObject = {
  }
}

export class UpdateSegmentStatusRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): UpdateSegmentStatusRequest;

  getNamespace(): string;
  setNamespace(value: string): UpdateSegmentStatusRequest;

  getShardName(): string;
  setShardName(value: string): UpdateSegmentStatusRequest;

  getSegmentSeq(): number;
  setSegmentSeq(value: number): UpdateSegmentStatusRequest;

  getCurStatus(): string;
  setCurStatus(value: string): UpdateSegmentStatusRequest;

  getNextStatus(): string;
  setNextStatus(value: string): UpdateSegmentStatusRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSegmentStatusRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSegmentStatusRequest): UpdateSegmentStatusRequest.AsObject;
  static serializeBinaryToWriter(message: UpdateSegmentStatusRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSegmentStatusRequest;
  static deserializeBinaryFromReader(message: UpdateSegmentStatusRequest, reader: jspb.BinaryReader): UpdateSegmentStatusRequest;
}

export namespace UpdateSegmentStatusRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
    segmentSeq: number,
    curStatus: string,
    nextStatus: string,
  }
}

export class UpdateSegmentStatusReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSegmentStatusReply.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSegmentStatusReply): UpdateSegmentStatusReply.AsObject;
  static serializeBinaryToWriter(message: UpdateSegmentStatusReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSegmentStatusReply;
  static deserializeBinaryFromReader(message: UpdateSegmentStatusReply, reader: jspb.BinaryReader): UpdateSegmentStatusReply;
}

export namespace UpdateSegmentStatusReply {
  export type AsObject = {
  }
}

export class ListSegmentMetaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListSegmentMetaRequest;

  getNamespace(): string;
  setNamespace(value: string): ListSegmentMetaRequest;

  getShardName(): string;
  setShardName(value: string): ListSegmentMetaRequest;

  getSegmentNo(): number;
  setSegmentNo(value: number): ListSegmentMetaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSegmentMetaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListSegmentMetaRequest): ListSegmentMetaRequest.AsObject;
  static serializeBinaryToWriter(message: ListSegmentMetaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSegmentMetaRequest;
  static deserializeBinaryFromReader(message: ListSegmentMetaRequest, reader: jspb.BinaryReader): ListSegmentMetaRequest;
}

export namespace ListSegmentMetaRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
    segmentNo: number,
  }
}

export class ListSegmentMetaReply extends jspb.Message {
  getSegments(): Uint8Array | string;
  getSegments_asU8(): Uint8Array;
  getSegments_asB64(): string;
  setSegments(value: Uint8Array | string): ListSegmentMetaReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSegmentMetaReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListSegmentMetaReply): ListSegmentMetaReply.AsObject;
  static serializeBinaryToWriter(message: ListSegmentMetaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSegmentMetaReply;
  static deserializeBinaryFromReader(message: ListSegmentMetaReply, reader: jspb.BinaryReader): ListSegmentMetaReply;
}

export namespace ListSegmentMetaReply {
  export type AsObject = {
    segments: Uint8Array | string,
  }
}

export class UpdateSegmentMetaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): UpdateSegmentMetaRequest;

  getNamespace(): string;
  setNamespace(value: string): UpdateSegmentMetaRequest;

  getShardName(): string;
  setShardName(value: string): UpdateSegmentMetaRequest;

  getSegmentNo(): number;
  setSegmentNo(value: number): UpdateSegmentMetaRequest;

  getStartOffset(): number;
  setStartOffset(value: number): UpdateSegmentMetaRequest;

  getEndOffset(): number;
  setEndOffset(value: number): UpdateSegmentMetaRequest;

  getStartTimestamp(): number;
  setStartTimestamp(value: number): UpdateSegmentMetaRequest;

  getEndTimestamp(): number;
  setEndTimestamp(value: number): UpdateSegmentMetaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSegmentMetaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSegmentMetaRequest): UpdateSegmentMetaRequest.AsObject;
  static serializeBinaryToWriter(message: UpdateSegmentMetaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSegmentMetaRequest;
  static deserializeBinaryFromReader(message: UpdateSegmentMetaRequest, reader: jspb.BinaryReader): UpdateSegmentMetaRequest;
}

export namespace UpdateSegmentMetaRequest {
  export type AsObject = {
    clusterName: string,
    namespace: string,
    shardName: string,
    segmentNo: number,
    startOffset: number,
    endOffset: number,
    startTimestamp: number,
    endTimestamp: number,
  }
}

export class UpdateSegmentMetaReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSegmentMetaReply.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSegmentMetaReply): UpdateSegmentMetaReply.AsObject;
  static serializeBinaryToWriter(message: UpdateSegmentMetaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSegmentMetaReply;
  static deserializeBinaryFromReader(message: UpdateSegmentMetaReply, reader: jspb.BinaryReader): UpdateSegmentMetaReply;
}

export namespace UpdateSegmentMetaReply {
  export type AsObject = {
  }
}

