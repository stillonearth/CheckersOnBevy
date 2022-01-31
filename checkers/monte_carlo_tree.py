import uuid
import numpy as np
import time
import torch.optim as optim
import torch


MCTS_N_SIMULATIONS = 3
MCTS_ROLLOUT_DEPTH = 10
MCTS_C = np.sqrt(2.0)
LR = 3e-4


class RandomPlayTree:
    """
    Random Play Tree (RPT) plays a game randomly. Base class for Monte Carlo Tree (MCT).

    - Sequence of moves is organized to a tree structure
    - Any node in tree can be expanded
    - RPT chooses a random move at each turn and plays the game unless game is ended.
    """
    
    def __init__(self, env, board_size):
        self.env = env
        self.root_node = Node(None, self.env.reset(), None, 1.0)
        self.board_size = board_size
        
    def pick_move(self, node):
        """Pick next move
        
        Parameters:
            node (Node): a game tree node

        Returns:
            (moves, probs): possible moves and their probabilities
        """
        possible_moves = node.possible_moves()

        if len(possible_moves) == 0:
            return None, 0.0
        
        index = np.random.choice(len(possible_moves))
        return tuple(possible_moves[index]), np.ones(len(possible_moves)) / len(possible_moves)
    
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

    def move(self, node, move, prob):
        """Make a move
        
        Parameters:
            node (Node): game tree node
            move (tuple): move
            prob (float32): move probability
        
        Returns:
            Node: new node
        """
        # Reset environment to node's state. Useful in MCTS unroll situation.
        
        action = {
            'piece': node.get_piece_by_coord(move[0], move[1]),
            'square': {
                'x': move[2],
                'y': move[3],
            }
        }

        self.env.reset(node.original_state)
        state, reward, done, _info = self.env.step(action)

        existing_nodes = list(filter(lambda n: n.move == move, node.children))
        
        # Whether this node has been already visited before
        if len(existing_nodes):
            new_node = existing_nodes[0]
            new_node.prob = prob
        else:
            new_node = Node(node, state, move, prob)
            new_node.is_terminal = done
            new_node.value = reward
            node.add_child(new_node)

        return new_node

    def simulate(self, node):
        """Simulation is MCTS is a sequence of moves that starts in current node and ends in terminal node. 
        During simulation moves are chosen w.r.t. rollout policy function which in usually uniform random.
        
        Parameters:
            node (Node): start node
        
        Returns:
            Node: terminal node
        """
        current_node = node
        
        while True:
            move, prob = self.pick_move(current_node)   

            if move is None: # no possible moves
                break
       
            current_node = self.move(current_node, move, prob)
            if current_node.is_terminal:
                break
        
        return current_node


class MonteCarloPlayTree(RandomPlayTree):

    def evaluate_node(self, node):
        return node.value
    
    def pick_move(self, node):
        """Pick next move"""
        
        leaf = self.traverse(node)  
        for _ in range(MCTS_N_SIMULATIONS):
            terminal_node = self.rollout(leaf, 0)
            value = self.evaluate_node(terminal_node)
            self.backpropagate(leaf, value)

        best_child = node.best_child()

        return best_child.move, best_child.prob

    def uct(self, node, c=MCTS_C):
        """
        UCT is a core of MCTS. It allows us to choose next node among visited nodes.
        
        Q_v/N_v                               - exploitation component (favors nodes that were winning)
        torch.sqrt(torch.log(N_v_parent)/N_v) - exploration component (favors node that weren't visited)
        c                                     - tradeoff
        
        In competetive games Q is always computed relative to player who moves.

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

        move, prob = self.traverse_policy(node)
        return self.move(node, move, prob)  

    def traverse_policy(self, node):
        """Traverse policy in uniform random"""
        unexplored_moves = list(node.possible_unexplored_moves())

        index = np.random.choice(len(unexplored_moves))
        move = unexplored_moves[index]
        return move, 1. / len(node.possible_moves())

    def rollout(self, node, depth):
        """Rollout a node according to a rollout policy."""
        if depth > MCTS_ROLLOUT_DEPTH:
            return node

        if node.is_terminal:
            return node

        move, prob = self.rollout_policy(node)
        new_node = self.move(node, move, prob)  
        return self.rollout(new_node, depth+1)

    def rollout_policy(self, node):
        """
        A Policy used to pick next best move
        In Non-Neural Monte Carlo Tree it is random uniform.
        """
        possible_moves = list(node.possible_moves())

        index = np.random.choice(len(possible_moves))
        move = possible_moves[index]
        return move, 1. / len(possible_moves)

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
    
    def rollout_policy(self, node):
        
        state = node.prepared_game_state()
        state_tensor = torch.from_numpy(state).float().to(self.device).unsqueeze(0)
        possible_moves = node.possible_moves(raw=True)
        possible_moves_tensor = torch.from_numpy(possible_moves).to(self.device).unsqueeze(0)
        probs_tensor, _ = self.actor_critic_network(state_tensor, possible_moves_tensor)
        probs_tensor = probs_tensor.squeeze().cpu().detach().numpy()

        table = probs_tensor / probs_tensor.sum()
        moves = np.argwhere(table>0).tolist()
        if len(moves) == 0:
            return None, 1.0
        probs = [probs_tensor[m[0], m[1], m[2], m[3]] for m in moves]
        index = np.random.choice(np.arange(len(probs)), p=probs)

        return moves[index], probs[index]

    def uct(self, node, c=MCTS_C):
        N_v = node.number_of_visits + 1
        N_v_parent = node.parent.number_of_visits + 1

        # TODO: test
        V_current = self.estimate_node_value(node)
        # for child in node.children:
        # V_current -= self.estimate_node_value(child)
        
        result = V_current/N_v + c * np.sqrt(np.log(N_v_parent)/N_v) * node.prob

        return result

    """Estimate node value with neural network"""
    def estimate_node_value(self, node):
        state = node.prepared_game_state(node.current_player())
        state_tensor = torch.from_numpy(state).float().to(self.device).unsqueeze(0)
        possible_moves = node.possible_moves(raw=True)
        possible_moves_tensor = torch.from_numpy(possible_moves).to(self.device).unsqueeze(0)
        _, v = self.actor_critic_network(state_tensor, possible_moves_tensor)
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
            print("number of moves: ", len(trajectory), )
            states = np.array([node.prepared_game_state(terminal_node.current_player()) for node in trajectory])
            # actions = np.array([node.action for node in trajectory])
            # action_probs = np.array([node.prob for node in trajectory])
            
            states_tensor = torch.from_numpy(states).float().to(self.device)
            possible_moves = np.array([node.possible_moves(raw=True) for node in trajectory])
            possible_moves_tensor = torch.from_numpy(possible_moves).to(self.device)
            # action_prob_tensors = torch.from_numpy(action_probs).float().to(self.device)
            probs, values = self.actor_critic_network(states_tensor, possible_moves_tensor)
            # Loss function. Core of alpha-zero
            loss_term_1 = (values - score).pow(2) #- (probs * torch.log(probs)).sum()).sum()
            loss_term_2 = 0
            for i, node in enumerate(trajectory):
                prob = probs[i]
                if node.move is not None:
                    prob = probs[i, node.move[0], node.move[1], node.move[2], node.move[3]]
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


"""Game Tree Node"""
class Node:
    
    def __init__(self, parent, original_state, move, prob=0.0, is_terminal=False):
        self.uuid = uuid.uuid1()
        self.parent = parent
        self.children = []
        
        # State
        # Keeping ndarray and original object
        # This probably consumes too much memory
        self.original_state = original_state
        self.is_terminal = is_terminal
        self.value = 0

        # MCT node properties
        self.number_of_visits = 0
        self.q_black = 0
        self.q_white = 0
        # Traversal properties
        self.prob = prob
        self.move = move

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

        # if player == 1:
        #     return np.flip(np.flip(state, 1), 0)

        return state

    def get_piece_by_id(self, piece_id):
        return next(
            filter(lambda p: p['id'] == piece_id, 
                self.original_state['pieces']
            ))

    def get_piece_by_coord(self, x, y):
        return next(
            filter(lambda p: p['x'] == x and p['y'] == y, 
                self.original_state['pieces']
            ))

    """White figures on board"""
    def whites(self):
        state = state_to_board(self.original_state)
        return state[1]
    
    """Black figures on board"""
    def blacks(self):
        state = state_to_board(self.original_state)
        return state[0]

    """Return 1 if current player plays black, and -1 for whites"""
    def current_player(self):
        if self.original_state['turn']['color'] == 'Black':
            return 1
        return 0
    
    def possible_moves(self, player=None, raw=False):
        """List of possible next moves"""
        if player == None:
            player = self.current_player()

        coords = []
        for piece_id in range(0, 18):
            piece = self.get_piece_by_id(piece_id)
            
            for move in self.original_state['moveset'][piece_id]:
                coords.append((piece['x'], piece['y'], move[0], move[1]))

        moves = np.zeros((8, 8, 8, 8))
        for c in coords:
            moves[c] = 1
        
        mask = self.possible_moves_mask(player)
        moves = moves * mask

        if raw:
            return moves

        return np.argwhere(moves).tolist()

    def possible_moves_mask(self, player=None):
        """Return list of possible next moves as int mask"""
        if player == None:
            player = self.current_player()

        coords = []
        for piece_id in range(0, 18):
            piece = self.get_piece_by_id(piece_id)

            if (piece['color'] == "Black" and player == 1) or (piece['color'] == "White" and player == 0):
                coords.append((piece['x'], piece['y']))

        moves = np.zeros((8, 8, 8, 8))
        for c in coords:
            moves[c] = 1

        return moves

    """How far node from root"""
    def depth(self):
        return len(self.unroll())

    """Whether node all of node's children were expanded"""
    def is_fully_expanded(self):
        return len(self.possible_unexplored_moves()) == 0 or self.is_terminal

    def best_uct(self, uct_func):
        """Pick child node with highest UCT"""
        return sorted(self.children, key=lambda node: uct_func(node), reverse=True)[0]

    """Pick unvisited child node"""
    def possible_unexplored_moves(self):
        possible_moves_set = set([tuple(m) for m in self.possible_moves()])
        explored_moves_set = set([tuple(m.move) for m in self.children])
        return possible_moves_set - explored_moves_set

    """Return best child nod"""
    def best_child(self):
        return sorted(self.children, key=lambda node: node.number_of_visits, reverse=True)[0]

    def is_root(self):
        return self.parent == None

    """Update node statistics"""
    def update_stats(self, result):
        self.number_of_visits += 1
        if result > 0:
            self.q_black += 1
        if result < 0:
            self.q_white += 1

    """Return list of nodes to root"""
    def unroll(self):
        nodes = [self]
        node = self
        while node.parent is not None:
            node = node.parent
            nodes.append(node)

        return nodes