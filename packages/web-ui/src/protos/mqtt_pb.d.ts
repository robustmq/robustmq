import * as jspb from 'google-protobuf'



export class GetShareSubLeaderRequest extends jspb.Message {
  getGroupName(): string;
  setGroupName(value: string): GetShareSubLeaderRequest;

  getClusterName(): string;
  setClusterName(value: string): GetShareSubLeaderRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShareSubLeaderRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetShareSubLeaderRequest): GetShareSubLeaderRequest.AsObject;
  static serializeBinaryToWriter(message: GetShareSubLeaderRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShareSubLeaderRequest;
  static deserializeBinaryFromReader(message: GetShareSubLeaderRequest, reader: jspb.BinaryReader): GetShareSubLeaderRequest;
}

export namespace GetShareSubLeaderRequest {
  export type AsObject = {
    groupName: string,
    clusterName: string,
  }
}

export class GetShareSubLeaderReply extends jspb.Message {
  getBrokerId(): number;
  setBrokerId(value: number): GetShareSubLeaderReply;

  getBrokerAddr(): string;
  setBrokerAddr(value: string): GetShareSubLeaderReply;

  getExtendInfo(): string;
  setExtendInfo(value: string): GetShareSubLeaderReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetShareSubLeaderReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetShareSubLeaderReply): GetShareSubLeaderReply.AsObject;
  static serializeBinaryToWriter(message: GetShareSubLeaderReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetShareSubLeaderReply;
  static deserializeBinaryFromReader(message: GetShareSubLeaderReply, reader: jspb.BinaryReader): GetShareSubLeaderReply;
}

export namespace GetShareSubLeaderReply {
  export type AsObject = {
    brokerId: number,
    brokerAddr: string,
    extendInfo: string,
  }
}

export class ListUserRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListUserRequest;

  getUserName(): string;
  setUserName(value: string): ListUserRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListUserRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListUserRequest): ListUserRequest.AsObject;
  static serializeBinaryToWriter(message: ListUserRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListUserRequest;
  static deserializeBinaryFromReader(message: ListUserRequest, reader: jspb.BinaryReader): ListUserRequest;
}

export namespace ListUserRequest {
  export type AsObject = {
    clusterName: string,
    userName: string,
  }
}

export class ListUserReply extends jspb.Message {
  getUsersList(): Array<Uint8Array | string>;
  setUsersList(value: Array<Uint8Array | string>): ListUserReply;
  clearUsersList(): ListUserReply;
  addUsers(value: Uint8Array | string, index?: number): ListUserReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListUserReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListUserReply): ListUserReply.AsObject;
  static serializeBinaryToWriter(message: ListUserReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListUserReply;
  static deserializeBinaryFromReader(message: ListUserReply, reader: jspb.BinaryReader): ListUserReply;
}

export namespace ListUserReply {
  export type AsObject = {
    usersList: Array<Uint8Array | string>,
  }
}

export class CreateUserRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateUserRequest;

  getUserName(): string;
  setUserName(value: string): CreateUserRequest;

  getContent(): Uint8Array | string;
  getContent_asU8(): Uint8Array;
  getContent_asB64(): string;
  setContent(value: Uint8Array | string): CreateUserRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateUserRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateUserRequest): CreateUserRequest.AsObject;
  static serializeBinaryToWriter(message: CreateUserRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateUserRequest;
  static deserializeBinaryFromReader(message: CreateUserRequest, reader: jspb.BinaryReader): CreateUserRequest;
}

export namespace CreateUserRequest {
  export type AsObject = {
    clusterName: string,
    userName: string,
    content: Uint8Array | string,
  }
}

export class CreateUserReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateUserReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateUserReply): CreateUserReply.AsObject;
  static serializeBinaryToWriter(message: CreateUserReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateUserReply;
  static deserializeBinaryFromReader(message: CreateUserReply, reader: jspb.BinaryReader): CreateUserReply;
}

export namespace CreateUserReply {
  export type AsObject = {
  }
}

export class DeleteUserRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteUserRequest;

  getUserName(): string;
  setUserName(value: string): DeleteUserRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteUserRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteUserRequest): DeleteUserRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteUserRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteUserRequest;
  static deserializeBinaryFromReader(message: DeleteUserRequest, reader: jspb.BinaryReader): DeleteUserRequest;
}

export namespace DeleteUserRequest {
  export type AsObject = {
    clusterName: string,
    userName: string,
  }
}

export class DeleteUserReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteUserReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteUserReply): DeleteUserReply.AsObject;
  static serializeBinaryToWriter(message: DeleteUserReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteUserReply;
  static deserializeBinaryFromReader(message: DeleteUserReply, reader: jspb.BinaryReader): DeleteUserReply;
}

export namespace DeleteUserReply {
  export type AsObject = {
  }
}

export class ListTopicRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListTopicRequest;

  getTopicName(): string;
  setTopicName(value: string): ListTopicRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListTopicRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListTopicRequest): ListTopicRequest.AsObject;
  static serializeBinaryToWriter(message: ListTopicRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListTopicRequest;
  static deserializeBinaryFromReader(message: ListTopicRequest, reader: jspb.BinaryReader): ListTopicRequest;
}

export namespace ListTopicRequest {
  export type AsObject = {
    clusterName: string,
    topicName: string,
  }
}

export class ListTopicReply extends jspb.Message {
  getTopicsList(): Array<Uint8Array | string>;
  setTopicsList(value: Array<Uint8Array | string>): ListTopicReply;
  clearTopicsList(): ListTopicReply;
  addTopics(value: Uint8Array | string, index?: number): ListTopicReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListTopicReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListTopicReply): ListTopicReply.AsObject;
  static serializeBinaryToWriter(message: ListTopicReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListTopicReply;
  static deserializeBinaryFromReader(message: ListTopicReply, reader: jspb.BinaryReader): ListTopicReply;
}

export namespace ListTopicReply {
  export type AsObject = {
    topicsList: Array<Uint8Array | string>,
  }
}

export class CreateTopicRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateTopicRequest;

  getTopicName(): string;
  setTopicName(value: string): CreateTopicRequest;

  getContent(): Uint8Array | string;
  getContent_asU8(): Uint8Array;
  getContent_asB64(): string;
  setContent(value: Uint8Array | string): CreateTopicRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateTopicRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateTopicRequest): CreateTopicRequest.AsObject;
  static serializeBinaryToWriter(message: CreateTopicRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateTopicRequest;
  static deserializeBinaryFromReader(message: CreateTopicRequest, reader: jspb.BinaryReader): CreateTopicRequest;
}

export namespace CreateTopicRequest {
  export type AsObject = {
    clusterName: string,
    topicName: string,
    content: Uint8Array | string,
  }
}

export class CreateTopicReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateTopicReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateTopicReply): CreateTopicReply.AsObject;
  static serializeBinaryToWriter(message: CreateTopicReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateTopicReply;
  static deserializeBinaryFromReader(message: CreateTopicReply, reader: jspb.BinaryReader): CreateTopicReply;
}

export namespace CreateTopicReply {
  export type AsObject = {
  }
}

export class DeleteTopicRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteTopicRequest;

  getTopicName(): string;
  setTopicName(value: string): DeleteTopicRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteTopicRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteTopicRequest): DeleteTopicRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteTopicRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteTopicRequest;
  static deserializeBinaryFromReader(message: DeleteTopicRequest, reader: jspb.BinaryReader): DeleteTopicRequest;
}

export namespace DeleteTopicRequest {
  export type AsObject = {
    clusterName: string,
    topicName: string,
  }
}

export class DeleteTopicReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteTopicReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteTopicReply): DeleteTopicReply.AsObject;
  static serializeBinaryToWriter(message: DeleteTopicReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteTopicReply;
  static deserializeBinaryFromReader(message: DeleteTopicReply, reader: jspb.BinaryReader): DeleteTopicReply;
}

export namespace DeleteTopicReply {
  export type AsObject = {
  }
}

export class SetTopicRetainMessageRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SetTopicRetainMessageRequest;

  getTopicName(): string;
  setTopicName(value: string): SetTopicRetainMessageRequest;

  getRetainMessage(): Uint8Array | string;
  getRetainMessage_asU8(): Uint8Array;
  getRetainMessage_asB64(): string;
  setRetainMessage(value: Uint8Array | string): SetTopicRetainMessageRequest;

  getRetainMessageExpiredAt(): number;
  setRetainMessageExpiredAt(value: number): SetTopicRetainMessageRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetTopicRetainMessageRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetTopicRetainMessageRequest): SetTopicRetainMessageRequest.AsObject;
  static serializeBinaryToWriter(message: SetTopicRetainMessageRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetTopicRetainMessageRequest;
  static deserializeBinaryFromReader(message: SetTopicRetainMessageRequest, reader: jspb.BinaryReader): SetTopicRetainMessageRequest;
}

export namespace SetTopicRetainMessageRequest {
  export type AsObject = {
    clusterName: string,
    topicName: string,
    retainMessage: Uint8Array | string,
    retainMessageExpiredAt: number,
  }
}

export class SetTopicRetainMessageReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetTopicRetainMessageReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetTopicRetainMessageReply): SetTopicRetainMessageReply.AsObject;
  static serializeBinaryToWriter(message: SetTopicRetainMessageReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetTopicRetainMessageReply;
  static deserializeBinaryFromReader(message: SetTopicRetainMessageReply, reader: jspb.BinaryReader): SetTopicRetainMessageReply;
}

export namespace SetTopicRetainMessageReply {
  export type AsObject = {
  }
}

export class ListSessionRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListSessionRequest;

  getClientId(): string;
  setClientId(value: string): ListSessionRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSessionRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListSessionRequest): ListSessionRequest.AsObject;
  static serializeBinaryToWriter(message: ListSessionRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSessionRequest;
  static deserializeBinaryFromReader(message: ListSessionRequest, reader: jspb.BinaryReader): ListSessionRequest;
}

export namespace ListSessionRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
  }
}

export class ListSessionReply extends jspb.Message {
  getSessionsList(): Array<string>;
  setSessionsList(value: Array<string>): ListSessionReply;
  clearSessionsList(): ListSessionReply;
  addSessions(value: string, index?: number): ListSessionReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSessionReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListSessionReply): ListSessionReply.AsObject;
  static serializeBinaryToWriter(message: ListSessionReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSessionReply;
  static deserializeBinaryFromReader(message: ListSessionReply, reader: jspb.BinaryReader): ListSessionReply;
}

export namespace ListSessionReply {
  export type AsObject = {
    sessionsList: Array<string>,
  }
}

export class CreateSessionRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateSessionRequest;

  getClientId(): string;
  setClientId(value: string): CreateSessionRequest;

  getSession(): string;
  setSession(value: string): CreateSessionRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateSessionRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateSessionRequest): CreateSessionRequest.AsObject;
  static serializeBinaryToWriter(message: CreateSessionRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateSessionRequest;
  static deserializeBinaryFromReader(message: CreateSessionRequest, reader: jspb.BinaryReader): CreateSessionRequest;
}

export namespace CreateSessionRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
    session: string,
  }
}

export class CreateSessionReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateSessionReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateSessionReply): CreateSessionReply.AsObject;
  static serializeBinaryToWriter(message: CreateSessionReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateSessionReply;
  static deserializeBinaryFromReader(message: CreateSessionReply, reader: jspb.BinaryReader): CreateSessionReply;
}

export namespace CreateSessionReply {
  export type AsObject = {
  }
}

export class UpdateSessionRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): UpdateSessionRequest;

  getClientId(): string;
  setClientId(value: string): UpdateSessionRequest;

  getConnectionId(): number;
  setConnectionId(value: number): UpdateSessionRequest;

  getBrokerId(): number;
  setBrokerId(value: number): UpdateSessionRequest;

  getReconnectTime(): number;
  setReconnectTime(value: number): UpdateSessionRequest;

  getDistinctTime(): number;
  setDistinctTime(value: number): UpdateSessionRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSessionRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSessionRequest): UpdateSessionRequest.AsObject;
  static serializeBinaryToWriter(message: UpdateSessionRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSessionRequest;
  static deserializeBinaryFromReader(message: UpdateSessionRequest, reader: jspb.BinaryReader): UpdateSessionRequest;
}

export namespace UpdateSessionRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
    connectionId: number,
    brokerId: number,
    reconnectTime: number,
    distinctTime: number,
  }
}

export class UpdateSessionReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSessionReply.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSessionReply): UpdateSessionReply.AsObject;
  static serializeBinaryToWriter(message: UpdateSessionReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSessionReply;
  static deserializeBinaryFromReader(message: UpdateSessionReply, reader: jspb.BinaryReader): UpdateSessionReply;
}

export namespace UpdateSessionReply {
  export type AsObject = {
  }
}

export class DeleteSessionRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteSessionRequest;

  getClientId(): string;
  setClientId(value: string): DeleteSessionRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSessionRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSessionRequest): DeleteSessionRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteSessionRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSessionRequest;
  static deserializeBinaryFromReader(message: DeleteSessionRequest, reader: jspb.BinaryReader): DeleteSessionRequest;
}

export namespace DeleteSessionRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
  }
}

export class DeleteSessionReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSessionReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSessionReply): DeleteSessionReply.AsObject;
  static serializeBinaryToWriter(message: DeleteSessionReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSessionReply;
  static deserializeBinaryFromReader(message: DeleteSessionReply, reader: jspb.BinaryReader): DeleteSessionReply;
}

export namespace DeleteSessionReply {
  export type AsObject = {
  }
}

export class SaveLastWillMessageRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SaveLastWillMessageRequest;

  getClientId(): string;
  setClientId(value: string): SaveLastWillMessageRequest;

  getLastWillMessage(): Uint8Array | string;
  getLastWillMessage_asU8(): Uint8Array;
  getLastWillMessage_asB64(): string;
  setLastWillMessage(value: Uint8Array | string): SaveLastWillMessageRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SaveLastWillMessageRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SaveLastWillMessageRequest): SaveLastWillMessageRequest.AsObject;
  static serializeBinaryToWriter(message: SaveLastWillMessageRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SaveLastWillMessageRequest;
  static deserializeBinaryFromReader(message: SaveLastWillMessageRequest, reader: jspb.BinaryReader): SaveLastWillMessageRequest;
}

export namespace SaveLastWillMessageRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
    lastWillMessage: Uint8Array | string,
  }
}

export class SaveLastWillMessageReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SaveLastWillMessageReply.AsObject;
  static toObject(includeInstance: boolean, msg: SaveLastWillMessageReply): SaveLastWillMessageReply.AsObject;
  static serializeBinaryToWriter(message: SaveLastWillMessageReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SaveLastWillMessageReply;
  static deserializeBinaryFromReader(message: SaveLastWillMessageReply, reader: jspb.BinaryReader): SaveLastWillMessageReply;
}

export namespace SaveLastWillMessageReply {
  export type AsObject = {
  }
}

export class ListAclRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListAclRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListAclRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListAclRequest): ListAclRequest.AsObject;
  static serializeBinaryToWriter(message: ListAclRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListAclRequest;
  static deserializeBinaryFromReader(message: ListAclRequest, reader: jspb.BinaryReader): ListAclRequest;
}

export namespace ListAclRequest {
  export type AsObject = {
    clusterName: string,
  }
}

export class ListAclReply extends jspb.Message {
  getAclsList(): Array<Uint8Array | string>;
  setAclsList(value: Array<Uint8Array | string>): ListAclReply;
  clearAclsList(): ListAclReply;
  addAcls(value: Uint8Array | string, index?: number): ListAclReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListAclReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListAclReply): ListAclReply.AsObject;
  static serializeBinaryToWriter(message: ListAclReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListAclReply;
  static deserializeBinaryFromReader(message: ListAclReply, reader: jspb.BinaryReader): ListAclReply;
}

export namespace ListAclReply {
  export type AsObject = {
    aclsList: Array<Uint8Array | string>,
  }
}

export class DeleteAclRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteAclRequest;

  getAcl(): Uint8Array | string;
  getAcl_asU8(): Uint8Array;
  getAcl_asB64(): string;
  setAcl(value: Uint8Array | string): DeleteAclRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteAclRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteAclRequest): DeleteAclRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteAclRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteAclRequest;
  static deserializeBinaryFromReader(message: DeleteAclRequest, reader: jspb.BinaryReader): DeleteAclRequest;
}

export namespace DeleteAclRequest {
  export type AsObject = {
    clusterName: string,
    acl: Uint8Array | string,
  }
}

export class DeleteAclReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteAclReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteAclReply): DeleteAclReply.AsObject;
  static serializeBinaryToWriter(message: DeleteAclReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteAclReply;
  static deserializeBinaryFromReader(message: DeleteAclReply, reader: jspb.BinaryReader): DeleteAclReply;
}

export namespace DeleteAclReply {
  export type AsObject = {
  }
}

export class CreateAclRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateAclRequest;

  getAcl(): Uint8Array | string;
  getAcl_asU8(): Uint8Array;
  getAcl_asB64(): string;
  setAcl(value: Uint8Array | string): CreateAclRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateAclRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateAclRequest): CreateAclRequest.AsObject;
  static serializeBinaryToWriter(message: CreateAclRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateAclRequest;
  static deserializeBinaryFromReader(message: CreateAclRequest, reader: jspb.BinaryReader): CreateAclRequest;
}

export namespace CreateAclRequest {
  export type AsObject = {
    clusterName: string,
    acl: Uint8Array | string,
  }
}

export class CreateAclReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateAclReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateAclReply): CreateAclReply.AsObject;
  static serializeBinaryToWriter(message: CreateAclReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateAclReply;
  static deserializeBinaryFromReader(message: CreateAclReply, reader: jspb.BinaryReader): CreateAclReply;
}

export namespace CreateAclReply {
  export type AsObject = {
  }
}

export class ListBlacklistRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListBlacklistRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListBlacklistRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListBlacklistRequest): ListBlacklistRequest.AsObject;
  static serializeBinaryToWriter(message: ListBlacklistRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListBlacklistRequest;
  static deserializeBinaryFromReader(message: ListBlacklistRequest, reader: jspb.BinaryReader): ListBlacklistRequest;
}

export namespace ListBlacklistRequest {
  export type AsObject = {
    clusterName: string,
  }
}

export class ListBlacklistReply extends jspb.Message {
  getBlacklistsList(): Array<Uint8Array | string>;
  setBlacklistsList(value: Array<Uint8Array | string>): ListBlacklistReply;
  clearBlacklistsList(): ListBlacklistReply;
  addBlacklists(value: Uint8Array | string, index?: number): ListBlacklistReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListBlacklistReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListBlacklistReply): ListBlacklistReply.AsObject;
  static serializeBinaryToWriter(message: ListBlacklistReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListBlacklistReply;
  static deserializeBinaryFromReader(message: ListBlacklistReply, reader: jspb.BinaryReader): ListBlacklistReply;
}

export namespace ListBlacklistReply {
  export type AsObject = {
    blacklistsList: Array<Uint8Array | string>,
  }
}

export class CreateBlacklistRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateBlacklistRequest;

  getBlacklist(): Uint8Array | string;
  getBlacklist_asU8(): Uint8Array;
  getBlacklist_asB64(): string;
  setBlacklist(value: Uint8Array | string): CreateBlacklistRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateBlacklistRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateBlacklistRequest): CreateBlacklistRequest.AsObject;
  static serializeBinaryToWriter(message: CreateBlacklistRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateBlacklistRequest;
  static deserializeBinaryFromReader(message: CreateBlacklistRequest, reader: jspb.BinaryReader): CreateBlacklistRequest;
}

export namespace CreateBlacklistRequest {
  export type AsObject = {
    clusterName: string,
    blacklist: Uint8Array | string,
  }
}

export class CreateBlacklistReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateBlacklistReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateBlacklistReply): CreateBlacklistReply.AsObject;
  static serializeBinaryToWriter(message: CreateBlacklistReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateBlacklistReply;
  static deserializeBinaryFromReader(message: CreateBlacklistReply, reader: jspb.BinaryReader): CreateBlacklistReply;
}

export namespace CreateBlacklistReply {
  export type AsObject = {
  }
}

export class DeleteBlacklistRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteBlacklistRequest;

  getBlacklistType(): string;
  setBlacklistType(value: string): DeleteBlacklistRequest;

  getResourceName(): string;
  setResourceName(value: string): DeleteBlacklistRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteBlacklistRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteBlacklistRequest): DeleteBlacklistRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteBlacklistRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteBlacklistRequest;
  static deserializeBinaryFromReader(message: DeleteBlacklistRequest, reader: jspb.BinaryReader): DeleteBlacklistRequest;
}

export namespace DeleteBlacklistRequest {
  export type AsObject = {
    clusterName: string,
    blacklistType: string,
    resourceName: string,
  }
}

export class DeleteBlacklistReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteBlacklistReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteBlacklistReply): DeleteBlacklistReply.AsObject;
  static serializeBinaryToWriter(message: DeleteBlacklistReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteBlacklistReply;
  static deserializeBinaryFromReader(message: DeleteBlacklistReply, reader: jspb.BinaryReader): DeleteBlacklistReply;
}

export namespace DeleteBlacklistReply {
  export type AsObject = {
  }
}

export class CreateTopicRewriteRuleRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateTopicRewriteRuleRequest;

  getAction(): string;
  setAction(value: string): CreateTopicRewriteRuleRequest;

  getSourceTopic(): string;
  setSourceTopic(value: string): CreateTopicRewriteRuleRequest;

  getDestTopic(): string;
  setDestTopic(value: string): CreateTopicRewriteRuleRequest;

  getRegex(): string;
  setRegex(value: string): CreateTopicRewriteRuleRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateTopicRewriteRuleRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateTopicRewriteRuleRequest): CreateTopicRewriteRuleRequest.AsObject;
  static serializeBinaryToWriter(message: CreateTopicRewriteRuleRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateTopicRewriteRuleRequest;
  static deserializeBinaryFromReader(message: CreateTopicRewriteRuleRequest, reader: jspb.BinaryReader): CreateTopicRewriteRuleRequest;
}

export namespace CreateTopicRewriteRuleRequest {
  export type AsObject = {
    clusterName: string,
    action: string,
    sourceTopic: string,
    destTopic: string,
    regex: string,
  }
}

export class CreateTopicRewriteRuleReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateTopicRewriteRuleReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateTopicRewriteRuleReply): CreateTopicRewriteRuleReply.AsObject;
  static serializeBinaryToWriter(message: CreateTopicRewriteRuleReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateTopicRewriteRuleReply;
  static deserializeBinaryFromReader(message: CreateTopicRewriteRuleReply, reader: jspb.BinaryReader): CreateTopicRewriteRuleReply;
}

export namespace CreateTopicRewriteRuleReply {
  export type AsObject = {
  }
}

export class DeleteTopicRewriteRuleRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteTopicRewriteRuleRequest;

  getAction(): string;
  setAction(value: string): DeleteTopicRewriteRuleRequest;

  getSourceTopic(): string;
  setSourceTopic(value: string): DeleteTopicRewriteRuleRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteTopicRewriteRuleRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteTopicRewriteRuleRequest): DeleteTopicRewriteRuleRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteTopicRewriteRuleRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteTopicRewriteRuleRequest;
  static deserializeBinaryFromReader(message: DeleteTopicRewriteRuleRequest, reader: jspb.BinaryReader): DeleteTopicRewriteRuleRequest;
}

export namespace DeleteTopicRewriteRuleRequest {
  export type AsObject = {
    clusterName: string,
    action: string,
    sourceTopic: string,
  }
}

export class DeleteTopicRewriteRuleReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteTopicRewriteRuleReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteTopicRewriteRuleReply): DeleteTopicRewriteRuleReply.AsObject;
  static serializeBinaryToWriter(message: DeleteTopicRewriteRuleReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteTopicRewriteRuleReply;
  static deserializeBinaryFromReader(message: DeleteTopicRewriteRuleReply, reader: jspb.BinaryReader): DeleteTopicRewriteRuleReply;
}

export namespace DeleteTopicRewriteRuleReply {
  export type AsObject = {
  }
}

export class ListTopicRewriteRuleRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListTopicRewriteRuleRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListTopicRewriteRuleRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListTopicRewriteRuleRequest): ListTopicRewriteRuleRequest.AsObject;
  static serializeBinaryToWriter(message: ListTopicRewriteRuleRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListTopicRewriteRuleRequest;
  static deserializeBinaryFromReader(message: ListTopicRewriteRuleRequest, reader: jspb.BinaryReader): ListTopicRewriteRuleRequest;
}

export namespace ListTopicRewriteRuleRequest {
  export type AsObject = {
    clusterName: string,
  }
}

export class ListTopicRewriteRuleReply extends jspb.Message {
  getTopicRewriteRulesList(): Array<Uint8Array | string>;
  setTopicRewriteRulesList(value: Array<Uint8Array | string>): ListTopicRewriteRuleReply;
  clearTopicRewriteRulesList(): ListTopicRewriteRuleReply;
  addTopicRewriteRules(value: Uint8Array | string, index?: number): ListTopicRewriteRuleReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListTopicRewriteRuleReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListTopicRewriteRuleReply): ListTopicRewriteRuleReply.AsObject;
  static serializeBinaryToWriter(message: ListTopicRewriteRuleReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListTopicRewriteRuleReply;
  static deserializeBinaryFromReader(message: ListTopicRewriteRuleReply, reader: jspb.BinaryReader): ListTopicRewriteRuleReply;
}

export namespace ListTopicRewriteRuleReply {
  export type AsObject = {
    topicRewriteRulesList: Array<Uint8Array | string>,
  }
}

export class SetSubscribeRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SetSubscribeRequest;

  getClientId(): string;
  setClientId(value: string): SetSubscribeRequest;

  getPath(): string;
  setPath(value: string): SetSubscribeRequest;

  getSubscribe(): Uint8Array | string;
  getSubscribe_asU8(): Uint8Array;
  getSubscribe_asB64(): string;
  setSubscribe(value: Uint8Array | string): SetSubscribeRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetSubscribeRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetSubscribeRequest): SetSubscribeRequest.AsObject;
  static serializeBinaryToWriter(message: SetSubscribeRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetSubscribeRequest;
  static deserializeBinaryFromReader(message: SetSubscribeRequest, reader: jspb.BinaryReader): SetSubscribeRequest;
}

export namespace SetSubscribeRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
    path: string,
    subscribe: Uint8Array | string,
  }
}

export class SetSubscribeReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetSubscribeReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetSubscribeReply): SetSubscribeReply.AsObject;
  static serializeBinaryToWriter(message: SetSubscribeReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetSubscribeReply;
  static deserializeBinaryFromReader(message: SetSubscribeReply, reader: jspb.BinaryReader): SetSubscribeReply;
}

export namespace SetSubscribeReply {
  export type AsObject = {
  }
}

export class DeleteSubscribeRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteSubscribeRequest;

  getClientId(): string;
  setClientId(value: string): DeleteSubscribeRequest;

  getPath(): string;
  setPath(value: string): DeleteSubscribeRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSubscribeRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSubscribeRequest): DeleteSubscribeRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteSubscribeRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSubscribeRequest;
  static deserializeBinaryFromReader(message: DeleteSubscribeRequest, reader: jspb.BinaryReader): DeleteSubscribeRequest;
}

export namespace DeleteSubscribeRequest {
  export type AsObject = {
    clusterName: string,
    clientId: string,
    path: string,
  }
}

export class DeleteSubscribeReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteSubscribeReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteSubscribeReply): DeleteSubscribeReply.AsObject;
  static serializeBinaryToWriter(message: DeleteSubscribeReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteSubscribeReply;
  static deserializeBinaryFromReader(message: DeleteSubscribeReply, reader: jspb.BinaryReader): DeleteSubscribeReply;
}

export namespace DeleteSubscribeReply {
  export type AsObject = {
  }
}

export class ListSubscribeRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListSubscribeRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSubscribeRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListSubscribeRequest): ListSubscribeRequest.AsObject;
  static serializeBinaryToWriter(message: ListSubscribeRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSubscribeRequest;
  static deserializeBinaryFromReader(message: ListSubscribeRequest, reader: jspb.BinaryReader): ListSubscribeRequest;
}

export namespace ListSubscribeRequest {
  export type AsObject = {
    clusterName: string,
  }
}

export class ListSubscribeReply extends jspb.Message {
  getSubscribesList(): Array<Uint8Array | string>;
  setSubscribesList(value: Array<Uint8Array | string>): ListSubscribeReply;
  clearSubscribesList(): ListSubscribeReply;
  addSubscribes(value: Uint8Array | string, index?: number): ListSubscribeReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListSubscribeReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListSubscribeReply): ListSubscribeReply.AsObject;
  static serializeBinaryToWriter(message: ListSubscribeReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListSubscribeReply;
  static deserializeBinaryFromReader(message: ListSubscribeReply, reader: jspb.BinaryReader): ListSubscribeReply;
}

export namespace ListSubscribeReply {
  export type AsObject = {
    subscribesList: Array<Uint8Array | string>,
  }
}

export class ListConnectorRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListConnectorRequest;

  getConnectorName(): string;
  setConnectorName(value: string): ListConnectorRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListConnectorRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListConnectorRequest): ListConnectorRequest.AsObject;
  static serializeBinaryToWriter(message: ListConnectorRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListConnectorRequest;
  static deserializeBinaryFromReader(message: ListConnectorRequest, reader: jspb.BinaryReader): ListConnectorRequest;
}

export namespace ListConnectorRequest {
  export type AsObject = {
    clusterName: string,
    connectorName: string,
  }
}

export class ListConnectorReply extends jspb.Message {
  getConnectorsList(): Array<Uint8Array | string>;
  setConnectorsList(value: Array<Uint8Array | string>): ListConnectorReply;
  clearConnectorsList(): ListConnectorReply;
  addConnectors(value: Uint8Array | string, index?: number): ListConnectorReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListConnectorReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListConnectorReply): ListConnectorReply.AsObject;
  static serializeBinaryToWriter(message: ListConnectorReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListConnectorReply;
  static deserializeBinaryFromReader(message: ListConnectorReply, reader: jspb.BinaryReader): ListConnectorReply;
}

export namespace ListConnectorReply {
  export type AsObject = {
    connectorsList: Array<Uint8Array | string>,
  }
}

export class CreateConnectorRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): CreateConnectorRequest;

  getConnectorName(): string;
  setConnectorName(value: string): CreateConnectorRequest;

  getConnector(): Uint8Array | string;
  getConnector_asU8(): Uint8Array;
  getConnector_asB64(): string;
  setConnector(value: Uint8Array | string): CreateConnectorRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateConnectorRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateConnectorRequest): CreateConnectorRequest.AsObject;
  static serializeBinaryToWriter(message: CreateConnectorRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateConnectorRequest;
  static deserializeBinaryFromReader(message: CreateConnectorRequest, reader: jspb.BinaryReader): CreateConnectorRequest;
}

export namespace CreateConnectorRequest {
  export type AsObject = {
    clusterName: string,
    connectorName: string,
    connector: Uint8Array | string,
  }
}

export class CreateConnectorReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateConnectorReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateConnectorReply): CreateConnectorReply.AsObject;
  static serializeBinaryToWriter(message: CreateConnectorReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateConnectorReply;
  static deserializeBinaryFromReader(message: CreateConnectorReply, reader: jspb.BinaryReader): CreateConnectorReply;
}

export namespace CreateConnectorReply {
  export type AsObject = {
  }
}

export class UpdateConnectorRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): UpdateConnectorRequest;

  getConnectorName(): string;
  setConnectorName(value: string): UpdateConnectorRequest;

  getConnector(): Uint8Array | string;
  getConnector_asU8(): Uint8Array;
  getConnector_asB64(): string;
  setConnector(value: Uint8Array | string): UpdateConnectorRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateConnectorRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateConnectorRequest): UpdateConnectorRequest.AsObject;
  static serializeBinaryToWriter(message: UpdateConnectorRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateConnectorRequest;
  static deserializeBinaryFromReader(message: UpdateConnectorRequest, reader: jspb.BinaryReader): UpdateConnectorRequest;
}

export namespace UpdateConnectorRequest {
  export type AsObject = {
    clusterName: string,
    connectorName: string,
    connector: Uint8Array | string,
  }
}

export class UpdateConnectorReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateConnectorReply.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateConnectorReply): UpdateConnectorReply.AsObject;
  static serializeBinaryToWriter(message: UpdateConnectorReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateConnectorReply;
  static deserializeBinaryFromReader(message: UpdateConnectorReply, reader: jspb.BinaryReader): UpdateConnectorReply;
}

export namespace UpdateConnectorReply {
  export type AsObject = {
  }
}

export class DeleteConnectorRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteConnectorRequest;

  getConnectorName(): string;
  setConnectorName(value: string): DeleteConnectorRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteConnectorRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteConnectorRequest): DeleteConnectorRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteConnectorRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteConnectorRequest;
  static deserializeBinaryFromReader(message: DeleteConnectorRequest, reader: jspb.BinaryReader): DeleteConnectorRequest;
}

export namespace DeleteConnectorRequest {
  export type AsObject = {
    clusterName: string,
    connectorName: string,
  }
}

export class DeleteConnectorReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteConnectorReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteConnectorReply): DeleteConnectorReply.AsObject;
  static serializeBinaryToWriter(message: DeleteConnectorReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteConnectorReply;
  static deserializeBinaryFromReader(message: DeleteConnectorReply, reader: jspb.BinaryReader): DeleteConnectorReply;
}

export namespace DeleteConnectorReply {
  export type AsObject = {
  }
}

export class ConnectorHeartbeatRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ConnectorHeartbeatRequest;

  getHeatbeatsList(): Array<ConnectorHeartbeatRaw>;
  setHeatbeatsList(value: Array<ConnectorHeartbeatRaw>): ConnectorHeartbeatRequest;
  clearHeatbeatsList(): ConnectorHeartbeatRequest;
  addHeatbeats(value?: ConnectorHeartbeatRaw, index?: number): ConnectorHeartbeatRaw;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ConnectorHeartbeatRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ConnectorHeartbeatRequest): ConnectorHeartbeatRequest.AsObject;
  static serializeBinaryToWriter(message: ConnectorHeartbeatRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ConnectorHeartbeatRequest;
  static deserializeBinaryFromReader(message: ConnectorHeartbeatRequest, reader: jspb.BinaryReader): ConnectorHeartbeatRequest;
}

export namespace ConnectorHeartbeatRequest {
  export type AsObject = {
    clusterName: string,
    heatbeatsList: Array<ConnectorHeartbeatRaw.AsObject>,
  }
}

export class ConnectorHeartbeatRaw extends jspb.Message {
  getConnectorName(): string;
  setConnectorName(value: string): ConnectorHeartbeatRaw;

  getBrokerId(): number;
  setBrokerId(value: number): ConnectorHeartbeatRaw;

  getHeartbeatTime(): number;
  setHeartbeatTime(value: number): ConnectorHeartbeatRaw;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ConnectorHeartbeatRaw.AsObject;
  static toObject(includeInstance: boolean, msg: ConnectorHeartbeatRaw): ConnectorHeartbeatRaw.AsObject;
  static serializeBinaryToWriter(message: ConnectorHeartbeatRaw, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ConnectorHeartbeatRaw;
  static deserializeBinaryFromReader(message: ConnectorHeartbeatRaw, reader: jspb.BinaryReader): ConnectorHeartbeatRaw;
}

export namespace ConnectorHeartbeatRaw {
  export type AsObject = {
    connectorName: string,
    brokerId: number,
    heartbeatTime: number,
  }
}

export class ConnectorHeartbeatReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ConnectorHeartbeatReply.AsObject;
  static toObject(includeInstance: boolean, msg: ConnectorHeartbeatReply): ConnectorHeartbeatReply.AsObject;
  static serializeBinaryToWriter(message: ConnectorHeartbeatReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ConnectorHeartbeatReply;
  static deserializeBinaryFromReader(message: ConnectorHeartbeatReply, reader: jspb.BinaryReader): ConnectorHeartbeatReply;
}

export namespace ConnectorHeartbeatReply {
  export type AsObject = {
  }
}

export class SetAutoSubscribeRuleRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): SetAutoSubscribeRuleRequest;

  getTopic(): string;
  setTopic(value: string): SetAutoSubscribeRuleRequest;

  getQos(): number;
  setQos(value: number): SetAutoSubscribeRuleRequest;

  getNoLocal(): boolean;
  setNoLocal(value: boolean): SetAutoSubscribeRuleRequest;

  getRetainAsPublished(): boolean;
  setRetainAsPublished(value: boolean): SetAutoSubscribeRuleRequest;

  getRetainedHandling(): number;
  setRetainedHandling(value: number): SetAutoSubscribeRuleRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetAutoSubscribeRuleRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetAutoSubscribeRuleRequest): SetAutoSubscribeRuleRequest.AsObject;
  static serializeBinaryToWriter(message: SetAutoSubscribeRuleRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetAutoSubscribeRuleRequest;
  static deserializeBinaryFromReader(message: SetAutoSubscribeRuleRequest, reader: jspb.BinaryReader): SetAutoSubscribeRuleRequest;
}

export namespace SetAutoSubscribeRuleRequest {
  export type AsObject = {
    clusterName: string,
    topic: string,
    qos: number,
    noLocal: boolean,
    retainAsPublished: boolean,
    retainedHandling: number,
  }
}

export class SetAutoSubscribeRuleReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetAutoSubscribeRuleReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetAutoSubscribeRuleReply): SetAutoSubscribeRuleReply.AsObject;
  static serializeBinaryToWriter(message: SetAutoSubscribeRuleReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetAutoSubscribeRuleReply;
  static deserializeBinaryFromReader(message: SetAutoSubscribeRuleReply, reader: jspb.BinaryReader): SetAutoSubscribeRuleReply;
}

export namespace SetAutoSubscribeRuleReply {
  export type AsObject = {
  }
}

export class DeleteAutoSubscribeRuleRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): DeleteAutoSubscribeRuleRequest;

  getTopic(): string;
  setTopic(value: string): DeleteAutoSubscribeRuleRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteAutoSubscribeRuleRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteAutoSubscribeRuleRequest): DeleteAutoSubscribeRuleRequest.AsObject;
  static serializeBinaryToWriter(message: DeleteAutoSubscribeRuleRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteAutoSubscribeRuleRequest;
  static deserializeBinaryFromReader(message: DeleteAutoSubscribeRuleRequest, reader: jspb.BinaryReader): DeleteAutoSubscribeRuleRequest;
}

export namespace DeleteAutoSubscribeRuleRequest {
  export type AsObject = {
    clusterName: string,
    topic: string,
  }
}

export class DeleteAutoSubscribeRuleReply extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteAutoSubscribeRuleReply.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteAutoSubscribeRuleReply): DeleteAutoSubscribeRuleReply.AsObject;
  static serializeBinaryToWriter(message: DeleteAutoSubscribeRuleReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteAutoSubscribeRuleReply;
  static deserializeBinaryFromReader(message: DeleteAutoSubscribeRuleReply, reader: jspb.BinaryReader): DeleteAutoSubscribeRuleReply;
}

export namespace DeleteAutoSubscribeRuleReply {
  export type AsObject = {
  }
}

export class ListAutoSubscribeRuleRequest extends jspb.Message {
  getClusterName(): string;
  setClusterName(value: string): ListAutoSubscribeRuleRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListAutoSubscribeRuleRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ListAutoSubscribeRuleRequest): ListAutoSubscribeRuleRequest.AsObject;
  static serializeBinaryToWriter(message: ListAutoSubscribeRuleRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListAutoSubscribeRuleRequest;
  static deserializeBinaryFromReader(message: ListAutoSubscribeRuleRequest, reader: jspb.BinaryReader): ListAutoSubscribeRuleRequest;
}

export namespace ListAutoSubscribeRuleRequest {
  export type AsObject = {
    clusterName: string,
  }
}

export class ListAutoSubscribeRuleReply extends jspb.Message {
  getAutoSubscribeRulesList(): Array<Uint8Array | string>;
  setAutoSubscribeRulesList(value: Array<Uint8Array | string>): ListAutoSubscribeRuleReply;
  clearAutoSubscribeRulesList(): ListAutoSubscribeRuleReply;
  addAutoSubscribeRules(value: Uint8Array | string, index?: number): ListAutoSubscribeRuleReply;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ListAutoSubscribeRuleReply.AsObject;
  static toObject(includeInstance: boolean, msg: ListAutoSubscribeRuleReply): ListAutoSubscribeRuleReply.AsObject;
  static serializeBinaryToWriter(message: ListAutoSubscribeRuleReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ListAutoSubscribeRuleReply;
  static deserializeBinaryFromReader(message: ListAutoSubscribeRuleReply, reader: jspb.BinaryReader): ListAutoSubscribeRuleReply;
}

export namespace ListAutoSubscribeRuleReply {
  export type AsObject = {
    autoSubscribeRulesList: Array<Uint8Array | string>,
  }
}

