// source: inner.proto
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

var validate_validate_pb = require('./validate/validate_pb.js');
goog.object.extend(proto, validate_validate_pb);
goog.exportSymbol('proto.placement.center.inner.BindSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.BindSchemaRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.ClusterStatusReply', null, global);
goog.exportSymbol('proto.placement.center.inner.ClusterStatusRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.ClusterType', null, global);
goog.exportSymbol('proto.placement.center.inner.CreateSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.CreateSchemaRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.DeleteIdempotentDataReply', null, global);
goog.exportSymbol('proto.placement.center.inner.DeleteIdempotentDataRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.DeleteResourceConfigReply', null, global);
goog.exportSymbol('proto.placement.center.inner.DeleteResourceConfigRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.DeleteSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.DeleteSchemaRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.ExistsIdempotentDataReply', null, global);
goog.exportSymbol('proto.placement.center.inner.ExistsIdempotentDataRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.GetOffsetDataReply', null, global);
goog.exportSymbol('proto.placement.center.inner.GetOffsetDataReplyOffset', null, global);
goog.exportSymbol('proto.placement.center.inner.GetOffsetDataRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.GetResourceConfigReply', null, global);
goog.exportSymbol('proto.placement.center.inner.GetResourceConfigRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.HeartbeatReply', null, global);
goog.exportSymbol('proto.placement.center.inner.HeartbeatRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.ListBindSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.ListBindSchemaRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.ListSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.ListSchemaRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.NodeListReply', null, global);
goog.exportSymbol('proto.placement.center.inner.NodeListRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.RegisterNodeReply', null, global);
goog.exportSymbol('proto.placement.center.inner.RegisterNodeRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.ReportMonitorReply', null, global);
goog.exportSymbol('proto.placement.center.inner.ReportMonitorRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.SaveOffsetDataReply', null, global);
goog.exportSymbol('proto.placement.center.inner.SaveOffsetDataRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.SaveOffsetDataRequestOffset', null, global);
goog.exportSymbol('proto.placement.center.inner.SendRaftConfChangeReply', null, global);
goog.exportSymbol('proto.placement.center.inner.SendRaftConfChangeRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.SendRaftMessageReply', null, global);
goog.exportSymbol('proto.placement.center.inner.SendRaftMessageRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.SetIdempotentDataReply', null, global);
goog.exportSymbol('proto.placement.center.inner.SetIdempotentDataRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.SetResourceConfigReply', null, global);
goog.exportSymbol('proto.placement.center.inner.SetResourceConfigRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.UnBindSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.UnBindSchemaRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.UnRegisterNodeReply', null, global);
goog.exportSymbol('proto.placement.center.inner.UnRegisterNodeRequest', null, global);
goog.exportSymbol('proto.placement.center.inner.UpdateSchemaReply', null, global);
goog.exportSymbol('proto.placement.center.inner.UpdateSchemaRequest', null, global);
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
proto.placement.center.inner.ClusterStatusRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ClusterStatusRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ClusterStatusRequest.displayName = 'proto.placement.center.inner.ClusterStatusRequest';
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
proto.placement.center.inner.ClusterStatusReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ClusterStatusReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ClusterStatusReply.displayName = 'proto.placement.center.inner.ClusterStatusReply';
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
proto.placement.center.inner.NodeListRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.NodeListRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.NodeListRequest.displayName = 'proto.placement.center.inner.NodeListRequest';
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
proto.placement.center.inner.NodeListReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.NodeListReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.NodeListReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.NodeListReply.displayName = 'proto.placement.center.inner.NodeListReply';
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
proto.placement.center.inner.RegisterNodeRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.RegisterNodeRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.RegisterNodeRequest.displayName = 'proto.placement.center.inner.RegisterNodeRequest';
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
proto.placement.center.inner.RegisterNodeReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.RegisterNodeReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.RegisterNodeReply.displayName = 'proto.placement.center.inner.RegisterNodeReply';
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
proto.placement.center.inner.UnRegisterNodeRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.UnRegisterNodeRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.UnRegisterNodeRequest.displayName = 'proto.placement.center.inner.UnRegisterNodeRequest';
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
proto.placement.center.inner.UnRegisterNodeReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.UnRegisterNodeReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.UnRegisterNodeReply.displayName = 'proto.placement.center.inner.UnRegisterNodeReply';
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
proto.placement.center.inner.HeartbeatRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.HeartbeatRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.HeartbeatRequest.displayName = 'proto.placement.center.inner.HeartbeatRequest';
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
proto.placement.center.inner.HeartbeatReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.HeartbeatReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.HeartbeatReply.displayName = 'proto.placement.center.inner.HeartbeatReply';
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
proto.placement.center.inner.ReportMonitorRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ReportMonitorRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ReportMonitorRequest.displayName = 'proto.placement.center.inner.ReportMonitorRequest';
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
proto.placement.center.inner.ReportMonitorReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ReportMonitorReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ReportMonitorReply.displayName = 'proto.placement.center.inner.ReportMonitorReply';
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
proto.placement.center.inner.SendRaftMessageRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SendRaftMessageRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SendRaftMessageRequest.displayName = 'proto.placement.center.inner.SendRaftMessageRequest';
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
proto.placement.center.inner.SendRaftMessageReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SendRaftMessageReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SendRaftMessageReply.displayName = 'proto.placement.center.inner.SendRaftMessageReply';
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
proto.placement.center.inner.SendRaftConfChangeRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SendRaftConfChangeRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SendRaftConfChangeRequest.displayName = 'proto.placement.center.inner.SendRaftConfChangeRequest';
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
proto.placement.center.inner.SendRaftConfChangeReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SendRaftConfChangeReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SendRaftConfChangeReply.displayName = 'proto.placement.center.inner.SendRaftConfChangeReply';
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
proto.placement.center.inner.SetResourceConfigRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.SetResourceConfigRequest.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.SetResourceConfigRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SetResourceConfigRequest.displayName = 'proto.placement.center.inner.SetResourceConfigRequest';
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
proto.placement.center.inner.SetResourceConfigReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SetResourceConfigReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SetResourceConfigReply.displayName = 'proto.placement.center.inner.SetResourceConfigReply';
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
proto.placement.center.inner.GetResourceConfigRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.GetResourceConfigRequest.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.GetResourceConfigRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.GetResourceConfigRequest.displayName = 'proto.placement.center.inner.GetResourceConfigRequest';
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
proto.placement.center.inner.GetResourceConfigReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.GetResourceConfigReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.GetResourceConfigReply.displayName = 'proto.placement.center.inner.GetResourceConfigReply';
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
proto.placement.center.inner.DeleteResourceConfigRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.DeleteResourceConfigRequest.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.DeleteResourceConfigRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.DeleteResourceConfigRequest.displayName = 'proto.placement.center.inner.DeleteResourceConfigRequest';
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
proto.placement.center.inner.DeleteResourceConfigReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.DeleteResourceConfigReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.DeleteResourceConfigReply.displayName = 'proto.placement.center.inner.DeleteResourceConfigReply';
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
proto.placement.center.inner.SetIdempotentDataRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SetIdempotentDataRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SetIdempotentDataRequest.displayName = 'proto.placement.center.inner.SetIdempotentDataRequest';
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
proto.placement.center.inner.SetIdempotentDataReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SetIdempotentDataReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SetIdempotentDataReply.displayName = 'proto.placement.center.inner.SetIdempotentDataReply';
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
proto.placement.center.inner.ExistsIdempotentDataRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ExistsIdempotentDataRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ExistsIdempotentDataRequest.displayName = 'proto.placement.center.inner.ExistsIdempotentDataRequest';
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
proto.placement.center.inner.ExistsIdempotentDataReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ExistsIdempotentDataReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ExistsIdempotentDataReply.displayName = 'proto.placement.center.inner.ExistsIdempotentDataReply';
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
proto.placement.center.inner.DeleteIdempotentDataRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.DeleteIdempotentDataRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.DeleteIdempotentDataRequest.displayName = 'proto.placement.center.inner.DeleteIdempotentDataRequest';
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
proto.placement.center.inner.DeleteIdempotentDataReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.DeleteIdempotentDataReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.DeleteIdempotentDataReply.displayName = 'proto.placement.center.inner.DeleteIdempotentDataReply';
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
proto.placement.center.inner.SaveOffsetDataRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.SaveOffsetDataRequest.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.SaveOffsetDataRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SaveOffsetDataRequest.displayName = 'proto.placement.center.inner.SaveOffsetDataRequest';
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
proto.placement.center.inner.SaveOffsetDataRequestOffset = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SaveOffsetDataRequestOffset, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SaveOffsetDataRequestOffset.displayName = 'proto.placement.center.inner.SaveOffsetDataRequestOffset';
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
proto.placement.center.inner.SaveOffsetDataReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.SaveOffsetDataReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.SaveOffsetDataReply.displayName = 'proto.placement.center.inner.SaveOffsetDataReply';
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
proto.placement.center.inner.GetOffsetDataRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.GetOffsetDataRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.GetOffsetDataRequest.displayName = 'proto.placement.center.inner.GetOffsetDataRequest';
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
proto.placement.center.inner.GetOffsetDataReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.GetOffsetDataReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.GetOffsetDataReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.GetOffsetDataReply.displayName = 'proto.placement.center.inner.GetOffsetDataReply';
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
proto.placement.center.inner.GetOffsetDataReplyOffset = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.GetOffsetDataReplyOffset, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.GetOffsetDataReplyOffset.displayName = 'proto.placement.center.inner.GetOffsetDataReplyOffset';
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
proto.placement.center.inner.ListSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ListSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ListSchemaRequest.displayName = 'proto.placement.center.inner.ListSchemaRequest';
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
proto.placement.center.inner.ListSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.ListSchemaReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.ListSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ListSchemaReply.displayName = 'proto.placement.center.inner.ListSchemaReply';
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
proto.placement.center.inner.CreateSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.CreateSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.CreateSchemaRequest.displayName = 'proto.placement.center.inner.CreateSchemaRequest';
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
proto.placement.center.inner.CreateSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.CreateSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.CreateSchemaReply.displayName = 'proto.placement.center.inner.CreateSchemaReply';
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
proto.placement.center.inner.UpdateSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.UpdateSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.UpdateSchemaRequest.displayName = 'proto.placement.center.inner.UpdateSchemaRequest';
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
proto.placement.center.inner.UpdateSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.UpdateSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.UpdateSchemaReply.displayName = 'proto.placement.center.inner.UpdateSchemaReply';
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
proto.placement.center.inner.DeleteSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.DeleteSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.DeleteSchemaRequest.displayName = 'proto.placement.center.inner.DeleteSchemaRequest';
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
proto.placement.center.inner.DeleteSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.DeleteSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.DeleteSchemaReply.displayName = 'proto.placement.center.inner.DeleteSchemaReply';
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
proto.placement.center.inner.ListBindSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.ListBindSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ListBindSchemaRequest.displayName = 'proto.placement.center.inner.ListBindSchemaRequest';
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
proto.placement.center.inner.ListBindSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, proto.placement.center.inner.ListBindSchemaReply.repeatedFields_, null);
};
goog.inherits(proto.placement.center.inner.ListBindSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.ListBindSchemaReply.displayName = 'proto.placement.center.inner.ListBindSchemaReply';
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
proto.placement.center.inner.BindSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.BindSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.BindSchemaRequest.displayName = 'proto.placement.center.inner.BindSchemaRequest';
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
proto.placement.center.inner.BindSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.BindSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.BindSchemaReply.displayName = 'proto.placement.center.inner.BindSchemaReply';
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
proto.placement.center.inner.UnBindSchemaRequest = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.UnBindSchemaRequest, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.UnBindSchemaRequest.displayName = 'proto.placement.center.inner.UnBindSchemaRequest';
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
proto.placement.center.inner.UnBindSchemaReply = function(opt_data) {
  jspb.Message.initialize(this, opt_data, 0, -1, null, null);
};
goog.inherits(proto.placement.center.inner.UnBindSchemaReply, jspb.Message);
if (goog.DEBUG && !COMPILED) {
  /**
   * @public
   * @override
   */
  proto.placement.center.inner.UnBindSchemaReply.displayName = 'proto.placement.center.inner.UnBindSchemaReply';
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
proto.placement.center.inner.ClusterStatusRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ClusterStatusRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ClusterStatusRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ClusterStatusRequest.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.ClusterStatusRequest}
 */
proto.placement.center.inner.ClusterStatusRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ClusterStatusRequest;
  return proto.placement.center.inner.ClusterStatusRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ClusterStatusRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ClusterStatusRequest}
 */
proto.placement.center.inner.ClusterStatusRequest.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.ClusterStatusRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ClusterStatusRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ClusterStatusRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ClusterStatusRequest.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.ClusterStatusReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ClusterStatusReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ClusterStatusReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ClusterStatusReply.toObject = function(includeInstance, msg) {
  var f, obj = {
content: jspb.Message.getFieldWithDefault(msg, 1, "")
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
 * @return {!proto.placement.center.inner.ClusterStatusReply}
 */
proto.placement.center.inner.ClusterStatusReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ClusterStatusReply;
  return proto.placement.center.inner.ClusterStatusReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ClusterStatusReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ClusterStatusReply}
 */
proto.placement.center.inner.ClusterStatusReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
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
proto.placement.center.inner.ClusterStatusReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ClusterStatusReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ClusterStatusReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ClusterStatusReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getContent();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
};


/**
 * optional string content = 1;
 * @return {string}
 */
proto.placement.center.inner.ClusterStatusReply.prototype.getContent = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ClusterStatusReply} returns this
 */
proto.placement.center.inner.ClusterStatusReply.prototype.setContent = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
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
proto.placement.center.inner.NodeListRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.NodeListRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.NodeListRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.NodeListRequest.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.NodeListRequest}
 */
proto.placement.center.inner.NodeListRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.NodeListRequest;
  return proto.placement.center.inner.NodeListRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.NodeListRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.NodeListRequest}
 */
proto.placement.center.inner.NodeListRequest.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.NodeListRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.NodeListRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.NodeListRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.NodeListRequest.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.NodeListRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.NodeListRequest} returns this
 */
proto.placement.center.inner.NodeListRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.NodeListReply.repeatedFields_ = [1];



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
proto.placement.center.inner.NodeListReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.NodeListReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.NodeListReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.NodeListReply.toObject = function(includeInstance, msg) {
  var f, obj = {
nodesList: msg.getNodesList_asB64()
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
 * @return {!proto.placement.center.inner.NodeListReply}
 */
proto.placement.center.inner.NodeListReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.NodeListReply;
  return proto.placement.center.inner.NodeListReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.NodeListReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.NodeListReply}
 */
proto.placement.center.inner.NodeListReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addNodes(value);
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
proto.placement.center.inner.NodeListReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.NodeListReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.NodeListReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.NodeListReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getNodesList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes nodes = 1;
 * @return {!Array<string>}
 */
proto.placement.center.inner.NodeListReply.prototype.getNodesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes nodes = 1;
 * This is a type-conversion wrapper around `getNodesList()`
 * @return {!Array<string>}
 */
proto.placement.center.inner.NodeListReply.prototype.getNodesList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getNodesList()));
};


/**
 * repeated bytes nodes = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getNodesList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.inner.NodeListReply.prototype.getNodesList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getNodesList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.inner.NodeListReply} returns this
 */
proto.placement.center.inner.NodeListReply.prototype.setNodesList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.NodeListReply} returns this
 */
proto.placement.center.inner.NodeListReply.prototype.addNodes = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.NodeListReply} returns this
 */
proto.placement.center.inner.NodeListReply.prototype.clearNodesList = function() {
  return this.setNodesList([]);
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
proto.placement.center.inner.RegisterNodeRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.RegisterNodeRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.RegisterNodeRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.RegisterNodeRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterType: jspb.Message.getFieldWithDefault(msg, 1, 0),
clusterName: jspb.Message.getFieldWithDefault(msg, 2, ""),
nodeIp: jspb.Message.getFieldWithDefault(msg, 3, ""),
nodeId: jspb.Message.getFieldWithDefault(msg, 4, 0),
nodeInnerAddr: jspb.Message.getFieldWithDefault(msg, 5, ""),
extendInfo: jspb.Message.getFieldWithDefault(msg, 6, "")
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
 * @return {!proto.placement.center.inner.RegisterNodeRequest}
 */
proto.placement.center.inner.RegisterNodeRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.RegisterNodeRequest;
  return proto.placement.center.inner.RegisterNodeRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.RegisterNodeRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.RegisterNodeRequest}
 */
proto.placement.center.inner.RegisterNodeRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!proto.placement.center.inner.ClusterType} */ (reader.readEnum());
      msg.setClusterType(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 3:
      var value = /** @type {string} */ (reader.readString());
      msg.setNodeIp(value);
      break;
    case 4:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setNodeId(value);
      break;
    case 5:
      var value = /** @type {string} */ (reader.readString());
      msg.setNodeInnerAddr(value);
      break;
    case 6:
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
proto.placement.center.inner.RegisterNodeRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.RegisterNodeRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.RegisterNodeRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.RegisterNodeRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterType();
  if (f !== 0.0) {
    writer.writeEnum(
      1,
      f
    );
  }
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getNodeIp();
  if (f.length > 0) {
    writer.writeString(
      3,
      f
    );
  }
  f = message.getNodeId();
  if (f !== 0) {
    writer.writeUint64(
      4,
      f
    );
  }
  f = message.getNodeInnerAddr();
  if (f.length > 0) {
    writer.writeString(
      5,
      f
    );
  }
  f = message.getExtendInfo();
  if (f.length > 0) {
    writer.writeString(
      6,
      f
    );
  }
};


/**
 * optional ClusterType cluster_type = 1;
 * @return {!proto.placement.center.inner.ClusterType}
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.getClusterType = function() {
  return /** @type {!proto.placement.center.inner.ClusterType} */ (jspb.Message.getFieldWithDefault(this, 1, 0));
};


/**
 * @param {!proto.placement.center.inner.ClusterType} value
 * @return {!proto.placement.center.inner.RegisterNodeRequest} returns this
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.setClusterType = function(value) {
  return jspb.Message.setProto3EnumField(this, 1, value);
};


/**
 * optional string cluster_name = 2;
 * @return {string}
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.RegisterNodeRequest} returns this
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string node_ip = 3;
 * @return {string}
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.getNodeIp = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.RegisterNodeRequest} returns this
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.setNodeIp = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};


/**
 * optional uint64 node_id = 4;
 * @return {number}
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.getNodeId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 4, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.RegisterNodeRequest} returns this
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.setNodeId = function(value) {
  return jspb.Message.setProto3IntField(this, 4, value);
};


/**
 * optional string node_inner_addr = 5;
 * @return {string}
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.getNodeInnerAddr = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 5, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.RegisterNodeRequest} returns this
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.setNodeInnerAddr = function(value) {
  return jspb.Message.setProto3StringField(this, 5, value);
};


/**
 * optional string extend_info = 6;
 * @return {string}
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.getExtendInfo = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 6, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.RegisterNodeRequest} returns this
 */
proto.placement.center.inner.RegisterNodeRequest.prototype.setExtendInfo = function(value) {
  return jspb.Message.setProto3StringField(this, 6, value);
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
proto.placement.center.inner.RegisterNodeReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.RegisterNodeReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.RegisterNodeReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.RegisterNodeReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.RegisterNodeReply}
 */
proto.placement.center.inner.RegisterNodeReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.RegisterNodeReply;
  return proto.placement.center.inner.RegisterNodeReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.RegisterNodeReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.RegisterNodeReply}
 */
proto.placement.center.inner.RegisterNodeReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.RegisterNodeReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.RegisterNodeReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.RegisterNodeReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.RegisterNodeReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.UnRegisterNodeRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.UnRegisterNodeRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.UnRegisterNodeRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnRegisterNodeRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterType: jspb.Message.getFieldWithDefault(msg, 1, 0),
clusterName: jspb.Message.getFieldWithDefault(msg, 2, ""),
nodeId: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.inner.UnRegisterNodeRequest}
 */
proto.placement.center.inner.UnRegisterNodeRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.UnRegisterNodeRequest;
  return proto.placement.center.inner.UnRegisterNodeRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.UnRegisterNodeRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.UnRegisterNodeRequest}
 */
proto.placement.center.inner.UnRegisterNodeRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!proto.placement.center.inner.ClusterType} */ (reader.readEnum());
      msg.setClusterType(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setNodeId(value);
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
proto.placement.center.inner.UnRegisterNodeRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.UnRegisterNodeRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.UnRegisterNodeRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnRegisterNodeRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterType();
  if (f !== 0.0) {
    writer.writeEnum(
      1,
      f
    );
  }
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getNodeId();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional ClusterType cluster_type = 1;
 * @return {!proto.placement.center.inner.ClusterType}
 */
proto.placement.center.inner.UnRegisterNodeRequest.prototype.getClusterType = function() {
  return /** @type {!proto.placement.center.inner.ClusterType} */ (jspb.Message.getFieldWithDefault(this, 1, 0));
};


/**
 * @param {!proto.placement.center.inner.ClusterType} value
 * @return {!proto.placement.center.inner.UnRegisterNodeRequest} returns this
 */
proto.placement.center.inner.UnRegisterNodeRequest.prototype.setClusterType = function(value) {
  return jspb.Message.setProto3EnumField(this, 1, value);
};


/**
 * optional string cluster_name = 2;
 * @return {string}
 */
proto.placement.center.inner.UnRegisterNodeRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.UnRegisterNodeRequest} returns this
 */
proto.placement.center.inner.UnRegisterNodeRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 node_id = 3;
 * @return {number}
 */
proto.placement.center.inner.UnRegisterNodeRequest.prototype.getNodeId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.UnRegisterNodeRequest} returns this
 */
proto.placement.center.inner.UnRegisterNodeRequest.prototype.setNodeId = function(value) {
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
proto.placement.center.inner.UnRegisterNodeReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.UnRegisterNodeReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.UnRegisterNodeReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnRegisterNodeReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.UnRegisterNodeReply}
 */
proto.placement.center.inner.UnRegisterNodeReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.UnRegisterNodeReply;
  return proto.placement.center.inner.UnRegisterNodeReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.UnRegisterNodeReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.UnRegisterNodeReply}
 */
proto.placement.center.inner.UnRegisterNodeReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.UnRegisterNodeReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.UnRegisterNodeReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.UnRegisterNodeReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnRegisterNodeReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.HeartbeatRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.HeartbeatRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.HeartbeatRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.HeartbeatRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterType: jspb.Message.getFieldWithDefault(msg, 1, 0),
clusterName: jspb.Message.getFieldWithDefault(msg, 2, ""),
nodeId: jspb.Message.getFieldWithDefault(msg, 4, 0)
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
 * @return {!proto.placement.center.inner.HeartbeatRequest}
 */
proto.placement.center.inner.HeartbeatRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.HeartbeatRequest;
  return proto.placement.center.inner.HeartbeatRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.HeartbeatRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.HeartbeatRequest}
 */
proto.placement.center.inner.HeartbeatRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!proto.placement.center.inner.ClusterType} */ (reader.readEnum());
      msg.setClusterType(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setClusterName(value);
      break;
    case 4:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setNodeId(value);
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
proto.placement.center.inner.HeartbeatRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.HeartbeatRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.HeartbeatRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.HeartbeatRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterType();
  if (f !== 0.0) {
    writer.writeEnum(
      1,
      f
    );
  }
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getNodeId();
  if (f !== 0) {
    writer.writeUint64(
      4,
      f
    );
  }
};


/**
 * optional ClusterType cluster_type = 1;
 * @return {!proto.placement.center.inner.ClusterType}
 */
proto.placement.center.inner.HeartbeatRequest.prototype.getClusterType = function() {
  return /** @type {!proto.placement.center.inner.ClusterType} */ (jspb.Message.getFieldWithDefault(this, 1, 0));
};


/**
 * @param {!proto.placement.center.inner.ClusterType} value
 * @return {!proto.placement.center.inner.HeartbeatRequest} returns this
 */
proto.placement.center.inner.HeartbeatRequest.prototype.setClusterType = function(value) {
  return jspb.Message.setProto3EnumField(this, 1, value);
};


/**
 * optional string cluster_name = 2;
 * @return {string}
 */
proto.placement.center.inner.HeartbeatRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.HeartbeatRequest} returns this
 */
proto.placement.center.inner.HeartbeatRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 node_id = 4;
 * @return {number}
 */
proto.placement.center.inner.HeartbeatRequest.prototype.getNodeId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 4, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.HeartbeatRequest} returns this
 */
proto.placement.center.inner.HeartbeatRequest.prototype.setNodeId = function(value) {
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
proto.placement.center.inner.HeartbeatReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.HeartbeatReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.HeartbeatReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.HeartbeatReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.HeartbeatReply}
 */
proto.placement.center.inner.HeartbeatReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.HeartbeatReply;
  return proto.placement.center.inner.HeartbeatReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.HeartbeatReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.HeartbeatReply}
 */
proto.placement.center.inner.HeartbeatReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.HeartbeatReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.HeartbeatReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.HeartbeatReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.HeartbeatReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.ReportMonitorRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ReportMonitorRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ReportMonitorRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ReportMonitorRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
nodeId: jspb.Message.getFieldWithDefault(msg, 2, 0),
cpuRate: jspb.Message.getFloatingPointFieldWithDefault(msg, 3, 0.0),
memoryRate: jspb.Message.getFloatingPointFieldWithDefault(msg, 4, 0.0),
diskRate: jspb.Message.getFloatingPointFieldWithDefault(msg, 5, 0.0),
networkRate: jspb.Message.getFloatingPointFieldWithDefault(msg, 6, 0.0)
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
 * @return {!proto.placement.center.inner.ReportMonitorRequest}
 */
proto.placement.center.inner.ReportMonitorRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ReportMonitorRequest;
  return proto.placement.center.inner.ReportMonitorRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ReportMonitorRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ReportMonitorRequest}
 */
proto.placement.center.inner.ReportMonitorRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      var value = /** @type {number} */ (reader.readUint64());
      msg.setNodeId(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readFloat());
      msg.setCpuRate(value);
      break;
    case 4:
      var value = /** @type {number} */ (reader.readFloat());
      msg.setMemoryRate(value);
      break;
    case 5:
      var value = /** @type {number} */ (reader.readFloat());
      msg.setDiskRate(value);
      break;
    case 6:
      var value = /** @type {number} */ (reader.readFloat());
      msg.setNetworkRate(value);
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
proto.placement.center.inner.ReportMonitorRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ReportMonitorRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ReportMonitorRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ReportMonitorRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getNodeId();
  if (f !== 0) {
    writer.writeUint64(
      2,
      f
    );
  }
  f = message.getCpuRate();
  if (f !== 0.0) {
    writer.writeFloat(
      3,
      f
    );
  }
  f = message.getMemoryRate();
  if (f !== 0.0) {
    writer.writeFloat(
      4,
      f
    );
  }
  f = message.getDiskRate();
  if (f !== 0.0) {
    writer.writeFloat(
      5,
      f
    );
  }
  f = message.getNetworkRate();
  if (f !== 0.0) {
    writer.writeFloat(
      6,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ReportMonitorRequest} returns this
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional uint64 node_id = 2;
 * @return {number}
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.getNodeId = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 2, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.ReportMonitorRequest} returns this
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.setNodeId = function(value) {
  return jspb.Message.setProto3IntField(this, 2, value);
};


/**
 * optional float cpu_rate = 3;
 * @return {number}
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.getCpuRate = function() {
  return /** @type {number} */ (jspb.Message.getFloatingPointFieldWithDefault(this, 3, 0.0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.ReportMonitorRequest} returns this
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.setCpuRate = function(value) {
  return jspb.Message.setProto3FloatField(this, 3, value);
};


/**
 * optional float memory_rate = 4;
 * @return {number}
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.getMemoryRate = function() {
  return /** @type {number} */ (jspb.Message.getFloatingPointFieldWithDefault(this, 4, 0.0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.ReportMonitorRequest} returns this
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.setMemoryRate = function(value) {
  return jspb.Message.setProto3FloatField(this, 4, value);
};


/**
 * optional float disk_rate = 5;
 * @return {number}
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.getDiskRate = function() {
  return /** @type {number} */ (jspb.Message.getFloatingPointFieldWithDefault(this, 5, 0.0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.ReportMonitorRequest} returns this
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.setDiskRate = function(value) {
  return jspb.Message.setProto3FloatField(this, 5, value);
};


/**
 * optional float network_rate = 6;
 * @return {number}
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.getNetworkRate = function() {
  return /** @type {number} */ (jspb.Message.getFloatingPointFieldWithDefault(this, 6, 0.0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.ReportMonitorRequest} returns this
 */
proto.placement.center.inner.ReportMonitorRequest.prototype.setNetworkRate = function(value) {
  return jspb.Message.setProto3FloatField(this, 6, value);
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
proto.placement.center.inner.ReportMonitorReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ReportMonitorReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ReportMonitorReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ReportMonitorReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.ReportMonitorReply}
 */
proto.placement.center.inner.ReportMonitorReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ReportMonitorReply;
  return proto.placement.center.inner.ReportMonitorReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ReportMonitorReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ReportMonitorReply}
 */
proto.placement.center.inner.ReportMonitorReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.ReportMonitorReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ReportMonitorReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ReportMonitorReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ReportMonitorReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.SendRaftMessageRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SendRaftMessageRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SendRaftMessageRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftMessageRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
message: msg.getMessage_asB64()
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
 * @return {!proto.placement.center.inner.SendRaftMessageRequest}
 */
proto.placement.center.inner.SendRaftMessageRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SendRaftMessageRequest;
  return proto.placement.center.inner.SendRaftMessageRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SendRaftMessageRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SendRaftMessageRequest}
 */
proto.placement.center.inner.SendRaftMessageRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setMessage(value);
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
proto.placement.center.inner.SendRaftMessageRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SendRaftMessageRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SendRaftMessageRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftMessageRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getMessage_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      1,
      f
    );
  }
};


/**
 * optional bytes message = 1;
 * @return {string}
 */
proto.placement.center.inner.SendRaftMessageRequest.prototype.getMessage = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * optional bytes message = 1;
 * This is a type-conversion wrapper around `getMessage()`
 * @return {string}
 */
proto.placement.center.inner.SendRaftMessageRequest.prototype.getMessage_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getMessage()));
};


/**
 * optional bytes message = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getMessage()`
 * @return {!Uint8Array}
 */
proto.placement.center.inner.SendRaftMessageRequest.prototype.getMessage_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getMessage()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.inner.SendRaftMessageRequest} returns this
 */
proto.placement.center.inner.SendRaftMessageRequest.prototype.setMessage = function(value) {
  return jspb.Message.setProto3BytesField(this, 1, value);
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
proto.placement.center.inner.SendRaftMessageReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SendRaftMessageReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SendRaftMessageReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftMessageReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.SendRaftMessageReply}
 */
proto.placement.center.inner.SendRaftMessageReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SendRaftMessageReply;
  return proto.placement.center.inner.SendRaftMessageReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SendRaftMessageReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SendRaftMessageReply}
 */
proto.placement.center.inner.SendRaftMessageReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.SendRaftMessageReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SendRaftMessageReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SendRaftMessageReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftMessageReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.SendRaftConfChangeRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SendRaftConfChangeRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SendRaftConfChangeRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftConfChangeRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
message: msg.getMessage_asB64()
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
 * @return {!proto.placement.center.inner.SendRaftConfChangeRequest}
 */
proto.placement.center.inner.SendRaftConfChangeRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SendRaftConfChangeRequest;
  return proto.placement.center.inner.SendRaftConfChangeRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SendRaftConfChangeRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SendRaftConfChangeRequest}
 */
proto.placement.center.inner.SendRaftConfChangeRequest.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setMessage(value);
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
proto.placement.center.inner.SendRaftConfChangeRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SendRaftConfChangeRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SendRaftConfChangeRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftConfChangeRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getMessage_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      1,
      f
    );
  }
};


/**
 * optional bytes message = 1;
 * @return {string}
 */
proto.placement.center.inner.SendRaftConfChangeRequest.prototype.getMessage = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * optional bytes message = 1;
 * This is a type-conversion wrapper around `getMessage()`
 * @return {string}
 */
proto.placement.center.inner.SendRaftConfChangeRequest.prototype.getMessage_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getMessage()));
};


/**
 * optional bytes message = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getMessage()`
 * @return {!Uint8Array}
 */
proto.placement.center.inner.SendRaftConfChangeRequest.prototype.getMessage_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getMessage()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.inner.SendRaftConfChangeRequest} returns this
 */
proto.placement.center.inner.SendRaftConfChangeRequest.prototype.setMessage = function(value) {
  return jspb.Message.setProto3BytesField(this, 1, value);
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
proto.placement.center.inner.SendRaftConfChangeReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SendRaftConfChangeReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SendRaftConfChangeReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftConfChangeReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.SendRaftConfChangeReply}
 */
proto.placement.center.inner.SendRaftConfChangeReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SendRaftConfChangeReply;
  return proto.placement.center.inner.SendRaftConfChangeReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SendRaftConfChangeReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SendRaftConfChangeReply}
 */
proto.placement.center.inner.SendRaftConfChangeReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.SendRaftConfChangeReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SendRaftConfChangeReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SendRaftConfChangeReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SendRaftConfChangeReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.SetResourceConfigRequest.repeatedFields_ = [2];



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
proto.placement.center.inner.SetResourceConfigRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SetResourceConfigRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SetResourceConfigRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetResourceConfigRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
resourcesList: (f = jspb.Message.getRepeatedField(msg, 2)) == null ? undefined : f,
config: msg.getConfig_asB64()
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
 * @return {!proto.placement.center.inner.SetResourceConfigRequest}
 */
proto.placement.center.inner.SetResourceConfigRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SetResourceConfigRequest;
  return proto.placement.center.inner.SetResourceConfigRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SetResourceConfigRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SetResourceConfigRequest}
 */
proto.placement.center.inner.SetResourceConfigRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.addResources(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setConfig(value);
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
proto.placement.center.inner.SetResourceConfigRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SetResourceConfigRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SetResourceConfigRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetResourceConfigRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getResourcesList();
  if (f.length > 0) {
    writer.writeRepeatedString(
      2,
      f
    );
  }
  f = message.getConfig_asU8();
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
proto.placement.center.inner.SetResourceConfigRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SetResourceConfigRequest} returns this
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * repeated string resources = 2;
 * @return {!Array<string>}
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.getResourcesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 2));
};


/**
 * @param {!Array<string>} value
 * @return {!proto.placement.center.inner.SetResourceConfigRequest} returns this
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.setResourcesList = function(value) {
  return jspb.Message.setField(this, 2, value || []);
};


/**
 * @param {string} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.SetResourceConfigRequest} returns this
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.addResources = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 2, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.SetResourceConfigRequest} returns this
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.clearResourcesList = function() {
  return this.setResourcesList([]);
};


/**
 * optional bytes config = 3;
 * @return {string}
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.getConfig = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes config = 3;
 * This is a type-conversion wrapper around `getConfig()`
 * @return {string}
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.getConfig_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getConfig()));
};


/**
 * optional bytes config = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getConfig()`
 * @return {!Uint8Array}
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.getConfig_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getConfig()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.inner.SetResourceConfigRequest} returns this
 */
proto.placement.center.inner.SetResourceConfigRequest.prototype.setConfig = function(value) {
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
proto.placement.center.inner.SetResourceConfigReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SetResourceConfigReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SetResourceConfigReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetResourceConfigReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.SetResourceConfigReply}
 */
proto.placement.center.inner.SetResourceConfigReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SetResourceConfigReply;
  return proto.placement.center.inner.SetResourceConfigReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SetResourceConfigReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SetResourceConfigReply}
 */
proto.placement.center.inner.SetResourceConfigReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.SetResourceConfigReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SetResourceConfigReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SetResourceConfigReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetResourceConfigReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.GetResourceConfigRequest.repeatedFields_ = [2];



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
proto.placement.center.inner.GetResourceConfigRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.GetResourceConfigRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.GetResourceConfigRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetResourceConfigRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
resourcesList: (f = jspb.Message.getRepeatedField(msg, 2)) == null ? undefined : f
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
 * @return {!proto.placement.center.inner.GetResourceConfigRequest}
 */
proto.placement.center.inner.GetResourceConfigRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.GetResourceConfigRequest;
  return proto.placement.center.inner.GetResourceConfigRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.GetResourceConfigRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.GetResourceConfigRequest}
 */
proto.placement.center.inner.GetResourceConfigRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.addResources(value);
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
proto.placement.center.inner.GetResourceConfigRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.GetResourceConfigRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.GetResourceConfigRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetResourceConfigRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getResourcesList();
  if (f.length > 0) {
    writer.writeRepeatedString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.GetResourceConfigRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.GetResourceConfigRequest} returns this
 */
proto.placement.center.inner.GetResourceConfigRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * repeated string resources = 2;
 * @return {!Array<string>}
 */
proto.placement.center.inner.GetResourceConfigRequest.prototype.getResourcesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 2));
};


/**
 * @param {!Array<string>} value
 * @return {!proto.placement.center.inner.GetResourceConfigRequest} returns this
 */
proto.placement.center.inner.GetResourceConfigRequest.prototype.setResourcesList = function(value) {
  return jspb.Message.setField(this, 2, value || []);
};


/**
 * @param {string} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.GetResourceConfigRequest} returns this
 */
proto.placement.center.inner.GetResourceConfigRequest.prototype.addResources = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 2, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.GetResourceConfigRequest} returns this
 */
proto.placement.center.inner.GetResourceConfigRequest.prototype.clearResourcesList = function() {
  return this.setResourcesList([]);
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
proto.placement.center.inner.GetResourceConfigReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.GetResourceConfigReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.GetResourceConfigReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetResourceConfigReply.toObject = function(includeInstance, msg) {
  var f, obj = {
config: msg.getConfig_asB64()
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
 * @return {!proto.placement.center.inner.GetResourceConfigReply}
 */
proto.placement.center.inner.GetResourceConfigReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.GetResourceConfigReply;
  return proto.placement.center.inner.GetResourceConfigReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.GetResourceConfigReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.GetResourceConfigReply}
 */
proto.placement.center.inner.GetResourceConfigReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setConfig(value);
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
proto.placement.center.inner.GetResourceConfigReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.GetResourceConfigReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.GetResourceConfigReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetResourceConfigReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getConfig_asU8();
  if (f.length > 0) {
    writer.writeBytes(
      1,
      f
    );
  }
};


/**
 * optional bytes config = 1;
 * @return {string}
 */
proto.placement.center.inner.GetResourceConfigReply.prototype.getConfig = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * optional bytes config = 1;
 * This is a type-conversion wrapper around `getConfig()`
 * @return {string}
 */
proto.placement.center.inner.GetResourceConfigReply.prototype.getConfig_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getConfig()));
};


/**
 * optional bytes config = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getConfig()`
 * @return {!Uint8Array}
 */
proto.placement.center.inner.GetResourceConfigReply.prototype.getConfig_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getConfig()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.inner.GetResourceConfigReply} returns this
 */
proto.placement.center.inner.GetResourceConfigReply.prototype.setConfig = function(value) {
  return jspb.Message.setProto3BytesField(this, 1, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.DeleteResourceConfigRequest.repeatedFields_ = [2];



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
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.DeleteResourceConfigRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.DeleteResourceConfigRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteResourceConfigRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
resourcesList: (f = jspb.Message.getRepeatedField(msg, 2)) == null ? undefined : f
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
 * @return {!proto.placement.center.inner.DeleteResourceConfigRequest}
 */
proto.placement.center.inner.DeleteResourceConfigRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.DeleteResourceConfigRequest;
  return proto.placement.center.inner.DeleteResourceConfigRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.DeleteResourceConfigRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.DeleteResourceConfigRequest}
 */
proto.placement.center.inner.DeleteResourceConfigRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.addResources(value);
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
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.DeleteResourceConfigRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.DeleteResourceConfigRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteResourceConfigRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getResourcesList();
  if (f.length > 0) {
    writer.writeRepeatedString(
      2,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.DeleteResourceConfigRequest} returns this
 */
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * repeated string resources = 2;
 * @return {!Array<string>}
 */
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.getResourcesList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 2));
};


/**
 * @param {!Array<string>} value
 * @return {!proto.placement.center.inner.DeleteResourceConfigRequest} returns this
 */
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.setResourcesList = function(value) {
  return jspb.Message.setField(this, 2, value || []);
};


/**
 * @param {string} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.DeleteResourceConfigRequest} returns this
 */
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.addResources = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 2, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.DeleteResourceConfigRequest} returns this
 */
proto.placement.center.inner.DeleteResourceConfigRequest.prototype.clearResourcesList = function() {
  return this.setResourcesList([]);
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
proto.placement.center.inner.DeleteResourceConfigReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.DeleteResourceConfigReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.DeleteResourceConfigReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteResourceConfigReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.DeleteResourceConfigReply}
 */
proto.placement.center.inner.DeleteResourceConfigReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.DeleteResourceConfigReply;
  return proto.placement.center.inner.DeleteResourceConfigReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.DeleteResourceConfigReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.DeleteResourceConfigReply}
 */
proto.placement.center.inner.DeleteResourceConfigReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.DeleteResourceConfigReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.DeleteResourceConfigReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.DeleteResourceConfigReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteResourceConfigReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.SetIdempotentDataRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SetIdempotentDataRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SetIdempotentDataRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetIdempotentDataRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
producerId: jspb.Message.getFieldWithDefault(msg, 2, ""),
seqNum: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.inner.SetIdempotentDataRequest}
 */
proto.placement.center.inner.SetIdempotentDataRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SetIdempotentDataRequest;
  return proto.placement.center.inner.SetIdempotentDataRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SetIdempotentDataRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SetIdempotentDataRequest}
 */
proto.placement.center.inner.SetIdempotentDataRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setProducerId(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setSeqNum(value);
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
proto.placement.center.inner.SetIdempotentDataRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SetIdempotentDataRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SetIdempotentDataRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetIdempotentDataRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getProducerId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSeqNum();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.SetIdempotentDataRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SetIdempotentDataRequest} returns this
 */
proto.placement.center.inner.SetIdempotentDataRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string producer_id = 2;
 * @return {string}
 */
proto.placement.center.inner.SetIdempotentDataRequest.prototype.getProducerId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SetIdempotentDataRequest} returns this
 */
proto.placement.center.inner.SetIdempotentDataRequest.prototype.setProducerId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 seq_num = 3;
 * @return {number}
 */
proto.placement.center.inner.SetIdempotentDataRequest.prototype.getSeqNum = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.SetIdempotentDataRequest} returns this
 */
proto.placement.center.inner.SetIdempotentDataRequest.prototype.setSeqNum = function(value) {
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
proto.placement.center.inner.SetIdempotentDataReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SetIdempotentDataReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SetIdempotentDataReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetIdempotentDataReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.SetIdempotentDataReply}
 */
proto.placement.center.inner.SetIdempotentDataReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SetIdempotentDataReply;
  return proto.placement.center.inner.SetIdempotentDataReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SetIdempotentDataReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SetIdempotentDataReply}
 */
proto.placement.center.inner.SetIdempotentDataReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.SetIdempotentDataReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SetIdempotentDataReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SetIdempotentDataReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SetIdempotentDataReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ExistsIdempotentDataRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ExistsIdempotentDataRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
producerId: jspb.Message.getFieldWithDefault(msg, 2, ""),
seqNum: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.inner.ExistsIdempotentDataRequest}
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ExistsIdempotentDataRequest;
  return proto.placement.center.inner.ExistsIdempotentDataRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ExistsIdempotentDataRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ExistsIdempotentDataRequest}
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setProducerId(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setSeqNum(value);
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
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ExistsIdempotentDataRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ExistsIdempotentDataRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getProducerId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSeqNum();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ExistsIdempotentDataRequest} returns this
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string producer_id = 2;
 * @return {string}
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.getProducerId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ExistsIdempotentDataRequest} returns this
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.setProducerId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 seq_num = 3;
 * @return {number}
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.getSeqNum = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.ExistsIdempotentDataRequest} returns this
 */
proto.placement.center.inner.ExistsIdempotentDataRequest.prototype.setSeqNum = function(value) {
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
proto.placement.center.inner.ExistsIdempotentDataReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ExistsIdempotentDataReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ExistsIdempotentDataReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ExistsIdempotentDataReply.toObject = function(includeInstance, msg) {
  var f, obj = {
exists: jspb.Message.getBooleanFieldWithDefault(msg, 1, false)
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
 * @return {!proto.placement.center.inner.ExistsIdempotentDataReply}
 */
proto.placement.center.inner.ExistsIdempotentDataReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ExistsIdempotentDataReply;
  return proto.placement.center.inner.ExistsIdempotentDataReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ExistsIdempotentDataReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ExistsIdempotentDataReply}
 */
proto.placement.center.inner.ExistsIdempotentDataReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {boolean} */ (reader.readBool());
      msg.setExists(value);
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
proto.placement.center.inner.ExistsIdempotentDataReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ExistsIdempotentDataReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ExistsIdempotentDataReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ExistsIdempotentDataReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getExists();
  if (f) {
    writer.writeBool(
      1,
      f
    );
  }
};


/**
 * optional bool exists = 1;
 * @return {boolean}
 */
proto.placement.center.inner.ExistsIdempotentDataReply.prototype.getExists = function() {
  return /** @type {boolean} */ (jspb.Message.getBooleanFieldWithDefault(this, 1, false));
};


/**
 * @param {boolean} value
 * @return {!proto.placement.center.inner.ExistsIdempotentDataReply} returns this
 */
proto.placement.center.inner.ExistsIdempotentDataReply.prototype.setExists = function(value) {
  return jspb.Message.setProto3BooleanField(this, 1, value);
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
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.DeleteIdempotentDataRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.DeleteIdempotentDataRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
producerId: jspb.Message.getFieldWithDefault(msg, 2, ""),
seqNum: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.inner.DeleteIdempotentDataRequest}
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.DeleteIdempotentDataRequest;
  return proto.placement.center.inner.DeleteIdempotentDataRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.DeleteIdempotentDataRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.DeleteIdempotentDataRequest}
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setProducerId(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setSeqNum(value);
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
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.DeleteIdempotentDataRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.DeleteIdempotentDataRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getProducerId();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSeqNum();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.DeleteIdempotentDataRequest} returns this
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string producer_id = 2;
 * @return {string}
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.getProducerId = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.DeleteIdempotentDataRequest} returns this
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.setProducerId = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 seq_num = 3;
 * @return {number}
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.getSeqNum = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.DeleteIdempotentDataRequest} returns this
 */
proto.placement.center.inner.DeleteIdempotentDataRequest.prototype.setSeqNum = function(value) {
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
proto.placement.center.inner.DeleteIdempotentDataReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.DeleteIdempotentDataReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.DeleteIdempotentDataReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteIdempotentDataReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.DeleteIdempotentDataReply}
 */
proto.placement.center.inner.DeleteIdempotentDataReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.DeleteIdempotentDataReply;
  return proto.placement.center.inner.DeleteIdempotentDataReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.DeleteIdempotentDataReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.DeleteIdempotentDataReply}
 */
proto.placement.center.inner.DeleteIdempotentDataReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.DeleteIdempotentDataReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.DeleteIdempotentDataReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.DeleteIdempotentDataReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteIdempotentDataReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.SaveOffsetDataRequest.repeatedFields_ = [3];



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
proto.placement.center.inner.SaveOffsetDataRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SaveOffsetDataRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SaveOffsetDataRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SaveOffsetDataRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
group: jspb.Message.getFieldWithDefault(msg, 2, ""),
offsetsList: jspb.Message.toObjectList(msg.getOffsetsList(),
    proto.placement.center.inner.SaveOffsetDataRequestOffset.toObject, includeInstance)
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
 * @return {!proto.placement.center.inner.SaveOffsetDataRequest}
 */
proto.placement.center.inner.SaveOffsetDataRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SaveOffsetDataRequest;
  return proto.placement.center.inner.SaveOffsetDataRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SaveOffsetDataRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SaveOffsetDataRequest}
 */
proto.placement.center.inner.SaveOffsetDataRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setGroup(value);
      break;
    case 3:
      var value = new proto.placement.center.inner.SaveOffsetDataRequestOffset;
      reader.readMessage(value,proto.placement.center.inner.SaveOffsetDataRequestOffset.deserializeBinaryFromReader);
      msg.addOffsets(value);
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
proto.placement.center.inner.SaveOffsetDataRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SaveOffsetDataRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SaveOffsetDataRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SaveOffsetDataRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getGroup();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getOffsetsList();
  if (f.length > 0) {
    writer.writeRepeatedMessage(
      3,
      f,
      proto.placement.center.inner.SaveOffsetDataRequestOffset.serializeBinaryToWriter
    );
  }
};


/**
 * optional string cluster_name = 1;
 * @return {string}
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SaveOffsetDataRequest} returns this
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string group = 2;
 * @return {string}
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.getGroup = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SaveOffsetDataRequest} returns this
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.setGroup = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * repeated SaveOffsetDataRequestOffset offsets = 3;
 * @return {!Array<!proto.placement.center.inner.SaveOffsetDataRequestOffset>}
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.getOffsetsList = function() {
  return /** @type{!Array<!proto.placement.center.inner.SaveOffsetDataRequestOffset>} */ (
    jspb.Message.getRepeatedWrapperField(this, proto.placement.center.inner.SaveOffsetDataRequestOffset, 3));
};


/**
 * @param {!Array<!proto.placement.center.inner.SaveOffsetDataRequestOffset>} value
 * @return {!proto.placement.center.inner.SaveOffsetDataRequest} returns this
*/
proto.placement.center.inner.SaveOffsetDataRequest.prototype.setOffsetsList = function(value) {
  return jspb.Message.setRepeatedWrapperField(this, 3, value);
};


/**
 * @param {!proto.placement.center.inner.SaveOffsetDataRequestOffset=} opt_value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.SaveOffsetDataRequestOffset}
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.addOffsets = function(opt_value, opt_index) {
  return jspb.Message.addToRepeatedWrapperField(this, 3, opt_value, proto.placement.center.inner.SaveOffsetDataRequestOffset, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.SaveOffsetDataRequest} returns this
 */
proto.placement.center.inner.SaveOffsetDataRequest.prototype.clearOffsetsList = function() {
  return this.setOffsetsList([]);
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
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SaveOffsetDataRequestOffset.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SaveOffsetDataRequestOffset} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.toObject = function(includeInstance, msg) {
  var f, obj = {
namespace: jspb.Message.getFieldWithDefault(msg, 1, ""),
shardName: jspb.Message.getFieldWithDefault(msg, 2, ""),
offset: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.inner.SaveOffsetDataRequestOffset}
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SaveOffsetDataRequestOffset;
  return proto.placement.center.inner.SaveOffsetDataRequestOffset.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SaveOffsetDataRequestOffset} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SaveOffsetDataRequestOffset}
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setNamespace(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setShardName(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setOffset(value);
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
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SaveOffsetDataRequestOffset.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SaveOffsetDataRequestOffset} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getNamespace();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getShardName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getOffset();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional string namespace = 1;
 * @return {string}
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.getNamespace = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SaveOffsetDataRequestOffset} returns this
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.setNamespace = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string shard_name = 2;
 * @return {string}
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.getShardName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.SaveOffsetDataRequestOffset} returns this
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.setShardName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 offset = 3;
 * @return {number}
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.getOffset = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.SaveOffsetDataRequestOffset} returns this
 */
proto.placement.center.inner.SaveOffsetDataRequestOffset.prototype.setOffset = function(value) {
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
proto.placement.center.inner.SaveOffsetDataReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.SaveOffsetDataReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.SaveOffsetDataReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SaveOffsetDataReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.SaveOffsetDataReply}
 */
proto.placement.center.inner.SaveOffsetDataReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.SaveOffsetDataReply;
  return proto.placement.center.inner.SaveOffsetDataReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.SaveOffsetDataReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.SaveOffsetDataReply}
 */
proto.placement.center.inner.SaveOffsetDataReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.SaveOffsetDataReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.SaveOffsetDataReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.SaveOffsetDataReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.SaveOffsetDataReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.GetOffsetDataRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.GetOffsetDataRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.GetOffsetDataRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetOffsetDataRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
group: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.inner.GetOffsetDataRequest}
 */
proto.placement.center.inner.GetOffsetDataRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.GetOffsetDataRequest;
  return proto.placement.center.inner.GetOffsetDataRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.GetOffsetDataRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.GetOffsetDataRequest}
 */
proto.placement.center.inner.GetOffsetDataRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setGroup(value);
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
proto.placement.center.inner.GetOffsetDataRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.GetOffsetDataRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.GetOffsetDataRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetOffsetDataRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getGroup();
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
proto.placement.center.inner.GetOffsetDataRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.GetOffsetDataRequest} returns this
 */
proto.placement.center.inner.GetOffsetDataRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string group = 2;
 * @return {string}
 */
proto.placement.center.inner.GetOffsetDataRequest.prototype.getGroup = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.GetOffsetDataRequest} returns this
 */
proto.placement.center.inner.GetOffsetDataRequest.prototype.setGroup = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.GetOffsetDataReply.repeatedFields_ = [1];



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
proto.placement.center.inner.GetOffsetDataReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.GetOffsetDataReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.GetOffsetDataReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetOffsetDataReply.toObject = function(includeInstance, msg) {
  var f, obj = {
offsetsList: jspb.Message.toObjectList(msg.getOffsetsList(),
    proto.placement.center.inner.GetOffsetDataReplyOffset.toObject, includeInstance)
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
 * @return {!proto.placement.center.inner.GetOffsetDataReply}
 */
proto.placement.center.inner.GetOffsetDataReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.GetOffsetDataReply;
  return proto.placement.center.inner.GetOffsetDataReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.GetOffsetDataReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.GetOffsetDataReply}
 */
proto.placement.center.inner.GetOffsetDataReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = new proto.placement.center.inner.GetOffsetDataReplyOffset;
      reader.readMessage(value,proto.placement.center.inner.GetOffsetDataReplyOffset.deserializeBinaryFromReader);
      msg.addOffsets(value);
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
proto.placement.center.inner.GetOffsetDataReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.GetOffsetDataReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.GetOffsetDataReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetOffsetDataReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getOffsetsList();
  if (f.length > 0) {
    writer.writeRepeatedMessage(
      1,
      f,
      proto.placement.center.inner.GetOffsetDataReplyOffset.serializeBinaryToWriter
    );
  }
};


/**
 * repeated GetOffsetDataReplyOffset offsets = 1;
 * @return {!Array<!proto.placement.center.inner.GetOffsetDataReplyOffset>}
 */
proto.placement.center.inner.GetOffsetDataReply.prototype.getOffsetsList = function() {
  return /** @type{!Array<!proto.placement.center.inner.GetOffsetDataReplyOffset>} */ (
    jspb.Message.getRepeatedWrapperField(this, proto.placement.center.inner.GetOffsetDataReplyOffset, 1));
};


/**
 * @param {!Array<!proto.placement.center.inner.GetOffsetDataReplyOffset>} value
 * @return {!proto.placement.center.inner.GetOffsetDataReply} returns this
*/
proto.placement.center.inner.GetOffsetDataReply.prototype.setOffsetsList = function(value) {
  return jspb.Message.setRepeatedWrapperField(this, 1, value);
};


/**
 * @param {!proto.placement.center.inner.GetOffsetDataReplyOffset=} opt_value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.GetOffsetDataReplyOffset}
 */
proto.placement.center.inner.GetOffsetDataReply.prototype.addOffsets = function(opt_value, opt_index) {
  return jspb.Message.addToRepeatedWrapperField(this, 1, opt_value, proto.placement.center.inner.GetOffsetDataReplyOffset, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.GetOffsetDataReply} returns this
 */
proto.placement.center.inner.GetOffsetDataReply.prototype.clearOffsetsList = function() {
  return this.setOffsetsList([]);
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
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.GetOffsetDataReplyOffset.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.GetOffsetDataReplyOffset} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.toObject = function(includeInstance, msg) {
  var f, obj = {
namespace: jspb.Message.getFieldWithDefault(msg, 1, ""),
shardName: jspb.Message.getFieldWithDefault(msg, 2, ""),
offset: jspb.Message.getFieldWithDefault(msg, 3, 0)
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
 * @return {!proto.placement.center.inner.GetOffsetDataReplyOffset}
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.GetOffsetDataReplyOffset;
  return proto.placement.center.inner.GetOffsetDataReplyOffset.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.GetOffsetDataReplyOffset} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.GetOffsetDataReplyOffset}
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {string} */ (reader.readString());
      msg.setNamespace(value);
      break;
    case 2:
      var value = /** @type {string} */ (reader.readString());
      msg.setShardName(value);
      break;
    case 3:
      var value = /** @type {number} */ (reader.readUint64());
      msg.setOffset(value);
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
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.GetOffsetDataReplyOffset.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.GetOffsetDataReplyOffset} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getNamespace();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getShardName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getOffset();
  if (f !== 0) {
    writer.writeUint64(
      3,
      f
    );
  }
};


/**
 * optional string namespace = 1;
 * @return {string}
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.getNamespace = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.GetOffsetDataReplyOffset} returns this
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.setNamespace = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string shard_name = 2;
 * @return {string}
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.getShardName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.GetOffsetDataReplyOffset} returns this
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.setShardName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional uint64 offset = 3;
 * @return {number}
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.getOffset = function() {
  return /** @type {number} */ (jspb.Message.getFieldWithDefault(this, 3, 0));
};


/**
 * @param {number} value
 * @return {!proto.placement.center.inner.GetOffsetDataReplyOffset} returns this
 */
proto.placement.center.inner.GetOffsetDataReplyOffset.prototype.setOffset = function(value) {
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
proto.placement.center.inner.ListSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ListSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ListSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.inner.ListSchemaRequest}
 */
proto.placement.center.inner.ListSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ListSchemaRequest;
  return proto.placement.center.inner.ListSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ListSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ListSchemaRequest}
 */
proto.placement.center.inner.ListSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
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
proto.placement.center.inner.ListSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ListSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ListSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
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
proto.placement.center.inner.ListSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ListSchemaRequest} returns this
 */
proto.placement.center.inner.ListSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.ListSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ListSchemaRequest} returns this
 */
proto.placement.center.inner.ListSchemaRequest.prototype.setSchemaName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.ListSchemaReply.repeatedFields_ = [1];



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
proto.placement.center.inner.ListSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ListSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ListSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListSchemaReply.toObject = function(includeInstance, msg) {
  var f, obj = {
schemasList: msg.getSchemasList_asB64()
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
 * @return {!proto.placement.center.inner.ListSchemaReply}
 */
proto.placement.center.inner.ListSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ListSchemaReply;
  return proto.placement.center.inner.ListSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ListSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ListSchemaReply}
 */
proto.placement.center.inner.ListSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addSchemas(value);
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
proto.placement.center.inner.ListSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ListSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ListSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListSchemaReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getSchemasList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes schemas = 1;
 * @return {!Array<string>}
 */
proto.placement.center.inner.ListSchemaReply.prototype.getSchemasList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes schemas = 1;
 * This is a type-conversion wrapper around `getSchemasList()`
 * @return {!Array<string>}
 */
proto.placement.center.inner.ListSchemaReply.prototype.getSchemasList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getSchemasList()));
};


/**
 * repeated bytes schemas = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getSchemasList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.inner.ListSchemaReply.prototype.getSchemasList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getSchemasList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.inner.ListSchemaReply} returns this
 */
proto.placement.center.inner.ListSchemaReply.prototype.setSchemasList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.ListSchemaReply} returns this
 */
proto.placement.center.inner.ListSchemaReply.prototype.addSchemas = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.ListSchemaReply} returns this
 */
proto.placement.center.inner.ListSchemaReply.prototype.clearSchemasList = function() {
  return this.setSchemasList([]);
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
proto.placement.center.inner.CreateSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.CreateSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.CreateSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.CreateSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, ""),
schema: msg.getSchema_asB64()
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
 * @return {!proto.placement.center.inner.CreateSchemaRequest}
 */
proto.placement.center.inner.CreateSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.CreateSchemaRequest;
  return proto.placement.center.inner.CreateSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.CreateSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.CreateSchemaRequest}
 */
proto.placement.center.inner.CreateSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setSchema(value);
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
proto.placement.center.inner.CreateSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.CreateSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.CreateSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.CreateSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSchema_asU8();
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
proto.placement.center.inner.CreateSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.CreateSchemaRequest} returns this
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.CreateSchemaRequest} returns this
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.setSchemaName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes schema = 3;
 * @return {string}
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.getSchema = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes schema = 3;
 * This is a type-conversion wrapper around `getSchema()`
 * @return {string}
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.getSchema_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getSchema()));
};


/**
 * optional bytes schema = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getSchema()`
 * @return {!Uint8Array}
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.getSchema_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getSchema()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.inner.CreateSchemaRequest} returns this
 */
proto.placement.center.inner.CreateSchemaRequest.prototype.setSchema = function(value) {
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
proto.placement.center.inner.CreateSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.CreateSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.CreateSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.CreateSchemaReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.CreateSchemaReply}
 */
proto.placement.center.inner.CreateSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.CreateSchemaReply;
  return proto.placement.center.inner.CreateSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.CreateSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.CreateSchemaReply}
 */
proto.placement.center.inner.CreateSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.CreateSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.CreateSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.CreateSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.CreateSchemaReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.UpdateSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.UpdateSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.UpdateSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UpdateSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, ""),
schema: msg.getSchema_asB64()
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
 * @return {!proto.placement.center.inner.UpdateSchemaRequest}
 */
proto.placement.center.inner.UpdateSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.UpdateSchemaRequest;
  return proto.placement.center.inner.UpdateSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.UpdateSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.UpdateSchemaRequest}
 */
proto.placement.center.inner.UpdateSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
      break;
    case 3:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.setSchema(value);
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
proto.placement.center.inner.UpdateSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.UpdateSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.UpdateSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UpdateSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
  if (f.length > 0) {
    writer.writeString(
      2,
      f
    );
  }
  f = message.getSchema_asU8();
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
proto.placement.center.inner.UpdateSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.UpdateSchemaRequest} returns this
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.UpdateSchemaRequest} returns this
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.setSchemaName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional bytes schema = 3;
 * @return {string}
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.getSchema = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * optional bytes schema = 3;
 * This is a type-conversion wrapper around `getSchema()`
 * @return {string}
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.getSchema_asB64 = function() {
  return /** @type {string} */ (jspb.Message.bytesAsB64(
      this.getSchema()));
};


/**
 * optional bytes schema = 3;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getSchema()`
 * @return {!Uint8Array}
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.getSchema_asU8 = function() {
  return /** @type {!Uint8Array} */ (jspb.Message.bytesAsU8(
      this.getSchema()));
};


/**
 * @param {!(string|Uint8Array)} value
 * @return {!proto.placement.center.inner.UpdateSchemaRequest} returns this
 */
proto.placement.center.inner.UpdateSchemaRequest.prototype.setSchema = function(value) {
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
proto.placement.center.inner.UpdateSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.UpdateSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.UpdateSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UpdateSchemaReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.UpdateSchemaReply}
 */
proto.placement.center.inner.UpdateSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.UpdateSchemaReply;
  return proto.placement.center.inner.UpdateSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.UpdateSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.UpdateSchemaReply}
 */
proto.placement.center.inner.UpdateSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.UpdateSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.UpdateSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.UpdateSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UpdateSchemaReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.DeleteSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.DeleteSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.DeleteSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, "")
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
 * @return {!proto.placement.center.inner.DeleteSchemaRequest}
 */
proto.placement.center.inner.DeleteSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.DeleteSchemaRequest;
  return proto.placement.center.inner.DeleteSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.DeleteSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.DeleteSchemaRequest}
 */
proto.placement.center.inner.DeleteSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
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
proto.placement.center.inner.DeleteSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.DeleteSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.DeleteSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
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
proto.placement.center.inner.DeleteSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.DeleteSchemaRequest} returns this
 */
proto.placement.center.inner.DeleteSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.DeleteSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.DeleteSchemaRequest} returns this
 */
proto.placement.center.inner.DeleteSchemaRequest.prototype.setSchemaName = function(value) {
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
proto.placement.center.inner.DeleteSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.DeleteSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.DeleteSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteSchemaReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.DeleteSchemaReply}
 */
proto.placement.center.inner.DeleteSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.DeleteSchemaReply;
  return proto.placement.center.inner.DeleteSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.DeleteSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.DeleteSchemaReply}
 */
proto.placement.center.inner.DeleteSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.DeleteSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.DeleteSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.DeleteSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.DeleteSchemaReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.ListBindSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ListBindSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ListBindSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListBindSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, ""),
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
 * @return {!proto.placement.center.inner.ListBindSchemaRequest}
 */
proto.placement.center.inner.ListBindSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ListBindSchemaRequest;
  return proto.placement.center.inner.ListBindSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ListBindSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ListBindSchemaRequest}
 */
proto.placement.center.inner.ListBindSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
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
proto.placement.center.inner.ListBindSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ListBindSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ListBindSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListBindSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
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
proto.placement.center.inner.ListBindSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ListBindSchemaRequest} returns this
 */
proto.placement.center.inner.ListBindSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.ListBindSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ListBindSchemaRequest} returns this
 */
proto.placement.center.inner.ListBindSchemaRequest.prototype.setSchemaName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string resource_name = 3;
 * @return {string}
 */
proto.placement.center.inner.ListBindSchemaRequest.prototype.getResourceName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.ListBindSchemaRequest} returns this
 */
proto.placement.center.inner.ListBindSchemaRequest.prototype.setResourceName = function(value) {
  return jspb.Message.setProto3StringField(this, 3, value);
};



/**
 * List of repeated fields within this message type.
 * @private {!Array<number>}
 * @const
 */
proto.placement.center.inner.ListBindSchemaReply.repeatedFields_ = [1];



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
proto.placement.center.inner.ListBindSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.ListBindSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.ListBindSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListBindSchemaReply.toObject = function(includeInstance, msg) {
  var f, obj = {
schemaBindsList: msg.getSchemaBindsList_asB64()
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
 * @return {!proto.placement.center.inner.ListBindSchemaReply}
 */
proto.placement.center.inner.ListBindSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.ListBindSchemaReply;
  return proto.placement.center.inner.ListBindSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.ListBindSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.ListBindSchemaReply}
 */
proto.placement.center.inner.ListBindSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
  while (reader.nextField()) {
    if (reader.isEndGroup()) {
      break;
    }
    var field = reader.getFieldNumber();
    switch (field) {
    case 1:
      var value = /** @type {!Uint8Array} */ (reader.readBytes());
      msg.addSchemaBinds(value);
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
proto.placement.center.inner.ListBindSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.ListBindSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.ListBindSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.ListBindSchemaReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getSchemaBindsList_asU8();
  if (f.length > 0) {
    writer.writeRepeatedBytes(
      1,
      f
    );
  }
};


/**
 * repeated bytes schema_binds = 1;
 * @return {!Array<string>}
 */
proto.placement.center.inner.ListBindSchemaReply.prototype.getSchemaBindsList = function() {
  return /** @type {!Array<string>} */ (jspb.Message.getRepeatedField(this, 1));
};


/**
 * repeated bytes schema_binds = 1;
 * This is a type-conversion wrapper around `getSchemaBindsList()`
 * @return {!Array<string>}
 */
proto.placement.center.inner.ListBindSchemaReply.prototype.getSchemaBindsList_asB64 = function() {
  return /** @type {!Array<string>} */ (jspb.Message.bytesListAsB64(
      this.getSchemaBindsList()));
};


/**
 * repeated bytes schema_binds = 1;
 * Note that Uint8Array is not supported on all browsers.
 * @see http://caniuse.com/Uint8Array
 * This is a type-conversion wrapper around `getSchemaBindsList()`
 * @return {!Array<!Uint8Array>}
 */
proto.placement.center.inner.ListBindSchemaReply.prototype.getSchemaBindsList_asU8 = function() {
  return /** @type {!Array<!Uint8Array>} */ (jspb.Message.bytesListAsU8(
      this.getSchemaBindsList()));
};


/**
 * @param {!(Array<!Uint8Array>|Array<string>)} value
 * @return {!proto.placement.center.inner.ListBindSchemaReply} returns this
 */
proto.placement.center.inner.ListBindSchemaReply.prototype.setSchemaBindsList = function(value) {
  return jspb.Message.setField(this, 1, value || []);
};


/**
 * @param {!(string|Uint8Array)} value
 * @param {number=} opt_index
 * @return {!proto.placement.center.inner.ListBindSchemaReply} returns this
 */
proto.placement.center.inner.ListBindSchemaReply.prototype.addSchemaBinds = function(value, opt_index) {
  return jspb.Message.addToRepeatedField(this, 1, value, opt_index);
};


/**
 * Clears the list making it empty but non-null.
 * @return {!proto.placement.center.inner.ListBindSchemaReply} returns this
 */
proto.placement.center.inner.ListBindSchemaReply.prototype.clearSchemaBindsList = function() {
  return this.setSchemaBindsList([]);
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
proto.placement.center.inner.BindSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.BindSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.BindSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.BindSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, ""),
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
 * @return {!proto.placement.center.inner.BindSchemaRequest}
 */
proto.placement.center.inner.BindSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.BindSchemaRequest;
  return proto.placement.center.inner.BindSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.BindSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.BindSchemaRequest}
 */
proto.placement.center.inner.BindSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
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
proto.placement.center.inner.BindSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.BindSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.BindSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.BindSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
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
proto.placement.center.inner.BindSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.BindSchemaRequest} returns this
 */
proto.placement.center.inner.BindSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.BindSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.BindSchemaRequest} returns this
 */
proto.placement.center.inner.BindSchemaRequest.prototype.setSchemaName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string resource_name = 3;
 * @return {string}
 */
proto.placement.center.inner.BindSchemaRequest.prototype.getResourceName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.BindSchemaRequest} returns this
 */
proto.placement.center.inner.BindSchemaRequest.prototype.setResourceName = function(value) {
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
proto.placement.center.inner.BindSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.BindSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.BindSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.BindSchemaReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.BindSchemaReply}
 */
proto.placement.center.inner.BindSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.BindSchemaReply;
  return proto.placement.center.inner.BindSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.BindSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.BindSchemaReply}
 */
proto.placement.center.inner.BindSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.BindSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.BindSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.BindSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.BindSchemaReply.serializeBinaryToWriter = function(message, writer) {
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
proto.placement.center.inner.UnBindSchemaRequest.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.UnBindSchemaRequest.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.UnBindSchemaRequest} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnBindSchemaRequest.toObject = function(includeInstance, msg) {
  var f, obj = {
clusterName: jspb.Message.getFieldWithDefault(msg, 1, ""),
schemaName: jspb.Message.getFieldWithDefault(msg, 2, ""),
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
 * @return {!proto.placement.center.inner.UnBindSchemaRequest}
 */
proto.placement.center.inner.UnBindSchemaRequest.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.UnBindSchemaRequest;
  return proto.placement.center.inner.UnBindSchemaRequest.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.UnBindSchemaRequest} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.UnBindSchemaRequest}
 */
proto.placement.center.inner.UnBindSchemaRequest.deserializeBinaryFromReader = function(msg, reader) {
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
      msg.setSchemaName(value);
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
proto.placement.center.inner.UnBindSchemaRequest.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.UnBindSchemaRequest.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.UnBindSchemaRequest} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnBindSchemaRequest.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
  f = message.getClusterName();
  if (f.length > 0) {
    writer.writeString(
      1,
      f
    );
  }
  f = message.getSchemaName();
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
proto.placement.center.inner.UnBindSchemaRequest.prototype.getClusterName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 1, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.UnBindSchemaRequest} returns this
 */
proto.placement.center.inner.UnBindSchemaRequest.prototype.setClusterName = function(value) {
  return jspb.Message.setProto3StringField(this, 1, value);
};


/**
 * optional string schema_name = 2;
 * @return {string}
 */
proto.placement.center.inner.UnBindSchemaRequest.prototype.getSchemaName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 2, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.UnBindSchemaRequest} returns this
 */
proto.placement.center.inner.UnBindSchemaRequest.prototype.setSchemaName = function(value) {
  return jspb.Message.setProto3StringField(this, 2, value);
};


/**
 * optional string resource_name = 3;
 * @return {string}
 */
proto.placement.center.inner.UnBindSchemaRequest.prototype.getResourceName = function() {
  return /** @type {string} */ (jspb.Message.getFieldWithDefault(this, 3, ""));
};


/**
 * @param {string} value
 * @return {!proto.placement.center.inner.UnBindSchemaRequest} returns this
 */
proto.placement.center.inner.UnBindSchemaRequest.prototype.setResourceName = function(value) {
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
proto.placement.center.inner.UnBindSchemaReply.prototype.toObject = function(opt_includeInstance) {
  return proto.placement.center.inner.UnBindSchemaReply.toObject(opt_includeInstance, this);
};


/**
 * Static version of the {@see toObject} method.
 * @param {boolean|undefined} includeInstance Deprecated. Whether to include
 *     the JSPB instance for transitional soy proto support:
 *     http://goto/soy-param-migration
 * @param {!proto.placement.center.inner.UnBindSchemaReply} msg The msg instance to transform.
 * @return {!Object}
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnBindSchemaReply.toObject = function(includeInstance, msg) {
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
 * @return {!proto.placement.center.inner.UnBindSchemaReply}
 */
proto.placement.center.inner.UnBindSchemaReply.deserializeBinary = function(bytes) {
  var reader = new jspb.BinaryReader(bytes);
  var msg = new proto.placement.center.inner.UnBindSchemaReply;
  return proto.placement.center.inner.UnBindSchemaReply.deserializeBinaryFromReader(msg, reader);
};


/**
 * Deserializes binary data (in protobuf wire format) from the
 * given reader into the given message object.
 * @param {!proto.placement.center.inner.UnBindSchemaReply} msg The message object to deserialize into.
 * @param {!jspb.BinaryReader} reader The BinaryReader to use.
 * @return {!proto.placement.center.inner.UnBindSchemaReply}
 */
proto.placement.center.inner.UnBindSchemaReply.deserializeBinaryFromReader = function(msg, reader) {
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
proto.placement.center.inner.UnBindSchemaReply.prototype.serializeBinary = function() {
  var writer = new jspb.BinaryWriter();
  proto.placement.center.inner.UnBindSchemaReply.serializeBinaryToWriter(this, writer);
  return writer.getResultBuffer();
};


/**
 * Serializes the given message to binary data (in protobuf wire
 * format), writing to the given BinaryWriter.
 * @param {!proto.placement.center.inner.UnBindSchemaReply} message
 * @param {!jspb.BinaryWriter} writer
 * @suppress {unusedLocalVariables} f is only used for nested messages
 */
proto.placement.center.inner.UnBindSchemaReply.serializeBinaryToWriter = function(message, writer) {
  var f = undefined;
};


/**
 * @enum {number}
 */
proto.placement.center.inner.ClusterType = {
  PLACEMENTCENTER: 0,
  JOURNALSERVER: 1,
  MQTTBROKERSERVER: 2,
  AMQPBROKERSERVER: 3
};

goog.object.extend(exports, proto.placement.center.inner);
