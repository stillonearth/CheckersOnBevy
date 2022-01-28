# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc

import environment_pb2 as environment__pb2


class EnvironmentStub(object):
    """Implementation of OpenAI environment 
    """

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.Reset = channel.unary_unary(
                '/environment.Environment/Reset',
                request_serializer=environment__pb2.ResetRequest.SerializeToString,
                response_deserializer=environment__pb2.JsonReply.FromString,
                )
        self.Step = channel.unary_unary(
                '/environment.Environment/Step',
                request_serializer=environment__pb2.StepRequest.SerializeToString,
                response_deserializer=environment__pb2.JsonReply.FromString,
                )
        self.CurrentState = channel.unary_unary(
                '/environment.Environment/CurrentState',
                request_serializer=environment__pb2.CurrentStateRequest.SerializeToString,
                response_deserializer=environment__pb2.JsonReply.FromString,
                )


class EnvironmentServicer(object):
    """Implementation of OpenAI environment 
    """

    def Reset(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Step(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def CurrentState(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_EnvironmentServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'Reset': grpc.unary_unary_rpc_method_handler(
                    servicer.Reset,
                    request_deserializer=environment__pb2.ResetRequest.FromString,
                    response_serializer=environment__pb2.JsonReply.SerializeToString,
            ),
            'Step': grpc.unary_unary_rpc_method_handler(
                    servicer.Step,
                    request_deserializer=environment__pb2.StepRequest.FromString,
                    response_serializer=environment__pb2.JsonReply.SerializeToString,
            ),
            'CurrentState': grpc.unary_unary_rpc_method_handler(
                    servicer.CurrentState,
                    request_deserializer=environment__pb2.CurrentStateRequest.FromString,
                    response_serializer=environment__pb2.JsonReply.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'environment.Environment', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class Environment(object):
    """Implementation of OpenAI environment 
    """

    @staticmethod
    def Reset(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/environment.Environment/Reset',
            environment__pb2.ResetRequest.SerializeToString,
            environment__pb2.JsonReply.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def Step(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/environment.Environment/Step',
            environment__pb2.StepRequest.SerializeToString,
            environment__pb2.JsonReply.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def CurrentState(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/environment.Environment/CurrentState',
            environment__pb2.CurrentStateRequest.SerializeToString,
            environment__pb2.JsonReply.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)
