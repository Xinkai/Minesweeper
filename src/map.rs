
extern crate rand;

use std::fmt;
use ansi_term::Colour::Red;

#[derive(Debug)]
pub enum Interaction {
    Opened,    // discovered
    Undiscovered,
    Flagged,    //
}

#[derive(Debug)]
pub struct Cell {
    pub mine: bool,
    pub interaction: Interaction,
    pub nearby: usize,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.interaction {
            Interaction::Undiscovered => {
                if self.mine {
                    write!(f, "{}", Red.paint("X"))
                } else {
                    write!(f, "{}", self.nearby)
                }
            },
            Interaction::Opened       => write!(f, " "),
            Interaction::Flagged      => write!(f, "!"),
        }
    }
}

#[derive(Default)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    mines: usize,
    pub grid: Vec<Cell>,
}

pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Map {
    pub fn populate(&mut self, width: usize, height: usize, mines: usize) -> () {
        use rand::distributions::{IndependentSample, Range};

        let mut rng = rand::thread_rng();

        self.width = width;
        self.height = height;
        self.mines = mines;

        let mut grid = vec![];
        for _ in 0..width * height {
            grid.push(Cell {
                mine: false,
                interaction: Interaction::Undiscovered,
                nearby: 9,
            });
        }

        let range = Range::new(0, width * height - 1);

        let mut mines_filled_count = 0;
        loop {
            let i = range.ind_sample(&mut rng);
            if !grid[i].mine {
                mines_filled_count += 1;
                grid[i].mine = true;
            }
            if mines_filled_count == mines {
                break;
            }
        }

        self.grid = grid;

        // calculate nearby cells
        for row in 0..height {
            for column in 0..width {
                let nearbys = self.get_nearby_cells(column, row);
                let nearby = nearbys.iter().map(|&(left, top)| {
                    (&self).is_mine(left, top)
                }).filter(|&val| { val }).count();
                self.grid[row * width + column].nearby = nearby;
            }
        }

    }

    pub fn is_mine(&self, column: usize, row: usize) -> bool {
        self.grid[row * self.width + column].mine
    }

    pub fn get_nearby_cells(&self, column: usize, row: usize) -> Vec<(usize, usize)> {
        let mut result = vec![];
        if column > 0 {
            result.push((column - 1, row));
            if row > 0 {
                result.push((column - 1, row - 1));
            }
            if row < self.height - 1 {
                result.push((column - 1, row + 1));
            }
        }
        if row > 0 {
            result.push((column, row - 1));
        }
        if row < self.height - 1 {
            result.push((column, row + 1));
        }
        if column < self.width - 1 {
            result.push((column + 1, row));
            if row > 0 {
                result.push((column + 1, row - 1));
            }
            if row < self.height - 1 {
                result.push((column + 1 , row + 1));
            }
        }
        result
    }

    pub fn get_adjacent_cells(&self, column: usize, row: usize) -> Vec<(usize, usize)> {
        let mut result = vec![];
        if column > 0 {
            result.push((column - 1, row));
        }
        if row > 0 {
            result.push((column, row - 1));
        }
        if row < self.height - 1 {
            result.push((column, row + 1));
        }
        if column < self.width - 1 {
            result.push((column + 1, row));
        }
        result
    }

    pub fn reveal(&mut self, column: usize, row: usize) {
        if self.is_mine(column, row) {
            return ();
        } else {
            match self.grid[row * self.width + column].interaction {
                Interaction::Opened => {
                    return ();
                },
                Interaction::Undiscovered => {
                    self.grid[row * self.width + column].interaction = Interaction::Opened;
                },
                _ => (),
            };
        };
        let adjacents = self.get_adjacent_cells(column, row);
        for &(column, row) in adjacents.iter() {
            self.reveal(column, row);
        }
    }
}


impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut repr = Vec::new();
        for l in 0..self.width {
            let mut line_repr = Vec::new();
            for t in 0..self.height {
                let n = self.width * l + t;
                line_repr.push(format!("{} ", self.grid[n]));
            }
            line_repr.push("\n".to_owned());

            let line = line_repr.join(" ");
            repr.push(line);
        }
        write!(f, "===== Map =====\n{}", repr.join("\n"))
    }
}
