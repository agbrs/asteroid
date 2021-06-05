#![no_std]
#![no_main]

use agb::display::{
    object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, Size},
    HEIGHT, WIDTH,
};

struct Character<'a> {
    object: ObjectAffine<'a>,
    matrix: AffineMatrix<'a>,
    position: Vector2D,
    velocity: Vector2D,
}

#[derive(Clone, Copy)]
struct Vector2D {
    x: i32,
    y: i32,
}

fn convert_array<T>(arr: &'static [u8]) -> &[T] {
    unsafe {
        &(arr as *const [u8] as *const [T]).as_ref().unwrap()
            [..arr.len() / core::mem::size_of::<T>()]
    }
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();

    let images = convert_array(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/graphics.img.bin"
    )));
    let palette = convert_array(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/graphics.pal.bin"
    )));

    let mut gfx = agb.display.video.tiled0();
    gfx.set_sprite_palette(palette);
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

    let mut angle = 0;

    let mut input = agb::input::ButtonController::new();

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

        character
            .object
            .set_x(((8 + character.position.x >> 8).rem_euclid(WIDTH + 16) - 16) as u16);
        character
            .object
            .set_y(((8 + character.position.y >> 8).rem_euclid(HEIGHT + 16) - 16) as u16);

        character.object.commit();

        vblank.wait_for_VBlank();
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
