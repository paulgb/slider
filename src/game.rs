use std::{collections::BTreeSet, fmt::Debug, hash::Hash};
use anyhow::Result;
use serde::Deserialize;
use crate::bidirectional_list::BidirectionalList;

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
    #[allow(unused)]
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

        true
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
}

pub struct GraphGenerator {
    nodes: BidirectionalList<GameConfiguration>,
    edges: BTreeSet<(usize, usize)>,
    queue: Vec<usize>,
    board: GameBoard,
    specification: GameSpecification,
}

impl GraphGenerator {
    pub fn new(specification: GameSpecification) -> Self {
        let board = GameBoard::new(specification.dimensions);
        let mut nodes = BidirectionalList::default();
        let idx = nodes.push(specification.as_configuration());
        let queue = vec![idx];

        GraphGenerator {
            specification,
            nodes,
            edges: Default::default(),
            queue,
            board,
        }
    }

    pub fn enqueue_configuration(
        nodes: &mut BidirectionalList<GameConfiguration>,
        queue: &mut Vec<usize>,
        edges: &mut BTreeSet<(usize, usize)>,
        configuration: GameConfiguration,
        neighbor: usize,
    ) {
        let idx = if let Some(idx) = nodes.get_index(&configuration) {
            // This configuration has already been visited; don't do anything.
            idx
        } else {
            let idx = nodes.push(configuration);
            queue.push(idx);

            idx
        };

        edges.insert((idx.min(neighbor), idx.max(neighbor)));
    }

    pub fn visit_node(&mut self, idx: usize) {
        let configuration = self.nodes.get(idx).unwrap().clone();
        self.board.clear();

        for (piece, position) in self
            .specification
            .pieces
            .iter()
            .zip(&configuration.positions)
        {
            self.board.place(piece, *position)
        }

        assert_eq!(18, self.board.bitmap.iter().filter(|d| **d).count());

        for piece_idx in 0..self.specification.pieces.len() {
            let piece = &self.specification.pieces[piece_idx];
            let position = configuration.positions[piece_idx];

            if piece.moves.0 {
                // Horizontal moves allowed
                let size = (1, piece.size.1);

                if position.0 > 0 && self.board.is_rect_clear((position.0 - 1, position.1), size) {
                    // this piece can move left
                    let mut configuration = configuration.clone();
                    configuration.positions[piece_idx] = (position.0 - 1, position.1);
                    Self::enqueue_configuration(
                        &mut self.nodes,
                        &mut self.queue,
                        &mut self.edges,
                        configuration,
                        idx,
                    );
                }

                if position.0 + piece.size.0 < self.board.width
                    && self
                        .board
                        .is_rect_clear((position.0 + piece.size.0, position.1), size)
                {
                    // this piece can move right
                    let mut configuration = configuration.clone();
                    configuration.positions[piece_idx] = (position.0 + 1, position.1);
                    Self::enqueue_configuration(
                        &mut self.nodes,
                        &mut self.queue,
                        &mut self.edges,
                        configuration,
                        idx,
                    );
                }
            }

            if piece.moves.1 {
                // Vertical moves allowed
                let size = (piece.size.0, 1);

                if position.1 > 0 && self.board.is_rect_clear((position.0, position.1 - 1), size) {
                    // this piece can move up
                    let mut configuration = configuration.clone();
                    configuration.positions[piece_idx] = (position.0, position.1 - 1);
                    Self::enqueue_configuration(
                        &mut self.nodes,
                        &mut self.queue,
                        &mut self.edges,
                        configuration,
                        idx,
                    );
                }

                if position.1 + piece.size.1 < self.board.height
                    && self
                        .board
                        .is_rect_clear((position.0, position.1 + piece.size.1), size)
                {
                    // this piece can move down
                    let mut configuration = configuration.clone();
                    configuration.positions[piece_idx] = (position.0, position.1 + 1);
                    Self::enqueue_configuration(
                        &mut self.nodes,
                        &mut self.queue,
                        &mut self.edges,
                        configuration,
                        idx,
                    );
                }
            }
        }
    }

    pub fn generate(
        &mut self,
    ) -> Result<(
        &BidirectionalList<GameConfiguration>,
        &BTreeSet<(usize, usize)>,
    )> {
        let mut step: u32 = 0;
        while !self.queue.is_empty() {
            if step % 1_000_000 == 0 {
                eprintln!(
                    "Step: {}, queue size: {}, visited: {}",
                    step,
                    self.queue.len(),
                    self.nodes.len() - self.queue.len()
                );    
            }
            step += 1;

            let idx = self.queue.pop().unwrap();
            self.visit_node(idx);
        }

        Ok((&self.nodes, &self.edges))
    }
}
