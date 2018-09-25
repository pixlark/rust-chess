extern crate sdl2;

use sdl2::event::Event;
use sdl2::image;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

#[derive(Copy, Clone)]
enum Side {
    White = 0,
    Black = 1,
}

#[derive(Copy, Clone)]
enum Piece {
    Pawn(Side),
    Knight(Side),
    Bishop(Side),
    Rook(Side),
    Queen(Side),
    King(Side),
}

impl Piece {
    fn texture_index(self: &Piece) -> (usize, usize) {
        // TODO(pixlark): Please tell me there's a better way to do this.
        match self {
            Piece::Pawn(side) => (*side as usize, 0),
            Piece::Knight(side) => (*side as usize, 1),
            Piece::Bishop(side) => (*side as usize, 2),
            Piece::Rook(side) => (*side as usize, 3),
            Piece::Queen(side) => (*side as usize, 4),
            Piece::King(side) => (*side as usize, 5),
        }
    }
}

/// Stores textures for all pieces, black and white
///    W B
///    0 1
/// P 0
/// N 1
/// B 2
/// R 3
/// Q 4
/// K 5
struct TextureTable<'a> {
    table: [[Option<Texture<'a>>; 6]; 2],
}

impl<'a> TextureTable<'a> {
    fn new(creator: &'a TextureCreator<WindowContext>) -> Result<TextureTable<'a>, String> {
        let mut table = TextureTable {
            table: Default::default(),
        };
        let mut path = std::env::current_exe().unwrap();
        for i in 0..3 {
            path.pop();
        }
        let names = [
            [
                "white_pawn.png",
                "white_knight.png",
                "white_bishop.png",
                "white_rook.png",
                "white_queen.png",
                "white_king.png",
            ],
            [
                "black_pawn.png",
                "black_knight.png",
                "black_bishop.png",
                "black_rook.png",
                "black_queen.png",
                "black_king.png",
            ],
        ];
        for side in 0..2 {
            for piece in 0..6 {
                path.push(names[side][piece]);
                table.table[side][piece] =
                    Option::Some(creator.load_texture(path.as_path().to_str().unwrap())?);
                path.pop();
            }
        }
        Result::Ok(table)
    }
}

#[derive(Copy, Clone)]
struct SquareProp {
    piece_visible: bool,
}

struct Board {
    /// [File][Rank]
    board: [[Option<Piece>; 8]; 8],
    props: [[SquareProp; 8]; 8],
}

#[derive(Debug, Copy, Clone)]
struct Pos {
    file: usize,
    rank: usize,
}

impl Pos {
    fn new(file: usize, rank: usize) -> Pos {
        Pos { file, rank }
    }
    fn from_ordinals(file: usize, rank: usize) -> Pos {
        Pos {
            file: file - 1,
            rank: rank - 1,
        }
    }
}

const SQUARE_SIZE: u32 = 64;
const WHITE_SQUARE: Color = Color {
    r: 174,
    g: 167,
    b: 149,
    a: 255,
};
const BLACK_SQUARE: Color = Color {
    r: 126,
    g: 118,
    b: 099,
    a: 255,
};
type WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;

impl Board {
    fn empty() -> Board {
        Board {
            board: [[Option::None; 8]; 8],
            props: [[SquareProp {
                piece_visible: true,
            }; 8]; 8],
        }
    }
    fn starting() -> Board {
        let mut board = Board::empty();
        for i in 1..9 {
            board.place(Piece::Pawn(Side::White), Pos::from_ordinals(i, 2));
            board.place(Piece::Pawn(Side::Black), Pos::from_ordinals(i, 7));
        }
        for i in [1usize, 8usize].iter() {
            board.place(Piece::Rook(Side::White), Pos::from_ordinals(*i, 1));
            board.place(Piece::Rook(Side::Black), Pos::from_ordinals(*i, 8));
        }
        for i in [2usize, 7usize].iter() {
            board.place(Piece::Knight(Side::White), Pos::from_ordinals(*i, 1));
            board.place(Piece::Knight(Side::Black), Pos::from_ordinals(*i, 8));
        }
        for i in [3usize, 6usize].iter() {
            board.place(Piece::Bishop(Side::White), Pos::from_ordinals(*i, 1));
            board.place(Piece::Bishop(Side::Black), Pos::from_ordinals(*i, 8));
        }
        board.place(Piece::Queen(Side::White), Pos::from_ordinals(4, 1));
        board.place(Piece::Queen(Side::Black), Pos::from_ordinals(4, 8));
        board.place(Piece::King(Side::White), Pos::from_ordinals(5, 1));
        board.place(Piece::King(Side::Black), Pos::from_ordinals(5, 8));
        board
    }
    fn place(self: &mut Board, piece: Piece, pos: Pos) {
        self.board[pos.file][pos.rank] = Option::Some(piece);
    }
    fn at(self: &Board, pos: Pos) -> Option<Piece> {
        self.board[pos.file][pos.rank]
    }
    fn draw_square(self: &Board, canvas: &mut WindowCanvas, pos: Pos) -> Result<(), String> {
        if (pos.file + pos.rank) % 2 == 0 {
            canvas.set_draw_color(WHITE_SQUARE);
        } else {
            canvas.set_draw_color(BLACK_SQUARE);
        }
        canvas.fill_rect(Rect::new(
            (pos.file as i32) * (SQUARE_SIZE as i32),
            (pos.rank as i32) * (SQUARE_SIZE as i32),
            SQUARE_SIZE,
            SQUARE_SIZE,
        ))?;
        Result::Ok(())
    }
    fn draw_piece(
        self: &Board,
        piece: Piece,
        pos: Pos,
        canvas: &mut WindowCanvas,
        texture_table: &TextureTable,
    ) {
        let tex_index = piece.texture_index();
        canvas.copy(
            texture_table.table[tex_index.0][tex_index.1]
                .as_ref()
                .unwrap(),
            Option::None,
            Option::Some(Rect::new(
                (pos.file as i32) * (SQUARE_SIZE as i32),
                (7 - pos.rank as i32) * (SQUARE_SIZE as i32),
                SQUARE_SIZE,
                SQUARE_SIZE,
            )),
        );
    }
    fn draw(
        self: &Board,
        canvas: &mut WindowCanvas,
        texture_table: &TextureTable,
    ) -> Result<(), String> {
        // Squares
        for rank in 0..8 {
            for file in 0..8 {
                self.draw_square(canvas, Pos::new(file, rank))?;
            }
        }
        // Pieces
        for rank in 0..8 {
            for file in 0..8 {
                if !self.props[file][rank].piece_visible {
                    continue;
                }
                let piece_op = self.at(Pos::new(file, rank));
                if piece_op.is_some() {
                    self.draw_piece(
                        piece_op.unwrap(),
                        Pos::new(file, rank),
                        canvas,
                        texture_table,
                    );
                }
            }
        }
        Result::Ok(())
    }
    fn coord_to_pos(coord: (i32, i32)) -> Pos {
        Pos::new(
            (coord.0 as f32 / SQUARE_SIZE as f32).floor() as usize,
            7 - (coord.1 as f32 / SQUARE_SIZE as f32).floor() as usize,
        )
    }
    fn get_click(self: &mut Board, pump: &sdl2::EventPump) -> Option<Pos> {
        let mouse_state = pump.mouse_state();
        if mouse_state.left() {
            let pos = Board::coord_to_pos((mouse_state.x(), mouse_state.y()));
            Option::Some(pos)
        } else {
            Option::None
        }
    }
}

#[derive(Debug)]
struct ControlState {
    active_piece: Option<Pos>,
}

fn update(board: &mut Board, control_state: &mut ControlState, pump: &sdl2::EventPump) {
    match board.get_click(pump) {
        Option::Some(pos) => {
            if control_state.active_piece.is_none() {
                control_state.active_piece = Option::Some(pos);
                board.props[pos.file][pos.rank].piece_visible = false;
            }
        }
        Option::None => {
            if control_state.active_piece.is_some() {
                let pos = control_state.active_piece.unwrap();
                board.props[pos.file][pos.rank].piece_visible = true;
                control_state.active_piece = Option::None;
            }
        }
    }
}

fn draw_transient_piece(
    canvas: &mut WindowCanvas,
    board: &Board,
    control_state: &ControlState,
    pump: &sdl2::EventPump,
    texture_table: &TextureTable,
) {
    let mouse_state = pump.mouse_state();
    if control_state.active_piece.is_some() {
        let pos = control_state.active_piece.unwrap();
        if board.at(pos).is_some() {
            let tex_index = board.at(pos).unwrap().texture_index();
            canvas.copy(
                texture_table.table[tex_index.0][tex_index.1]
                    .as_ref()
                    .unwrap(),
                Option::None,
                Option::Some(Rect::new(
                    mouse_state.x() - (SQUARE_SIZE as i32 / 2),
                    mouse_state.y() - (SQUARE_SIZE as i32 / 2),
                    SQUARE_SIZE,
                    SQUARE_SIZE,
                )),
            );            
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();

    let sdl_image_context = image::init(image::InitFlag::all()).unwrap();

    let window = video_system
        .window("SDL2 from Rust", SQUARE_SIZE * 8, SQUARE_SIZE * 8)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let texture_table = TextureTable::new(&texture_creator).unwrap();

    let mut board: Board = Board::starting();

    let mut control_state = ControlState {
        active_piece: Option::None,
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => (),
            }
        }
        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x00));

        update(&mut board, &mut control_state, &event_pump);
        //println!("{:?}", control_state);

        board.draw(&mut canvas, &texture_table).unwrap();
        draw_transient_piece(&mut canvas, &board, &control_state, &event_pump, &texture_table);
        canvas.present();
    }
}
