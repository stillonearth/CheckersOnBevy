import checkers
import threading
import numpy as np


class Env():
    def __init__(self):
        self.checkers_env = checkers.new()

    def reset(self):
        return state_to_np(self.checkers_env.reset())

    def step(self, action):

        step = self.checkers_env.step(action)

        return (state_to_np(step.obs.state), step.reward, step.is_done, {})


def state_to_np(state):
    board = np.zeros((8, 8))

    for p in state.pieces:
        if p.get_color() == "white":
            board[(p.x, p.y)] = 1
        else:
            board[(p.x, p.y)] = -1
    
    return np.flip(board.T, 1)


def main():

    env = Env()
 
    print(env.reset())


main()