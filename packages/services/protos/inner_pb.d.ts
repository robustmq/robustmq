import * as jspb from 'google-protobuf'

import * as validate_validate_pb from './validate/validate_pb'; // proto import: "validate/validate.proto"


export class ClusterStatusRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ClusterStatusRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ClusterStatusRequest): ClusterStatusRequest.AsObject;
  static serializeBinaryToWriter(message: ClusterStatusRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ClusterStatusRequest;
  static deserializeBinaryFromReader(message: ClusterStatusRequest, reader: jspb.BinaryReader): ClusterStatusRequest;
}

export namespace ClusterStatusRequest {
  export type AsObject = {
  }
}

export class ClusterStatusReply extends jspb.Message {
  getContent(): string;
  setContent(value: string): ClusterStatusReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ClusterStatusReply.AsObject;
  static toObject(includeInstance: boolean, msg: ClusterStatusReply): ClusterStatusReply.AsObject;
  static serializeBinaryToWriter(message: ClusterStatusReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ClusterStatusReply;
  static deserializeBinaryFromReader(message: ClusterStatusReply, reader: jspb.BinaryReader): ClusterStatusReply;
}

export namespace ClusterStatusReply {
  export type AsObject = {
    content: string,
  }
}

export class NodeListRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): NodeListRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): NodeListRequest.AsObject;
  static toObject(includeInstance: boolean, msg: NodeListRequest): NodeListRequest.AsObject;
  static serializeBinaryToWriter(message: NodeListRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): NodeListRequest;
  static deserializeBinaryFromReader(message: NodeListRequest, reader: jspb.BinaryReader): NodeListRequest;
}

export namespace NodeListRequest {
  export type AsObject = {
    clusterName: string,
  }
}

export class NodeListReply extends jspb.Message {
  getNodesList(): Array<Uint8Array | string>;
  setNodesList(value: Array<Uint8Array | string>): NodeListReply;
  clearNodesList(): NodeListReply;
  addNodes(value: Uint8Array | string, index?: number): NodeListReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): NodeListReply.AsObject;
  static toObject(includeInstance: boolean, msg: NodeListReply): NodeListReply.AsObject;
  static serializeBinaryToWriter(message: NodeListReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): NodeListReply;
  static deserializeBinaryFromReader(message: NodeListReply, reader: jspb.BinaryReader): NodeListReply;
}

export namespace NodeListReply {
  export type AsObject = {
    nodesList: Array<Uint8Array | string>,
  }
}

export class RegisterNodeRequest extends jspb.Message {
  getClusterType(): ClusterType;
  setClusterType(value: ClusterType): RegisterNodeRequest;

  getClusterName(): string;
  setClusterName(value: string): RegisterNodeRequest;

  getNodeIp(): string;
  setNodeIp(value: string): RegisterNodeRequest;

  getNodeId(): number;
  setNodeId(value: number): RegisterNodeRequest;

  getNodeInnerAddr(): string;
  setNodeInnerAddr(value: string): RegisterNodeRequest;

  getExtendInfo(): string;
  setExtendInfo(value: string): RegisterNodeRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RegisterNodeRequest.AsObject;
  static toObject(includeInstance: boolean, msg: RegisterNodeRequest): RegisterNodeRequest.AsObject;
  static serializeBinaryToWriter(message: RegisterNodeRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RegisterNodeRequest;
  static deserializeBinaryFromReader(message: RegisterNodeRequest, reader: jspb.BinaryReader): RegisterNodeRequest;
}

export namespace RegisterNodeRequest {
  export type AsObject = {
    clusterType: ClusterType,
    clusterName: string,
    nodeIp: string,
    nodeId: number,
    nodeInnerAddr: string,
    extendInfo: string,
  }
}

export class RegisterNodeReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RegisterNodeReply.AsObject;
  static toObject(includeInstance: boolean, msg: RegisterNodeReply): RegisterNodeReply.AsObject;
  static serializeBinaryToWriter(message: RegisterNodeReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RegisterNodeReply;
  static deserializeBinaryFromReader(message: RegisterNodeReply, reader: jspb.BinaryReader): RegisterNodeReply;
}

export namespace RegisterNodeReply {
  export type AsObject = {
  }
}

export class UnRegisterNodeRequest extends jspb.Message {
  getClusterType(): ClusterType;
  setClusterType(value: ClusterType): UnRegisterNodeRequest;

  getClusterName(): string;
  setClusterName(value: string): UnRegisterNodeRequest;

  getNodeId(): number;
  setNodeId(value: number): UnRegisterNodeRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UnRegisterNodeRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UnRegisterNodeRequest): UnRegisterNodeRequest.AsObject;
  static serializeBinaryToWriter(message: UnRegisterNodeRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UnRegisterNodeRequest;
  static deserializeBinaryFromReader(message: UnRegisterNodeRequest, reader: jspb.BinaryReader): UnRegisterNodeRequest;
}

export namespace UnRegisterNodeRequest {
  export type AsObject = {
    clusterType: ClusterType,
    clusterName: string,
    nodeId: number,
  }
}

export class UnRegisterNodeReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UnRegisterNodeReply.AsObject;
  static toObject(includeInstance: boolean, msg: UnRegisterNodeReply): UnRegisterNodeReply.AsObject;
  static serializeBinaryToWriter(message: UnRegisterNodeReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UnRegisterNodeReply;
  static deserializeBinaryFromReader(message: UnRegisterNodeReply, reader: jspb.BinaryReader): UnRegisterNodeReply;
}

export namespace UnRegisterNodeReply {
  export type AsObject = {
  }
}

export class HeartbeatRequest extends jspb.Message {
  getClusterType(): ClusterType;
  setClusterType(value: ClusterType): HeartbeatRequest;

  getClusterName(): string;
  setClusterName(value: string): HeartbeatRequest;

  getNodeId(): number;
  setNodeId(value: number): HeartbeatRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): HeartbeatRequest.AsObject;
  static toObject(includeInstance: boolean, msg: HeartbeatRequest): HeartbeatRequest.AsObject;
  static serializeBinaryToWriter(message: HeartbeatRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): HeartbeatRequest;
  static deserializeBinaryFromReader(message: HeartbeatRequest, reader: jspb.BinaryReader): HeartbeatRequest;
}

export namespace HeartbeatRequest {
  export type AsObject = {
    clusterType: ClusterType,
    clusterName: string,
    nodeId: number,
  }
}

export class HeartbeatReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): HeartbeatReply.AsObject;
  static toObject(includeInstance: boolean, msg: HeartbeatReply): HeartbeatReply.AsObject;
  static serializeBinaryToWriter(message: HeartbeatReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): HeartbeatReply;
  static deserializeBinaryFromReader(message: HeartbeatReply, reader: jspb.BinaryReader): HeartbeatReply;
}

export namespace HeartbeatReply {
  export type AsObject = {
  }
}

export class ReportMonitorRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ReportMonitorRequest;

  getNodeId(): number;
  setNodeId(value: number): ReportMonitorRequest;

  getCpuRate(): number;
  setCpuRate(value: number): ReportMonitorRequest;

  getMemoryRate(): number;
  setMemoryRate(value: number): ReportMonitorRequest;

  getDiskRate(): number;
  setDiskRate(value: number): ReportMonitorRequest;

  getNetworkRate(): number;
  setNetworkRate(value: number): ReportMonitorRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReportMonitorRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ReportMonitorRequest): ReportMonitorRequest.AsObject;
  static serializeBinaryToWriter(message: ReportMonitorRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReportMonitorRequest;
  static deserializeBinaryFromReader(message: ReportMonitorRequest, reader: jspb.BinaryReader): ReportMonitorRequest;
}

export namespace ReportMonitorRequest {
  export type AsObject = {
    clusterName: string,
    nodeId: number,
    cpuRate: number,
    memoryRate: number,
    diskRate: number,
    networkRate: number,
  }
}

export class ReportMonitorReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReportMonitorReply.AsObject;
  static toObject(includeInstance: boolean, msg: ReportMonitorReply): ReportMonitorReply.AsObject;
  static serializeBinaryToWriter(message: ReportMonitorReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReportMonitorReply;
  static deserializeBinaryFromReader(message: ReportMonitorReply, reader: jspb.BinaryReader): ReportMonitorReply;
}

export namespace ReportMonitorReply {
  export type AsObject = {
  }
}

export class SendRaftMessageRequest extends jspb.Message {
  getMessage(): Uint8Array | string;
  getMessage_asU8(): Uint8Array;
  getMessage_asB64(): string;
  setMessage(value: Uint8Array | string): SendRaftMessageRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SendRaftMessageRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SendRaftMessageRequest): SendRaftMessageRequest.AsObject;
  static serializeBinaryToWriter(message: SendRaftMessageRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SendRaftMessageRequest;
  static deserializeBinaryFromReader(message: SendRaftMessageRequest, reader: jspb.BinaryReader): SendRaftMessageRequest;
}

export namespace SendRaftMessageRequest {
  export type AsObject = {
    message: Uint8Array | string,
  }
}

export class SendRaftMessageReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SendRaftMessageReply.AsObject;
  static toObject(includeInstance: boolean, msg: SendRaftMessageReply): SendRaftMessageReply.AsObject;
  static serializeBinaryToWriter(message: SendRaftMessageReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SendRaftMessageReply;
  static deserializeBinaryFromReader(message: SendRaftMessageReply, reader: jspb.BinaryReader): SendRaftMessageReply;
}

export namespace SendRaftMessageReply {
  export type AsObject = {
  }
}

export class SendRaftConfChangeRequest extends jspb.Message {
  getMessage(): Uint8Array | string;
  getMessage_asU8(): Uint8Array;
  getMessage_asB64(): string;
  setMessage(value: Uint8Array | string): SendRaftConfChangeRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SendRaftConfChangeRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SendRaftConfChangeRequest): SendRaftConfChangeRequest.AsObject;
  static serializeBinaryToWriter(message: SendRaftConfChangeRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SendRaftConfChangeRequest;
  static deserializeBinaryFromReader(message: SendRaftConfChangeRequest, reader: jspb.BinaryReader): SendRaftConfChangeRequest;
}

export namespace SendRaftConfChangeRequest {
  export type AsObject = {
    message: Uint8Array | string,
  }
}

export class SendRaftConfChangeReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SendRaftConfChangeReply.AsObject;
  static toObject(includeInstance: boolean, msg: SendRaftConfChangeReply): SendRaftConfChangeReply.AsObject;
  static serializeBinaryToWriter(message: SendRaftConfChangeReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SendRaftConfChangeReply;
  static deserializeBinaryFromReader(message: SendRaftConfChangeReply, reader: jspb.BinaryReader): SendRaftConfChangeReply;
}

export namespace SendRaftConfChangeReply {
  export type AsObject = {
  }
}

export class SetResourceConfigRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SetResourceConfigRequest;

  getResourcesList(): Array<string>;
  setResourcesList(value: Array<string>): SetResourceConfigRequest;
  clearResourcesList(): SetResourceConfigRequest;
  addResources(value: string, index?: number): SetResourceConfigRequest;

  getConfig(): Uint8Array | string;
  getConfig_asU8(): Uint8Array;
  getConfig_asB64(): string;
  setConfig(value: Uint8Array | string): SetResourceConfigRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetResourceConfigRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetResourceConfigRequest): SetResourceConfigRequest.AsObject;
  static serializeBinaryToWriter(message: SetResourceConfigRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetResourceConfigRequest;
  static deserializeBinaryFromReader(message: SetResourceConfigRequest, reader: jspb.BinaryReader): SetResourceConfigRequest;
}

export namespace SetResourceConfigRequest {
  export type AsObject = {
    clusterName: string,
    resourcesList: Array<string>,
    config: Uint8Array | string,
  }
}

export class SetResourceConfigReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetResourceConfigReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetResourceConfigReply): SetResourceConfigReply.AsObject;
  static serializeBinaryToWriter(message: SetResourceConfigReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetResourceConfigReply;
  static deserializeBinaryFromReader(message: SetResourceConfigReply, reader: jspb.BinaryReader): SetResourceConfigReply;
}

export namespace SetResourceConfigReply {
  export type AsObject = {
  }
}

export class GetResourceConfigRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): GetResourceConfigRequest;

  getResourcesList(): Array<string>;
  setResourcesList(value: Array<string>): GetResourceConfigRequest;
  clearResourcesList(): GetResourceConfigRequest;
  addResources(value: string, index?: number): GetResourceConfigRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetResourceConfigRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetResourceConfigRequest): GetResourceConfigRequest.AsObject;
  static serializeBinaryToWriter(message: GetResourceConfigRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetResourceConfigRequest;
  static deserializeBinaryFromReader(message: GetResourceConfigRequest, reader: jspb.BinaryReader): GetResourceConfigRequest;
}

export namespace GetResourceConfigRequest {
  export type AsObject = {
    clusterName: string,
    resourcesList: Array<string>,
  }
}

export class GetResourceConfigReply extends jspb.Message {
  getConfig(): Uint8Array | string;
  getConfig_asU8(): Uint8Array;
  getConfig_asB64(): string;
  setConfig(value: Uint8Array | string): GetResourceConfigReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetResourceConfigReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetResourceConfigReply): GetResourceConfigReply.AsObject;
  static serializeBinaryToWriter(message: GetResourceConfigReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetResourceConfigReply;
  static deserializeBinaryFromReader(message: GetResourceConfigReply, reader: jspb.BinaryReader): GetResourceConfigReply;
}

export namespace GetResourceConfigReply {
  export type AsObject = {
    config: Uint8Array | string,
  }
}

export class DeleteResourceConfigRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteResourceConfigRequest;

  getResourcesList(): Array<string>;
  setResourcesList(value: Array<string>): DeleteResourceConfigRequest;
  clearResourcesList(): DeleteResourceConfigRequest;
  addResources(value: string, index?: number): DeleteResourceConfigRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteResourceConfigRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteResourceConfigRequest): DeleteResourceConfigRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteResourceConfigRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteResourceConfigRequest;
  static deserializeBinaryFromReader(message: DeleteResourceConfigRequest, reader: jspb.BinaryReader): DeleteResourceConfigRequest;
}

export namespace DeleteResourceConfigRequest {
  export type AsObject = {
    clusterName: string,
    resourcesList: Array<string>,
  }
}

export class DeleteResourceConfigReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteResourceConfigReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteResourceConfigReply): DeleteResourceConfigReply.AsObject;
  static serializeBinaryToWriter(message: DeleteResourceConfigReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteResourceConfigReply;
  static deserializeBinaryFromReader(message: DeleteResourceConfigReply, reader: jspb.BinaryReader): DeleteResourceConfigReply;
}

export namespace DeleteResourceConfigReply {
  export type AsObject = {
  }
}

export class SetIdempotentDataRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SetIdempotentDataRequest;

  getProducerId(): string;
  setProducerId(value: string): SetIdempotentDataRequest;

  getSeqNum(): number;
  setSeqNum(value: number): SetIdempotentDataRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetIdempotentDataRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetIdempotentDataRequest): SetIdempotentDataRequest.AsObject;
  static serializeBinaryToWriter(message: SetIdempotentDataRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetIdempotentDataRequest;
  static deserializeBinaryFromReader(message: SetIdempotentDataRequest, reader: jspb.BinaryReader): SetIdempotentDataRequest;
}

export namespace SetIdempotentDataRequest {
  export type AsObject = {
    clusterName: string,
    producerId: string,
    seqNum: number,
  }
}

export class SetIdempotentDataReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetIdempotentDataReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetIdempotentDataReply): SetIdempotentDataReply.AsObject;
  static serializeBinaryToWriter(message: SetIdempotentDataReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetIdempotentDataReply;
  static deserializeBinaryFromReader(message: SetIdempotentDataReply, reader: jspb.BinaryReader): SetIdempotentDataReply;
}

export namespace SetIdempotentDataReply {
  export type AsObject = {
  }
}

export class ExistsIdempotentDataRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ExistsIdempotentDataRequest;

  getProducerId(): string;
  setProducerId(value: string): ExistsIdempotentDataRequest;

  getSeqNum(): number;
  setSeqNum(value: number): ExistsIdempotentDataRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ExistsIdempotentDataRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ExistsIdempotentDataRequest): ExistsIdempotentDataRequest.AsObject;
  static serializeBinaryToWriter(message: ExistsIdempotentDataRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ExistsIdempotentDataRequest;
  static deserializeBinaryFromReader(message: ExistsIdempotentDataRequest, reader: jspb.BinaryReader): ExistsIdempotentDataRequest;
}

export namespace ExistsIdempotentDataRequest {
  export type AsObject = {
    clusterName: string,
    producerId: string,
    seqNum: number,
  }
}

export class ExistsIdempotentDataReply extends jspb.Message {
  getExists(): boolean;
  setExists(value: boolean): ExistsIdempotentDataReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ExistsIdempotentDataReply.AsObject;
  static toObject(includeInstance: boolean, msg: ExistsIdempotentDataReply): ExistsIdempotentDataReply.AsObject;
  static serializeBinaryToWriter(message: ExistsIdempotentDataReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ExistsIdempotentDataReply;
  static deserializeBinaryFromReader(message: ExistsIdempotentDataReply, reader: jspb.BinaryReader): ExistsIdempotentDataReply;
}

export namespace ExistsIdempotentDataReply {
  export type AsObject = {
    exists: boolean,
  }
}

export class DeleteIdempotentDataRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteIdempotentDataRequest;

  getProducerId(): string;
  setProducerId(value: string): DeleteIdempotentDataRequest;

  getSeqNum(): number;
  setSeqNum(value: number): DeleteIdempotentDataRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteIdempotentDataRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteIdempotentDataRequest): DeleteIdempotentDataRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteIdempotentDataRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteIdempotentDataRequest;
  static deserializeBinaryFromReader(message: DeleteIdempotentDataRequest, reader: jspb.BinaryReader): DeleteIdempotentDataRequest;
}

export namespace DeleteIdempotentDataRequest {
  export type AsObject = {
    clusterName: string,
    producerId: string,
    seqNum: number,
  }
}

export class DeleteIdempotentDataReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteIdempotentDataReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteIdempotentDataReply): DeleteIdempotentDataReply.AsObject;
  static serializeBinaryToWriter(message: DeleteIdempotentDataReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteIdempotentDataReply;
  static deserializeBinaryFromReader(message: DeleteIdempotentDataReply, reader: jspb.BinaryReader): DeleteIdempotentDataReply;
}

export namespace DeleteIdempotentDataReply {
  export type AsObject = {
  }
}

export class SaveOffsetDataRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SaveOffsetDataRequest;

  getGroup(): string;
  setGroup(value: string): SaveOffsetDataRequest;

  getOffsetsList(): Array<SaveOffsetDataRequestOffset>;
  setOffsetsList(value: Array<SaveOffsetDataRequestOffset>): SaveOffsetDataRequest;
  clearOffsetsList(): SaveOffsetDataRequest;
  addOffsets(value?: SaveOffsetDataRequestOffset, index?: number): SaveOffsetDataRequestOffset;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SaveOffsetDataRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SaveOffsetDataRequest): SaveOffsetDataRequest.AsObject;
  static serializeBinaryToWriter(message: SaveOffsetDataRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SaveOffsetDataRequest;
  static deserializeBinaryFromReader(message: SaveOffsetDataRequest, reader: jspb.BinaryReader): SaveOffsetDataRequest;
}

export namespace SaveOffsetDataRequest {
  export type AsObject = {
    clusterName: string,
    group: string,
    offsetsList: Array<SaveOffsetDataRequestOffset.AsObject>,
  }
}

export class SaveOffsetDataRequestOffset extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): SaveOffsetDataRequestOffset;

  getShardName(): string;
  setShardName(value: string): SaveOffsetDataRequestOffset;

  getOffset(): number;
  setOffset(value: number): SaveOffsetDataRequestOffset;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SaveOffsetDataRequestOffset.AsObject;
  static toObject(includeInstance: boolean, msg: SaveOffsetDataRequestOffset): SaveOffsetDataRequestOffset.AsObject;
  static serializeBinaryToWriter(message: SaveOffsetDataRequestOffset, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SaveOffsetDataRequestOffset;
  static deserializeBinaryFromReader(message: SaveOffsetDataRequestOffset, reader: jspb.BinaryReader): SaveOffsetDataRequestOffset;
}

export namespace SaveOffsetDataRequestOffset {
  export type AsObject = {
    namespace: string,
    shardName: string,
    offset: number,
  }
}

export class SaveOffsetDataReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SaveOffsetDataReply.AsObject;
  static toObject(includeInstance: boolean, msg: SaveOffsetDataReply): SaveOffsetDataReply.AsObject;
  static serializeBinaryToWriter(message: SaveOffsetDataReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SaveOffsetDataReply;
  static deserializeBinaryFromReader(message: SaveOffsetDataReply, reader: jspb.BinaryReader): SaveOffsetDataReply;
}

export namespace SaveOffsetDataReply {
  export type AsObject = {
  }
}

export class GetOffsetDataRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): GetOffsetDataRequest;

  getGroup(): string;
  setGroup(value: string): GetOffsetDataRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetOffsetDataRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetOffsetDataRequest): GetOffsetDataRequest.AsObject;
  static serializeBinaryToWriter(message: GetOffsetDataRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetOffsetDataRequest;
  static deserializeBinaryFromReader(message: GetOffsetDataRequest, reader: jspb.BinaryReader): GetOffsetDataRequest;
}

export namespace GetOffsetDataRequest {
  export type AsObject = {
    clusterName: string,
    group: string,
  }
}

export class GetOffsetDataReply extends jspb.Message {
  getOffsetsList(): Array<GetOffsetDataReplyOffset>;
  setOffsetsList(value: Array<GetOffsetDataReplyOffset>): GetOffsetDataReply;
  clearOffsetsList(): GetOffsetDataReply;
  addOffsets(value?: GetOffsetDataReplyOffset, index?: number): GetOffsetDataReplyOffset;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetOffsetDataReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetOffsetDataReply): GetOffsetDataReply.AsObject;
  static serializeBinaryToWriter(message: GetOffsetDataReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetOffsetDataReply;
  static deserializeBinaryFromReader(message: GetOffsetDataReply, reader: jspb.BinaryReader): GetOffsetDataReply;
}

export namespace GetOffsetDataReply {
  export type AsObject = {
    offsetsList: Array<GetOffsetDataReplyOffset.AsObject>,
  }
}

export class GetOffsetDataReplyOffset extends jspb.Message {
  getNamespace(): string;
  setNamespace(value: string): GetOffsetDataReplyOffset;

  getShardName(): string;
  setShardName(value: string): GetOffsetDataReplyOffset;

  getOffset(): number;
  setOffset(value: number): GetOffsetDataReplyOffset;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetOffsetDataReplyOffset.AsObject;
  static toObject(includeInstance: boolean, msg: GetOffsetDataReplyOffset): GetOffsetDataReplyOffset.AsObject;
  static serializeBinaryToWriter(message: GetOffsetDataReplyOffset, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetOffsetDataReplyOffset;
  static deserializeBinaryFromReader(message: GetOffsetDataReplyOffset, reader: jspb.BinaryReader): GetOffsetDataReplyOffset;
}

export namespace GetOffsetDataReplyOffset {
  export type AsObject = {
    namespace: string,
    shardName: string,
    offset: number,
  }
}

export class ListSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): ListSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListSchemaRequest): ListSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: ListSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSchemaRequest;
  static deserializeBinaryFromReader(message: ListSchemaRequest, reader: jspb.BinaryReader): ListSchemaRequest;
}

export namespace ListSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
  }
}

export class ListSchemaReply extends jspb.Message {
  getSchemasList(): Array<Uint8Array | string>;
  setSchemasList(value: Array<Uint8Array | string>): ListSchemaReply;
  clearSchemasList(): ListSchemaReply;
  addSchemas(value: Uint8Array | string, index?: number): ListSchemaReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListSchemaReply): ListSchemaReply.AsObject;
  static serializeBinaryToWriter(message: ListSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSchemaReply;
  static deserializeBinaryFromReader(message: ListSchemaReply, reader: jspb.BinaryReader): ListSchemaReply;
}

export namespace ListSchemaReply {
  export type AsObject = {
    schemasList: Array<Uint8Array | string>,
  }
}

export class CreateSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): CreateSchemaRequest;

  getSchema(): Uint8Array | string;
  getSchema_asU8(): Uint8Array;
  getSchema_asB64(): string;
  setSchema(value: Uint8Array | string): CreateSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateSchemaRequest): CreateSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: CreateSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateSchemaRequest;
  static deserializeBinaryFromReader(message: CreateSchemaRequest, reader: jspb.BinaryReader): CreateSchemaRequest;
}

export namespace CreateSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
    schema: Uint8Array | string,
  }
}

export class CreateSchemaReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateSchemaReply): CreateSchemaReply.AsObject;
  static serializeBinaryToWriter(message: CreateSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateSchemaReply;
  static deserializeBinaryFromReader(message: CreateSchemaReply, reader: jspb.BinaryReader): CreateSchemaReply;
}

export namespace CreateSchemaReply {
  export type AsObject = {
  }
}

export class UpdateSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): UpdateSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): UpdateSchemaRequest;

  getSchema(): Uint8Array | string;
  getSchema_asU8(): Uint8Array;
  getSchema_asB64(): string;
  setSchema(value: Uint8Array | string): UpdateSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSchemaRequest): UpdateSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: UpdateSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSchemaRequest;
  static deserializeBinaryFromReader(message: UpdateSchemaRequest, reader: jspb.BinaryReader): UpdateSchemaRequest;
}

export namespace UpdateSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
    schema: Uint8Array | string,
  }
}

export class UpdateSchemaReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSchemaReply): UpdateSchemaReply.AsObject;
  static serializeBinaryToWriter(message: UpdateSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSchemaReply;
  static deserializeBinaryFromReader(message: UpdateSchemaReply, reader: jspb.BinaryReader): UpdateSchemaReply;
}

export namespace UpdateSchemaReply {
  export type AsObject = {
  }
}

export class DeleteSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): DeleteSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSchemaRequest): DeleteSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSchemaRequest;
  static deserializeBinaryFromReader(message: DeleteSchemaRequest, reader: jspb.BinaryReader): DeleteSchemaRequest;
}

export namespace DeleteSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
  }
}

export class DeleteSchemaReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSchemaReply): DeleteSchemaReply.AsObject;
  static serializeBinaryToWriter(message: DeleteSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSchemaReply;
  static deserializeBinaryFromReader(message: DeleteSchemaReply, reader: jspb.BinaryReader): DeleteSchemaReply;
}

export namespace DeleteSchemaReply {
  export type AsObject = {
  }
}

export class ListBindSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListBindSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): ListBindSchemaRequest;

  getResourceName(): string;
  setResourceName(value: string): ListBindSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListBindSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListBindSchemaRequest): ListBindSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: ListBindSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListBindSchemaRequest;
  static deserializeBinaryFromReader(message: ListBindSchemaRequest, reader: jspb.BinaryReader): ListBindSchemaRequest;
}

export namespace ListBindSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
    resourceName: string,
  }
}

export class ListBindSchemaReply extends jspb.Message {
  getSchemaBindsList(): Array<Uint8Array | string>;
  setSchemaBindsList(value: Array<Uint8Array | string>): ListBindSchemaReply;
  clearSchemaBindsList(): ListBindSchemaReply;
  addSchemaBinds(value: Uint8Array | string, index?: number): ListBindSchemaReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListBindSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListBindSchemaReply): ListBindSchemaReply.AsObject;
  static serializeBinaryToWriter(message: ListBindSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListBindSchemaReply;
  static deserializeBinaryFromReader(message: ListBindSchemaReply, reader: jspb.BinaryReader): ListBindSchemaReply;
}

export namespace ListBindSchemaReply {
  export type AsObject = {
    schemaBindsList: Array<Uint8Array | string>,
  }
}

export class BindSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): BindSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): BindSchemaRequest;

  getResourceName(): string;
  setResourceName(value: string): BindSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): BindSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: BindSchemaRequest): BindSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: BindSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): BindSchemaRequest;
  static deserializeBinaryFromReader(message: BindSchemaRequest, reader: jspb.BinaryReader): BindSchemaRequest;
}

export namespace BindSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
    resourceName: string,
  }
}

export class BindSchemaReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): BindSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: BindSchemaReply): BindSchemaReply.AsObject;
  static serializeBinaryToWriter(message: BindSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): BindSchemaReply;
  static deserializeBinaryFromReader(message: BindSchemaReply, reader: jspb.BinaryReader): BindSchemaReply;
}

export namespace BindSchemaReply {
  export type AsObject = {
  }
}

export class UnBindSchemaRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): UnBindSchemaRequest;

  getSchemaName(): string;
  setSchemaName(value: string): UnBindSchemaRequest;

  getResourceName(): string;
  setResourceName(value: string): UnBindSchemaRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UnBindSchemaRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UnBindSchemaRequest): UnBindSchemaRequest.AsObject;
  static serializeBinaryToWriter(message: UnBindSchemaRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UnBindSchemaRequest;
  static deserializeBinaryFromReader(message: UnBindSchemaRequest, reader: jspb.BinaryReader): UnBindSchemaRequest;
}

export namespace UnBindSchemaRequest {
  export type AsObject = {
    clusterName: string,
    schemaName: string,
    resourceName: string,
  }
}

export class UnBindSchemaReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UnBindSchemaReply.AsObject;
  static toObject(includeInstance: boolean, msg: UnBindSchemaReply): UnBindSchemaReply.AsObject;
  static serializeBinaryToWriter(message: UnBindSchemaReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UnBindSchemaReply;
  static deserializeBinaryFromReader(message: UnBindSchemaReply, reader: jspb.BinaryReader): UnBindSchemaReply;
}

export namespace UnBindSchemaReply {
  export type AsObject = {
  }
}

export enum ClusterType { 
  PLACEMENTCENTER = 0,
  JOURNALSERVER = 1,
  MQTTBROKERSERVER = 2,
  AMQPBROKERSERVER = 3,
}
