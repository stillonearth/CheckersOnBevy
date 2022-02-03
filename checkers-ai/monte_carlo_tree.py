import uuid
import numpy as np
import time
import torch.optim as optim
import torch


MCTS_N_SIMULATIONS = 30
MCTS_ROLLOUT_DEPTH = 100
MCTS_C = np.sqrt(2.0)
LR = 3e-4


class RandomPlayTree:
    """
    Random Play Tree (RPT) plays a game randomly. Base class for Monte Carlo Tree (MCT).

    - Sequence of actions is organized to a tree structure
    - Any node in tree can be expanded
    - RPT chooses a random action at each turn and plays the game unless game is ended.
    """
    
    def __init__(self, env, board_size):
        self.env = env
        self.root_node = Node(parent=None, state=self.env.reset(), action=None, reward=0, prob=1.0)
        self.board_size = board_size
        
    def pick_action(self, node):
        """Pick next action
        
        Parameters:
            node (Node): a game tree node

        Returns:
            (actions, probs): possible actions and their probabilities
        """
        possible_actions = node.possible_actions()

        if len(possible_actions) == 0:
            return None, 0.0
        
        index = np.random.choice(len(possible_actions))
        return tuple(possible_actions[index]), np.ones(len(possible_actions)) / len(possible_actions)
    
    def find_node(self, uuid):
        """Find tree node by UUID
        
        Parameters:
            uuid (string): node uuid

        Returns:
            Node: game tree node
        """
        def _find_node(parent_node):
            if parent_node.uuid == uuid:
                return parent_node
            for node in parent_node.children.values():
                return _find_node(node)
        return _find_node(self.root_node)

    def act(self, node, action, prob):
        """Make a action
        
        Parameters:
            node (Node): game tree node
            action (tuple): action
            prob (float32): action probability
        
        Returns:
            Node: new node
        """
        # Reset environment to node's state. Useful in MCTS unroll situation.
        
        serialized_action = {
            'piece': node.get_piece_by_coord(action[0], action[1]),
            'square': {'x': action[2], 'y': action[3]}
        }

        self.env.reset(node.state)
        state, reward, done, _info = self.env.step(serialized_action)

        existing_nodes = list(filter(lambda n: n.action == action, node.children))
        
        # Whether this node has been already visited before
        if len(existing_nodes):
            new_node = existing_nodes[0]
            new_node.prob = prob
        else:
            new_node = Node(parent=node, state=state, action=action, reward=reward, prob=prob, is_terminal=done)
            node.add_child(new_node)

        return new_node

    def simulate(self, node):
        """Simulation is MCTS is a sequence of actions that starts in current node and ends in terminal node. 
        During simulation actions are chosen w.r.t. rollout policy function which in usually uniform random.
        
        Parameters:
            node (Node): start node
        
        Returns:
            Node: terminal node
        """
        current_node = node
        
        while True:
            action, prob = self.pick_action(current_node)   

            if action is None: # no possible actions
                break

            current_node = self.act(current_node, action, prob)
            if current_node.is_terminal:
                break
        
        return current_node


class MonteCarloPlayTree(RandomPlayTree):

    def evaluate_node(self, node):
        if node.current_player() == 1:
            return node.reward
        else:
            return -node.reward
    
    def pick_action(self, node):
        """Pick next action"""
        
        leaf = self.traverse(node)  
        for _ in range(MCTS_N_SIMULATIONS):
            terminal_node = self.rollout(leaf, 0)
            value = self.evaluate_node(terminal_node)
            self.backpropagate(leaf, value)

        best_child = node.best_uct(self.uct)

        return best_child.action, best_child.prob

    def uct(self, node, c=MCTS_C):
        """
        UCT is a core of MCTS. It allows us to choose next node among visited nodes.
        
        Q_v/N_v                               - exploitation component (favors nodes that were winning)
        torch.sqrt(torch.log(N_v_parent)/N_v) - exploration component (favors node that weren't visited)
        c                                     - tradeoff
        
        In competitive games Q is always computed relative to player who moves.

        Parameters
        ----------
        player: int
            1 for blacks
            -1 for whites
        c: float
            Constant for exploration/exploitation tradeoff
        """

        if node.current_player() == 1:
            Q_v = node.q_black
        else:
            Q_v = node.q_white
        N_v = node.number_of_visits + 1
        N_v_parent = node.parent.number_of_visits + 1
        
        return np.sum(Q_v/N_v + c*np.sqrt(np.log(N_v_parent)/N_v))

    def traverse(self, node):
        """
        Traverse a node. 
        Pick a path prioritizing highest UTC for fully explored nodes and random uniform otherwise.
        """

        if node.is_terminal:
            return node

        if node.is_fully_expanded():
            return self.traverse(node.best_uct(self.uct))

        action, prob = self.traverse_policy(node)
        return self.act(node, action, prob)  

    def traverse_policy(self, node):
        """Traverse policy in uniform random"""
        unexplored_actions = list(node.possible_unexplored_actions())

        index = np.random.choice(len(unexplored_actions))
        action = unexplored_actions[index]
        return action, 1. / len(node.possible_actions())

    def rollout(self, node, depth):
        """Rollout a node according to a rollout policy."""
        if depth > MCTS_ROLLOUT_DEPTH:
            return node

        if node.is_terminal:
            return node

        action, prob = self.rollout_policy(node)
        new_node = self.act(node, action, prob)  
        return self.rollout(new_node, depth+1)

    def rollout_policy(self, node):
        """
        A Policy used to pick next best action
        In Non-Neural Monte Carlo Tree it is random uniform.
        """
        possible_actions = list(node.possible_actions())

        index = np.random.choice(len(possible_actions))
        action = possible_actions[index]
        return action, 1. / len(possible_actions)

    def backpropagate(self, node, result):
        """Backpropagate node's statistics all the way up to root node"""
        node.update_stats(result)
        if node.is_root():
            return
        self.backpropagate(node.parent, result)


"""MCTS with neural augmentations"""
class GuidedMonteCarloPlayTree(MonteCarloPlayTree):

    def __init__(self, env, tree_size, actor_critic_network, device):
        super(GuidedMonteCarloPlayTree, self).__init__(env, tree_size)
        self.actor_critic_network = actor_critic_network
        self.optimizer = optim.Adam(self.actor_critic_network.parameters(), lr=LR)
        self.device = device

    def correct_probs_with_possible_actions(self, node, probs_tensor):
        possible_actions = node.possible_actions(raw=True)
        possible_actions_tensor = torch.from_numpy(possible_actions).to(self.device).view(self.board_size**4) 
        probs_tensor *= possible_actions_tensor
        probs_tensor = probs_tensor.view(
            1, self.board_size, self.board_size, self.board_size, self.board_size
        ).squeeze()

        return probs_tensor / probs_tensor.sum()
    
    def rollout_policy(self, node):
        
        state = node.prepared_game_state()
        state_tensor = torch.from_numpy(state).float().to(self.device).unsqueeze(0)
        probs_tensor, _ = self.actor_critic_network(state_tensor)
        probs_tensor = self.correct_probs_with_possible_actions(node, probs_tensor) \
            .cpu().detach().numpy()
        actions = np.argwhere(probs_tensor>0).tolist()
        probs = np.array([probs_tensor[m[0], m[1], m[2], m[3]] for m in actions])
        probs /= np.sum(probs)
        index = np.random.choice(np.arange(len(probs)), p=probs)

        return actions[index], probs[index]

    def uct(self, node, c=MCTS_C):
        N_v = node.number_of_visits + 1
        N_v_parent = node.parent.number_of_visits + 1

        # TODO: test
        V_current = self.estimate_node_value(node)
        for child in node.children:
            V_current -= self.estimate_node_value(child)
        
        result = V_current/N_v + c * np.sqrt(np.log(N_v_parent)/N_v) * node.prob

        return result

    """Estimate node value with neural network"""
    def estimate_node_value(self, node):
        state = node.prepared_game_state(node.current_player())
        state_tensor = torch.from_numpy(state).float().to(self.device).unsqueeze(0)
        _, v = self.actor_critic_network(state_tensor)
        return v.detach().cpu().numpy().sum()

    """
    Train guided MCTS
    AlphaZero algorithm:
    1. Initialize actor-critic
    2. Simulate a game
    3. Compute loss
    4. Repeat
    """
    def train(self, n_iterations):
        for i in range(n_iterations):
            print("Iteration #", i,)
            terminal_node = self.simulate(self.root_node)
            last_player = terminal_node.current_player()
            winning_player = np.sign(self.evaluate_node(terminal_node))
            if last_player == winning_player:
                score = 1
            else:
                score = -1

            trajectory = terminal_node.unroll()
            print("number of actions: ", len(trajectory), )
            
            states = np.array([node.prepared_game_state(terminal_node.current_player()) for node in trajectory])
            states_tensor = torch.from_numpy(states).float().to(self.device)
            probs, values = self.actor_critic_network(states_tensor)
            
            # Loss function. Core of alpha-zero
            # loss = ((values - score).pow(2) - probs * torch.log(probs+1e-5)).sum()

            loss_term_1 = (values - score).pow(2) #- (probs * torch.log(probs)).sum()).sum()
            loss_term_2 = 0
            for i, node in enumerate(trajectory):
                prob = probs[i]
                if node.action is not None:
                    prob = prob.view(self.board_size,self.board_size,self.board_size,self.board_size)[node.action[0], node.action[1], node.action[2], node.action[3]]
                    loss_term_2 += node.prob * torch.log(prob+1e-5)

            loss = (loss_term_1 - loss_term_2).sum()

            self.optimizer.zero_grad()
            loss.backward()
            self.optimizer.step()
            print("Loss:", loss)


def state_to_board(state):
    board = np.zeros((5, 8, 8))
    for piece in state['pieces']:
        if piece['color'] == "Black":
            board[0, piece['x'], piece['y']] = 1
        else: 
            board[1, piece['x'], piece['y']] = 1
        board[2] = 1 if state['turn']['color'] == "Black" else 0

    return board


class Node:
    """Game Tree Node"""
    def __init__(self, parent, state, action, reward, is_terminal=False, prob=0.0):
        self.uuid = uuid.uuid1()
        self.parent = parent
        self.children = []
        
        self.state = state
        self.action = action
        self.reward = reward
        self.is_terminal = is_terminal

        # MCT properties
        self.number_of_visits = 0
        self.q_black = 0
        self.q_white = 0
        
        # Traversal properties
        self.prob = prob
        
    def add_child(self, node):
        self.children.append(node)

    def prepared_game_state(self, player=None):
        """
        Prepare game state X from perspective of current player
        [
            [ 1 -1 -1 ]
            [ 1  0  0 ]
            [ 0  0 -1 ]
        ]

        
        Where  
            1:  current player
            -1: opposing player
            
        """

        if player == None:
            player = self.current_player()

        # take advantage of game symmetry        
        state = self.blacks() - self.whites() if player == 1 else self.whites() - self.blacks()

        return state

    def get_piece_by_id(self, piece_id):
        return next(
            filter(lambda p: p['id'] == piece_id, 
                self.state['pieces']
            ))

    def get_piece_by_coord(self, x, y):
        return next(
            filter(lambda p: p['x'] == x and p['y'] == y, 
                self.state['pieces']
            ))

    def whites(self):
        """White figures on board"""
        state = state_to_board(self.state)
        return state[1]
    
    def blacks(self):
        """Black figures on board"""
        state = state_to_board(self.state)
        return state[0]

    def current_player(self):
        """Return 1 if current player plays black, and -1 for whites"""
        if self.state['turn']['color'] == 'Black':
            return 1
        return 0
    
    def possible_actions(self, player=None, raw=False):
        """List of possible next actions"""
        if player == None:
            player = self.current_player()

        coords = []
        for piece_id in range(0, 18):
            piece = self.get_piece_by_id(piece_id)
            
            for action in self.state['moveset'][piece_id]:
                coords.append((piece['x'], piece['y'], action[0], action[1]))

        actions = np.zeros((8, 8, 8, 8))
        for c in coords:
            actions[c] = 1
        
        mask = self.possible_actions_mask(player)
        actions = actions * mask

        if raw:
            return actions

        return np.argwhere(actions).tolist()

    def possible_actions_mask(self, player=None):
        """Return list of possible next actions as i8 mask"""
        if player == None:
            player = self.current_player()

        coords = []
        for piece_id in range(0, 18):
            piece = self.get_piece_by_id(piece_id)

            if (piece['color'] == "Black" and player == 1) or (piece['color'] == "White" and player == 0):
                coords.append((piece['x'], piece['y']))

        actions = np.zeros((8, 8, 8, 8))
        for c in coords:
            actions[c] = 1

        return actions

    def depth(self):
        """How far node from root"""
        return len(self.unroll())

    """Whether node all of node's children were expanded"""
    def is_fully_expanded(self):
        return len(self.possible_unexplored_actions()) == 0 or self.is_terminal

    def best_uct(self, uct_func):
        """Pick child node with highest UCT"""
        return sorted(self.children, key=lambda node: uct_func(node), reverse=True)[0]

    def possible_unexplored_actions(self):
        """Unvisited possibled nodes"""
        possible_actions_set = set([tuple(m) for m in self.possible_actions()])
        explored_actions_set = set([tuple(m.action) for m in self.children])
        return possible_actions_set - explored_actions_set

    def best_child(self):
        """Return best child node according to node's number of visits"""
        return sorted(self.children, key=lambda node: node.number_of_visits, reverse=True)[0]

    def is_root(self):
        return self.parent == None

    def update_stats(self, result):
        """Update node statistics"""
        self.number_of_visits += 1
        if result > 0:
            self.q_black += 1
        if result < 0:
            self.q_white += 1

    def unroll(self):
        """Return list of nodes to root"""
        nodes = [self]
        node = self
        while node.parent is not None:
            node = node.parent
            nodes.append(node)

        return nodes