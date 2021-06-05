#![no_std]
#![no_main]

use agb::display::{
    object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, ObjectStandard, Size},
    HEIGHT, WIDTH,
};

use agb::number::Num;

struct Character<'a> {
    object: ObjectAffine<'a>,
    matrix: AffineMatrix<'a>,
    position: Vector2D,
    velocity: Vector2D,
}

struct Bullet<'a> {
    object: ObjectStandard<'a>,
    position: Vector2D,
    velocity: Vector2D,
    present: bool,
}

#[derive(Clone, Copy)]
struct Vector2D {
    x: Num<8>,
    y: Num<8>,
}

mod sprite_sheet {
    include!(concat!(env!("OUT_DIR"), "/sprite_sheet.rs"));
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();

    let images = sprite_sheet::TILE_DATA;
    let palette = sprite_sheet::PALETTE_DATA;

    let mut gfx = agb.display.video.tiled0();
    gfx.set_sprite_palettes(palette);
    gfx.set_sprite_tilemap(images);

    let vblank = agb.display.vblank.get();
    let mut objs = gfx.object;
    objs.enable();

    let mut character = Character {
        object: objs.get_object_affine(),
        matrix: objs.get_affine(),
        position: Vector2D {
            x: (WIDTH / 2).into(),
            y: (HEIGHT / 2).into(),
        },
        velocity: Vector2D {
            x: 0.into(),
            y: 0.into(),
        },
    };

    character.object.set_affine_mat(&character.matrix);
    character.object.show();
    character.object.set_sprite_size(Size::S16x16);

    character.matrix.attributes = agb::syscall::affine_matrix(1 << 8, 1 << 8, 0);
    character.object.commit();
    character.matrix.commit();

    let mut bullet = Bullet {
        object: objs.get_object_standard(),
        position: Vector2D {
            x: 0.into(),
            y: 0.into(),
        },
        velocity: Vector2D {
            x: 0.into(),
            y: 0.into(),
        },
        present: false,
    };

    bullet.object.set_tile_id(4);

    let mut angle = 0.into();

    let mut input = agb::input::ButtonController::new();

    let screen_bounds = Vector2D {
        x: WIDTH.into(),
        y: HEIGHT.into(),
    };

    loop {
        input.update();

        angle -= input.x_tri() as i32;
        character.matrix.attributes = AffineMatrixAttributes {
            p_a: cos(angle).to_raw() as i16,
            p_b: -sin(angle).to_raw() as i16,
            p_c: sin(angle).to_raw() as i16,
            p_d: cos(angle).to_raw() as i16,
        };
        character.matrix.commit();

        let acceleration = if input.is_pressed(agb::input::Button::A) {
            1
        } else {
            0
        };

        character.velocity.x += cos(angle) / 5 * acceleration;
        character.velocity.y += -sin(angle) / 5 * acceleration;

        character.velocity.x = character.velocity.x * 120 / 121;
        character.velocity.y = character.velocity.y * 120 / 121;

        character.position.x += character.velocity.x;
        character.position.y += character.velocity.y;

        character.position.wrap_to_bounds(16, screen_bounds);

        character
            .object
            .set_x(character.position.x.int() as u16 - 8);
        character
            .object
            .set_y(character.position.y.int() as u16 - 8);

        character.object.commit();

        if input.is_just_pressed(agb::input::Button::B) {
            bullet.position = character.position;
            bullet.velocity = character.velocity;
            bullet.velocity.x += cos(angle) * 2;
            bullet.velocity.y += -sin(angle) * 2;
            bullet.present = true;
        }

        if bullet.present {
            bullet.position.x += bullet.velocity.x;
            bullet.position.y += bullet.velocity.y;
            bullet.position.wrap_to_bounds(8, screen_bounds);
            bullet.object.set_x(bullet.position.x.int() as u16 - 4);
            bullet.object.set_y(bullet.position.y.int() as u16 - 4);
            bullet.object.show();
            bullet.object.commit();
        } else {
            bullet.object.hide();
            bullet.object.commit();
        }

        vblank.wait_for_VBlank();
    }
}

impl Vector2D {
    fn wrap_to_bounds(&mut self, size: i32, bounds: Vector2D) {
        self.x = (self.x + size / 2).rem_euclid(bounds.x + size) - size / 2;
        self.y = (self.y + size / 2).rem_euclid(bounds.y + size) - size / 2;
    }
}

fn sin_quadrent(n: i32) -> Num<8> {
    SINE_LUT[n.rem_euclid(32) as usize]
}

fn sin(n: i32) -> Num<8> {
    let quadrent = (n >> 5).rem_euclid(4);
    match quadrent {
        0 => sin_quadrent(n),
        1 => sin_quadrent(-n - 1),
        2 => -sin_quadrent(n),
        3 => -sin_quadrent(-n - 1),
        _ => unreachable!(),
    }
}

fn cos(n: i32) -> Num<8> {
    sin(n + 32)
}

const SINE_LUT: [Num<8>; 32] = [
    Num::from_raw(0),
    Num::from_raw(13),
    Num::from_raw(25),
    Num::from_raw(37),
    Num::from_raw(50),
    Num::from_raw(62),
    Num::from_raw(74),
    Num::from_raw(86),
    Num::from_raw(98),
    Num::from_raw(109),
    Num::from_raw(120),
    Num::from_raw(131),
    Num::from_raw(142),
    Num::from_raw(152),
    Num::from_raw(162),
    Num::from_raw(171),
    Num::from_raw(180),
    Num::from_raw(189),
    Num::from_raw(197),
    Num::from_raw(205),
    Num::from_raw(212),
    Num::from_raw(219),
    Num::from_raw(225),
    Num::from_raw(231),
    Num::from_raw(236),
    Num::from_raw(240),
    Num::from_raw(244),
    Num::from_raw(247),
    Num::from_raw(250),
    Num::from_raw(252),
    Num::from_raw(254),
    Num::from_raw(255),
];
