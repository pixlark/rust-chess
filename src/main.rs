extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Copy, Clone)]
enum Piece {
    None,
    Rook,
}

struct Board {
    /// [File][Rank]
    board: [[Piece; 8]; 8],
}

struct Pos {
    file: usize,
    rank: usize,
}

const SQUARE_SIZE: u32 = 64;
type WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;

impl Board {
    fn empty() -> Board {
        Board {
            board: [[Piece::None; 8]; 8],
        }
    }
    fn place(self: &mut Board, square: Piece, pos: Pos) {
        self.board[pos.file][pos.rank] = square;
    }
    fn draw_square(self: &Board, canvas: &mut WindowCanvas, pos: Pos)
                   -> Result<(), String> {
        if (pos.file + pos.rank) % 2 == 0 {
            canvas.set_draw_color(Color::RGB(0xff, 0x00, 0x00));
        } else {
            canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        }
        canvas.fill_rect(Rect::new((pos.file as i32) * (SQUARE_SIZE as i32),
                                   (pos.rank as i32) * (SQUARE_SIZE as i32),
                                   SQUARE_SIZE,
                                   SQUARE_SIZE,))?;
        Result::Ok(())
    }
    fn draw(self: &Board, canvas: &mut WindowCanvas)
            -> Result<(), String> {
        //canvas.set_draw_color(Color::RGB(0xff, 0x00, 0x00));
        //canvas.fill_rect(Rect::new(10, 10, 100, 100))?;
        for rank in 0..8 {
            for file in 0..8 {
                self.draw_square(canvas, Pos { file, rank })?;
            }
        }
        Result::Ok(())
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();

    let window = video_system
        .window("SDL2 from Rust", SQUARE_SIZE * 8, SQUARE_SIZE * 8)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut board: Board = Board::empty();
    board.place(Piece::Rook, Pos { file: 4, rank: 4 });

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => (),
            }
        }
        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        canvas.clear();
        board.draw(&mut canvas).unwrap();
        canvas.present();
    }
}
