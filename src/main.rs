#![no_std]
#![no_main]

use agb::display::{
    object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, ObjectStandard, Size},
    HEIGHT, WIDTH,
};

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
    x: i32,
    y: i32,
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
            x: (WIDTH / 2) << 8,
            y: (HEIGHT / 2) << 8,
        },
        velocity: Vector2D { x: 0, y: 0 },
    };

    character.object.set_affine_mat(&character.matrix);
    character.object.show();
    character.object.set_sprite_size(Size::S16x16);

    character.matrix.attributes = agb::syscall::affine_matrix(1 << 8, 1 << 8, 0);
    character.object.commit();
    character.matrix.commit();

    let mut bullet = Bullet {
        object: objs.get_object_standard(),
        position: Vector2D { x: 0, y: 0 },
        velocity: Vector2D { x: 0, y: 0 },
        present: false,
    };

    bullet.object.set_tile_id(4);

    let mut angle = 0;

    let mut input = agb::input::ButtonController::new();

    let screen_bounds = Vector2D {
        x: WIDTH << 8,
        y: HEIGHT << 8,
    };

    loop {
        input.update();

        angle -= input.x_tri() as i16;
        character.matrix.attributes = AffineMatrixAttributes {
            p_a: cos(angle),
            p_b: -sin(angle),
            p_c: sin(angle),
            p_d: cos(angle),
        };
        character.matrix.commit();

        let acceleration = if input.is_pressed(agb::input::Button::A) {
            1
        } else {
            0
        };

        character.velocity.x += acceleration * cos(angle) as i32 >> 5;
        character.velocity.y += acceleration * -sin(angle) as i32 >> 5;

        character.velocity.x = 120 * character.velocity.x / 121;
        character.velocity.y = 120 * character.velocity.y / 121;

        character.position.x += character.velocity.x;
        character.position.y += character.velocity.y;

        character.position.wrap_to_bounds(16 << 8, screen_bounds);

        character
            .object
            .set_x((character.position.x >> 8) as u16 - 8);
        character
            .object
            .set_y((character.position.y >> 8) as u16 - 8);

        character.object.commit();

        if input.is_just_pressed(agb::input::Button::B) {
            bullet.position = character.position;
            bullet.velocity = character.velocity;
            bullet.velocity.x += cos(angle) as i32 * 2;
            bullet.velocity.y += -sin(angle) as i32 * 2;
            bullet.present = true;
        }

        if bullet.present {
            bullet.position.x += bullet.velocity.x;
            bullet.position.y += bullet.velocity.y;
            bullet.position.wrap_to_bounds(8 << 8, screen_bounds);
            bullet.object.set_x((bullet.position.x >> 8) as u16 - 4);
            bullet.object.set_y((bullet.position.y >> 8) as u16 - 4);
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

fn sin_quadrent(n: i16) -> i16 {
    SINE_LUT[n.rem_euclid(32) as usize]
}

fn sin(n: i16) -> i16 {
    let quadrent = (n >> 5).rem_euclid(4);
    match quadrent {
        0 => sin_quadrent(n),
        1 => sin_quadrent(-n - 1),
        2 => -sin_quadrent(n),
        3 => -sin_quadrent(-n - 1),
        _ => unreachable!(),
    }
}

fn cos(n: i16) -> i16 {
    sin(n + 32)
}

const SINE_LUT: [i16; 32] = [
    0, 13, 25, 38, 50, 62, 74, 86, 98, 109, 121, 132, 142, 152, 162, 172, 181, 190, 198, 206, 213,
    220, 226, 231, 237, 241, 245, 248, 251, 253, 255, 256,
];
