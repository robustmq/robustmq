// source: record.proto
/**
 * @fileoverview
 * @enhanceable
 * @suppress {missingRequire} reports error on implicit type usages.
 * @suppress {messageConventions} JS Compiler reports an error if a variable or
 *     field starts with 'MSG_' and isn't a translatable message.
 * @public
 */
// GENERATED CODE -- DO NOT EDIT!
/* eslint-disable */
// @ts-nocheck

var jspb = require('google-protobuf');
var goog = jspb;
var global =
    (typeof globalThis !== 'undefined' && globalThis) ||
    (typeof window !== 'undefined' && window) ||
    (typeof global !== 'undefined' && global) ||
    (typeof self !== 'undefined' && self) ||
    (function () { return this; }).call(null) ||
    Function('return this')();

goog.exportSymbol('proto.journal.record.JournalRecord', null, global);
/**
 * Generated by JsPbCodeGenerator.
 * @param {Array=} opt_data Optional initial data array, typically from a
 * server response, or constructed directly in Javascript. The array is used
 * in place and becomes part of the constructed object. It is not cloned.
 * If no data is provided, the constructed object will be empty, but still
 * valid.
 * @extends {jspb.Message}
 * @constructor
 */
proto.journal.record.JournalRecord = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.journal.record.JournalRecord.repeatedFields_, null);
};
goog.inherits(proto.journal.record.JournalRecord, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.journal.record.JournalRecord.displayName = 'proto.journal.record.JournalRecord';
}

/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.journal.record.JournalRecord.repeatedFields_ = [6];



if (jspb.Message.GENERATE_TO_OBJECT) {
/**
 * Creates an object representation of this proto.
 * Field names that are reserved in JavaScript and will be renamed to pb_name.
 * Optional fields that are not set will be set to undefined.
 * To access a reserved field use, foo.pb_<name>, eg, foo.pb_default.
 * For the list of reserved names please see:
 *     net/proto2/compiler/js/internal/generator.cc#kKeyword.
 * @param {boolean=} opt_includeInstance Deprecated. whether to include the
 *     JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @return {!Object}
 */
proto.journal.record.JournalRecord.prototype.toObject = function(opt_includeInstance) {
  return proto.journal.record.JournalRecord.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.journal.record.JournalRecord} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.journal.record.JournalRecord.toObject = function(includeInstance, msg) {
  var f, obj = {
producerId: jspb.Message.getFieldWithDefault(msg, 1, ""),
pkid: jspb.Message.getFieldWithDefault(msg, 2, 0),
key: jspb.Message.getFieldWithDefault(msg, 3, ""),
content: msg.getContent_asB64(),
createTime: jspb.Message.getFieldWithDefault(msg, 5, 0),
tagsList: (f = jspb.Message.getRepeatedField(msg, 6)) == null ? undefined : f,
offset: jspb.Message.getFieldWithDefault(msg, 7, 0),
namespace: jspb.Message.getFieldWithDefault(msg, 8, ""),
shardName: jspb.Message.getFieldWithDefault(msg, 9, ""),
segment: jspb.Message.getFieldWithDefault(msg, 10, 0)
  };

  if (includeInstance) {
    obj.$jspbMessageInstance = msg;
  }
  return obj;
};
}


/**
 * Deserializes binary data (in protobuf wire format).
 * @param {jspb.ByteSource} bytes The bytes to deserialize.
 * @return {!proto.journal.record.JournalRecord}
 */
proto.journal.record.JournalRecord.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.journal.record.JournalRecord;
  return proto.journal.record.JournalRecord.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.journal.record.JournalRecord} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.journal.record.JournalRecord}
 */
proto.journal.record.JournalRecord.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setProducerId(value);
      break;
    case 2:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setPkid(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setKey(value);
      break;
    case 4:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setContent(value);
      break;
    case 5:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setCreateTime(value);
      break;
    case 6:
      var value = /** @type {string} */ (reader.readString());
      msg.addTags(value);
      break;
    case 7:
      var value = /** @type {number} */ (reader.readInt64());
      msg.setOffset(value);
      break;
    case 8:
      var value = /** @type {string} */ (reader.readString());
      msg.setNamespace(value);
      break;
    case 9:
      var value = /** @type {string} */ (reader.readString());
      msg.setShardName(value);
      break;
    case 10:
      var value = /** @type {number} */ (reader.readUint32());
      msg.setSegment(value);
      break;
    default:
      reader.skipField();
      break;
    }
  }
  return msg;
};


/**
 * Serializes the message to binary data (in protobuf wire format).
 * @return {!Uint8Array}
 */
proto.journal.record.JournalRecord.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.journal.record.JournalRecord.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.journal.record.JournalRecord} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.journal.record.JournalRecord.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getProducerId();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getPkid();
  if (f !== 0) {
    writer.writeUint64(
      2,
      f
    );
  }
  f = message.getKey();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
  f = message.getContent_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      4,
      f
    );
  }
  f = message.getCreateTime();
  if (f !== 0) {
    writer.writeUint64(
      5,
      f
    );
  }
  f = message.getTagsList();
  if (f.length > 0) {
    writer.writeRepeatedString(
      6,
      f
    );
  }
  f = message.getOffset();
  if (f !== 0) {
    writer.writeInt64(
      7,
      f
    );
  }
  f = message.getNamespace();
  if (f.length > 0) {
    writer.writeString(
      8,
      f
    );
  }
  f = message.getShardName();
  if (f.length > 0) {
    writer.writeString(
      9,
      f
    );
  }
  f = message.getSegment();
  if (f !== 0) {
    writer.writeUint32(
      10,
      f
    );
  }
};


/**
 * optional string producer_id = 1;
 * @return {string}
 */
proto.journal.record.JournalRecord.prototype.getProducerId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setProducerId = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional uint64 pkid = 2;
 * @return {number}
 */
proto.journal.record.JournalRecord.prototype.getPkid = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 2, 0));
};


/**
 * @param {number} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setPkid = function(value) {
  return jspb.Message.setProto3IntField(this, 2, value);
};


/**
 * optional string key = 3;
 * @return {string}
 */
proto.journal.record.JournalRecord.prototype.getKey = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setKey = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};


/**
 * optional bytes content = 4;
 * @return {string}
 */
proto.journal.record.JournalRecord.prototype.getContent = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 4, ""));
};


/**
 * optional bytes content = 4;
 * This is a type-conversion wrapper around `getContent()`
 * @return {string}
 */
proto.journal.record.JournalRecord.prototype.getContent_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getContent()));
};


/**
 * optional bytes content = 4;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getContent()`
 * @return {!Uint8Array}
 */
proto.journal.record.JournalRecord.prototype.getContent_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getContent()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setContent = function(value) {
  return jspb.Message.setProto3BytesField(this, 4, value);
};


/**
 * optional uint64 create_time = 5;
 * @return {number}
 */
proto.journal.record.JournalRecord.prototype.getCreateTime = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 5, 0));
};


/**
 * @param {number} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setCreateTime = function(value) {
  return jspb.Message.setProto3IntField(this, 5, value);
};


/**
 * repeated string tags = 6;
 * @return {!Array<string>}
 */
proto.journal.record.JournalRecord.prototype.getTagsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 6));
};


/**
 * @param {!Array<string>} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setTagsList = function(value) {
  return jspb.Message.setField(this, 6, value || []);
};


/**
 * @param {string} value
 * @param {number=} opt_index
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.addTags = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 6, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.clearTagsList = function() {
  return this.setTagsList([]);
};


/**
 * optional int64 offset = 7;
 * @return {number}
 */
proto.journal.record.JournalRecord.prototype.getOffset = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 7, 0));
};


/**
 * @param {number} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setOffset = function(value) {
  return jspb.Message.setProto3IntField(this, 7, value);
};


/**
 * optional string namespace = 8;
 * @return {string}
 */
proto.journal.record.JournalRecord.prototype.getNamespace = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 8, ""));
};


/**
 * @param {string} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setNamespace = function(value) {
  return jspb.Message.setProto3StringField(this, 8, value);
};


/**
 * optional string shard_name = 9;
 * @return {string}
 */
proto.journal.record.JournalRecord.prototype.getShardName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 9, ""));
};


/**
 * @param {string} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setShardName = function(value) {
  return jspb.Message.setProto3StringField(this, 9, value);
};


/**
 * optional uint32 segment = 10;
 * @return {number}
 */
proto.journal.record.JournalRecord.prototype.getSegment = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 10, 0));
};


/**
 * @param {number} value
 * @return {!proto.journal.record.JournalRecord} returns this
 */
proto.journal.record.JournalRecord.prototype.setSegment = function(value) {
  return jspb.Message.setProto3IntField(this, 10, value);
};


goog.object.extend(exports, proto.journal.record);
