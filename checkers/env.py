import grpc
import environment_pb2
import environment_pb2_grpc
import json
import numpy as np

def state_to_board(state):
    board = np.zeros((5+18, 8, 8))
    for piece in state['pieces']:
        if piece['color'] == "Black":
            board[0, 7-piece['x'], piece['y']] = 1
        else: 
            board[1, 7-piece['x'], piece['y']] = 1
        board[2, 7-piece['x'], piece['y']] = piece['id']
        board[3] = 1 if state['turn']['color'] == "Black" else 0

    for i in range(0, 18):
        for p in state['moveset'][i]:
            board[5+i, 7-p[0], p[1]] = 1

    return board

class Env:

    def __init__(self):
        self.channel = grpc.insecure_channel('localhost:50051')
        self.stub = environment_pb2_grpc.EnvironmentStub(self.channel)

    def reset(self, state=None):
        state_json = ""
        if state is not None:
            state_json = json.dumps(state)
        response = self.stub.Reset(environment_pb2.ResetRequest(state=state_json))
        state = json.loads(response.json)
        return state

    def step(self, action):
        response = self.stub.Action(environment_pb2.StepRequest(action=json.dumps(action)))
        step = json.loads(response.json)

        board = state_to_board(step.state)
        if np.abs(step.reward) == 10:
            board[4] = 1

        return step

    def current_state(self):
        response = self.stub.CurrentState(environment_pb2.CurrentStateRequest())
        state = json.loads(response.json)
        return state
