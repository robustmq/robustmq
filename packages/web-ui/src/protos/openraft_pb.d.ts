import * as jspb from 'google-protobuf'



export class VoteRequest extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): VoteRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): VoteRequest.AsObject;
  static toObject(includeInstance: boolean, msg: VoteRequest): VoteRequest.AsObject;
  static serializeBinaryToWriter(message: VoteRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): VoteRequest;
  static deserializeBinaryFromReader(message: VoteRequest, reader: jspb.BinaryReader): VoteRequest;
}

export namespace VoteRequest {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class VoteReply extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): VoteReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): VoteReply.AsObject;
  static toObject(includeInstance: boolean, msg: VoteReply): VoteReply.AsObject;
  static serializeBinaryToWriter(message: VoteReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): VoteReply;
  static deserializeBinaryFromReader(message: VoteReply, reader: jspb.BinaryReader): VoteReply;
}

export namespace VoteReply {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class AppendRequest extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): AppendRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AppendRequest.AsObject;
  static toObject(includeInstance: boolean, msg: AppendRequest): AppendRequest.AsObject;
  static serializeBinaryToWriter(message: AppendRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AppendRequest;
  static deserializeBinaryFromReader(message: AppendRequest, reader: jspb.BinaryReader): AppendRequest;
}

export namespace AppendRequest {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class AppendReply extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): AppendReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AppendReply.AsObject;
  static toObject(includeInstance: boolean, msg: AppendReply): AppendReply.AsObject;
  static serializeBinaryToWriter(message: AppendReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AppendReply;
  static deserializeBinaryFromReader(message: AppendReply, reader: jspb.BinaryReader): AppendReply;
}

export namespace AppendReply {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class SnapshotRequest extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): SnapshotRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SnapshotRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SnapshotRequest): SnapshotRequest.AsObject;
  static serializeBinaryToWriter(message: SnapshotRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SnapshotRequest;
  static deserializeBinaryFromReader(message: SnapshotRequest, reader: jspb.BinaryReader): SnapshotRequest;
}

export namespace SnapshotRequest {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class SnapshotReply extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): SnapshotReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SnapshotReply.AsObject;
  static toObject(includeInstance: boolean, msg: SnapshotReply): SnapshotReply.AsObject;
  static serializeBinaryToWriter(message: SnapshotReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SnapshotReply;
  static deserializeBinaryFromReader(message: SnapshotReply, reader: jspb.BinaryReader): SnapshotReply;
}

export namespace SnapshotReply {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class AddLearnerRequest extends jspb.Message {
  getNodeId(): number;
  setNodeId(value: number): AddLearnerRequest;

  getNode(): Node | undefined;
  setNode(value?: Node): AddLearnerRequest;
  hasNode(): boolean;
  clearNode(): AddLearnerRequest;

  getBlocking(): boolean;
  setBlocking(value: boolean): AddLearnerRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AddLearnerRequest.AsObject;
  static toObject(includeInstance: boolean, msg: AddLearnerRequest): AddLearnerRequest.AsObject;
  static serializeBinaryToWriter(message: AddLearnerRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AddLearnerRequest;
  static deserializeBinaryFromReader(message: AddLearnerRequest, reader: jspb.BinaryReader): AddLearnerRequest;
}

export namespace AddLearnerRequest {
  export type AsObject = {
    nodeId: number,
    node?: Node.AsObject,
    blocking: boolean,
  }
}

export class Node extends jspb.Message {
  getRpcAddr(): string;
  setRpcAddr(value: string): Node;

  getNodeId(): number;
  setNodeId(value: number): Node;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Node.AsObject;
  static toObject(includeInstance: boolean, msg: Node): Node.AsObject;
  static serializeBinaryToWriter(message: Node, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Node;
  static deserializeBinaryFromReader(message: Node, reader: jspb.BinaryReader): Node;
}

export namespace Node {
  export type AsObject = {
    rpcAddr: string,
    nodeId: number,
  }
}

export class AddLearnerReply extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): AddLearnerReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AddLearnerReply.AsObject;
  static toObject(includeInstance: boolean, msg: AddLearnerReply): AddLearnerReply.AsObject;
  static serializeBinaryToWriter(message: AddLearnerReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AddLearnerReply;
  static deserializeBinaryFromReader(message: AddLearnerReply, reader: jspb.BinaryReader): AddLearnerReply;
}

export namespace AddLearnerReply {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

export class ChangeMembershipRequest extends jspb.Message {
  getMembersList(): Array<number>;
  setMembersList(value: Array<number>): ChangeMembershipRequest;
  clearMembersList(): ChangeMembershipRequest;
  addMembers(value: number, index?: number): ChangeMembershipRequest;

  getRetain(): boolean;
  setRetain(value: boolean): ChangeMembershipRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ChangeMembershipRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ChangeMembershipRequest): ChangeMembershipRequest.AsObject;
  static serializeBinaryToWriter(message: ChangeMembershipRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ChangeMembershipRequest;
  static deserializeBinaryFromReader(message: ChangeMembershipRequest, reader: jspb.BinaryReader): ChangeMembershipRequest;
}

export namespace ChangeMembershipRequest {
  export type AsObject = {
    membersList: Array<number>,
    retain: boolean,
  }
}

export class ChangeMembershipReply extends jspb.Message {
  getValue(): Uint8Array | string;
  getValue_asU8(): Uint8Array;
  getValue_asB64(): string;
  setValue(value: Uint8Array | string): ChangeMembershipReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ChangeMembershipReply.AsObject;
  static toObject(includeInstance: boolean, msg: ChangeMembershipReply): ChangeMembershipReply.AsObject;
  static serializeBinaryToWriter(message: ChangeMembershipReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ChangeMembershipReply;
  static deserializeBinaryFromReader(message: ChangeMembershipReply, reader: jspb.BinaryReader): ChangeMembershipReply;
}

export namespace ChangeMembershipReply {
  export type AsObject = {
    value: Uint8Array | string,
  }
}

