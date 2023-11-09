use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const MOVE_LIMIT: u16 = 33;
const CHAIN_LIMIT: u16 = 5;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum GameTermination {
    White(u8),
    Black(u8),
    Draw,
    Unterminated,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum MoveType {
    Invalid,
    Take,
    Regular,
    Pass,
}

pub type Position = (u8, u8);

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTurn {
    pub color: Color,
    pub turn_count: u16,
    pub chain_count: u16,
    pub chain_piece_id: i16,
}

impl Default for PlayerTurn {
    fn default() -> Self {
        PlayerTurn {
            color: Color::White,
            turn_count: 0,
            chain_count: 0,
            chain_piece_id: 0,
        }
    }
}

impl PlayerTurn {
    pub fn change(&mut self) {
        self.color = match self.color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        self.turn_count += 1;
        self.chain_count = 0;
        self.chain_piece_id = -1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum PieceType {
    Normal,
    King,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    pub y: u8,
    pub x: u8,
    pub id: u8,
}

impl Piece {
    pub fn move_to_square(&mut self, square: Square) {
        self.x = square.x;
        self.y = square.y;
    }

    pub fn is_move_valid(&self, new_square: Square, pieces: &Vec<Piece>) -> MoveType {
        if self.x == new_square.x && self.y == new_square.y {
            return MoveType::Invalid;
        }

        let is_square_occupied = pieces
            .iter()
            .filter(|p| p.x == new_square.x && p.y == new_square.y)
            .count()
            > 0;

        if is_square_occupied {
            return MoveType::Invalid;
        }

        let collisions =
            self.path_collisions((self.x, self.y), (new_square.x, new_square.y), pieces);

        let collision_count = collisions.len();

        // move to empty square
        if collision_count == 0 {
            let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                == (self.x as i8 - new_square.x as i8).abs()
                && (self.x as i8 - new_square.x as i8).abs() == 1;

            let direction = (new_square.x as i8 - self.x as i8).signum();

            if diagonal_move
                && ((self.piece_type == PieceType::Normal
                    && self.color == Color::Black
                    && direction == -1)
                    || (self.piece_type == PieceType::Normal
                        && self.color == Color::White
                        && direction == 1)
                    || self.piece_type == PieceType::King)
            {
                MoveType::Regular
            } else {
                MoveType::Invalid
            }
        } else if collision_count == 1 {
            let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                == (self.x as i8 - new_square.x as i8).abs()
                && (self.x as i8 - new_square.x as i8).abs() == 2;

            let direction = (new_square.x as i8 - self.x as i8).signum();

            if diagonal_move
                && ((self.piece_type == PieceType::Normal
                    && self.color == Color::Black
                    && direction == -1)
                    || (self.piece_type == PieceType::Normal
                        && self.color == Color::White
                        && direction == 1)
                    || self.piece_type == PieceType::King)
            {
                if collisions[0].color != self.color {
                    MoveType::Take
                } else {
                    MoveType::Invalid
                }
            } else {
                MoveType::Invalid
            }
        } else {
            MoveType::Invalid
        }
    }

    pub fn path_collisions(
        &self,
        begin: (u8, u8),
        end: (u8, u8),
        pieces: &Vec<Piece>,
    ) -> Vec<Piece> {
        let mut collisions: Vec<Piece> = Vec::new();
        // Diagonals
        let x_diff = (begin.0 as i8 - end.0 as i8).abs();
        let y_diff = (begin.1 as i8 - end.1 as i8).abs();
        if x_diff == y_diff {
            for i in 1..x_diff {
                let pos = if begin.0 < end.0 && begin.1 < end.1 {
                    (begin.0 + i as u8, begin.1 + i as u8)
                } else if begin.0 < end.0 && begin.1 > end.1 {
                    (begin.0 + i as u8, begin.1 - i as u8)
                } else if begin.0 > end.0 && begin.1 < end.1 {
                    (begin.0 - i as u8, begin.1 + i as u8)
                } else {
                    (begin.0 - i as u8, begin.1 - i as u8)
                };

                if let Some(piece) = find_piece_at_position(pos, pieces) {
                    collisions.push(piece);
                }
            }
        }

        collisions
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}
impl Square {
    pub fn color(&self) -> Color {
        if (self.x + self.y + 1) % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub pieces: Vec<Piece>,
    pub removed_pieces: Vec<Piece>,
    pub turn: PlayerTurn,
    pub moveset: [Vec<Position>; 24],
}

#[derive(Resource, Debug, Clone)]
pub struct Game {
    pub state: GameState,
    pub squares: Vec<Square>,
}

impl Default for Game {
    fn default() -> Self {
        let mut pieces: Vec<Piece> = Vec::new();
        let mut i: u8 = 0;

        for (x, y) in white_start_positions() {
            pieces.push(Piece {
                color: Color::White,
                piece_type: PieceType::Normal,
                x,
                y,
                id: i,
            });
            i += 1;
        }

        for (x, y) in black_start_positions() {
            pieces.push(Piece {
                color: Color::Black,
                piece_type: PieceType::Normal,
                x,
                y,
                id: i,
            });
            i += 1;
        }

        let mut squares: Vec<Square> = Vec::new();

        // Spawn 64 squares
        for i in 0..8 {
            for j in 0..8 {
                squares.push(Square { x: i, y: j });
            }
        }

        Game {
            squares,
            state: GameState {
                pieces,
                removed_pieces: Vec::new(),
                moveset: Default::default(),
                turn: PlayerTurn {
                    color: Color::White,
                    chain_count: 0,
                    turn_count: 0,
                    chain_piece_id: -1,
                },
            },
        }
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            ..Default::default()
        }
    }

    #[allow(clippy::comparison_chain)]
    pub fn check_termination(&self) -> GameTermination {
        // Game end condition check
        let number_of_whites = self
            .state
            .pieces
            .iter()
            .filter(|p| (p.color == Color::White))
            .count();

        let number_of_blacks = self
            .state
            .pieces
            .iter()
            .filter(|p| (p.color == Color::Black))
            .count();

        if self.state.turn.turn_count > MOVE_LIMIT || number_of_whites == 0 || number_of_blacks == 0
        {
            if number_of_whites > number_of_blacks {
                return GameTermination::White(number_of_whites as u8);
            } else if number_of_whites < number_of_blacks {
                return GameTermination::Black(number_of_blacks as u8);
            } else {
                return GameTermination::Draw;
            }
        }

        GameTermination::Unterminated
    }

    pub fn step(
        &mut self,
        mut piece: Piece,
        square: Square,
    ) -> (MoveType, &GameState, GameTermination) {
        let mut move_type: MoveType = MoveType::Invalid;

        // chain limit met
        if self.state.turn.chain_count > CHAIN_LIMIT {
            self.state.turn.change();
            return (MoveType::Regular, &self.state, self.check_termination());
        }

        match piece.is_move_valid(square, &self.state.pieces) {
            MoveType::Take => {
                move_type = MoveType::Take;
                let collision = piece.path_collisions(
                    (piece.x, piece.y),
                    (square.x, square.y),
                    &self.state.pieces,
                )[0];

                piece.move_to_square(square);

                for p in self.state.pieces.iter_mut() {
                    if p.id == piece.id {
                        p.x = piece.x;
                        p.y = piece.y;

                        if (p.color == Color::White && p.x == 7)
                            || (p.color == Color::Black && p.x == 0)
                        {
                            p.piece_type = PieceType::King;
                        }
                    }
                }

                if collision.color != piece.color {
                    self.state.removed_pieces.push(collision);
                    self.state.pieces.retain(|p| p.id != collision.id);
                }

                let moveset = self.possible_moves();
                let moveset = &moveset[piece.id as usize];

                if moveset
                    .iter()
                    .filter(|m| {
                        piece.is_move_valid(Square { x: m.0, y: m.1 }, &self.state.pieces)
                            == MoveType::Take
                    })
                    .count()
                    > 0
                {
                    self.state.turn.chain_count += 1;
                    self.state.turn.chain_piece_id = piece.id as i16;
                } else {
                    move_type = MoveType::Regular;
                    self.state.turn.change();
                }
            }
            MoveType::Regular => {
                piece.move_to_square(square);

                for p in self.state.pieces.iter_mut() {
                    if p.id == piece.id {
                        p.x = piece.x;
                        p.y = piece.y;

                        if (p.color == Color::White && p.x == 7)
                            || (p.color == Color::Black && p.x == 0)
                        {
                            p.piece_type = PieceType::King;
                        }
                    }
                }

                move_type = MoveType::Regular;
                self.state.turn.change();
            }
            MoveType::Pass => {
                move_type = MoveType::Pass;
                self.state.turn.change();
            }
            _ => {}
        }

        (move_type, &self.state, self.check_termination())
    }

    pub fn possible_moves(&self) -> [Vec<Position>; 24] {
        let mut moveset: [Vec<Position>; 24] = Default::default();

        for i in 0..24 {
            let p = self.state.pieces.iter().find(|p| p.id == i);

            if p.is_none() {
                continue;
            }

            let p = p.unwrap();

            if self.state.turn.chain_count > 0 && (p.id as i16) != self.state.turn.chain_piece_id {
                continue;
            }

            // move to same position is passing a turn
            // moveset[i as usize].push((p.x, p.y) as Position);

            for s in self.squares.iter() {
                match p.is_move_valid(*s, &self.state.pieces) {
                    MoveType::Take | MoveType::Regular => {
                        let position: Position = (s.x, s.y);
                        moveset[i as usize].push(position);
                    }
                    _ => {}
                }
            }
        }

        moveset
    }
}

pub fn find_piece_at_position(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<Piece> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(*piece);
        }
    }
    None
}

pub fn white_start_positions() -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for i in 0..3 {
        for j in (0..8).step_by(2) {
            let p: Position = (i as u8, j + (i % 2) as u8);
            positions.push(p);
        }
    }

    positions
}

pub fn black_start_positions() -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for i in 5..8 {
        for j in (0..8).step_by(2) {
            let p: Position = (i as u8, j + (i % 2) as u8);
            positions.push(p);
        }
    }

    positions
}
