#![no_std]
#![no_main]

use agb::display::{
    object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, ObjectStandard, Size},
    HEIGHT, WIDTH,
};

use agb::number::{change_base, Number};

struct Character<'a> {
    object: ObjectAffine<'a>,
    matrix: AffineMatrix<'a>,
    position: Vector2D,
    velocity: Vector2D,
    angle: Number<8>,
}

struct Bullet<'a> {
    object: ObjectStandard<'a>,
    position: Vector2D,
    velocity: Vector2D,
    present: bool,
}

#[derive(Clone, Copy)]
struct Vector2D {
    x: Number<10>,
    y: Number<10>,
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
        angle: 0.into(),
    };

    character.object.set_affine_mat(&character.matrix);
    character.object.show();
    character.object.set_sprite_size(Size::S16x16);
    character.object.set_tile_id(0);

    character.matrix.attributes = agb::syscall::affine_matrix(1.into(), 1.into(), 0);
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

    bullet.object.set_tile_id(8);

    let mut input = agb::input::ButtonController::new();

    let screen_bounds = Vector2D {
        x: WIDTH.into(),
        y: HEIGHT.into(),
    };

    let one: Number<8> = 1.into();

    loop {
        input.update();

        character.angle -= one * (input.x_tri() as i32) / 100;
        character.matrix.attributes = AffineMatrixAttributes {
            p_a: character.angle.cos().to_raw() as i16,
            p_b: -character.angle.sin().to_raw() as i16,
            p_c: character.angle.sin().to_raw() as i16,
            p_d: character.angle.cos().to_raw() as i16,
        };
        character.matrix.commit();

        let acceleration = if input.is_pressed(agb::input::Button::A) {
            character.object.set_tile_id(4);
            1
        } else {
            character.object.set_tile_id(0);
            0
        };

        character.velocity.x += change_base(character.angle.cos()) / 40 * acceleration;
        character.velocity.y += -change_base(character.angle.sin()) / 40 * acceleration;

        character.velocity.x = character.velocity.x * 120 / 121;
        character.velocity.y = character.velocity.y * 120 / 121;

        character.position.x += character.velocity.x;
        character.position.y += character.velocity.y;

        character.position.wrap_to_bounds(16, screen_bounds);

        character
            .object
            .set_x((character.position.x.floor() - 8) as u16);
        character
            .object
            .set_y((character.position.y.floor() - 8) as u16);

        character.object.commit();

        if input.is_just_pressed(agb::input::Button::B) {
            bullet.position = character.position;
            bullet.velocity = character.velocity;
            bullet.velocity.x += change_base(character.angle.cos()) * 2;
            bullet.velocity.y += -change_base(character.angle.sin()) * 2;
            bullet.present = true;
        }

        if bullet.present {
            bullet.position.x += bullet.velocity.x;
            bullet.position.y += bullet.velocity.y;
            bullet.position.wrap_to_bounds(8, screen_bounds);
            bullet.object.set_x((bullet.position.x.floor() - 4) as u16);
            bullet.object.set_y((bullet.position.y.floor() - 4) as u16);
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
