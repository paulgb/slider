use std::{collections::HashSet, fmt::Debug};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum GoalSpecification {
    #[serde(rename = "position")]
    Position((usize, usize)),
}

#[derive(Deserialize, Debug)]
pub struct PieceSpecification {
    size: (usize, usize),
    position: (usize, usize),
    moves: (bool, bool),
}

#[derive(Deserialize, Debug)]
pub struct GameSpecification {
    dimensions: (usize, usize),
    pieces: Vec<PieceSpecification>,
    goal: GoalSpecification,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct GameConfiguration {
    positions: Vec<(usize, usize)>,
}

struct GameBoard {
    bitmap: Vec<bool>,
    width: usize,
    height: usize,
}

impl Debug for GameBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in 0..self.height {
            for y in 0..self.width {
                if self.bitmap[y + x * self.width] {
                    write!(f, "X")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl GameBoard {
    pub fn place(&mut self, piece: &PieceSpecification, position: (usize, usize)) {
        for x_ in 0..piece.size.0 {
            let x = x_ + position.0;
            for y_ in 0..piece.size.1 {
                let y = y_ + position.1;
                self.bitmap[x + y * self.width] = true
            }
        }
    }

    pub fn is_rect_clear(&self, position: (usize, usize), size: (usize, usize)) -> bool {
        for x_ in 0..size.0 {
            let x = x_ + position.0;

            assert!(x < self.width);
            for y_ in 0..size.1 {
                let y = y_ + position.1;
                if self.bitmap[x + y * self.width] {
                    return false;
                }
            }
        }

        return true;
    }

    pub fn clear(&mut self) {
        self.bitmap.fill(false);
    }

    pub fn new(dimensions: (usize, usize)) -> Self {
        GameBoard {
            bitmap: vec![false; dimensions.0 * dimensions.1],
            width: dimensions.0,
            height: dimensions.1,
        }
    }
}

impl GameSpecification {
    pub fn as_configuration(&self) -> GameConfiguration {
        let positions = self.pieces.iter().map(|d| d.position).collect();
        GameConfiguration { positions }
    }

    pub fn valid_neighbors(
        &self,
        configuration: &GameConfiguration,
        board: &mut GameBoard,
        result_buf: &mut Vec<GameConfiguration>,
    ) {
        result_buf.clear();
        board.clear();

        for (piece, position) in self.pieces.iter().zip(&configuration.positions) {
            board.place(piece, *position)
        }

        //println!("{:?}", board);
        assert_eq!(18, board.bitmap.iter().filter(|d| **d).count());

        for piece_idx in 0..self.pieces.len() {
            let piece = &self.pieces[piece_idx];
            let position = configuration.positions[piece_idx];

            if piece.moves.0 {
                // Horizontal moves allowed
                let size = (1, piece.size.1);

                if position.0 > 0 && board.is_rect_clear((position.0 - 1, position.1), size) {
                    // this piece can move left

                    result_buf.push(configuration.clone());
                    result_buf.last_mut().unwrap().positions[piece_idx] =
                        (position.0 - 1, position.1)
                }

                if position.0 + piece.size.0 < board.width
                    && board.is_rect_clear((position.0 + piece.size.0, position.1), size)
                {
                    // this piece can move right

                    result_buf.push(configuration.clone());
                    result_buf.last_mut().unwrap().positions[piece_idx] =
                        (position.0 + 1, position.1)
                }
            }

            if piece.moves.1 {
                // Vertical moves allowed
                let size = (piece.size.0, 1);

                if position.1 > 0 && board.is_rect_clear((position.0, position.1 - 1), size) {
                    // this piece can move up

                    result_buf.push(configuration.clone());
                    result_buf.last_mut().unwrap().positions[piece_idx] =
                        (position.0, position.1 - 1)
                }

                if position.1 + piece.size.1 < board.height
                    && board.is_rect_clear((position.0, position.1 + piece.size.1), size)
                {
                    // this piece can move down

                    result_buf.push(configuration.clone());
                    result_buf.last_mut().unwrap().positions[piece_idx] =
                        (position.0, position.1 + 1)
                }
            }
        }
    }

    pub fn solve(&self) -> Result<()> {
        let mut visited: HashSet<GameConfiguration> = vec![self.as_configuration()].into_iter().collect();
        let mut queue: Vec<GameConfiguration> = vec![self.as_configuration()];
        let mut board: GameBoard = GameBoard::new(self.dimensions);
        let mut neighbors = Vec::new();

        let mut step: u32 = 0;
        while !queue.is_empty() {
            println!("Step: {}, queue size: {}, visited: {}", step, queue.len(), visited.len() - queue.len());
            step += 1;

            let configuration = queue.pop().unwrap();
            self.valid_neighbors(&configuration, &mut board, &mut neighbors);

            for neighbor in neighbors.drain(..) {
                if visited.contains(&neighbor) {
                    continue;
                } else {
                    queue.push(neighbor.clone());
                    visited.insert(neighbor);
                }
            }
        }

        Ok(())
    }
}
