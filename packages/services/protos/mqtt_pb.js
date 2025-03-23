// source: mqtt.proto
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

goog.exportSymbol('proto.placement.center.mqtt.ConnectorHeartbeatRaw', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ConnectorHeartbeatReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ConnectorHeartbeatRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateAclReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateAclRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateBlacklistReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateBlacklistRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateConnectorReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateConnectorRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateSessionReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateSessionRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateTopicReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateTopicRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateTopicRewriteRuleReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateTopicRewriteRuleRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateUserReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.CreateUserRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteAclReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteAclRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteBlacklistReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteBlacklistRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteConnectorReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteConnectorRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteSessionReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteSessionRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteSubscribeReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteSubscribeRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteTopicReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteTopicRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteTopicRewriteRuleReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteUserReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.DeleteUserRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.GetShareSubLeaderReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.GetShareSubLeaderRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListAclReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListAclRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListAutoSubscribeRuleReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListAutoSubscribeRuleRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListBlacklistReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListBlacklistRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListConnectorReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListConnectorRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListSessionReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListSessionRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListSubscribeReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListSubscribeRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListTopicReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListTopicRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListTopicRewriteRuleReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListTopicRewriteRuleRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListUserReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.ListUserRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SaveLastWillMessageReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SaveLastWillMessageRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SetAutoSubscribeRuleReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SetAutoSubscribeRuleRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SetSubscribeReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SetSubscribeRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SetTopicRetainMessageReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.SetTopicRetainMessageRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.UpdateConnectorReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.UpdateConnectorRequest', null, global);
goog.exportSymbol('proto.placement.center.mqtt.UpdateSessionReply', null, global);
goog.exportSymbol('proto.placement.center.mqtt.UpdateSessionRequest', null, global);
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
proto.placement.center.mqtt.GetShareSubLeaderRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.GetShareSubLeaderRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.GetShareSubLeaderRequest.displayName = 'proto.placement.center.mqtt.GetShareSubLeaderRequest';
}
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
proto.placement.center.mqtt.GetShareSubLeaderReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.GetShareSubLeaderReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.GetShareSubLeaderReply.displayName = 'proto.placement.center.mqtt.GetShareSubLeaderReply';
}
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
proto.placement.center.mqtt.ListUserRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListUserRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListUserRequest.displayName = 'proto.placement.center.mqtt.ListUserRequest';
}
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
proto.placement.center.mqtt.ListUserReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListUserReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListUserReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListUserReply.displayName = 'proto.placement.center.mqtt.ListUserReply';
}
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
proto.placement.center.mqtt.CreateUserRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateUserRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateUserRequest.displayName = 'proto.placement.center.mqtt.CreateUserRequest';
}
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
proto.placement.center.mqtt.CreateUserReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateUserReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateUserReply.displayName = 'proto.placement.center.mqtt.CreateUserReply';
}
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
proto.placement.center.mqtt.DeleteUserRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteUserRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteUserRequest.displayName = 'proto.placement.center.mqtt.DeleteUserRequest';
}
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
proto.placement.center.mqtt.DeleteUserReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteUserReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteUserReply.displayName = 'proto.placement.center.mqtt.DeleteUserReply';
}
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
proto.placement.center.mqtt.ListTopicRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListTopicRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListTopicRequest.displayName = 'proto.placement.center.mqtt.ListTopicRequest';
}
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
proto.placement.center.mqtt.ListTopicReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListTopicReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListTopicReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListTopicReply.displayName = 'proto.placement.center.mqtt.ListTopicReply';
}
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
proto.placement.center.mqtt.CreateTopicRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateTopicRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateTopicRequest.displayName = 'proto.placement.center.mqtt.CreateTopicRequest';
}
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
proto.placement.center.mqtt.CreateTopicReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateTopicReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateTopicReply.displayName = 'proto.placement.center.mqtt.CreateTopicReply';
}
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
proto.placement.center.mqtt.DeleteTopicRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteTopicRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteTopicRequest.displayName = 'proto.placement.center.mqtt.DeleteTopicRequest';
}
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
proto.placement.center.mqtt.DeleteTopicReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteTopicReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteTopicReply.displayName = 'proto.placement.center.mqtt.DeleteTopicReply';
}
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
proto.placement.center.mqtt.SetTopicRetainMessageRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SetTopicRetainMessageRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SetTopicRetainMessageRequest.displayName = 'proto.placement.center.mqtt.SetTopicRetainMessageRequest';
}
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
proto.placement.center.mqtt.SetTopicRetainMessageReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SetTopicRetainMessageReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SetTopicRetainMessageReply.displayName = 'proto.placement.center.mqtt.SetTopicRetainMessageReply';
}
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
proto.placement.center.mqtt.ListSessionRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListSessionRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListSessionRequest.displayName = 'proto.placement.center.mqtt.ListSessionRequest';
}
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
proto.placement.center.mqtt.ListSessionReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListSessionReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListSessionReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListSessionReply.displayName = 'proto.placement.center.mqtt.ListSessionReply';
}
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
proto.placement.center.mqtt.CreateSessionRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateSessionRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateSessionRequest.displayName = 'proto.placement.center.mqtt.CreateSessionRequest';
}
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
proto.placement.center.mqtt.CreateSessionReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateSessionReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateSessionReply.displayName = 'proto.placement.center.mqtt.CreateSessionReply';
}
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
proto.placement.center.mqtt.UpdateSessionRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.UpdateSessionRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.UpdateSessionRequest.displayName = 'proto.placement.center.mqtt.UpdateSessionRequest';
}
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
proto.placement.center.mqtt.UpdateSessionReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.UpdateSessionReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.UpdateSessionReply.displayName = 'proto.placement.center.mqtt.UpdateSessionReply';
}
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
proto.placement.center.mqtt.DeleteSessionRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteSessionRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteSessionRequest.displayName = 'proto.placement.center.mqtt.DeleteSessionRequest';
}
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
proto.placement.center.mqtt.DeleteSessionReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteSessionReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteSessionReply.displayName = 'proto.placement.center.mqtt.DeleteSessionReply';
}
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
proto.placement.center.mqtt.SaveLastWillMessageRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SaveLastWillMessageRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SaveLastWillMessageRequest.displayName = 'proto.placement.center.mqtt.SaveLastWillMessageRequest';
}
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
proto.placement.center.mqtt.SaveLastWillMessageReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SaveLastWillMessageReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SaveLastWillMessageReply.displayName = 'proto.placement.center.mqtt.SaveLastWillMessageReply';
}
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
proto.placement.center.mqtt.ListAclRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListAclRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListAclRequest.displayName = 'proto.placement.center.mqtt.ListAclRequest';
}
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
proto.placement.center.mqtt.ListAclReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListAclReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListAclReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListAclReply.displayName = 'proto.placement.center.mqtt.ListAclReply';
}
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
proto.placement.center.mqtt.DeleteAclRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteAclRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteAclRequest.displayName = 'proto.placement.center.mqtt.DeleteAclRequest';
}
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
proto.placement.center.mqtt.DeleteAclReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteAclReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteAclReply.displayName = 'proto.placement.center.mqtt.DeleteAclReply';
}
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
proto.placement.center.mqtt.CreateAclRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateAclRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateAclRequest.displayName = 'proto.placement.center.mqtt.CreateAclRequest';
}
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
proto.placement.center.mqtt.CreateAclReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateAclReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateAclReply.displayName = 'proto.placement.center.mqtt.CreateAclReply';
}
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
proto.placement.center.mqtt.ListBlacklistRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListBlacklistRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListBlacklistRequest.displayName = 'proto.placement.center.mqtt.ListBlacklistRequest';
}
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
proto.placement.center.mqtt.ListBlacklistReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListBlacklistReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListBlacklistReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListBlacklistReply.displayName = 'proto.placement.center.mqtt.ListBlacklistReply';
}
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
proto.placement.center.mqtt.CreateBlacklistRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateBlacklistRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateBlacklistRequest.displayName = 'proto.placement.center.mqtt.CreateBlacklistRequest';
}
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
proto.placement.center.mqtt.CreateBlacklistReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateBlacklistReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateBlacklistReply.displayName = 'proto.placement.center.mqtt.CreateBlacklistReply';
}
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
proto.placement.center.mqtt.DeleteBlacklistRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteBlacklistRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteBlacklistRequest.displayName = 'proto.placement.center.mqtt.DeleteBlacklistRequest';
}
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
proto.placement.center.mqtt.DeleteBlacklistReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteBlacklistReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteBlacklistReply.displayName = 'proto.placement.center.mqtt.DeleteBlacklistReply';
}
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
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateTopicRewriteRuleRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.displayName = 'proto.placement.center.mqtt.CreateTopicRewriteRuleRequest';
}
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
proto.placement.center.mqtt.CreateTopicRewriteRuleReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateTopicRewriteRuleReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateTopicRewriteRuleReply.displayName = 'proto.placement.center.mqtt.CreateTopicRewriteRuleReply';
}
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
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.displayName = 'proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest';
}
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
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteTopicRewriteRuleReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.displayName = 'proto.placement.center.mqtt.DeleteTopicRewriteRuleReply';
}
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
proto.placement.center.mqtt.ListTopicRewriteRuleRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListTopicRewriteRuleRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListTopicRewriteRuleRequest.displayName = 'proto.placement.center.mqtt.ListTopicRewriteRuleRequest';
}
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
proto.placement.center.mqtt.ListTopicRewriteRuleReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListTopicRewriteRuleReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListTopicRewriteRuleReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListTopicRewriteRuleReply.displayName = 'proto.placement.center.mqtt.ListTopicRewriteRuleReply';
}
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
proto.placement.center.mqtt.SetSubscribeRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SetSubscribeRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SetSubscribeRequest.displayName = 'proto.placement.center.mqtt.SetSubscribeRequest';
}
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
proto.placement.center.mqtt.SetSubscribeReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SetSubscribeReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SetSubscribeReply.displayName = 'proto.placement.center.mqtt.SetSubscribeReply';
}
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
proto.placement.center.mqtt.DeleteSubscribeRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteSubscribeRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteSubscribeRequest.displayName = 'proto.placement.center.mqtt.DeleteSubscribeRequest';
}
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
proto.placement.center.mqtt.DeleteSubscribeReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteSubscribeReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteSubscribeReply.displayName = 'proto.placement.center.mqtt.DeleteSubscribeReply';
}
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
proto.placement.center.mqtt.ListSubscribeRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListSubscribeRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListSubscribeRequest.displayName = 'proto.placement.center.mqtt.ListSubscribeRequest';
}
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
proto.placement.center.mqtt.ListSubscribeReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListSubscribeReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListSubscribeReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListSubscribeReply.displayName = 'proto.placement.center.mqtt.ListSubscribeReply';
}
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
proto.placement.center.mqtt.ListConnectorRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListConnectorRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListConnectorRequest.displayName = 'proto.placement.center.mqtt.ListConnectorRequest';
}
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
proto.placement.center.mqtt.ListConnectorReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListConnectorReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListConnectorReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListConnectorReply.displayName = 'proto.placement.center.mqtt.ListConnectorReply';
}
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
proto.placement.center.mqtt.CreateConnectorRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateConnectorRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateConnectorRequest.displayName = 'proto.placement.center.mqtt.CreateConnectorRequest';
}
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
proto.placement.center.mqtt.CreateConnectorReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.CreateConnectorReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.CreateConnectorReply.displayName = 'proto.placement.center.mqtt.CreateConnectorReply';
}
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
proto.placement.center.mqtt.UpdateConnectorRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.UpdateConnectorRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.UpdateConnectorRequest.displayName = 'proto.placement.center.mqtt.UpdateConnectorRequest';
}
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
proto.placement.center.mqtt.UpdateConnectorReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.UpdateConnectorReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.UpdateConnectorReply.displayName = 'proto.placement.center.mqtt.UpdateConnectorReply';
}
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
proto.placement.center.mqtt.DeleteConnectorRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteConnectorRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteConnectorRequest.displayName = 'proto.placement.center.mqtt.DeleteConnectorRequest';
}
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
proto.placement.center.mqtt.DeleteConnectorReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteConnectorReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteConnectorReply.displayName = 'proto.placement.center.mqtt.DeleteConnectorReply';
}
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
proto.placement.center.mqtt.ConnectorHeartbeatRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ConnectorHeartbeatRequest.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ConnectorHeartbeatRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ConnectorHeartbeatRequest.displayName = 'proto.placement.center.mqtt.ConnectorHeartbeatRequest';
}
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
proto.placement.center.mqtt.ConnectorHeartbeatRaw = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ConnectorHeartbeatRaw, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ConnectorHeartbeatRaw.displayName = 'proto.placement.center.mqtt.ConnectorHeartbeatRaw';
}
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
proto.placement.center.mqtt.ConnectorHeartbeatReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ConnectorHeartbeatReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ConnectorHeartbeatReply.displayName = 'proto.placement.center.mqtt.ConnectorHeartbeatReply';
}
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
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SetAutoSubscribeRuleRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.displayName = 'proto.placement.center.mqtt.SetAutoSubscribeRuleRequest';
}
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
proto.placement.center.mqtt.SetAutoSubscribeRuleReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.SetAutoSubscribeRuleReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.SetAutoSubscribeRuleReply.displayName = 'proto.placement.center.mqtt.SetAutoSubscribeRuleReply';
}
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
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.displayName = 'proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest';
}
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
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.displayName = 'proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply';
}
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
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.mqtt.ListAutoSubscribeRuleRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.displayName = 'proto.placement.center.mqtt.ListAutoSubscribeRuleRequest';
}
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
proto.placement.center.mqtt.ListAutoSubscribeRuleReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.mqtt.ListAutoSubscribeRuleReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.mqtt.ListAutoSubscribeRuleReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.mqtt.ListAutoSubscribeRuleReply.displayName = 'proto.placement.center.mqtt.ListAutoSubscribeRuleReply';
}



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
proto.placement.center.mqtt.GetShareSubLeaderRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.GetShareSubLeaderRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.GetShareSubLeaderRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
groupName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clusterName: jspb.Message.getFieldWithDefault(msg, 3, "")
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
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderRequest}
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.GetShareSubLeaderRequest;
  return proto.placement.center.mqtt.GetShareSubLeaderRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.GetShareSubLeaderRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderRequest}
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setGroupName(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
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
proto.placement.center.mqtt.GetShareSubLeaderRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.GetShareSubLeaderRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.GetShareSubLeaderRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getGroupName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
};


/**
 * optional string group_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.prototype.getGroupName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderRequest} returns this
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.prototype.setGroupName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string cluster_name = 3;
 * @return {string}
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderRequest} returns this
 */
proto.placement.center.mqtt.GetShareSubLeaderRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};





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
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.GetShareSubLeaderReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.GetShareSubLeaderReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.toObject = function(includeInstance, msg) {
  var f, obj = {
brokerId: jspb.Message.getFieldWithDefault(msg, 1, 0),
brokerAddr: jspb.Message.getFieldWithDefault(msg, 2, ""),
extendInfo: jspb.Message.getFieldWithDefault(msg, 3, "")
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
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderReply}
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.GetShareSubLeaderReply;
  return proto.placement.center.mqtt.GetShareSubLeaderReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.GetShareSubLeaderReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderReply}
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setBrokerId(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setBrokerAddr(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setExtendInfo(value);
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
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.GetShareSubLeaderReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.GetShareSubLeaderReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getBrokerId();
  if (f !== 0) {
    writer.writeUint64(
      1,
      f
    );
  }
  f = message.getBrokerAddr();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getExtendInfo();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
};


/**
 * optional uint64 broker_id = 1;
 * @return {number}
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.getBrokerId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 1, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderReply} returns this
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.setBrokerId = function(value) {
  return jspb.Message.setProto3IntField(this, 1, value);
};


/**
 * optional string broker_addr = 2;
 * @return {string}
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.getBrokerAddr = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderReply} returns this
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.setBrokerAddr = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string extend_info = 3;
 * @return {string}
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.getExtendInfo = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.GetShareSubLeaderReply} returns this
 */
proto.placement.center.mqtt.GetShareSubLeaderReply.prototype.setExtendInfo = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};





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
proto.placement.center.mqtt.ListUserRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListUserRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListUserRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListUserRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
userName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.ListUserRequest}
 */
proto.placement.center.mqtt.ListUserRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListUserRequest;
  return proto.placement.center.mqtt.ListUserRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListUserRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListUserRequest}
 */
proto.placement.center.mqtt.ListUserRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setUserName(value);
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
proto.placement.center.mqtt.ListUserRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListUserRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListUserRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListUserRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getUserName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListUserRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListUserRequest} returns this
 */
proto.placement.center.mqtt.ListUserRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string user_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.ListUserRequest.prototype.getUserName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListUserRequest} returns this
 */
proto.placement.center.mqtt.ListUserRequest.prototype.setUserName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListUserReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListUserReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListUserReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListUserReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListUserReply.toObject = function(includeInstance, msg) {
  var f, obj = {
usersList: msg.getUsersList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListUserReply}
 */
proto.placement.center.mqtt.ListUserReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListUserReply;
  return proto.placement.center.mqtt.ListUserReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListUserReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListUserReply}
 */
proto.placement.center.mqtt.ListUserReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addUsers(value);
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
proto.placement.center.mqtt.ListUserReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListUserReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListUserReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListUserReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getUsersList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes users = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListUserReply.prototype.getUsersList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes users = 1;
 * This is a type-conversion wrapper around `getUsersList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListUserReply.prototype.getUsersList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getUsersList()));
};


/**
 * repeated bytes users = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getUsersList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListUserReply.prototype.getUsersList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getUsersList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListUserReply} returns this
 */
proto.placement.center.mqtt.ListUserReply.prototype.setUsersList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListUserReply} returns this
 */
proto.placement.center.mqtt.ListUserReply.prototype.addUsers = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListUserReply} returns this
 */
proto.placement.center.mqtt.ListUserReply.prototype.clearUsersList = function() {
  return this.setUsersList([]);
};





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
proto.placement.center.mqtt.CreateUserRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateUserRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateUserRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateUserRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
userName: jspb.Message.getFieldWithDefault(msg, 2, ""),
content: msg.getContent_asB64()
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
 * @return {!proto.placement.center.mqtt.CreateUserRequest}
 */
proto.placement.center.mqtt.CreateUserRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateUserRequest;
  return proto.placement.center.mqtt.CreateUserRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateUserRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateUserRequest}
 */
proto.placement.center.mqtt.CreateUserRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setUserName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setContent(value);
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
proto.placement.center.mqtt.CreateUserRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateUserRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateUserRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateUserRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getUserName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getContent_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateUserRequest} returns this
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string user_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.getUserName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateUserRequest} returns this
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.setUserName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes content = 3;
 * @return {string}
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.getContent = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes content = 3;
 * This is a type-conversion wrapper around `getContent()`
 * @return {string}
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.getContent_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getContent()));
};


/**
 * optional bytes content = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getContent()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.getContent_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getContent()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.CreateUserRequest} returns this
 */
proto.placement.center.mqtt.CreateUserRequest.prototype.setContent = function(value) {
  return jspb.Message.setProto3BytesField(this, 3, value);
};





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
proto.placement.center.mqtt.CreateUserReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateUserReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateUserReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateUserReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateUserReply}
 */
proto.placement.center.mqtt.CreateUserReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateUserReply;
  return proto.placement.center.mqtt.CreateUserReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateUserReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateUserReply}
 */
proto.placement.center.mqtt.CreateUserReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateUserReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateUserReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateUserReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateUserReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteUserRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteUserRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteUserRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteUserRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
userName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.DeleteUserRequest}
 */
proto.placement.center.mqtt.DeleteUserRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteUserRequest;
  return proto.placement.center.mqtt.DeleteUserRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteUserRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteUserRequest}
 */
proto.placement.center.mqtt.DeleteUserRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setUserName(value);
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
proto.placement.center.mqtt.DeleteUserRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteUserRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteUserRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteUserRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getUserName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteUserRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteUserRequest} returns this
 */
proto.placement.center.mqtt.DeleteUserRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string user_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteUserRequest.prototype.getUserName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteUserRequest} returns this
 */
proto.placement.center.mqtt.DeleteUserRequest.prototype.setUserName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};





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
proto.placement.center.mqtt.DeleteUserReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteUserReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteUserReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteUserReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteUserReply}
 */
proto.placement.center.mqtt.DeleteUserReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteUserReply;
  return proto.placement.center.mqtt.DeleteUserReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteUserReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteUserReply}
 */
proto.placement.center.mqtt.DeleteUserReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteUserReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteUserReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteUserReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteUserReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListTopicRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListTopicRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListTopicRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
topicName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.ListTopicRequest}
 */
proto.placement.center.mqtt.ListTopicRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListTopicRequest;
  return proto.placement.center.mqtt.ListTopicRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListTopicRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListTopicRequest}
 */
proto.placement.center.mqtt.ListTopicRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setTopicName(value);
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
proto.placement.center.mqtt.ListTopicRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListTopicRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListTopicRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getTopicName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListTopicRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListTopicRequest} returns this
 */
proto.placement.center.mqtt.ListTopicRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string topic_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.ListTopicRequest.prototype.getTopicName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListTopicRequest} returns this
 */
proto.placement.center.mqtt.ListTopicRequest.prototype.setTopicName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListTopicReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListTopicReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListTopicReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListTopicReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicReply.toObject = function(includeInstance, msg) {
  var f, obj = {
topicsList: msg.getTopicsList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListTopicReply}
 */
proto.placement.center.mqtt.ListTopicReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListTopicReply;
  return proto.placement.center.mqtt.ListTopicReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListTopicReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListTopicReply}
 */
proto.placement.center.mqtt.ListTopicReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addTopics(value);
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
proto.placement.center.mqtt.ListTopicReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListTopicReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListTopicReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getTopicsList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes topics = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListTopicReply.prototype.getTopicsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes topics = 1;
 * This is a type-conversion wrapper around `getTopicsList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListTopicReply.prototype.getTopicsList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getTopicsList()));
};


/**
 * repeated bytes topics = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getTopicsList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListTopicReply.prototype.getTopicsList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getTopicsList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListTopicReply} returns this
 */
proto.placement.center.mqtt.ListTopicReply.prototype.setTopicsList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListTopicReply} returns this
 */
proto.placement.center.mqtt.ListTopicReply.prototype.addTopics = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListTopicReply} returns this
 */
proto.placement.center.mqtt.ListTopicReply.prototype.clearTopicsList = function() {
  return this.setTopicsList([]);
};





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
proto.placement.center.mqtt.CreateTopicRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateTopicRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateTopicRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
topicName: jspb.Message.getFieldWithDefault(msg, 2, ""),
content: msg.getContent_asB64()
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
 * @return {!proto.placement.center.mqtt.CreateTopicRequest}
 */
proto.placement.center.mqtt.CreateTopicRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateTopicRequest;
  return proto.placement.center.mqtt.CreateTopicRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateTopicRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateTopicRequest}
 */
proto.placement.center.mqtt.CreateTopicRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setTopicName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setContent(value);
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
proto.placement.center.mqtt.CreateTopicRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateTopicRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateTopicRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getTopicName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getContent_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string topic_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.getTopicName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.setTopicName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes content = 3;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.getContent = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes content = 3;
 * This is a type-conversion wrapper around `getContent()`
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.getContent_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getContent()));
};


/**
 * optional bytes content = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getContent()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.getContent_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getContent()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.CreateTopicRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRequest.prototype.setContent = function(value) {
  return jspb.Message.setProto3BytesField(this, 3, value);
};





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
proto.placement.center.mqtt.CreateTopicReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateTopicReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateTopicReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateTopicReply}
 */
proto.placement.center.mqtt.CreateTopicReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateTopicReply;
  return proto.placement.center.mqtt.CreateTopicReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateTopicReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateTopicReply}
 */
proto.placement.center.mqtt.CreateTopicReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateTopicReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateTopicReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateTopicReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteTopicRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteTopicRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteTopicRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
topicName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.DeleteTopicRequest}
 */
proto.placement.center.mqtt.DeleteTopicRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteTopicRequest;
  return proto.placement.center.mqtt.DeleteTopicRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteTopicRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteTopicRequest}
 */
proto.placement.center.mqtt.DeleteTopicRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setTopicName(value);
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
proto.placement.center.mqtt.DeleteTopicRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteTopicRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteTopicRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getTopicName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteTopicRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteTopicRequest} returns this
 */
proto.placement.center.mqtt.DeleteTopicRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string topic_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteTopicRequest.prototype.getTopicName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteTopicRequest} returns this
 */
proto.placement.center.mqtt.DeleteTopicRequest.prototype.setTopicName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};





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
proto.placement.center.mqtt.DeleteTopicReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteTopicReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteTopicReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteTopicReply}
 */
proto.placement.center.mqtt.DeleteTopicReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteTopicReply;
  return proto.placement.center.mqtt.DeleteTopicReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteTopicReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteTopicReply}
 */
proto.placement.center.mqtt.DeleteTopicReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteTopicReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteTopicReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteTopicReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SetTopicRetainMessageRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
topicName: jspb.Message.getFieldWithDefault(msg, 2, ""),
retainMessage: msg.getRetainMessage_asB64(),
retainMessageExpiredAt: jspb.Message.getFieldWithDefault(msg, 4, 0)
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
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageRequest}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SetTopicRetainMessageRequest;
  return proto.placement.center.mqtt.SetTopicRetainMessageRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageRequest}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setTopicName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setRetainMessage(value);
      break;
    case 4:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setRetainMessageExpiredAt(value);
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
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SetTopicRetainMessageRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getTopicName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getRetainMessage_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      3,
      f
    );
  }
  f = message.getRetainMessageExpiredAt();
  if (f !== 0) {
    writer.writeUint64(
      4,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} returns this
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string topic_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.getTopicName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} returns this
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.setTopicName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes retain_message = 3;
 * @return {string}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.getRetainMessage = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes retain_message = 3;
 * This is a type-conversion wrapper around `getRetainMessage()`
 * @return {string}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.getRetainMessage_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getRetainMessage()));
};


/**
 * optional bytes retain_message = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getRetainMessage()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.getRetainMessage_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getRetainMessage()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} returns this
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.setRetainMessage = function(value) {
  return jspb.Message.setProto3BytesField(this, 3, value);
};


/**
 * optional uint64 retain_message_expired_at = 4;
 * @return {number}
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.getRetainMessageExpiredAt = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 4, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageRequest} returns this
 */
proto.placement.center.mqtt.SetTopicRetainMessageRequest.prototype.setRetainMessageExpiredAt = function(value) {
  return jspb.Message.setProto3IntField(this, 4, value);
};





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
proto.placement.center.mqtt.SetTopicRetainMessageReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SetTopicRetainMessageReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SetTopicRetainMessageReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetTopicRetainMessageReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageReply}
 */
proto.placement.center.mqtt.SetTopicRetainMessageReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SetTopicRetainMessageReply;
  return proto.placement.center.mqtt.SetTopicRetainMessageReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SetTopicRetainMessageReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SetTopicRetainMessageReply}
 */
proto.placement.center.mqtt.SetTopicRetainMessageReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.SetTopicRetainMessageReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SetTopicRetainMessageReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SetTopicRetainMessageReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetTopicRetainMessageReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListSessionRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListSessionRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListSessionRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSessionRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.ListSessionRequest}
 */
proto.placement.center.mqtt.ListSessionRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListSessionRequest;
  return proto.placement.center.mqtt.ListSessionRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListSessionRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListSessionRequest}
 */
proto.placement.center.mqtt.ListSessionRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
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
proto.placement.center.mqtt.ListSessionRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListSessionRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListSessionRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSessionRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListSessionRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListSessionRequest} returns this
 */
proto.placement.center.mqtt.ListSessionRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 2;
 * @return {string}
 */
proto.placement.center.mqtt.ListSessionRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListSessionRequest} returns this
 */
proto.placement.center.mqtt.ListSessionRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListSessionReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListSessionReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListSessionReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListSessionReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSessionReply.toObject = function(includeInstance, msg) {
  var f, obj = {
sessionsList: (f = jspb.Message.getRepeatedField(msg, 1)) == null ? undefined : f
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
 * @return {!proto.placement.center.mqtt.ListSessionReply}
 */
proto.placement.center.mqtt.ListSessionReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListSessionReply;
  return proto.placement.center.mqtt.ListSessionReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListSessionReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListSessionReply}
 */
proto.placement.center.mqtt.ListSessionReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.addSessions(value);
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
proto.placement.center.mqtt.ListSessionReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListSessionReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListSessionReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSessionReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getSessionsList();
  if (f.length > 0) {
    writer.writeRepeatedString(
      1,
      f
    );
  }
};


/**
 * repeated string sessions = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListSessionReply.prototype.getSessionsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * @param {!Array<string>} value
 * @return {!proto.placement.center.mqtt.ListSessionReply} returns this
 */
proto.placement.center.mqtt.ListSessionReply.prototype.setSessionsList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {string} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListSessionReply} returns this
 */
proto.placement.center.mqtt.ListSessionReply.prototype.addSessions = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListSessionReply} returns this
 */
proto.placement.center.mqtt.ListSessionReply.prototype.clearSessionsList = function() {
  return this.setSessionsList([]);
};





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
proto.placement.center.mqtt.CreateSessionRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateSessionRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateSessionRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateSessionRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 2, ""),
session: jspb.Message.getFieldWithDefault(msg, 3, "")
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
 * @return {!proto.placement.center.mqtt.CreateSessionRequest}
 */
proto.placement.center.mqtt.CreateSessionRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateSessionRequest;
  return proto.placement.center.mqtt.CreateSessionRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateSessionRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateSessionRequest}
 */
proto.placement.center.mqtt.CreateSessionRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setSession(value);
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
proto.placement.center.mqtt.CreateSessionRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateSessionRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateSessionRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateSessionRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSession();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateSessionRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateSessionRequest} returns this
 */
proto.placement.center.mqtt.CreateSessionRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateSessionRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateSessionRequest} returns this
 */
proto.placement.center.mqtt.CreateSessionRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string session = 3;
 * @return {string}
 */
proto.placement.center.mqtt.CreateSessionRequest.prototype.getSession = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateSessionRequest} returns this
 */
proto.placement.center.mqtt.CreateSessionRequest.prototype.setSession = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};





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
proto.placement.center.mqtt.CreateSessionReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateSessionReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateSessionReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateSessionReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateSessionReply}
 */
proto.placement.center.mqtt.CreateSessionReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateSessionReply;
  return proto.placement.center.mqtt.CreateSessionReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateSessionReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateSessionReply}
 */
proto.placement.center.mqtt.CreateSessionReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateSessionReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateSessionReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateSessionReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateSessionReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.UpdateSessionRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.UpdateSessionRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.UpdateSessionRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateSessionRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 2, ""),
connectionId: jspb.Message.getFieldWithDefault(msg, 3, 0),
brokerId: jspb.Message.getFieldWithDefault(msg, 4, 0),
reconnectTime: jspb.Message.getFieldWithDefault(msg, 5, 0),
distinctTime: jspb.Message.getFieldWithDefault(msg, 6, 0)
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
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest}
 */
proto.placement.center.mqtt.UpdateSessionRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.UpdateSessionRequest;
  return proto.placement.center.mqtt.UpdateSessionRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.UpdateSessionRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest}
 */
proto.placement.center.mqtt.UpdateSessionRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setConnectionId(value);
      break;
    case 4:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setBrokerId(value);
      break;
    case 5:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setReconnectTime(value);
      break;
    case 6:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setDistinctTime(value);
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
proto.placement.center.mqtt.UpdateSessionRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.UpdateSessionRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.UpdateSessionRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateSessionRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getConnectionId();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
  f = message.getBrokerId();
  if (f !== 0) {
    writer.writeUint64(
      4,
      f
    );
  }
  f = message.getReconnectTime();
  if (f !== 0) {
    writer.writeUint64(
      5,
      f
    );
  }
  f = message.getDistinctTime();
  if (f !== 0) {
    writer.writeUint64(
      6,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest} returns this
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 2;
 * @return {string}
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest} returns this
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 connection_id = 3;
 * @return {number}
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.getConnectionId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest} returns this
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.setConnectionId = function(value) {
  return jspb.Message.setProto3IntField(this, 3, value);
};


/**
 * optional uint64 broker_id = 4;
 * @return {number}
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.getBrokerId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 4, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest} returns this
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.setBrokerId = function(value) {
  return jspb.Message.setProto3IntField(this, 4, value);
};


/**
 * optional uint64 reconnect_time = 5;
 * @return {number}
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.getReconnectTime = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 5, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest} returns this
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.setReconnectTime = function(value) {
  return jspb.Message.setProto3IntField(this, 5, value);
};


/**
 * optional uint64 distinct_time = 6;
 * @return {number}
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.getDistinctTime = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 6, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.UpdateSessionRequest} returns this
 */
proto.placement.center.mqtt.UpdateSessionRequest.prototype.setDistinctTime = function(value) {
  return jspb.Message.setProto3IntField(this, 6, value);
};





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
proto.placement.center.mqtt.UpdateSessionReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.UpdateSessionReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.UpdateSessionReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateSessionReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.UpdateSessionReply}
 */
proto.placement.center.mqtt.UpdateSessionReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.UpdateSessionReply;
  return proto.placement.center.mqtt.UpdateSessionReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.UpdateSessionReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.UpdateSessionReply}
 */
proto.placement.center.mqtt.UpdateSessionReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.UpdateSessionReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.UpdateSessionReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.UpdateSessionReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateSessionReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteSessionRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteSessionRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteSessionRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSessionRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.DeleteSessionRequest}
 */
proto.placement.center.mqtt.DeleteSessionRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteSessionRequest;
  return proto.placement.center.mqtt.DeleteSessionRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteSessionRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteSessionRequest}
 */
proto.placement.center.mqtt.DeleteSessionRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
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
proto.placement.center.mqtt.DeleteSessionRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteSessionRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteSessionRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSessionRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteSessionRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteSessionRequest} returns this
 */
proto.placement.center.mqtt.DeleteSessionRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteSessionRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteSessionRequest} returns this
 */
proto.placement.center.mqtt.DeleteSessionRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};





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
proto.placement.center.mqtt.DeleteSessionReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteSessionReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteSessionReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSessionReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteSessionReply}
 */
proto.placement.center.mqtt.DeleteSessionReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteSessionReply;
  return proto.placement.center.mqtt.DeleteSessionReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteSessionReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteSessionReply}
 */
proto.placement.center.mqtt.DeleteSessionReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteSessionReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteSessionReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteSessionReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSessionReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SaveLastWillMessageRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SaveLastWillMessageRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 2, ""),
lastWillMessage: msg.getLastWillMessage_asB64()
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
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageRequest}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SaveLastWillMessageRequest;
  return proto.placement.center.mqtt.SaveLastWillMessageRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SaveLastWillMessageRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageRequest}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setLastWillMessage(value);
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
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SaveLastWillMessageRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SaveLastWillMessageRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getLastWillMessage_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageRequest} returns this
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 2;
 * @return {string}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageRequest} returns this
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes last_will_message = 3;
 * @return {string}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.getLastWillMessage = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes last_will_message = 3;
 * This is a type-conversion wrapper around `getLastWillMessage()`
 * @return {string}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.getLastWillMessage_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getLastWillMessage()));
};


/**
 * optional bytes last_will_message = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getLastWillMessage()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.getLastWillMessage_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getLastWillMessage()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageRequest} returns this
 */
proto.placement.center.mqtt.SaveLastWillMessageRequest.prototype.setLastWillMessage = function(value) {
  return jspb.Message.setProto3BytesField(this, 3, value);
};





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
proto.placement.center.mqtt.SaveLastWillMessageReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SaveLastWillMessageReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SaveLastWillMessageReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SaveLastWillMessageReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageReply}
 */
proto.placement.center.mqtt.SaveLastWillMessageReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SaveLastWillMessageReply;
  return proto.placement.center.mqtt.SaveLastWillMessageReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SaveLastWillMessageReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SaveLastWillMessageReply}
 */
proto.placement.center.mqtt.SaveLastWillMessageReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.SaveLastWillMessageReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SaveLastWillMessageReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SaveLastWillMessageReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SaveLastWillMessageReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListAclRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListAclRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListAclRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAclRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, "")
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
 * @return {!proto.placement.center.mqtt.ListAclRequest}
 */
proto.placement.center.mqtt.ListAclRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListAclRequest;
  return proto.placement.center.mqtt.ListAclRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListAclRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListAclRequest}
 */
proto.placement.center.mqtt.ListAclRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
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
proto.placement.center.mqtt.ListAclRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListAclRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListAclRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAclRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListAclRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListAclRequest} returns this
 */
proto.placement.center.mqtt.ListAclRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListAclReply.repeatedFields_ = [2];



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
proto.placement.center.mqtt.ListAclReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListAclReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListAclReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAclReply.toObject = function(includeInstance, msg) {
  var f, obj = {
aclsList: msg.getAclsList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListAclReply}
 */
proto.placement.center.mqtt.ListAclReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListAclReply;
  return proto.placement.center.mqtt.ListAclReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListAclReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListAclReply}
 */
proto.placement.center.mqtt.ListAclReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 2:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addAcls(value);
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
proto.placement.center.mqtt.ListAclReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListAclReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListAclReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAclReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getAclsList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      2,
      f
    );
  }
};


/**
 * repeated bytes acls = 2;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListAclReply.prototype.getAclsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 2));
};


/**
 * repeated bytes acls = 2;
 * This is a type-conversion wrapper around `getAclsList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListAclReply.prototype.getAclsList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getAclsList()));
};


/**
 * repeated bytes acls = 2;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getAclsList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListAclReply.prototype.getAclsList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getAclsList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListAclReply} returns this
 */
proto.placement.center.mqtt.ListAclReply.prototype.setAclsList = function(value) {
  return jspb.Message.setField(this, 2, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListAclReply} returns this
 */
proto.placement.center.mqtt.ListAclReply.prototype.addAcls = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 2, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListAclReply} returns this
 */
proto.placement.center.mqtt.ListAclReply.prototype.clearAclsList = function() {
  return this.setAclsList([]);
};





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
proto.placement.center.mqtt.DeleteAclRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteAclRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteAclRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAclRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
acl: msg.getAcl_asB64()
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
 * @return {!proto.placement.center.mqtt.DeleteAclRequest}
 */
proto.placement.center.mqtt.DeleteAclRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteAclRequest;
  return proto.placement.center.mqtt.DeleteAclRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteAclRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteAclRequest}
 */
proto.placement.center.mqtt.DeleteAclRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setAcl(value);
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
proto.placement.center.mqtt.DeleteAclRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteAclRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteAclRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAclRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getAcl_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteAclRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteAclRequest} returns this
 */
proto.placement.center.mqtt.DeleteAclRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional bytes acl = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteAclRequest.prototype.getAcl = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * optional bytes acl = 2;
 * This is a type-conversion wrapper around `getAcl()`
 * @return {string}
 */
proto.placement.center.mqtt.DeleteAclRequest.prototype.getAcl_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getAcl()));
};


/**
 * optional bytes acl = 2;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getAcl()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.DeleteAclRequest.prototype.getAcl_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getAcl()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.DeleteAclRequest} returns this
 */
proto.placement.center.mqtt.DeleteAclRequest.prototype.setAcl = function(value) {
  return jspb.Message.setProto3BytesField(this, 2, value);
};





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
proto.placement.center.mqtt.DeleteAclReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteAclReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteAclReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAclReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteAclReply}
 */
proto.placement.center.mqtt.DeleteAclReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteAclReply;
  return proto.placement.center.mqtt.DeleteAclReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteAclReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteAclReply}
 */
proto.placement.center.mqtt.DeleteAclReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteAclReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteAclReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteAclReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAclReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.CreateAclRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateAclRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateAclRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateAclRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
acl: msg.getAcl_asB64()
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
 * @return {!proto.placement.center.mqtt.CreateAclRequest}
 */
proto.placement.center.mqtt.CreateAclRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateAclRequest;
  return proto.placement.center.mqtt.CreateAclRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateAclRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateAclRequest}
 */
proto.placement.center.mqtt.CreateAclRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setAcl(value);
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
proto.placement.center.mqtt.CreateAclRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateAclRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateAclRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateAclRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getAcl_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateAclRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateAclRequest} returns this
 */
proto.placement.center.mqtt.CreateAclRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional bytes acl = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateAclRequest.prototype.getAcl = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * optional bytes acl = 2;
 * This is a type-conversion wrapper around `getAcl()`
 * @return {string}
 */
proto.placement.center.mqtt.CreateAclRequest.prototype.getAcl_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getAcl()));
};


/**
 * optional bytes acl = 2;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getAcl()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.CreateAclRequest.prototype.getAcl_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getAcl()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.CreateAclRequest} returns this
 */
proto.placement.center.mqtt.CreateAclRequest.prototype.setAcl = function(value) {
  return jspb.Message.setProto3BytesField(this, 2, value);
};





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
proto.placement.center.mqtt.CreateAclReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateAclReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateAclReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateAclReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateAclReply}
 */
proto.placement.center.mqtt.CreateAclReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateAclReply;
  return proto.placement.center.mqtt.CreateAclReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateAclReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateAclReply}
 */
proto.placement.center.mqtt.CreateAclReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateAclReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateAclReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateAclReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateAclReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListBlacklistRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListBlacklistRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListBlacklistRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListBlacklistRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, "")
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
 * @return {!proto.placement.center.mqtt.ListBlacklistRequest}
 */
proto.placement.center.mqtt.ListBlacklistRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListBlacklistRequest;
  return proto.placement.center.mqtt.ListBlacklistRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListBlacklistRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListBlacklistRequest}
 */
proto.placement.center.mqtt.ListBlacklistRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
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
proto.placement.center.mqtt.ListBlacklistRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListBlacklistRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListBlacklistRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListBlacklistRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListBlacklistRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListBlacklistRequest} returns this
 */
proto.placement.center.mqtt.ListBlacklistRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListBlacklistReply.repeatedFields_ = [2];



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
proto.placement.center.mqtt.ListBlacklistReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListBlacklistReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListBlacklistReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListBlacklistReply.toObject = function(includeInstance, msg) {
  var f, obj = {
blacklistsList: msg.getBlacklistsList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListBlacklistReply}
 */
proto.placement.center.mqtt.ListBlacklistReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListBlacklistReply;
  return proto.placement.center.mqtt.ListBlacklistReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListBlacklistReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListBlacklistReply}
 */
proto.placement.center.mqtt.ListBlacklistReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 2:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addBlacklists(value);
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
proto.placement.center.mqtt.ListBlacklistReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListBlacklistReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListBlacklistReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListBlacklistReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getBlacklistsList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      2,
      f
    );
  }
};


/**
 * repeated bytes blacklists = 2;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListBlacklistReply.prototype.getBlacklistsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 2));
};


/**
 * repeated bytes blacklists = 2;
 * This is a type-conversion wrapper around `getBlacklistsList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListBlacklistReply.prototype.getBlacklistsList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getBlacklistsList()));
};


/**
 * repeated bytes blacklists = 2;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getBlacklistsList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListBlacklistReply.prototype.getBlacklistsList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getBlacklistsList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListBlacklistReply} returns this
 */
proto.placement.center.mqtt.ListBlacklistReply.prototype.setBlacklistsList = function(value) {
  return jspb.Message.setField(this, 2, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListBlacklistReply} returns this
 */
proto.placement.center.mqtt.ListBlacklistReply.prototype.addBlacklists = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 2, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListBlacklistReply} returns this
 */
proto.placement.center.mqtt.ListBlacklistReply.prototype.clearBlacklistsList = function() {
  return this.setBlacklistsList([]);
};





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
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateBlacklistRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateBlacklistRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateBlacklistRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
blacklist: msg.getBlacklist_asB64()
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
 * @return {!proto.placement.center.mqtt.CreateBlacklistRequest}
 */
proto.placement.center.mqtt.CreateBlacklistRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateBlacklistRequest;
  return proto.placement.center.mqtt.CreateBlacklistRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateBlacklistRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateBlacklistRequest}
 */
proto.placement.center.mqtt.CreateBlacklistRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setBlacklist(value);
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
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateBlacklistRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateBlacklistRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateBlacklistRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getBlacklist_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateBlacklistRequest} returns this
 */
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional bytes blacklist = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.getBlacklist = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * optional bytes blacklist = 2;
 * This is a type-conversion wrapper around `getBlacklist()`
 * @return {string}
 */
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.getBlacklist_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getBlacklist()));
};


/**
 * optional bytes blacklist = 2;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getBlacklist()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.getBlacklist_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getBlacklist()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.CreateBlacklistRequest} returns this
 */
proto.placement.center.mqtt.CreateBlacklistRequest.prototype.setBlacklist = function(value) {
  return jspb.Message.setProto3BytesField(this, 2, value);
};





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
proto.placement.center.mqtt.CreateBlacklistReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateBlacklistReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateBlacklistReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateBlacklistReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateBlacklistReply}
 */
proto.placement.center.mqtt.CreateBlacklistReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateBlacklistReply;
  return proto.placement.center.mqtt.CreateBlacklistReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateBlacklistReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateBlacklistReply}
 */
proto.placement.center.mqtt.CreateBlacklistReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateBlacklistReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateBlacklistReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateBlacklistReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateBlacklistReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteBlacklistRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteBlacklistRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
blacklistType: jspb.Message.getFieldWithDefault(msg, 2, ""),
resourceName: jspb.Message.getFieldWithDefault(msg, 3, "")
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
 * @return {!proto.placement.center.mqtt.DeleteBlacklistRequest}
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteBlacklistRequest;
  return proto.placement.center.mqtt.DeleteBlacklistRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteBlacklistRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteBlacklistRequest}
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setBlacklistType(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setResourceName(value);
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
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteBlacklistRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteBlacklistRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getBlacklistType();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getResourceName();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteBlacklistRequest} returns this
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string blacklist_type = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.getBlacklistType = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteBlacklistRequest} returns this
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.setBlacklistType = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string resource_name = 3;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.getResourceName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteBlacklistRequest} returns this
 */
proto.placement.center.mqtt.DeleteBlacklistRequest.prototype.setResourceName = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};





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
proto.placement.center.mqtt.DeleteBlacklistReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteBlacklistReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteBlacklistReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteBlacklistReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteBlacklistReply}
 */
proto.placement.center.mqtt.DeleteBlacklistReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteBlacklistReply;
  return proto.placement.center.mqtt.DeleteBlacklistReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteBlacklistReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteBlacklistReply}
 */
proto.placement.center.mqtt.DeleteBlacklistReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteBlacklistReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteBlacklistReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteBlacklistReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteBlacklistReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
action: jspb.Message.getFieldWithDefault(msg, 2, ""),
sourceTopic: jspb.Message.getFieldWithDefault(msg, 3, ""),
destTopic: jspb.Message.getFieldWithDefault(msg, 4, ""),
regex: jspb.Message.getFieldWithDefault(msg, 5, "")
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
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateTopicRewriteRuleRequest;
  return proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setAction(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setSourceTopic(value);
      break;
    case 4:
      var value = /** @type {string} */ (reader.readString());
      msg.setDestTopic(value);
      break;
    case 5:
      var value = /** @type {string} */ (reader.readString());
      msg.setRegex(value);
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
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getAction();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSourceTopic();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
  f = message.getDestTopic();
  if (f.length > 0) {
    writer.writeString(
      4,
      f
    );
  }
  f = message.getRegex();
  if (f.length > 0) {
    writer.writeString(
      5,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string action = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.getAction = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.setAction = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string source_topic = 3;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.getSourceTopic = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.setSourceTopic = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};


/**
 * optional string dest_topic = 4;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.getDestTopic = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 4, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.setDestTopic = function(value) {
  return jspb.Message.setProto3StringField(this, 4, value);
};


/**
 * optional string regex = 5;
 * @return {string}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.getRegex = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 5, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleRequest.prototype.setRegex = function(value) {
  return jspb.Message.setProto3StringField(this, 5, value);
};





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
proto.placement.center.mqtt.CreateTopicRewriteRuleReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateTopicRewriteRuleReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateTopicRewriteRuleReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleReply}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateTopicRewriteRuleReply;
  return proto.placement.center.mqtt.CreateTopicRewriteRuleReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateTopicRewriteRuleReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateTopicRewriteRuleReply}
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateTopicRewriteRuleReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateTopicRewriteRuleReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateTopicRewriteRuleReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateTopicRewriteRuleReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
action: jspb.Message.getFieldWithDefault(msg, 2, ""),
sourceTopic: jspb.Message.getFieldWithDefault(msg, 3, "")
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
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest;
  return proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setAction(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setSourceTopic(value);
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
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getAction();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSourceTopic();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string action = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.getAction = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.setAction = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string source_topic = 3;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.getSourceTopic = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleRequest.prototype.setSourceTopic = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};





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
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteTopicRewriteRuleReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleReply}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteTopicRewriteRuleReply;
  return proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteTopicRewriteRuleReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteTopicRewriteRuleReply}
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteTopicRewriteRuleReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteTopicRewriteRuleReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListTopicRewriteRuleRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListTopicRewriteRuleRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, "")
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
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleRequest}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListTopicRewriteRuleRequest;
  return proto.placement.center.mqtt.ListTopicRewriteRuleRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListTopicRewriteRuleRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleRequest}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
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
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListTopicRewriteRuleRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListTopicRewriteRuleRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleRequest} returns this
 */
proto.placement.center.mqtt.ListTopicRewriteRuleRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListTopicRewriteRuleReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListTopicRewriteRuleReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.toObject = function(includeInstance, msg) {
  var f, obj = {
topicRewriteRulesList: msg.getTopicRewriteRulesList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleReply}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListTopicRewriteRuleReply;
  return proto.placement.center.mqtt.ListTopicRewriteRuleReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListTopicRewriteRuleReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleReply}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addTopicRewriteRules(value);
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
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListTopicRewriteRuleReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListTopicRewriteRuleReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getTopicRewriteRulesList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes topic_rewrite_rules = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.getTopicRewriteRulesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes topic_rewrite_rules = 1;
 * This is a type-conversion wrapper around `getTopicRewriteRulesList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.getTopicRewriteRulesList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getTopicRewriteRulesList()));
};


/**
 * repeated bytes topic_rewrite_rules = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getTopicRewriteRulesList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.getTopicRewriteRulesList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getTopicRewriteRulesList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleReply} returns this
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.setTopicRewriteRulesList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleReply} returns this
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.addTopicRewriteRules = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListTopicRewriteRuleReply} returns this
 */
proto.placement.center.mqtt.ListTopicRewriteRuleReply.prototype.clearTopicRewriteRulesList = function() {
  return this.setTopicRewriteRulesList([]);
};





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
proto.placement.center.mqtt.SetSubscribeRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SetSubscribeRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SetSubscribeRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetSubscribeRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 3, ""),
path: jspb.Message.getFieldWithDefault(msg, 4, ""),
subscribe: msg.getSubscribe_asB64()
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
 * @return {!proto.placement.center.mqtt.SetSubscribeRequest}
 */
proto.placement.center.mqtt.SetSubscribeRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SetSubscribeRequest;
  return proto.placement.center.mqtt.SetSubscribeRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SetSubscribeRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SetSubscribeRequest}
 */
proto.placement.center.mqtt.SetSubscribeRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
      break;
    case 4:
      var value = /** @type {string} */ (reader.readString());
      msg.setPath(value);
      break;
    case 5:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setSubscribe(value);
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
proto.placement.center.mqtt.SetSubscribeRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SetSubscribeRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SetSubscribeRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetSubscribeRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
  f = message.getPath();
  if (f.length > 0) {
    writer.writeString(
      4,
      f
    );
  }
  f = message.getSubscribe_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      5,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetSubscribeRequest} returns this
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 3;
 * @return {string}
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetSubscribeRequest} returns this
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};


/**
 * optional string path = 4;
 * @return {string}
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.getPath = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 4, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetSubscribeRequest} returns this
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.setPath = function(value) {
  return jspb.Message.setProto3StringField(this, 4, value);
};


/**
 * optional bytes subscribe = 5;
 * @return {string}
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.getSubscribe = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 5, ""));
};


/**
 * optional bytes subscribe = 5;
 * This is a type-conversion wrapper around `getSubscribe()`
 * @return {string}
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.getSubscribe_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getSubscribe()));
};


/**
 * optional bytes subscribe = 5;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getSubscribe()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.getSubscribe_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getSubscribe()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.SetSubscribeRequest} returns this
 */
proto.placement.center.mqtt.SetSubscribeRequest.prototype.setSubscribe = function(value) {
  return jspb.Message.setProto3BytesField(this, 5, value);
};





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
proto.placement.center.mqtt.SetSubscribeReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SetSubscribeReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SetSubscribeReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetSubscribeReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.SetSubscribeReply}
 */
proto.placement.center.mqtt.SetSubscribeReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SetSubscribeReply;
  return proto.placement.center.mqtt.SetSubscribeReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SetSubscribeReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SetSubscribeReply}
 */
proto.placement.center.mqtt.SetSubscribeReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.SetSubscribeReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SetSubscribeReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SetSubscribeReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetSubscribeReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteSubscribeRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteSubscribeRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
clientId: jspb.Message.getFieldWithDefault(msg, 2, ""),
path: jspb.Message.getFieldWithDefault(msg, 3, "")
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
 * @return {!proto.placement.center.mqtt.DeleteSubscribeRequest}
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteSubscribeRequest;
  return proto.placement.center.mqtt.DeleteSubscribeRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteSubscribeRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteSubscribeRequest}
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClientId(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setPath(value);
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
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteSubscribeRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteSubscribeRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getClientId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getPath();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteSubscribeRequest} returns this
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string client_id = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.getClientId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteSubscribeRequest} returns this
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.setClientId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string path = 3;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.getPath = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteSubscribeRequest} returns this
 */
proto.placement.center.mqtt.DeleteSubscribeRequest.prototype.setPath = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};





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
proto.placement.center.mqtt.DeleteSubscribeReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteSubscribeReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteSubscribeReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSubscribeReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteSubscribeReply}
 */
proto.placement.center.mqtt.DeleteSubscribeReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteSubscribeReply;
  return proto.placement.center.mqtt.DeleteSubscribeReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteSubscribeReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteSubscribeReply}
 */
proto.placement.center.mqtt.DeleteSubscribeReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteSubscribeReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteSubscribeReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteSubscribeReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteSubscribeReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListSubscribeRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListSubscribeRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListSubscribeRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSubscribeRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, "")
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
 * @return {!proto.placement.center.mqtt.ListSubscribeRequest}
 */
proto.placement.center.mqtt.ListSubscribeRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListSubscribeRequest;
  return proto.placement.center.mqtt.ListSubscribeRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListSubscribeRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListSubscribeRequest}
 */
proto.placement.center.mqtt.ListSubscribeRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
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
proto.placement.center.mqtt.ListSubscribeRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListSubscribeRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListSubscribeRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSubscribeRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListSubscribeRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListSubscribeRequest} returns this
 */
proto.placement.center.mqtt.ListSubscribeRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListSubscribeReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListSubscribeReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListSubscribeReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListSubscribeReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSubscribeReply.toObject = function(includeInstance, msg) {
  var f, obj = {
subscribesList: msg.getSubscribesList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListSubscribeReply}
 */
proto.placement.center.mqtt.ListSubscribeReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListSubscribeReply;
  return proto.placement.center.mqtt.ListSubscribeReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListSubscribeReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListSubscribeReply}
 */
proto.placement.center.mqtt.ListSubscribeReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addSubscribes(value);
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
proto.placement.center.mqtt.ListSubscribeReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListSubscribeReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListSubscribeReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListSubscribeReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getSubscribesList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes subscribes = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListSubscribeReply.prototype.getSubscribesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes subscribes = 1;
 * This is a type-conversion wrapper around `getSubscribesList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListSubscribeReply.prototype.getSubscribesList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getSubscribesList()));
};


/**
 * repeated bytes subscribes = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getSubscribesList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListSubscribeReply.prototype.getSubscribesList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getSubscribesList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListSubscribeReply} returns this
 */
proto.placement.center.mqtt.ListSubscribeReply.prototype.setSubscribesList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListSubscribeReply} returns this
 */
proto.placement.center.mqtt.ListSubscribeReply.prototype.addSubscribes = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListSubscribeReply} returns this
 */
proto.placement.center.mqtt.ListSubscribeReply.prototype.clearSubscribesList = function() {
  return this.setSubscribesList([]);
};





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
proto.placement.center.mqtt.ListConnectorRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListConnectorRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListConnectorRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListConnectorRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
connectorName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.ListConnectorRequest}
 */
proto.placement.center.mqtt.ListConnectorRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListConnectorRequest;
  return proto.placement.center.mqtt.ListConnectorRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListConnectorRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListConnectorRequest}
 */
proto.placement.center.mqtt.ListConnectorRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setConnectorName(value);
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
proto.placement.center.mqtt.ListConnectorRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListConnectorRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListConnectorRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListConnectorRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getConnectorName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListConnectorRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListConnectorRequest} returns this
 */
proto.placement.center.mqtt.ListConnectorRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string connector_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.ListConnectorRequest.prototype.getConnectorName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListConnectorRequest} returns this
 */
proto.placement.center.mqtt.ListConnectorRequest.prototype.setConnectorName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListConnectorReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListConnectorReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListConnectorReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListConnectorReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListConnectorReply.toObject = function(includeInstance, msg) {
  var f, obj = {
connectorsList: msg.getConnectorsList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListConnectorReply}
 */
proto.placement.center.mqtt.ListConnectorReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListConnectorReply;
  return proto.placement.center.mqtt.ListConnectorReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListConnectorReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListConnectorReply}
 */
proto.placement.center.mqtt.ListConnectorReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addConnectors(value);
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
proto.placement.center.mqtt.ListConnectorReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListConnectorReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListConnectorReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListConnectorReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getConnectorsList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes connectors = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListConnectorReply.prototype.getConnectorsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes connectors = 1;
 * This is a type-conversion wrapper around `getConnectorsList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListConnectorReply.prototype.getConnectorsList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getConnectorsList()));
};


/**
 * repeated bytes connectors = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getConnectorsList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListConnectorReply.prototype.getConnectorsList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getConnectorsList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListConnectorReply} returns this
 */
proto.placement.center.mqtt.ListConnectorReply.prototype.setConnectorsList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListConnectorReply} returns this
 */
proto.placement.center.mqtt.ListConnectorReply.prototype.addConnectors = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListConnectorReply} returns this
 */
proto.placement.center.mqtt.ListConnectorReply.prototype.clearConnectorsList = function() {
  return this.setConnectorsList([]);
};





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
proto.placement.center.mqtt.CreateConnectorRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateConnectorRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateConnectorRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateConnectorRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
connectorName: jspb.Message.getFieldWithDefault(msg, 2, ""),
connector: msg.getConnector_asB64()
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
 * @return {!proto.placement.center.mqtt.CreateConnectorRequest}
 */
proto.placement.center.mqtt.CreateConnectorRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateConnectorRequest;
  return proto.placement.center.mqtt.CreateConnectorRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateConnectorRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateConnectorRequest}
 */
proto.placement.center.mqtt.CreateConnectorRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setConnectorName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setConnector(value);
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
proto.placement.center.mqtt.CreateConnectorRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateConnectorRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateConnectorRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateConnectorRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getConnectorName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getConnector_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateConnectorRequest} returns this
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string connector_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.getConnectorName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.CreateConnectorRequest} returns this
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.setConnectorName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes connector = 3;
 * @return {string}
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.getConnector = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes connector = 3;
 * This is a type-conversion wrapper around `getConnector()`
 * @return {string}
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.getConnector_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getConnector()));
};


/**
 * optional bytes connector = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getConnector()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.getConnector_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getConnector()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.CreateConnectorRequest} returns this
 */
proto.placement.center.mqtt.CreateConnectorRequest.prototype.setConnector = function(value) {
  return jspb.Message.setProto3BytesField(this, 3, value);
};





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
proto.placement.center.mqtt.CreateConnectorReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.CreateConnectorReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.CreateConnectorReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateConnectorReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.CreateConnectorReply}
 */
proto.placement.center.mqtt.CreateConnectorReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.CreateConnectorReply;
  return proto.placement.center.mqtt.CreateConnectorReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.CreateConnectorReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.CreateConnectorReply}
 */
proto.placement.center.mqtt.CreateConnectorReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.CreateConnectorReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.CreateConnectorReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.CreateConnectorReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.CreateConnectorReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.UpdateConnectorRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.UpdateConnectorRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateConnectorRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
connectorName: jspb.Message.getFieldWithDefault(msg, 2, ""),
connector: msg.getConnector_asB64()
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
 * @return {!proto.placement.center.mqtt.UpdateConnectorRequest}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.UpdateConnectorRequest;
  return proto.placement.center.mqtt.UpdateConnectorRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.UpdateConnectorRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.UpdateConnectorRequest}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setConnectorName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setConnector(value);
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
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.UpdateConnectorRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.UpdateConnectorRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateConnectorRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getConnectorName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getConnector_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.UpdateConnectorRequest} returns this
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string connector_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.getConnectorName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.UpdateConnectorRequest} returns this
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.setConnectorName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes connector = 3;
 * @return {string}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.getConnector = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes connector = 3;
 * This is a type-conversion wrapper around `getConnector()`
 * @return {string}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.getConnector_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getConnector()));
};


/**
 * optional bytes connector = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getConnector()`
 * @return {!Uint8Array}
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.getConnector_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getConnector()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.mqtt.UpdateConnectorRequest} returns this
 */
proto.placement.center.mqtt.UpdateConnectorRequest.prototype.setConnector = function(value) {
  return jspb.Message.setProto3BytesField(this, 3, value);
};





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
proto.placement.center.mqtt.UpdateConnectorReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.UpdateConnectorReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.UpdateConnectorReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateConnectorReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.UpdateConnectorReply}
 */
proto.placement.center.mqtt.UpdateConnectorReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.UpdateConnectorReply;
  return proto.placement.center.mqtt.UpdateConnectorReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.UpdateConnectorReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.UpdateConnectorReply}
 */
proto.placement.center.mqtt.UpdateConnectorReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.UpdateConnectorReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.UpdateConnectorReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.UpdateConnectorReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.UpdateConnectorReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteConnectorRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteConnectorRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteConnectorRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteConnectorRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
connectorName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.DeleteConnectorRequest}
 */
proto.placement.center.mqtt.DeleteConnectorRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteConnectorRequest;
  return proto.placement.center.mqtt.DeleteConnectorRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteConnectorRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteConnectorRequest}
 */
proto.placement.center.mqtt.DeleteConnectorRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setConnectorName(value);
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
proto.placement.center.mqtt.DeleteConnectorRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteConnectorRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteConnectorRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteConnectorRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getConnectorName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteConnectorRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteConnectorRequest} returns this
 */
proto.placement.center.mqtt.DeleteConnectorRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string connector_name = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteConnectorRequest.prototype.getConnectorName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteConnectorRequest} returns this
 */
proto.placement.center.mqtt.DeleteConnectorRequest.prototype.setConnectorName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};





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
proto.placement.center.mqtt.DeleteConnectorReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteConnectorReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteConnectorReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteConnectorReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteConnectorReply}
 */
proto.placement.center.mqtt.DeleteConnectorReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteConnectorReply;
  return proto.placement.center.mqtt.DeleteConnectorReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteConnectorReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteConnectorReply}
 */
proto.placement.center.mqtt.DeleteConnectorReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteConnectorReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteConnectorReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteConnectorReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteConnectorReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.repeatedFields_ = [2];



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
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ConnectorHeartbeatRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
heatbeatsList: jspb.Message.toObjectList(msg.getHeatbeatsList(),
    proto.placement.center.mqtt.ConnectorHeartbeatRaw.toObject, includeInstance)
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
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRequest}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ConnectorHeartbeatRequest;
  return proto.placement.center.mqtt.ConnectorHeartbeatRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRequest}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = new proto.placement.center.mqtt.ConnectorHeartbeatRaw;
      reader.readMessage(value,proto.placement.center.mqtt.ConnectorHeartbeatRaw.deserializeBinaryFromReader);
      msg.addHeatbeats(value);
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
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ConnectorHeartbeatRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getHeatbeatsList();
  if (f.length > 0) {
    writer.writeRepeatedMessage(
      2,
      f,
      proto.placement.center.mqtt.ConnectorHeartbeatRaw.serializeBinaryToWriter
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRequest} returns this
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * repeated ConnectorHeartbeatRaw heatbeats = 2;
 * @return {!Array<!proto.placement.center.mqtt.ConnectorHeartbeatRaw>}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.getHeatbeatsList = function() {
  return /** @type{!Array<!proto.placement.center.mqtt.ConnectorHeartbeatRaw>} */ (
    jspb.Message.getRepeatedWrapperField(this, proto.placement.center.mqtt.ConnectorHeartbeatRaw, 2));
};


/**
 * @param {!Array<!proto.placement.center.mqtt.ConnectorHeartbeatRaw>} value
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRequest} returns this
*/
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.setHeatbeatsList = function(value) {
  return jspb.Message.setRepeatedWrapperField(this, 2, value);
};


/**
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRaw=} opt_value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRaw}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.addHeatbeats = function(opt_value, opt_index) {
  return jspb.Message.addToRepeatedWrapperField(this, 2, opt_value, proto.placement.center.mqtt.ConnectorHeartbeatRaw, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRequest} returns this
 */
proto.placement.center.mqtt.ConnectorHeartbeatRequest.prototype.clearHeatbeatsList = function() {
  return this.setHeatbeatsList([]);
};





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
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ConnectorHeartbeatRaw.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRaw} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.toObject = function(includeInstance, msg) {
  var f, obj = {
connectorName: jspb.Message.getFieldWithDefault(msg, 1, ""),
brokerId: jspb.Message.getFieldWithDefault(msg, 2, 0),
heartbeatTime: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRaw}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ConnectorHeartbeatRaw;
  return proto.placement.center.mqtt.ConnectorHeartbeatRaw.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRaw} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRaw}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setConnectorName(value);
      break;
    case 2:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setBrokerId(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setHeartbeatTime(value);
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
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ConnectorHeartbeatRaw.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatRaw} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getConnectorName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getBrokerId();
  if (f !== 0) {
    writer.writeUint64(
      2,
      f
    );
  }
  f = message.getHeartbeatTime();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional string connector_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.getConnectorName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRaw} returns this
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.setConnectorName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional uint64 broker_id = 2;
 * @return {number}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.getBrokerId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 2, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRaw} returns this
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.setBrokerId = function(value) {
  return jspb.Message.setProto3IntField(this, 2, value);
};


/**
 * optional uint64 heartbeat_time = 3;
 * @return {number}
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.getHeartbeatTime = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatRaw} returns this
 */
proto.placement.center.mqtt.ConnectorHeartbeatRaw.prototype.setHeartbeatTime = function(value) {
  return jspb.Message.setProto3IntField(this, 3, value);
};





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
proto.placement.center.mqtt.ConnectorHeartbeatReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ConnectorHeartbeatReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ConnectorHeartbeatReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatReply}
 */
proto.placement.center.mqtt.ConnectorHeartbeatReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ConnectorHeartbeatReply;
  return proto.placement.center.mqtt.ConnectorHeartbeatReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ConnectorHeartbeatReply}
 */
proto.placement.center.mqtt.ConnectorHeartbeatReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.ConnectorHeartbeatReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ConnectorHeartbeatReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ConnectorHeartbeatReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ConnectorHeartbeatReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
topic: jspb.Message.getFieldWithDefault(msg, 2, ""),
qos: jspb.Message.getFieldWithDefault(msg, 3, 0),
noLocal: jspb.Message.getBooleanFieldWithDefault(msg, 4, false),
retainAsPublished: jspb.Message.getBooleanFieldWithDefault(msg, 5, false),
retainedHandling: jspb.Message.getFieldWithDefault(msg, 6, 0)
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
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SetAutoSubscribeRuleRequest;
  return proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setTopic(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint32());
      msg.setQos(value);
      break;
    case 4:
      var value = /** @type {boolean} */ (reader.readBool());
      msg.setNoLocal(value);
      break;
    case 5:
      var value = /** @type {boolean} */ (reader.readBool());
      msg.setRetainAsPublished(value);
      break;
    case 6:
      var value = /** @type {number} */ (reader.readUint32());
      msg.setRetainedHandling(value);
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
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getTopic();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getQos();
  if (f !== 0) {
    writer.writeUint32(
      3,
      f
    );
  }
  f = message.getNoLocal();
  if (f) {
    writer.writeBool(
      4,
      f
    );
  }
  f = message.getRetainAsPublished();
  if (f) {
    writer.writeBool(
      5,
      f
    );
  }
  f = message.getRetainedHandling();
  if (f !== 0) {
    writer.writeUint32(
      6,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string topic = 2;
 * @return {string}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.getTopic = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.setTopic = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint32 qos = 3;
 * @return {number}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.getQos = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.setQos = function(value) {
  return jspb.Message.setProto3IntField(this, 3, value);
};


/**
 * optional bool no_local = 4;
 * @return {boolean}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.getNoLocal = function() {
  return /** @type {boolean} */ (jspb.Message.getBooleanFieldWithDefault(this, 4, false));
};


/**
 * @param {boolean} value
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.setNoLocal = function(value) {
  return jspb.Message.setProto3BooleanField(this, 4, value);
};


/**
 * optional bool retain_as_published = 5;
 * @return {boolean}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.getRetainAsPublished = function() {
  return /** @type {boolean} */ (jspb.Message.getBooleanFieldWithDefault(this, 5, false));
};


/**
 * @param {boolean} value
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.setRetainAsPublished = function(value) {
  return jspb.Message.setProto3BooleanField(this, 5, value);
};


/**
 * optional uint32 retained_handling = 6;
 * @return {number}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.getRetainedHandling = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 6, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleRequest.prototype.setRetainedHandling = function(value) {
  return jspb.Message.setProto3IntField(this, 6, value);
};





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
proto.placement.center.mqtt.SetAutoSubscribeRuleReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.SetAutoSubscribeRuleReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.SetAutoSubscribeRuleReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleReply}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.SetAutoSubscribeRuleReply;
  return proto.placement.center.mqtt.SetAutoSubscribeRuleReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.SetAutoSubscribeRuleReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.SetAutoSubscribeRuleReply}
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.SetAutoSubscribeRuleReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.SetAutoSubscribeRuleReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.SetAutoSubscribeRuleReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.SetAutoSubscribeRuleReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
topic: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest}
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest;
  return proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest}
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setTopic(value);
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
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getTopic();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string topic = 2;
 * @return {string}
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.prototype.getTopic = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleRequest.prototype.setTopic = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};





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
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.toObject = function(includeInstance, msg) {
  var f, obj = {

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
 * @return {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply}
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply;
  return proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply}
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
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
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.DeleteAutoSubscribeRuleReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};





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
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListAutoSubscribeRuleRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, "")
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
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleRequest}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListAutoSubscribeRuleRequest;
  return proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListAutoSubscribeRuleRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleRequest}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
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
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListAutoSubscribeRuleRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleRequest} returns this
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.repeatedFields_ = [1];



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
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.mqtt.ListAutoSubscribeRuleReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.toObject = function(includeInstance, msg) {
  var f, obj = {
autoSubscribeRulesList: msg.getAutoSubscribeRulesList_asB64()
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
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.mqtt.ListAutoSubscribeRuleReply;
  return proto.placement.center.mqtt.ListAutoSubscribeRuleReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addAutoSubscribeRules(value);
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
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.mqtt.ListAutoSubscribeRuleReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getAutoSubscribeRulesList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes auto_subscribe_rules = 1;
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.getAutoSubscribeRulesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes auto_subscribe_rules = 1;
 * This is a type-conversion wrapper around `getAutoSubscribeRulesList()`
 * @return {!Array<string>}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.getAutoSubscribeRulesList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getAutoSubscribeRulesList()));
};


/**
 * repeated bytes auto_subscribe_rules = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getAutoSubscribeRulesList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.getAutoSubscribeRulesList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getAutoSubscribeRulesList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply} returns this
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.setAutoSubscribeRulesList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply} returns this
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.addAutoSubscribeRules = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.mqtt.ListAutoSubscribeRuleReply} returns this
 */
proto.placement.center.mqtt.ListAutoSubscribeRuleReply.prototype.clearAutoSubscribeRulesList = function() {
  return this.setAutoSubscribeRulesList([]);
};


goog.object.extend(exports, proto.placement.center.mqtt);
