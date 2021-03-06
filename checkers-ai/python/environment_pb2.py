# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: environment.proto
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from google.protobuf import reflection as _reflection
from google.protobuf import symbol_database as _symbol_database
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor.FileDescriptor(
  name='environment.proto',
  package='environment',
  syntax='proto3',
  serialized_options=None,
  create_key=_descriptor._internal_create_key,
  serialized_pb=b'\n\x11\x65nvironment.proto\x12\x0b\x65nvironment\"\x1d\n\x0cResetRequest\x12\r\n\x05state\x18\x01 \x01(\t\"\x1d\n\x0bStepRequest\x12\x0e\n\x06\x61\x63tion\x18\x01 \x01(\t\"\x15\n\x13\x43urrentStateRequest\"\x19\n\tJsonReply\x12\x0c\n\x04json\x18\x01 \x01(\t2\xcd\x01\n\x0b\x45nvironment\x12:\n\x05Reset\x12\x19.environment.ResetRequest\x1a\x16.environment.JsonReply\x12\x38\n\x04Step\x12\x18.environment.StepRequest\x1a\x16.environment.JsonReply\x12H\n\x0c\x43urrentState\x12 .environment.CurrentStateRequest\x1a\x16.environment.JsonReplyb\x06proto3'
)




_RESETREQUEST = _descriptor.Descriptor(
  name='ResetRequest',
  full_name='environment.ResetRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='state', full_name='environment.ResetRequest.state', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=34,
  serialized_end=63,
)


_STEPREQUEST = _descriptor.Descriptor(
  name='StepRequest',
  full_name='environment.StepRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='action', full_name='environment.StepRequest.action', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=65,
  serialized_end=94,
)


_CURRENTSTATEREQUEST = _descriptor.Descriptor(
  name='CurrentStateRequest',
  full_name='environment.CurrentStateRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=96,
  serialized_end=117,
)


_JSONREPLY = _descriptor.Descriptor(
  name='JsonReply',
  full_name='environment.JsonReply',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='json', full_name='environment.JsonReply.json', index=0,
      number=1, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=119,
  serialized_end=144,
)

DESCRIPTOR.message_types_by_name['ResetRequest'] = _RESETREQUEST
DESCRIPTOR.message_types_by_name['StepRequest'] = _STEPREQUEST
DESCRIPTOR.message_types_by_name['CurrentStateRequest'] = _CURRENTSTATEREQUEST
DESCRIPTOR.message_types_by_name['JsonReply'] = _JSONREPLY
_sym_db.RegisterFileDescriptor(DESCRIPTOR)

ResetRequest = _reflection.GeneratedProtocolMessageType('ResetRequest', (_message.Message,), {
  'DESCRIPTOR' : _RESETREQUEST,
  '__module__' : 'environment_pb2'
  # @@protoc_insertion_point(class_scope:environment.ResetRequest)
  })
_sym_db.RegisterMessage(ResetRequest)

StepRequest = _reflection.GeneratedProtocolMessageType('StepRequest', (_message.Message,), {
  'DESCRIPTOR' : _STEPREQUEST,
  '__module__' : 'environment_pb2'
  # @@protoc_insertion_point(class_scope:environment.StepRequest)
  })
_sym_db.RegisterMessage(StepRequest)

CurrentStateRequest = _reflection.GeneratedProtocolMessageType('CurrentStateRequest', (_message.Message,), {
  'DESCRIPTOR' : _CURRENTSTATEREQUEST,
  '__module__' : 'environment_pb2'
  # @@protoc_insertion_point(class_scope:environment.CurrentStateRequest)
  })
_sym_db.RegisterMessage(CurrentStateRequest)

JsonReply = _reflection.GeneratedProtocolMessageType('JsonReply', (_message.Message,), {
  'DESCRIPTOR' : _JSONREPLY,
  '__module__' : 'environment_pb2'
  # @@protoc_insertion_point(class_scope:environment.JsonReply)
  })
_sym_db.RegisterMessage(JsonReply)



_ENVIRONMENT = _descriptor.ServiceDescriptor(
  name='Environment',
  full_name='environment.Environment',
  file=DESCRIPTOR,
  index=0,
  serialized_options=None,
  create_key=_descriptor._internal_create_key,
  serialized_start=147,
  serialized_end=352,
  methods=[
  _descriptor.MethodDescriptor(
    name='Reset',
    full_name='environment.Environment.Reset',
    index=0,
    containing_service=None,
    input_type=_RESETREQUEST,
    output_type=_JSONREPLY,
    serialized_options=None,
    create_key=_descriptor._internal_create_key,
  ),
  _descriptor.MethodDescriptor(
    name='Step',
    full_name='environment.Environment.Step',
    index=1,
    containing_service=None,
    input_type=_STEPREQUEST,
    output_type=_JSONREPLY,
    serialized_options=None,
    create_key=_descriptor._internal_create_key,
  ),
  _descriptor.MethodDescriptor(
    name='CurrentState',
    full_name='environment.Environment.CurrentState',
    index=2,
    containing_service=None,
    input_type=_CURRENTSTATEREQUEST,
    output_type=_JSONREPLY,
    serialized_options=None,
    create_key=_descriptor._internal_create_key,
  ),
])
_sym_db.RegisterServiceDescriptor(_ENVIRONMENT)

DESCRIPTOR.services_by_name['Environment'] = _ENVIRONMENT

# @@protoc_insertion_point(module_scope)
