import * as jspb from 'google-protobuf'



export class JournalEngineError extends jspb.Message {
  getCode(): string;
  setCode(value: string): JournalEngineError;

  getError(): string;
  setError(value: string): JournalEngineError;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): JournalEngineError.AsObject;
  static toObject(includeInstance: boolean, msg: JournalEngineError): JournalEngineError.AsObject;
  static serializeBinaryToWriter(message: JournalEngineError, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): JournalEngineError;
  static deserializeBinaryFromReader(message: JournalEngineError, reader: jspb.BinaryReader): JournalEngineError;
}

export namespace JournalEngineError {
  export type AsObject = {
    code: string,
    error: string,
  }
}

export class ReqHeader extends jspb.Message {
  getApiKey(): ApiKey;
  setApiKey(value: ApiKey): ReqHeader;

  getApiVersion(): ApiVersion;
  setApiVersion(value: ApiVersion): ReqHeader;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReqHeader.AsObject;
  static toObject(includeInstance: boolean, msg: ReqHeader): ReqHeader.AsObject;
  static serializeBinaryToWriter(message: ReqHeader, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReqHeader;
  static deserializeBinaryFromReader(message: ReqHeader, reader: jspb.BinaryReader): ReqHeader;
}

export namespace ReqHeader {
  export type AsObject = {
    apiKey: ApiKey,
    apiVersion: ApiVersion,
  }
}

export class RespHeader extends jspb.Message {
  getApiKey(): ApiKey;
  setApiKey(value: ApiKey): RespHeader;

  getApiVersion(): ApiVersion;
  setApiVersion(value: ApiVersion): RespHeader;

  getError(): JournalEngineError | undefined;
  setError(value?: JournalEngineError): RespHeader;
  hasError(): boolean;
  clearError(): RespHeader;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RespHeader.AsObject;
  static toObject(includeInstance: boolean, msg: RespHeader): RespHeader.AsObject;
  static serializeBinaryToWriter(message: RespHeader, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RespHeader;
  static deserializeBinaryFromReader(message: RespHeader, reader: jspb.BinaryReader): RespHeader;
}

export namespace RespHeader {
  export type AsObject = {
    apiKey: ApiKey,
    apiVersion: ApiVersion,
    error?: JournalEngineError.AsObject,
  }
}

export class ClientSegmentMetadata extends jspb.Message {
  getSegmentNo(): number;
  setSegmentNo(value: number): ClientSegmentMetadata;

  getLeader(): number;
  setLeader(value: number): ClientSegmentMetadata;

  getReplicasList(): Array<number>;
  setReplicasList(value: Array<number>): ClientSegmentMetadata;
  clearReplicasList(): ClientSegmentMetadata;
  addReplicas(value: number, index?: number): ClientSegmentMetadata;

  getStartOffset(): number;
  setStartOffset(value: number): ClientSegmentMetadata;

  getEndOffset(): number;
  setEndOffset(value: number): ClientSegmentMetadata;

  getStartTimestamp(): number;
  setStartTimestamp(value: number): ClientSegmentMetadata;

  getEndTimestamp(): number;
  setEndTimestamp(value: number): ClientSegmentMetadata;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ClientSegmentMetadata.AsObject;
  static toObject(includeInstance: boolean, msg: ClientSegmentMetadata): ClientSegmentMetadata.AsObject;
  static serializeBinaryToWriter(message: ClientSegmentMetadata, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ClientSegmentMetadata;
  static deserializeBinaryFromReader(message: ClientSegmentMetadata, reader: jspb.BinaryReader): ClientSegmentMetadata;
}

export namespace ClientSegmentMetadata {
  export type AsObject = {
    segmentNo: number,
    leader: number,
    replicasList: Array<number>,
    startOffset: number,
    endOffset: number,
    startTimestamp: number,
    endTimestamp: number,
  }
}

export class GetClusterMetadataReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): GetClusterMetadataReq;
  hasHeader(): boolean;
  clearHeader(): GetClusterMetadataReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetClusterMetadataReq.AsObject;
  static toObject(includeInstance: boolean, msg: GetClusterMetadataReq): GetClusterMetadataReq.AsObject;
  static serializeBinaryToWriter(message: GetClusterMetadataReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetClusterMetadataReq;
  static deserializeBinaryFromReader(message: GetClusterMetadataReq, reader: jspb.BinaryReader): GetClusterMetadataReq;
}

export namespace GetClusterMetadataReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
  }
}

export class GetClusterMetadataRespBody extends jspb.Message {
  getNodesList(): Array<GetClusterMetadataNode>;
  setNodesList(value: Array<GetClusterMetadataNode>): GetClusterMetadataRespBody;
  clearNodesList(): GetClusterMetadataRespBody;
  addNodes(value?: GetClusterMetadataNode, index?: number): GetClusterMetadataNode;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetClusterMetadataRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: GetClusterMetadataRespBody): GetClusterMetadataRespBody.AsObject;
  static serializeBinaryToWriter(message: GetClusterMetadataRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetClusterMetadataRespBody;
  static deserializeBinaryFromReader(message: GetClusterMetadataRespBody, reader: jspb.BinaryReader): GetClusterMetadataRespBody;
}

export namespace GetClusterMetadataRespBody {
  export type AsObject = {
    nodesList: Array<GetClusterMetadataNode.AsObject>,
  }
}

export class GetClusterMetadataNode extends jspb.Message {
  getNodeId(): number;
  setNodeId(value: number): GetClusterMetadataNode;

  getTcpAddr(): string;
  setTcpAddr(value: string): GetClusterMetadataNode;

  getTcpsAddr(): string;
  setTcpsAddr(value: string): GetClusterMetadataNode;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetClusterMetadataNode.AsObject;
  static toObject(includeInstance: boolean, msg: GetClusterMetadataNode): GetClusterMetadataNode.AsObject;
  static serializeBinaryToWriter(message: GetClusterMetadataNode, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetClusterMetadataNode;
  static deserializeBinaryFromReader(message: GetClusterMetadataNode, reader: jspb.BinaryReader): GetClusterMetadataNode;
}

export namespace GetClusterMetadataNode {
  export type AsObject = {
    nodeId: number,
    tcpAddr: string,
    tcpsAddr: string,
  }
}

export class GetClusterMetadataResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): GetClusterMetadataResp;
  hasHeader(): boolean;
  clearHeader(): GetClusterMetadataResp;

  getBody(): GetClusterMetadataRespBody | undefined;
  setBody(value?: GetClusterMetadataRespBody): GetClusterMetadataResp;
  hasBody(): boolean;
  clearBody(): GetClusterMetadataResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetClusterMetadataResp.AsObject;
  static toObject(includeInstance: boolean, msg: GetClusterMetadataResp): GetClusterMetadataResp.AsObject;
  static serializeBinaryToWriter(message: GetClusterMetadataResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetClusterMetadataResp;
  static deserializeBinaryFromReader(message: GetClusterMetadataResp, reader: jspb.BinaryReader): GetClusterMetadataResp;
}

export namespace GetClusterMetadataResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: GetClusterMetadataRespBody.AsObject,
  }
}

export class CreateShardReqBody extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): CreateShardReqBody;

  getShardName(): string;
  setShardName(value: string): CreateShardReqBody;

  getReplicaNum(): number;
  setReplicaNum(value: number): CreateShardReqBody;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateShardReqBody.AsObject;
  static toObject(includeInstance: boolean, msg: CreateShardReqBody): CreateShardReqBody.AsObject;
  static serializeBinaryToWriter(message: CreateShardReqBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateShardReqBody;
  static deserializeBinaryFromReader(message: CreateShardReqBody, reader: jspb.BinaryReader): CreateShardReqBody;
}

export namespace CreateShardReqBody {
  export type AsObject = {
    namespace: string,
    shardName: string,
    replicaNum: number,
  }
}

export class CreateShardRespBody extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateShardRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: CreateShardRespBody): CreateShardRespBody.AsObject;
  static serializeBinaryToWriter(message: CreateShardRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateShardRespBody;
  static deserializeBinaryFromReader(message: CreateShardRespBody, reader: jspb.BinaryReader): CreateShardRespBody;
}

export namespace CreateShardRespBody {
  export type AsObject = {
  }
}

export class CreateShardReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): CreateShardReq;
  hasHeader(): boolean;
  clearHeader(): CreateShardReq;

  getBody(): CreateShardReqBody | undefined;
  setBody(value?: CreateShardReqBody): CreateShardReq;
  hasBody(): boolean;
  clearBody(): CreateShardReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateShardReq.AsObject;
  static toObject(includeInstance: boolean, msg: CreateShardReq): CreateShardReq.AsObject;
  static serializeBinaryToWriter(message: CreateShardReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateShardReq;
  static deserializeBinaryFromReader(message: CreateShardReq, reader: jspb.BinaryReader): CreateShardReq;
}

export namespace CreateShardReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
    body?: CreateShardReqBody.AsObject,
  }
}

export class CreateShardResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): CreateShardResp;
  hasHeader(): boolean;
  clearHeader(): CreateShardResp;

  getBody(): CreateShardRespBody | undefined;
  setBody(value?: CreateShardRespBody): CreateShardResp;
  hasBody(): boolean;
  clearBody(): CreateShardResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateShardResp.AsObject;
  static toObject(includeInstance: boolean, msg: CreateShardResp): CreateShardResp.AsObject;
  static serializeBinaryToWriter(message: CreateShardResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateShardResp;
  static deserializeBinaryFromReader(message: CreateShardResp, reader: jspb.BinaryReader): CreateShardResp;
}

export namespace CreateShardResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: CreateShardRespBody.AsObject,
  }
}

export class DeleteShardReqBody extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): DeleteShardReqBody;

  getShardName(): string;
  setShardName(value: string): DeleteShardReqBody;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteShardReqBody.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteShardReqBody): DeleteShardReqBody.AsObject;
  static serializeBinaryToWriter(message: DeleteShardReqBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteShardReqBody;
  static deserializeBinaryFromReader(message: DeleteShardReqBody, reader: jspb.BinaryReader): DeleteShardReqBody;
}

export namespace DeleteShardReqBody {
  export type AsObject = {
    namespace: string,
    shardName: string,
  }
}

export class DeleteShardRespBody extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteShardRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteShardRespBody): DeleteShardRespBody.AsObject;
  static serializeBinaryToWriter(message: DeleteShardRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteShardRespBody;
  static deserializeBinaryFromReader(message: DeleteShardRespBody, reader: jspb.BinaryReader): DeleteShardRespBody;
}

export namespace DeleteShardRespBody {
  export type AsObject = {
  }
}

export class DeleteShardReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): DeleteShardReq;
  hasHeader(): boolean;
  clearHeader(): DeleteShardReq;

  getBody(): DeleteShardReqBody | undefined;
  setBody(value?: DeleteShardReqBody): DeleteShardReq;
  hasBody(): boolean;
  clearBody(): DeleteShardReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteShardReq.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteShardReq): DeleteShardReq.AsObject;
  static serializeBinaryToWriter(message: DeleteShardReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteShardReq;
  static deserializeBinaryFromReader(message: DeleteShardReq, reader: jspb.BinaryReader): DeleteShardReq;
}

export namespace DeleteShardReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
    body?: DeleteShardReqBody.AsObject,
  }
}

export class DeleteShardResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): DeleteShardResp;
  hasHeader(): boolean;
  clearHeader(): DeleteShardResp;

  getBody(): DeleteShardRespBody | undefined;
  setBody(value?: DeleteShardRespBody): DeleteShardResp;
  hasBody(): boolean;
  clearBody(): DeleteShardResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteShardResp.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteShardResp): DeleteShardResp.AsObject;
  static serializeBinaryToWriter(message: DeleteShardResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteShardResp;
  static deserializeBinaryFromReader(message: DeleteShardResp, reader: jspb.BinaryReader): DeleteShardResp;
}

export namespace DeleteShardResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: DeleteShardRespBody.AsObject,
  }
}

export class GetShardMetadataReqBody extends jspb.Message {
  getShardsList(): Array<GetShardMetadataReqShard>;
  setShardsList(value: Array<GetShardMetadataReqShard>): GetShardMetadataReqBody;
  clearShardsList(): GetShardMetadataReqBody;
  addShards(value?: GetShardMetadataReqShard, index?: number): GetShardMetadataReqShard;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShardMetadataReqBody.AsObject;
  static toObject(includeInstance: boolean, msg: GetShardMetadataReqBody): GetShardMetadataReqBody.AsObject;
  static serializeBinaryToWriter(message: GetShardMetadataReqBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShardMetadataReqBody;
  static deserializeBinaryFromReader(message: GetShardMetadataReqBody, reader: jspb.BinaryReader): GetShardMetadataReqBody;
}

export namespace GetShardMetadataReqBody {
  export type AsObject = {
    shardsList: Array<GetShardMetadataReqShard.AsObject>,
  }
}

export class GetShardMetadataReqShard extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): GetShardMetadataReqShard;

  getShardName(): string;
  setShardName(value: string): GetShardMetadataReqShard;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShardMetadataReqShard.AsObject;
  static toObject(includeInstance: boolean, msg: GetShardMetadataReqShard): GetShardMetadataReqShard.AsObject;
  static serializeBinaryToWriter(message: GetShardMetadataReqShard, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShardMetadataReqShard;
  static deserializeBinaryFromReader(message: GetShardMetadataReqShard, reader: jspb.BinaryReader): GetShardMetadataReqShard;
}

export namespace GetShardMetadataReqShard {
  export type AsObject = {
    namespace: string,
    shardName: string,
  }
}

export class GetShardMetadataReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): GetShardMetadataReq;
  hasHeader(): boolean;
  clearHeader(): GetShardMetadataReq;

  getBody(): GetShardMetadataReqBody | undefined;
  setBody(value?: GetShardMetadataReqBody): GetShardMetadataReq;
  hasBody(): boolean;
  clearBody(): GetShardMetadataReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShardMetadataReq.AsObject;
  static toObject(includeInstance: boolean, msg: GetShardMetadataReq): GetShardMetadataReq.AsObject;
  static serializeBinaryToWriter(message: GetShardMetadataReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShardMetadataReq;
  static deserializeBinaryFromReader(message: GetShardMetadataReq, reader: jspb.BinaryReader): GetShardMetadataReq;
}

export namespace GetShardMetadataReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
    body?: GetShardMetadataReqBody.AsObject,
  }
}

export class GetShardMetadataRespBody extends jspb.Message {
  getShardsList(): Array<GetShardMetadataRespShard>;
  setShardsList(value: Array<GetShardMetadataRespShard>): GetShardMetadataRespBody;
  clearShardsList(): GetShardMetadataRespBody;
  addShards(value?: GetShardMetadataRespShard, index?: number): GetShardMetadataRespShard;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShardMetadataRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: GetShardMetadataRespBody): GetShardMetadataRespBody.AsObject;
  static serializeBinaryToWriter(message: GetShardMetadataRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShardMetadataRespBody;
  static deserializeBinaryFromReader(message: GetShardMetadataRespBody, reader: jspb.BinaryReader): GetShardMetadataRespBody;
}

export namespace GetShardMetadataRespBody {
  export type AsObject = {
    shardsList: Array<GetShardMetadataRespShard.AsObject>,
  }
}

export class GetShardMetadataRespShard extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): GetShardMetadataRespShard;

  getShard(): string;
  setShard(value: string): GetShardMetadataRespShard;

  getActiveSegment(): number;
  setActiveSegment(value: number): GetShardMetadataRespShard;

  getActiveSegmentLeader(): number;
  setActiveSegmentLeader(value: number): GetShardMetadataRespShard;

  getSegmentsList(): Array<ClientSegmentMetadata>;
  setSegmentsList(value: Array<ClientSegmentMetadata>): GetShardMetadataRespShard;
  clearSegmentsList(): GetShardMetadataRespShard;
  addSegments(value?: ClientSegmentMetadata, index?: number): ClientSegmentMetadata;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShardMetadataRespShard.AsObject;
  static toObject(includeInstance: boolean, msg: GetShardMetadataRespShard): GetShardMetadataRespShard.AsObject;
  static serializeBinaryToWriter(message: GetShardMetadataRespShard, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShardMetadataRespShard;
  static deserializeBinaryFromReader(message: GetShardMetadataRespShard, reader: jspb.BinaryReader): GetShardMetadataRespShard;
}

export namespace GetShardMetadataRespShard {
  export type AsObject = {
    namespace: string,
    shard: string,
    activeSegment: number,
    activeSegmentLeader: number,
    segmentsList: Array<ClientSegmentMetadata.AsObject>,
  }
}

export class GetShardMetadataResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): GetShardMetadataResp;
  hasHeader(): boolean;
  clearHeader(): GetShardMetadataResp;

  getBody(): GetShardMetadataRespBody | undefined;
  setBody(value?: GetShardMetadataRespBody): GetShardMetadataResp;
  hasBody(): boolean;
  clearBody(): GetShardMetadataResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShardMetadataResp.AsObject;
  static toObject(includeInstance: boolean, msg: GetShardMetadataResp): GetShardMetadataResp.AsObject;
  static serializeBinaryToWriter(message: GetShardMetadataResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShardMetadataResp;
  static deserializeBinaryFromReader(message: GetShardMetadataResp, reader: jspb.BinaryReader): GetShardMetadataResp;
}

export namespace GetShardMetadataResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: GetShardMetadataRespBody.AsObject,
  }
}

export class WriteReqBody extends jspb.Message {
  getDataList(): Array<WriteReqSegmentMessages>;
  setDataList(value: Array<WriteReqSegmentMessages>): WriteReqBody;
  clearDataList(): WriteReqBody;
  addData(value?: WriteReqSegmentMessages, index?: number): WriteReqSegmentMessages;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteReqBody.AsObject;
  static toObject(includeInstance: boolean, msg: WriteReqBody): WriteReqBody.AsObject;
  static serializeBinaryToWriter(message: WriteReqBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteReqBody;
  static deserializeBinaryFromReader(message: WriteReqBody, reader: jspb.BinaryReader): WriteReqBody;
}

export namespace WriteReqBody {
  export type AsObject = {
    dataList: Array<WriteReqSegmentMessages.AsObject>,
  }
}

export class WriteReqSegmentMessages extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): WriteReqSegmentMessages;

  getShardName(): string;
  setShardName(value: string): WriteReqSegmentMessages;

  getSegment(): number;
  setSegment(value: number): WriteReqSegmentMessages;

  getMessagesList(): Array<WriteReqMessages>;
  setMessagesList(value: Array<WriteReqMessages>): WriteReqSegmentMessages;
  clearMessagesList(): WriteReqSegmentMessages;
  addMessages(value?: WriteReqMessages, index?: number): WriteReqMessages;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteReqSegmentMessages.AsObject;
  static toObject(includeInstance: boolean, msg: WriteReqSegmentMessages): WriteReqSegmentMessages.AsObject;
  static serializeBinaryToWriter(message: WriteReqSegmentMessages, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteReqSegmentMessages;
  static deserializeBinaryFromReader(message: WriteReqSegmentMessages, reader: jspb.BinaryReader): WriteReqSegmentMessages;
}

export namespace WriteReqSegmentMessages {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segment: number,
    messagesList: Array<WriteReqMessages.AsObject>,
  }
}

export class WriteReqMessages extends jspb.Message {
  getPkid(): number;
  setPkid(value: number): WriteReqMessages;

  getKey(): string;
  setKey(value: string): WriteReqMessages;

  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): WriteReqMessages;

  getTagsList(): Array<string>;
  setTagsList(value: Array<string>): WriteReqMessages;
  clearTagsList(): WriteReqMessages;
  addTags(value: string, index?: number): WriteReqMessages;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteReqMessages.AsObject;
  static toObject(includeInstance: boolean, msg: WriteReqMessages): WriteReqMessages.AsObject;
  static serializeBinaryToWriter(message: WriteReqMessages, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteReqMessages;
  static deserializeBinaryFromReader(message: WriteReqMessages, reader: jspb.BinaryReader): WriteReqMessages;
}

export namespace WriteReqMessages {
  export type AsObject = {
    pkid: number,
    key: string,
    value: Uint8Array | string,
    tagsList: Array<string>,
  }
}

export class WriteRespBody extends jspb.Message {
  getStatusList(): Array<WriteRespMessage>;
  setStatusList(value: Array<WriteRespMessage>): WriteRespBody;
  clearStatusList(): WriteRespBody;
  addStatus(value?: WriteRespMessage, index?: number): WriteRespMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: WriteRespBody): WriteRespBody.AsObject;
  static serializeBinaryToWriter(message: WriteRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteRespBody;
  static deserializeBinaryFromReader(message: WriteRespBody, reader: jspb.BinaryReader): WriteRespBody;
}

export namespace WriteRespBody {
  export type AsObject = {
    statusList: Array<WriteRespMessage.AsObject>,
  }
}

export class WriteRespMessage extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): WriteRespMessage;

  getShardName(): string;
  setShardName(value: string): WriteRespMessage;

  getSegment(): number;
  setSegment(value: number): WriteRespMessage;

  getMessagesList(): Array<WriteRespMessageStatus>;
  setMessagesList(value: Array<WriteRespMessageStatus>): WriteRespMessage;
  clearMessagesList(): WriteRespMessage;
  addMessages(value?: WriteRespMessageStatus, index?: number): WriteRespMessageStatus;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteRespMessage.AsObject;
  static toObject(includeInstance: boolean, msg: WriteRespMessage): WriteRespMessage.AsObject;
  static serializeBinaryToWriter(message: WriteRespMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteRespMessage;
  static deserializeBinaryFromReader(message: WriteRespMessage, reader: jspb.BinaryReader): WriteRespMessage;
}

export namespace WriteRespMessage {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segment: number,
    messagesList: Array<WriteRespMessageStatus.AsObject>,
  }
}

export class WriteRespMessageStatus extends jspb.Message {
  getOffset(): number;
  setOffset(value: number): WriteRespMessageStatus;

  getPkid(): number;
  setPkid(value: number): WriteRespMessageStatus;

  getError(): JournalEngineError | undefined;
  setError(value?: JournalEngineError): WriteRespMessageStatus;
  hasError(): boolean;
  clearError(): WriteRespMessageStatus;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteRespMessageStatus.AsObject;
  static toObject(includeInstance: boolean, msg: WriteRespMessageStatus): WriteRespMessageStatus.AsObject;
  static serializeBinaryToWriter(message: WriteRespMessageStatus, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteRespMessageStatus;
  static deserializeBinaryFromReader(message: WriteRespMessageStatus, reader: jspb.BinaryReader): WriteRespMessageStatus;
}

export namespace WriteRespMessageStatus {
  export type AsObject = {
    offset: number,
    pkid: number,
    error?: JournalEngineError.AsObject,
  }
}

export class WriteReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): WriteReq;
  hasHeader(): boolean;
  clearHeader(): WriteReq;

  getBody(): WriteReqBody | undefined;
  setBody(value?: WriteReqBody): WriteReq;
  hasBody(): boolean;
  clearBody(): WriteReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteReq.AsObject;
  static toObject(includeInstance: boolean, msg: WriteReq): WriteReq.AsObject;
  static serializeBinaryToWriter(message: WriteReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteReq;
  static deserializeBinaryFromReader(message: WriteReq, reader: jspb.BinaryReader): WriteReq;
}

export namespace WriteReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
    body?: WriteReqBody.AsObject,
  }
}

export class WriteResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): WriteResp;
  hasHeader(): boolean;
  clearHeader(): WriteResp;

  getBody(): WriteRespBody | undefined;
  setBody(value?: WriteRespBody): WriteResp;
  hasBody(): boolean;
  clearBody(): WriteResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): WriteResp.AsObject;
  static toObject(includeInstance: boolean, msg: WriteResp): WriteResp.AsObject;
  static serializeBinaryToWriter(message: WriteResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): WriteResp;
  static deserializeBinaryFromReader(message: WriteResp, reader: jspb.BinaryReader): WriteResp;
}

export namespace WriteResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: WriteRespBody.AsObject,
  }
}

export class ReadReqBody extends jspb.Message {
  getMessagesList(): Array<ReadReqMessage>;
  setMessagesList(value: Array<ReadReqMessage>): ReadReqBody;
  clearMessagesList(): ReadReqBody;
  addMessages(value?: ReadReqMessage, index?: number): ReadReqMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadReqBody.AsObject;
  static toObject(includeInstance: boolean, msg: ReadReqBody): ReadReqBody.AsObject;
  static serializeBinaryToWriter(message: ReadReqBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadReqBody;
  static deserializeBinaryFromReader(message: ReadReqBody, reader: jspb.BinaryReader): ReadReqBody;
}

export namespace ReadReqBody {
  export type AsObject = {
    messagesList: Array<ReadReqMessage.AsObject>,
  }
}

export class ReadReqMessage extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): ReadReqMessage;

  getShardName(): string;
  setShardName(value: string): ReadReqMessage;

  getSegment(): number;
  setSegment(value: number): ReadReqMessage;

  getReadyType(): ReadType;
  setReadyType(value: ReadType): ReadReqMessage;

  getFilter(): ReadReqFilter | undefined;
  setFilter(value?: ReadReqFilter): ReadReqMessage;
  hasFilter(): boolean;
  clearFilter(): ReadReqMessage;

  getOptions(): ReadReqOptions | undefined;
  setOptions(value?: ReadReqOptions): ReadReqMessage;
  hasOptions(): boolean;
  clearOptions(): ReadReqMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadReqMessage.AsObject;
  static toObject(includeInstance: boolean, msg: ReadReqMessage): ReadReqMessage.AsObject;
  static serializeBinaryToWriter(message: ReadReqMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadReqMessage;
  static deserializeBinaryFromReader(message: ReadReqMessage, reader: jspb.BinaryReader): ReadReqMessage;
}

export namespace ReadReqMessage {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segment: number,
    readyType: ReadType,
    filter?: ReadReqFilter.AsObject,
    options?: ReadReqOptions.AsObject,
  }
}

export class ReadReqFilter extends jspb.Message {
  getTimestamp(): number;
  setTimestamp(value: number): ReadReqFilter;

  getOffset(): number;
  setOffset(value: number): ReadReqFilter;

  getKey(): string;
  setKey(value: string): ReadReqFilter;

  getTag(): string;
  setTag(value: string): ReadReqFilter;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadReqFilter.AsObject;
  static toObject(includeInstance: boolean, msg: ReadReqFilter): ReadReqFilter.AsObject;
  static serializeBinaryToWriter(message: ReadReqFilter, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadReqFilter;
  static deserializeBinaryFromReader(message: ReadReqFilter, reader: jspb.BinaryReader): ReadReqFilter;
}

export namespace ReadReqFilter {
  export type AsObject = {
    timestamp: number,
    offset: number,
    key: string,
    tag: string,
  }
}

export class ReadReqOptions extends jspb.Message {
  getMaxSize(): number;
  setMaxSize(value: number): ReadReqOptions;

  getMaxRecord(): number;
  setMaxRecord(value: number): ReadReqOptions;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadReqOptions.AsObject;
  static toObject(includeInstance: boolean, msg: ReadReqOptions): ReadReqOptions.AsObject;
  static serializeBinaryToWriter(message: ReadReqOptions, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadReqOptions;
  static deserializeBinaryFromReader(message: ReadReqOptions, reader: jspb.BinaryReader): ReadReqOptions;
}

export namespace ReadReqOptions {
  export type AsObject = {
    maxSize: number,
    maxRecord: number,
  }
}

export class ReadRespBody extends jspb.Message {
  getMessagesList(): Array<ReadRespSegmentMessage>;
  setMessagesList(value: Array<ReadRespSegmentMessage>): ReadRespBody;
  clearMessagesList(): ReadRespBody;
  addMessages(value?: ReadRespSegmentMessage, index?: number): ReadRespSegmentMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: ReadRespBody): ReadRespBody.AsObject;
  static serializeBinaryToWriter(message: ReadRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadRespBody;
  static deserializeBinaryFromReader(message: ReadRespBody, reader: jspb.BinaryReader): ReadRespBody;
}

export namespace ReadRespBody {
  export type AsObject = {
    messagesList: Array<ReadRespSegmentMessage.AsObject>,
  }
}

export class ReadRespSegmentMessage extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): ReadRespSegmentMessage;

  getShardName(): string;
  setShardName(value: string): ReadRespSegmentMessage;

  getSegment(): number;
  setSegment(value: number): ReadRespSegmentMessage;

  getMessagesList(): Array<ReadRespMessage>;
  setMessagesList(value: Array<ReadRespMessage>): ReadRespSegmentMessage;
  clearMessagesList(): ReadRespSegmentMessage;
  addMessages(value?: ReadRespMessage, index?: number): ReadRespMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadRespSegmentMessage.AsObject;
  static toObject(includeInstance: boolean, msg: ReadRespSegmentMessage): ReadRespSegmentMessage.AsObject;
  static serializeBinaryToWriter(message: ReadRespSegmentMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadRespSegmentMessage;
  static deserializeBinaryFromReader(message: ReadRespSegmentMessage, reader: jspb.BinaryReader): ReadRespSegmentMessage;
}

export namespace ReadRespSegmentMessage {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segment: number,
    messagesList: Array<ReadRespMessage.AsObject>,
  }
}

export class ReadRespMessage extends jspb.Message {
  getOffset(): number;
  setOffset(value: number): ReadRespMessage;

  getKey(): string;
  setKey(value: string): ReadRespMessage;

  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): ReadRespMessage;

  getTagsList(): Array<string>;
  setTagsList(value: Array<string>): ReadRespMessage;
  clearTagsList(): ReadRespMessage;
  addTags(value: string, index?: number): ReadRespMessage;

  getTimestamp(): number;
  setTimestamp(value: number): ReadRespMessage;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadRespMessage.AsObject;
  static toObject(includeInstance: boolean, msg: ReadRespMessage): ReadRespMessage.AsObject;
  static serializeBinaryToWriter(message: ReadRespMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadRespMessage;
  static deserializeBinaryFromReader(message: ReadRespMessage, reader: jspb.BinaryReader): ReadRespMessage;
}

export namespace ReadRespMessage {
  export type AsObject = {
    offset: number,
    key: string,
    value: Uint8Array | string,
    tagsList: Array<string>,
    timestamp: number,
  }
}

export class ReadReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): ReadReq;
  hasHeader(): boolean;
  clearHeader(): ReadReq;

  getBody(): ReadReqBody | undefined;
  setBody(value?: ReadReqBody): ReadReq;
  hasBody(): boolean;
  clearBody(): ReadReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadReq.AsObject;
  static toObject(includeInstance: boolean, msg: ReadReq): ReadReq.AsObject;
  static serializeBinaryToWriter(message: ReadReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadReq;
  static deserializeBinaryFromReader(message: ReadReq, reader: jspb.BinaryReader): ReadReq;
}

export namespace ReadReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
    body?: ReadReqBody.AsObject,
  }
}

export class ReadResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): ReadResp;
  hasHeader(): boolean;
  clearHeader(): ReadResp;

  getBody(): ReadRespBody | undefined;
  setBody(value?: ReadRespBody): ReadResp;
  hasBody(): boolean;
  clearBody(): ReadResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReadResp.AsObject;
  static toObject(includeInstance: boolean, msg: ReadResp): ReadResp.AsObject;
  static serializeBinaryToWriter(message: ReadResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReadResp;
  static deserializeBinaryFromReader(message: ReadResp, reader: jspb.BinaryReader): ReadResp;
}

export namespace ReadResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: ReadRespBody.AsObject,
  }
}

export class FetchOffsetReqBody extends jspb.Message {
  getShardsList(): Array<FetchOffsetShard>;
  setShardsList(value: Array<FetchOffsetShard>): FetchOffsetReqBody;
  clearShardsList(): FetchOffsetReqBody;
  addShards(value?: FetchOffsetShard, index?: number): FetchOffsetShard;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FetchOffsetReqBody.AsObject;
  static toObject(includeInstance: boolean, msg: FetchOffsetReqBody): FetchOffsetReqBody.AsObject;
  static serializeBinaryToWriter(message: FetchOffsetReqBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FetchOffsetReqBody;
  static deserializeBinaryFromReader(message: FetchOffsetReqBody, reader: jspb.BinaryReader): FetchOffsetReqBody;
}

export namespace FetchOffsetReqBody {
  export type AsObject = {
    shardsList: Array<FetchOffsetShard.AsObject>,
  }
}

export class FetchOffsetShard extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): FetchOffsetShard;

  getShardName(): string;
  setShardName(value: string): FetchOffsetShard;

  getSegmentNo(): number;
  setSegmentNo(value: number): FetchOffsetShard;

  getTimestamp(): number;
  setTimestamp(value: number): FetchOffsetShard;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FetchOffsetShard.AsObject;
  static toObject(includeInstance: boolean, msg: FetchOffsetShard): FetchOffsetShard.AsObject;
  static serializeBinaryToWriter(message: FetchOffsetShard, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FetchOffsetShard;
  static deserializeBinaryFromReader(message: FetchOffsetShard, reader: jspb.BinaryReader): FetchOffsetShard;
}

export namespace FetchOffsetShard {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segmentNo: number,
    timestamp: number,
  }
}

export class FetchOffsetReq extends jspb.Message {
  getHeader(): ReqHeader | undefined;
  setHeader(value?: ReqHeader): FetchOffsetReq;
  hasHeader(): boolean;
  clearHeader(): FetchOffsetReq;

  getBody(): FetchOffsetReqBody | undefined;
  setBody(value?: FetchOffsetReqBody): FetchOffsetReq;
  hasBody(): boolean;
  clearBody(): FetchOffsetReq;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FetchOffsetReq.AsObject;
  static toObject(includeInstance: boolean, msg: FetchOffsetReq): FetchOffsetReq.AsObject;
  static serializeBinaryToWriter(message: FetchOffsetReq, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FetchOffsetReq;
  static deserializeBinaryFromReader(message: FetchOffsetReq, reader: jspb.BinaryReader): FetchOffsetReq;
}

export namespace FetchOffsetReq {
  export type AsObject = {
    header?: ReqHeader.AsObject,
    body?: FetchOffsetReqBody.AsObject,
  }
}

export class FetchOffsetRespBody extends jspb.Message {
  getShardOffsetsList(): Array<FetchOffsetShardMeta>;
  setShardOffsetsList(value: Array<FetchOffsetShardMeta>): FetchOffsetRespBody;
  clearShardOffsetsList(): FetchOffsetRespBody;
  addShardOffsets(value?: FetchOffsetShardMeta, index?: number): FetchOffsetShardMeta;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FetchOffsetRespBody.AsObject;
  static toObject(includeInstance: boolean, msg: FetchOffsetRespBody): FetchOffsetRespBody.AsObject;
  static serializeBinaryToWriter(message: FetchOffsetRespBody, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FetchOffsetRespBody;
  static deserializeBinaryFromReader(message: FetchOffsetRespBody, reader: jspb.BinaryReader): FetchOffsetRespBody;
}

export namespace FetchOffsetRespBody {
  export type AsObject = {
    shardOffsetsList: Array<FetchOffsetShardMeta.AsObject>,
  }
}

export class FetchOffsetShardMeta extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): FetchOffsetShardMeta;

  getShardName(): string;
  setShardName(value: string): FetchOffsetShardMeta;

  getSegmentNo(): number;
  setSegmentNo(value: number): FetchOffsetShardMeta;

  getOffset(): number;
  setOffset(value: number): FetchOffsetShardMeta;

  getError(): JournalEngineError | undefined;
  setError(value?: JournalEngineError): FetchOffsetShardMeta;
  hasError(): boolean;
  clearError(): FetchOffsetShardMeta;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FetchOffsetShardMeta.AsObject;
  static toObject(includeInstance: boolean, msg: FetchOffsetShardMeta): FetchOffsetShardMeta.AsObject;
  static serializeBinaryToWriter(message: FetchOffsetShardMeta, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FetchOffsetShardMeta;
  static deserializeBinaryFromReader(message: FetchOffsetShardMeta, reader: jspb.BinaryReader): FetchOffsetShardMeta;
}

export namespace FetchOffsetShardMeta {
  export type AsObject = {
    namespace: string,
    shardName: string,
    segmentNo: number,
    offset: number,
    error?: JournalEngineError.AsObject,
  }
}

export class FetchOffsetResp extends jspb.Message {
  getHeader(): RespHeader | undefined;
  setHeader(value?: RespHeader): FetchOffsetResp;
  hasHeader(): boolean;
  clearHeader(): FetchOffsetResp;

  getBody(): FetchOffsetRespBody | undefined;
  setBody(value?: FetchOffsetRespBody): FetchOffsetResp;
  hasBody(): boolean;
  clearBody(): FetchOffsetResp;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): FetchOffsetResp.AsObject;
  static toObject(includeInstance: boolean, msg: FetchOffsetResp): FetchOffsetResp.AsObject;
  static serializeBinaryToWriter(message: FetchOffsetResp, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): FetchOffsetResp;
  static deserializeBinaryFromReader(message: FetchOffsetResp, reader: jspb.BinaryReader): FetchOffsetResp;
}

export namespace FetchOffsetResp {
  export type AsObject = {
    header?: RespHeader.AsObject,
    body?: FetchOffsetRespBody.AsObject,
  }
}

export enum ApiKey { 
  UNIMPLEMENTED = 0,
  READ = 1,
  WRITE = 2,
  CREATESHARD = 3,
  DELETESHARD = 4,
  GETSHARDMETADATA = 5,
  GETCLUSTERMETADATA = 6,
  FETCHOFFSET = 7,
}
export enum ApiVersion { 
  V0 = 0,
}
export enum ReadType { 
  OFFSET = 0,
  KEY = 1,
  TAG = 2,
}
export enum AutoOffsetStrategy { 
  EARLIEST = 0,
  LATEST = 1,
  NONE = 2,
}
