import grpc
import environment_pb2
import environment_pb2_grpc
import json


class Env:

    def __init__(self):
        self.channel = grpc.insecure_channel('localhost:50051')
        self.stub = environment_pb2_grpc.EnvironmentStub(self.channel)

    def reset(self, state=None):
        state_json = "" if state is None else json.dumps(state)
        response = self.stub.Reset(environment_pb2.ResetRequest(state=state_json))
        return json.loads(response.json)

    def step(self, action):
        response = self.stub.Step(environment_pb2.StepRequest(action=json.dumps(action)))
        state = json.loads(response.json)

        return state['obs'], state['reward'], state['is_done'], {}

    def current_state(self):
        response = self.stub.CurrentState(environment_pb2.CurrentStateRequest())
        return json.loads(response.json)
