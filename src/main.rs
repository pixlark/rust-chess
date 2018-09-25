extern crate sdl2;

use std::option::Option;
use std::vec::Vec;

use sdl2::event::Event;
use sdl2::image;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Side {
    White = 0,
    Black = 1,
}

impl Side {
    fn flipped(self: &Side) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum PieceType {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone)]
struct Piece {
    piece_type: PieceType,
    side: Side,
}

impl Piece {
    fn new(piece_type: PieceType, side: Side) -> Piece {
        Piece { piece_type, side }
    }
    fn texture_index(self: &Piece) -> (usize, usize) {
        (self.side as usize, self.piece_type as usize)
    }
}

#[derive(Debug, Copy, Clone)]
struct Move {
    pos: Pos,
    capture: Option<Piece>,
}

impl Move {
    fn new(pos: Pos, capture: Option<Piece>) -> Move {
        Move { pos, capture }
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
                    Some(creator.load_texture(path.as_path().to_str().unwrap())?);
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

#[derive(Debug, Copy, Clone, PartialEq)]
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

// Signed pos, useful for calculating possible piece positions
#[derive(Debug, Copy, Clone, PartialEq)]
struct SignedPos {
    file: i32,
    rank: i32,
}

impl SignedPos {
    fn new(file: i32, rank: i32) -> SignedPos {
        SignedPos { file, rank }
    }
    fn from_pos(pos: Pos) -> SignedPos {
        SignedPos {
            file: pos.file as i32,
            rank: pos.rank as i32,
        }
    }
    fn to_pos(self: &SignedPos) -> Pos {
        if !self.valid() {
            panic!("Tried to convert invalid SignedPos to Pos");
        }
        Pos {
            file: self.file as usize,
            rank: self.rank as usize,
        }
    }
    fn valid(self: &SignedPos) -> bool {
        self.rank >= 0 && self.rank < 8 && self.file >= 0 && self.file < 8
    }
}
//

impl Board {
    fn empty() -> Board {
        Board {
            board: [[None; 8]; 8],
            props: [[SquareProp {
                piece_visible: true,
            }; 8]; 8],
        }
    }
    fn starting() -> Board {
        let mut board = Board::empty();
        for i in 1..9 {
            board.place(
                Piece::new(PieceType::Pawn, Side::White),
                Pos::from_ordinals(i, 2),
            );
            board.place(
                Piece::new(PieceType::Pawn, Side::Black),
                Pos::from_ordinals(i, 7),
            );
        }
        for i in [1usize, 8usize].iter() {
            board.place(
                Piece::new(PieceType::Rook, Side::White),
                Pos::from_ordinals(*i, 1),
            );
            board.place(
                Piece::new(PieceType::Rook, Side::Black),
                Pos::from_ordinals(*i, 8),
            );
        }
        for i in [2usize, 7usize].iter() {
            board.place(
                Piece::new(PieceType::Knight, Side::White),
                Pos::from_ordinals(*i, 1),
            );
            board.place(
                Piece::new(PieceType::Knight, Side::Black),
                Pos::from_ordinals(*i, 8),
            );
        }
        for i in [3usize, 6usize].iter() {
            board.place(
                Piece::new(PieceType::Bishop, Side::White),
                Pos::from_ordinals(*i, 1),
            );
            board.place(
                Piece::new(PieceType::Bishop, Side::Black),
                Pos::from_ordinals(*i, 8),
            );
        }
        board.place(
            Piece::new(PieceType::Queen, Side::White),
            Pos::from_ordinals(4, 1),
        );
        board.place(
            Piece::new(PieceType::Queen, Side::Black),
            Pos::from_ordinals(4, 8),
        );
        board.place(
            Piece::new(PieceType::King, Side::White),
            Pos::from_ordinals(5, 1),
        );
        board.place(
            Piece::new(PieceType::King, Side::Black),
            Pos::from_ordinals(5, 8),
        );
        board
    }
    fn capture_on_line(
        self: &Board,
        side: Side,
        pos: Pos,
        variant: SignedPos,
        extents: Option<usize>,
    ) -> Option<Pos> {
        let mut vpos = SignedPos::from_pos(pos);
        let mut len: usize = 0;
        loop {
            vpos.file += variant.file;
            vpos.rank += variant.rank;
            len += 1;
            if extents.map(|e| len > e).unwrap_or(false) || !vpos.valid() || self
                .at(vpos.to_pos())
                .map(|p| p.side == side)
                .unwrap_or(false)
            {
                break;
            }
        }
        None
    }
    fn test_line(self: &Board, pos: Pos, variant: SignedPos, extents: Option<usize>) -> Vec<Move> {
        // Note: This should never fail as an unwrap, which means it's
        // ok to unwrap, right? I mean, if the game state is that
        // messed up, crashing is ok, I think.
        let side = self.at(pos).unwrap().side;
        let mut mvec = Vec::new();
        let mut vpos = SignedPos::from_pos(pos);
        let mut len: usize = 0;
        loop {
            vpos.file += variant.file;
            vpos.rank += variant.rank;
            len += 1;
            // TODO(pixlark): Should I go with a slightly longer if
            // statement here for the sake of clarity?
            if extents.map(|e| len > e).unwrap_or(false) || !vpos.valid() {
                break;
            }
            let piece = self.at(vpos.to_pos());
            if piece.is_some() {
                let piece = piece.unwrap();
                if piece.side != side {
                    mvec.push(Move::new(vpos.to_pos(), Some(piece)));
                }
                break;
            }
            mvec.push(Move::new(vpos.to_pos(), None));
        }
        mvec
    }
    fn move_squares_lateral(self: &Board, pos: Pos, extents: Option<usize>) -> Vec<Move> {
        let mut mvec = Vec::new();
        let variants = [
            SignedPos::new(0, 1),  // N
            SignedPos::new(0, -1), // S
            SignedPos::new(1, 0),  // E
            SignedPos::new(-1, 0), // W
        ];
        for variant in variants.iter() {
            mvec.append(&mut self.test_line(pos, *variant, extents));
        }
        mvec
    }
    fn move_squares_diagonal(self: &Board, pos: Pos, extents: Option<usize>) -> Vec<Move> {
        let mut mvec = Vec::new();
        let variants = [
            SignedPos::new(1, 1),   // NE
            SignedPos::new(1, -1),  // SE
            SignedPos::new(-1, -1), // SW
            SignedPos::new(-1, 1),  // NW
        ];
        for variant in variants.iter() {
            mvec.append(&mut self.test_line(pos, *variant, extents));
        }
        mvec
    }
    fn move_squares_knight(self: &Board, pos: Pos) -> Vec<Move> {
        let side = self.at(pos).unwrap().side;
        let jump_square_offsets = [
            SignedPos::new(-1, -2),
            SignedPos::new(-2, -1),
            SignedPos::new(-2, 1),
            SignedPos::new(-1, 2),
            SignedPos::new(1, 2),
            SignedPos::new(2, 1),
            SignedPos::new(2, -1),
            SignedPos::new(1, -2),
        ];
        let mut mvec = Vec::new();
        for offset in jump_square_offsets.iter() {
            let mut npos = SignedPos::from_pos(pos);
            npos.file += offset.file;
            npos.rank += offset.rank;
            if npos.valid() {
                let piece = self.at(npos.to_pos());
                if piece.is_some() {
                    let piece = piece.unwrap();
                    if piece.side != side {
                        mvec.push(Move::new(npos.to_pos(), Some(piece)));
                    }
                } else {
                    mvec.push(Move::new(npos.to_pos(), None))
                }
            }
        }
        mvec
    }
    fn move_squares_pawn(self: &Board, pos: Pos) -> Vec<Move> {
        let side = self.at(pos).unwrap().side;
        let mut mvec = Vec::new();
        // Movement
        let fwd = match side {
            Side::White => SignedPos::new(pos.file as i32, pos.rank as i32 + 1),
            Side::Black => SignedPos::new(pos.file as i32, pos.rank as i32 - 1),
        };
        if fwd.valid() && self.at(fwd.to_pos()).is_none() {
            mvec.push(Move::new(fwd.to_pos(), None));
        }
        // Capturing
        let caps = [
            SignedPos::new(fwd.file - 1, fwd.rank),
            SignedPos::new(fwd.file + 1, fwd.rank),
        ];
        for cap in caps.iter() {
            if cap.valid() {
                let piece = self.at(cap.to_pos());
                if piece.is_some() {
                    let piece = piece.unwrap();
                    if piece.side != side {
                        mvec.push(Move::new(cap.to_pos(), Some(piece)));
                    }
                }
            }
        }
        mvec
    }
    fn move_squares(self: &Board, piece: Piece, pos: Pos) -> Vec<Move> {
        // TODO(pixlark): Combine piece and pos, just use self.at()?
        match piece.piece_type {
            PieceType::Pawn => self.move_squares_pawn(pos),
            PieceType::Knight => self.move_squares_knight(pos),
            PieceType::Bishop => self.move_squares_diagonal(pos, None),
            PieceType::Rook => self.move_squares_lateral(pos, None),
            PieceType::Queen => {
                let mut lateral = self.move_squares_lateral(pos, None);
                lateral.append(&mut self.move_squares_diagonal(pos, None));
                lateral
            }
            PieceType::King => {
                let mut lateral = self.move_squares_lateral(pos, Some(1));
                lateral.append(&mut self.move_squares_diagonal(pos, Some(1)));
                lateral
            }
            _ => panic!("Movement not implemented for this yet..."),
        }
    }
    fn place(self: &mut Board, piece: Piece, pos: Pos) {
        self.board[pos.file][pos.rank] = Some(piece);
    }
    fn at(self: &Board, pos: Pos) -> Option<Piece> {
        self.board[pos.file][pos.rank]
    }
    fn mov(self: &mut Board, start: Pos, end: Pos) {
        if start == end {
            return;
        }
        if self.at(start).is_some() {
            let start_piece = self.at(start).unwrap();
            let move_squares = self.move_squares(start_piece, start);
            let square = move_squares.iter().find(|&x| x.pos == end);
            if square.is_some() {
                self.place(start_piece, end);
                self.board[start.file][start.rank] = None;
            }
        } else {
            panic!(
                "Game state has become invalid --- \
                 tried to move a piece that doesn't exist."
            );
        }
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
            None,
            Some(Rect::new(
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
            Some(pos)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct ControlState {
    active_piece: Option<Pos>,
    turn: Side,
}

fn update(board: &mut Board, control_state: &mut ControlState, pump: &sdl2::EventPump) {
    let mouse_state = pump.mouse_state(); // TODO(pixlark): Perhaps just pass MouseState rather than EventPump?
    match board.get_click(pump) {
        Some(pos) => {
            if board.at(pos).is_some()
                && board.at(pos).unwrap().side == control_state.turn
                && control_state.active_piece.is_none()
            {
                control_state.active_piece = Some(pos);
                board.props[pos.file][pos.rank].piece_visible = false;
            }
        }
        None => {
            if control_state.active_piece.is_some() {
                let pos = control_state.active_piece.unwrap();
                board.mov(pos, Board::coord_to_pos((mouse_state.x(), mouse_state.y())));
                board.props[pos.file][pos.rank].piece_visible = true;
                control_state.active_piece = None;
                //control_state.turn = control_state.turn.flipped();
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
                None,
                Some(Rect::new(
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

    //let mut board: Board = Board::starting();
    let mut board: Board = Board::empty();
    board.place(
        Piece::new(PieceType::Rook, Side::White),
        Pos::from_ordinals(3, 3),
    );
    board.place(
        Piece::new(PieceType::Pawn, Side::White),
        Pos::from_ordinals(5, 5),
    );
    board.place(
        Piece::new(PieceType::Pawn, Side::Black),
        Pos::from_ordinals(6, 6),
    );
    board.place(
        Piece::new(PieceType::Bishop, Side::White),
        Pos::from_ordinals(4, 6),
    );
    board.place(
        Piece::new(PieceType::Queen, Side::White),
        Pos::from_ordinals(8, 8),
    );
    board.place(
        Piece::new(PieceType::King, Side::White),
        Pos::from_ordinals(1, 2),
    );
    board.place(
        Piece::new(PieceType::Knight, Side::White),
        Pos::from_ordinals(8, 1),
    );

    let mut control_state = ControlState {
        active_piece: None,
        turn: Side::Black,
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

        board.draw(&mut canvas, &texture_table).unwrap();
        draw_transient_piece(
            &mut canvas,
            &board,
            &control_state,
            &event_pump,
            &texture_table,
        );
        canvas.present();
    }
}
