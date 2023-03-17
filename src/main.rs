use rand::{thread_rng, Rng};
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::{IntoParallelIterator, ParallelIterator, IntoParallelRefIterator};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};
use sdl2::render::Canvas;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const GRID_WIDTH: usize = 80;
const GRID_HEIGHT: usize = 60;
const CELL_SIZE: u32 = 10;

#[derive(Clone)]
enum Cell {
    Alive,
    Dead,
}

type Grid = Vec<Vec<Cell>>;

fn random(max: usize) -> usize {
    thread_rng().gen_range(0..max)
}

fn init_grid() -> Grid {
    (0..GRID_HEIGHT).into_par_iter().map(|_| {
        (0..GRID_WIDTH).into_par_iter().map(|_| match random(4) {
            0 => Cell::Alive,
            _ => Cell::Dead,
        }).collect()
    }).collect()
}

fn display_cell(canvas: &mut Canvas<sdl2::video::Window>, x: i32, y: i32, cell: &Cell) {
    let rect = Rect::new(x * CELL_SIZE as i32, y * CELL_SIZE as i32, CELL_SIZE, CELL_SIZE);
    match cell {
        Cell::Alive => canvas.set_draw_color(Color::RGB(255, 255, 255)),
        Cell::Dead => canvas.set_draw_color(Color::RGB(0,0,0)),
    }
    canvas.fill_rect(rect).unwrap();
}

fn display_grid(canvas: &mut Canvas<sdl2::video::Window>, grid: &Grid){
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            display_cell(canvas, x as i32, y as i32, &grid[y][x]);
        }
    }
}

fn get_index(x: isize, y: isize) -> (usize, usize) {
    let x = if x < 0 {
        GRID_WIDTH as isize + x
    } else if x >= GRID_WIDTH as isize {
        x - GRID_WIDTH as isize
    } else {
        x
    };

    let y =if y < 0 {
        GRID_HEIGHT as isize +y
    } else if y >= GRID_HEIGHT as isize{
        y - GRID_HEIGHT as isize
    } else {
        y
    };
    (x as usize, y as usize)
}

fn count_neighbours(grid: &Grid, x: usize, y:usize) -> u8 {
    let mut count = 0;
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue;
            }
            let (nx, ny) = get_index(x as isize + i, y as isize + j);
            match grid[ny][nx] {
                Cell::Alive => count += 1,
                Cell::Dead => (),
            }
        }
    }
    count
}

fn next_grid(grid: &Grid) -> Grid {
    grid.par_iter().enumerate().map(|(y, row)| {
        row.par_iter().enumerate().map(|(x, cell)| match cell {
            Cell::Alive => match count_neighbours(grid, x, y){
                2 | 3 => Cell::Alive,
                _ => Cell::Dead,
            },
            Cell::Dead => match count_neighbours(grid, x, y) {
                3 => Cell::Alive,
                _ => Cell::Dead,
            },
        }).collect()
    }).collect()
}

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().unwrap();

    let window =video_subsystem.window("Game Of Life", WINDOW_HEIGHT, WINDOW_WIDTH)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas =window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut grid = init_grid();
    let mut event_pump =sdl2_context.event_pump().unwrap();
    let mut timer = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter(){
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }

            if timer.elapsed() >= Duration::from_millis(100) {
                grid = next_grid(&grid);
                timer = Instant::now();
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        display_grid(&mut canvas, &grid);
        canvas.present();
    }
}
