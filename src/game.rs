use bevy::prelude::*;
use pyo3::prelude::*;

use dict_derive::{FromPyObject, IntoPyObject};

#[derive(Debug)]
pub enum GameTermination {
    White,
    Black,
    Unterminated,
}

#[derive(Debug, Clone)]
pub enum MoveType {
    Invalid,
    JumpOver,
    Regular,
}

pub type Position = (u8, u8);

#[derive(PartialEq, Debug, Clone)]
#[pyclass]
pub struct PlayerTurn {
    // #[pyo3(get, set)]
    pub color: Color,
    #[pyo3(get, set)]
    pub turn_count: u8,
    #[pyo3(get, set)]
    pub chain_count: u8,
}

impl Default for PlayerTurn {
    fn default() -> Self {
        PlayerTurn {
            color: Color::White,
            turn_count: 0,
            chain_count: 0,
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
    }
}

#[derive(Component, Debug, Clone, PartialEq, Copy)]
#[pyclass]
pub struct Piece {
    pub color: Color,
    #[pyo3(get, set)]
    pub y: u8,
    #[pyo3(get, set)]
    pub x: u8,
    #[pyo3(get, set)]
    pub id: u8,
}

#[pymethods]
impl Piece {
    pub fn get_color(&self) -> String {
        match self.color {
            Color::Black => "black".to_string(),
            Color::White => "white".to_string(),
        }
    }
}

impl Piece {
    pub fn move_to_square(&mut self, square: Square) {
        self.x = square.x;
        self.y = square.y;
    }

    pub fn translation(&self) -> Vec3 {
        let v1 = Vec3::new(self.x as f32, 0.1, self.y as f32);
        return v1;
    }

    pub fn is_move_valid(&self, new_square: Square, pieces: &Vec<Piece>) -> MoveType {
        let is_square_occopied = pieces
            .iter()
            .filter(|p| p.x == new_square.x && p.y == new_square.y)
            .count()
            == 1;

        if is_square_occopied {
            return MoveType::Invalid;
        }

        let collision_count =
            self.is_path_empty((self.x, self.y), (new_square.x, new_square.y), pieces);

        // move to empty square
        if collision_count == 0 {
            let horizontal_move =
                (self.x as i8 - new_square.x as i8).abs() == 1 && (self.y == new_square.y);
            let vertical_move =
                (self.y as i8 - new_square.y as i8).abs() == 1 && (self.x == new_square.x);
            let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                == (self.x as i8 - new_square.x as i8).abs()
                && (self.x as i8 - new_square.x as i8).abs() == 1;

            if horizontal_move || vertical_move || diagonal_move {
                return MoveType::Regular;
            } else {
                return MoveType::Invalid;
            }
        } else if collision_count == 1 {
            let horizontal_move =
                (self.x as i8 - new_square.x as i8).abs() == 2 && (self.y == new_square.y);
            let vertical_move =
                (self.y as i8 - new_square.y as i8).abs() == 2 && (self.x == new_square.x);
            let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                == (self.x as i8 - new_square.x as i8).abs()
                && (self.x as i8 - new_square.x as i8).abs() == 2;
            if horizontal_move || vertical_move || diagonal_move {
                return MoveType::JumpOver;
            } else {
                return MoveType::Invalid;
            }
        } else {
            return MoveType::Invalid;
        }
    }

    pub fn is_path_empty(&self, begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> u8 {
        let mut collision_count: u8 = 0;
        // Same column
        if begin.0 == end.0 {
            for piece in pieces {
                if piece.x == begin.0
                    && ((piece.y > begin.1 && piece.y < end.1)
                        || (piece.y > end.1 && piece.y < begin.1))
                {
                    collision_count += 1;
                }
            }
        }
        // Same row
        if begin.1 == end.1 {
            for piece in pieces {
                if piece.y == begin.1
                    && ((piece.x > begin.0 && piece.x < end.0)
                        || (piece.x > end.0 && piece.x < begin.0))
                {
                    collision_count += 1;
                }
            }
        }

        // Diagonals
        let x_diff = (begin.0 as i8 - end.0 as i8).abs();
        let y_diff = (begin.1 as i8 - end.1 as i8).abs();
        if x_diff == y_diff {
            for i in 1..x_diff {
                let pos = if begin.0 < end.0 && begin.1 < end.1 {
                    // left bottom - right top
                    (begin.0 + i as u8, begin.1 + i as u8)
                } else if begin.0 < end.0 && begin.1 > end.1 {
                    // left top - right bottom
                    (begin.0 + i as u8, begin.1 - i as u8)
                } else if begin.0 > end.0 && begin.1 < end.1 {
                    // right bottom - left top
                    (begin.0 - i as u8, begin.1 + i as u8)
                } else {
                    // begin.0 > end.0 && begin.1 > end.1
                    // right top - left bottom
                    (begin.0 - i as u8, begin.1 - i as u8)
                };

                if color_of_square(pos, pieces).is_some() {
                    collision_count += 1;
                }
            }
        }

        return collision_count;
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Color {
    White,
    Black,
}

#[derive(Component, Copy, Clone, Debug, FromPyObject, IntoPyObject)]
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

#[derive(Debug, PartialEq, Default, Clone)]
#[pyclass]
pub struct GameState {
    #[pyo3(get, set)]
    pub pieces: Vec<Piece>,
    pub turn: PlayerTurn,
}

#[derive(Debug, Clone)]
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
                x,
                y,
                id: i,
            });
            i += 1;
        }

        for (x, y) in black_start_positions() {
            pieces.push(Piece {
                color: Color::Black,
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

        return Game {
            squares,
            state: GameState {
                pieces,
                turn: PlayerTurn {
                    color: Color::White,
                    chain_count: 0,
                    turn_count: 0,
                },
            },
        };
    }
}

impl Game {
    pub fn new() -> Game {
        return Game {
            ..Default::default()
        };
    }

    pub fn check_termination(&self) -> GameTermination {
        let piece_in_set = |p: &Piece, collection: Vec<Position>| -> bool {
            let cnt = collection
                .iter()
                .filter(|e| e.0 == p.x && e.1 == p.y)
                .count();
            return cnt > 0;
        };

        // Game end condition check
        let number_of_whites = self
            .state
            .pieces
            .iter()
            .filter(|p| (p.color == Color::White) && piece_in_set(p, black_start_positions()))
            .count();

        let number_of_blacks = self
            .state
            .pieces
            .iter()
            .filter(|p| (p.color == Color::Black) && piece_in_set(p, white_start_positions()))
            .count();

        if number_of_whites == 9 || number_of_blacks == 9 {
            return match self.state.turn.color {
                Color::White => GameTermination::Black,
                Color::Black => GameTermination::White,
            };
        }

        if self.state.turn.turn_count >= 40 {
            if number_of_whites > number_of_blacks {
                return GameTermination::White;
            } else {
                return GameTermination::Black;
            }
        }

        return GameTermination::Unterminated;
    }

    pub fn step(
        &mut self,
        piece: &mut Piece,
        square: Square,
    ) -> (MoveType, &GameState, GameTermination) {
        let mut move_type: MoveType = MoveType::Invalid;

        if self.state.turn.chain_count > 10 {
            self.state.turn.change();
            return (MoveType::Regular, &self.state, self.check_termination());
        }

        match piece.is_move_valid(square, &self.state.pieces) {
            MoveType::JumpOver => {
                piece.move_to_square(square);
                move_type = MoveType::JumpOver;
                self.state.turn.chain_count += 1;
            }
            MoveType::Regular => {
                piece.move_to_square(square);
                move_type = MoveType::Regular;
                self.state.turn.change();
            }
            _ => {}
        }

        for p in self.state.pieces.iter_mut() {
            if p.id == piece.id {
                p.x = piece.x;
                p.y = piece.y;
            }
        }

        return (move_type, &self.state, self.check_termination());
    }
}

pub fn color_of_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<Color> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.color);
        }
    }
    None
}

pub fn white_start_positions() -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for i in 0..3 {
        for j in 5..8 {
            let p: Position = (i as u8, j as u8);
            positions.push(p);
        }
    }

    return positions;
}

pub fn black_start_positions() -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for i in 5..8 {
        for j in 0..3 {
            let p: Position = (i as u8, j as u8);
            positions.push(p);
        }
    }

    return positions;
}
