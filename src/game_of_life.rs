use super::utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Grid {
    width: u32,
    height: u32,
    cells_front_buf: Vec<Cell>,
    cells_back_buf: Vec<Cell>,
}

// Public methods, exported to JavaScript.
impl Grid {
    pub fn new(width: u32, height: u32) -> Grid {
        utils::set_panic_hook();
        let cells_front_buf: Vec<Cell> = (0..width * height).map(|i| {
            if i % 2 == 0 || i % 7 == 0 {
                Cell::Alive
            } else {
                Cell::Dead
            }
        }).collect();
        let cells_back_buf = cells_front_buf.clone();
        Grid {
            width,
            height,
            cells_front_buf,
            cells_back_buf,
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn cells(&self) -> *const Cell { self.cells_front_buf.as_ptr() }

    pub fn tick(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells_front_buf[idx];
                let live_neighbors = self.count_alive_neighbors(row, col);

                let next_cell = match (cell, live_neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (otherwise, _) => otherwise,
                };

                self.cells_back_buf[idx] = next_cell;
            }
        }
        std::mem::swap(&mut self.cells_front_buf, &mut self.cells_back_buf);
        utils::log!("Time: {}", utils::now());
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn render_canvas(&self, canvas: &mut web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        const CELL_SIZE: f64 = 5.0;
        const CELL_STRIDE: f64 = CELL_SIZE + 1.0;
        const GRID_COLOR: &str = "#CCCCCC";
        const DEAD_COLOR: &str = "#FFFFFF";
        const ALIVE_COLOR: &str = "#000000";
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        
        // grid
        ctx.begin_path();
        ctx.set_stroke_style(&JsValue::from_str(GRID_COLOR));

        let grid_width = self.width as f64;
        let grid_height = self.height as f64;
        for i in 0..=self.width {
            ctx.move_to((i as f64) * CELL_STRIDE + 1.0, 0.0);
            ctx.line_to((i as f64) * CELL_STRIDE + 1.0, CELL_STRIDE * grid_width + 1.0);
        }

        for j in 0..=self.height {
            ctx.move_to(0.0, (j as f64) * CELL_STRIDE + 1.0);
            ctx.line_to(CELL_STRIDE * grid_height + 1.0, (j as f64) * CELL_STRIDE + 1.0);
        }
        ctx.stroke();

        // cells
        ctx.begin_path();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let color = if self.cells_front_buf[idx] == Cell::Dead { DEAD_COLOR } else { ALIVE_COLOR };
                ctx.set_fill_style(&JsValue::from_str(color));
                ctx.fill_rect(
                    (col as f64) * CELL_STRIDE + 1.0,
                    (row as f64) * CELL_STRIDE + 1.0,
                    CELL_SIZE, CELL_SIZE);
            }
        }
        ctx.stroke();

        Ok(())
    }
}

// private
impl Grid {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn count_alive_neighbors(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for dy in &[self.height - 1, 0, 1] {
            for dx in &[self.width - 1, 0, 1] {
                if *dx == 0 && *dy == 0 { continue; }
                let neighbor_row = (row + dy) % self.height;
                let neighbor_col = (column + dx) % self.width;
                count += self.cells_front_buf[self.get_index(neighbor_row, neighbor_col)] as u8;
            }
        }
        count
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for row in self.cells_front_buf.chunks(self.width as usize) {
            for &cell in row {
                let symbol = if cell == Cell::Dead {'◻'} else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}