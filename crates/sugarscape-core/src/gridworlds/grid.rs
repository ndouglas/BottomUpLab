//! Boards: the reference's ASCII layouts, the four moves, walls.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

impl Action {
    /// The paper's order, which is also the tie-break order of the greedy policy.
    pub const ALL: [Action; 4] = [Action::Up, Action::Down, Action::Left, Action::Right];

    pub fn index(self) -> usize {
        match self {
            Action::Up => 0,
            Action::Down => 1,
            Action::Left => 2,
            Action::Right => 3,
        }
    }

    pub fn from_index(i: usize) -> Action {
        Action::ALL[i]
    }

    /// (row change, column change).
    pub fn delta(self) -> (i32, i32) {
        match self {
            Action::Up => (-1, 0),
            Action::Down => (1, 0),
            Action::Left => (0, -1),
            Action::Right => (0, 1),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Action::Up => "up",
            Action::Down => "down",
            Action::Left => "left",
            Action::Right => "right",
        }
    }
}

/// A layout as the reference writes it: row 0 at the top, `#` for walls, every row the same width.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    cells: Vec<char>,
}

impl Grid {
    pub fn parse(rows: &[&str]) -> Grid {
        let cols = rows[0].chars().count();
        let mut cells = Vec::with_capacity(rows.len() * cols);
        for row in rows {
            assert_eq!(row.chars().count(), cols, "ragged layout");
            cells.extend(row.chars());
        }
        Grid {
            rows: rows.len(),
            cols,
            cells,
        }
    }

    pub fn at(&self, c: Cell) -> char {
        self.cells[self.index(c)]
    }

    pub fn index(&self, c: Cell) -> usize {
        c.row * self.cols + c.col
    }

    pub fn cell(&self, index: usize) -> Cell {
        Cell {
            row: index / self.cols,
            col: index % self.cols,
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The layout's characters, row-major (the starting board before any object moves).
    pub fn chars(&self) -> Vec<char> {
        self.cells.clone()
    }

    /// The one cell holding `ch`; layouts are constants, so a missing character is a bug.
    pub fn find(&self, ch: char) -> Cell {
        self.find_all(ch)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no {ch:?} in the layout"))
    }

    /// Every cell holding `ch`, row-major.
    pub fn find_all(&self, ch: char) -> Vec<Cell> {
        (0..self.len())
            .filter(|&i| self.cells[i] == ch)
            .map(|i| self.cell(i))
            .collect()
    }

    pub fn is_wall(&self, c: Cell) -> bool {
        self.at(c) == '#'
    }

    /// The adjacent cell in direction `a`, or `None` off the board.
    pub fn neighbor(&self, c: Cell, a: Action) -> Option<Cell> {
        let (dr, dc) = a.delta();
        let row = c.row as i32 + dr;
        let col = c.col as i32 + dc;
        (row >= 0 && col >= 0 && (row as usize) < self.rows && (col as usize) < self.cols)
            .then_some(Cell {
                row: row as usize,
                col: col as usize,
            })
    }

    /// Where a move from `from` in direction `a` ends: `from` itself against a wall or the edge.
    pub fn step(&self, from: Cell, a: Action) -> Cell {
        match self.neighbor(from, a) {
            Some(next) if !self.is_wall(next) => next,
            _ => from,
        }
    }

    pub fn manhattan(a: Cell, b: Cell) -> usize {
        a.row.abs_diff(b.row) + a.col.abs_diff(b.col)
    }

    /// Every non-wall cell, row-major.
    pub fn floor_cells(&self) -> Vec<Cell> {
        (0..self.len())
            .filter(|&i| self.cells[i] != '#')
            .map(|i| self.cell(i))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOARD: [&str; 4] = ["#####", "#A  #", "# # #", "#####"];

    #[test]
    fn parsing_reads_rows_top_down_and_finds_cells() {
        let g = Grid::parse(&BOARD);
        assert_eq!((g.rows, g.cols, g.len()), (4, 5, 20));
        assert_eq!(g.find('A'), Cell { row: 1, col: 1 });
        assert_eq!(g.at(Cell { row: 2, col: 2 }), '#');
        assert_eq!(g.find_all(' ').len(), 4);
        assert_eq!(g.index(Cell { row: 2, col: 3 }), 13);
        assert_eq!(g.cell(13), Cell { row: 2, col: 3 });
        assert_eq!(g.floor_cells().len(), 5, "A counts as floor");
    }

    #[test]
    fn moves_stop_at_walls_and_the_board_edge() {
        let g = Grid::parse(&BOARD);
        let a = Cell { row: 1, col: 1 };
        assert_eq!(g.step(a, Action::Right), Cell { row: 1, col: 2 });
        assert_eq!(g.step(a, Action::Up), a, "wall above");
        assert_eq!(g.step(a, Action::Left), a, "wall left");
        assert_eq!(
            g.step(Cell { row: 1, col: 3 }, Action::Down),
            Cell { row: 2, col: 3 }
        );
        assert_eq!(g.neighbor(Cell { row: 0, col: 0 }, Action::Up), None);
        assert_eq!(Grid::manhattan(a, Cell { row: 2, col: 3 }), 3);
    }

    #[test]
    fn actions_index_in_the_papers_order() {
        assert_eq!(Action::ALL.map(Action::index), [0, 1, 2, 3]);
        assert_eq!(Action::from_index(2), Action::Left);
        assert_eq!(Action::Down.delta(), (1, 0));
        assert_eq!(
            Action::ALL.map(Action::name),
            ["up", "down", "left", "right"]
        );
        assert_eq!(serde_json::to_string(&Action::Right).unwrap(), "\"right\"");
    }
}
