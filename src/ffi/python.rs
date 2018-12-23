use crate::rules::game::Game;
use crate::rules::game::GobanSizes::Custom;

use libc::uint32_t;
use crate::pieces::util::Coord;
use crate::rules::game::Move;


#[repr(C)]
pub struct CoordC {
    x: uint32_t,
    y: uint32_t,
}


impl Into<Coord> for CoordC {
    fn into(self) -> Coord {
        (self.x as usize, self.y as usize)
    }
}


#[no_mangle]
pub extern fn new_game(size: uint32_t) -> *mut Game {
    Box::into_raw(Box::new(Game::new(Custom(size as usize))))
}

#[no_mangle]
pub extern fn play_game(ptr: *mut Game, coord: CoordC) {
    let mut game = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    let c: (usize, usize) = coord.into();
    game.play(&Move::Play(c.0, c.1));
}

#[no_mangle]
pub extern fn free_game(ptr: *mut Game) {
    if ptr.is_null() {
        return;
    } else {
        unsafe {
            Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern fn print_game(ptr: *mut Game) {
    let mut game = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };
    game.display();
}

