#![no_std]
#![no_main]

use agb::display::{
    object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, ObjectStandard, Size},
    HEIGHT, WIDTH,
};

use agb::sound::{Channel1, DutyCycle, EnvelopeSettings, Noise, SoundDirection, SweepSettings};

use agb::number::{change_base, Number};

struct RandomNumberGenerator {
    state: [u32; 4],
}

impl RandomNumberGenerator {
    fn next(&mut self) -> i32 {
        let result = (self.state[0].wrapping_add(self.state[3]))
            .rotate_left(7)
            .wrapping_mul(9);
        let t = self.state[1].wrapping_shr(9);

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(11);

        result as i32
    }
}

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

struct Asteroid<'a> {
    object: ObjectAffine<'a>,
    matrix: AffineMatrix<'a>,
    position: Vector2D,
    velocity: Vector2D,
    angle: Number<8>,
    angular_velocity: Number<8>,
}

mod sprite_sheet {
    include!(concat!(env!("OUT_DIR"), "/sprite_sheet.rs"));
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();
    agb.sound.enable();

    let images = sprite_sheet::TILE_DATA;
    let palette = sprite_sheet::PALETTE_DATA;

    let mut gfx = agb.display.video.tiled0();
    gfx.set_sprite_palettes(palette);
    gfx.set_sprite_tilemap(images);

    gfx.set_background_palettes(palette);

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

    let mut rng = RandomNumberGenerator {
        state: [0, 1, 2, 3],
    };

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

    let mut game_frame_count = 0;
    let mut asteroids: [Option<Asteroid>; 8] = Default::default();

    let one_number_8: Number<8> = 1.into();
    let one: Number<10> = 1.into();

    loop {
        game_frame_count += 1;

        if !bullet.present {
            input.update();
        }

        character.angle -= one_number_8 * (input.x_tri() as i32) / 100;
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
            input.update();
            bullet.position = character.position;
            bullet.velocity = character.velocity;
            bullet.velocity.x += change_base(character.angle.cos()) * 2;
            bullet.velocity.y += -change_base(character.angle.sin()) * 2;
            bullet.present = true;
            shoot_sound(agb.sound.channel1())
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

        if game_frame_count % 256 == 0 {
            let mut new_asteroid = Asteroid {
                object: objs.get_object_affine(),
                matrix: objs.get_affine(),
                position: Vector2D {
                    x: (WIDTH / 2).into(),
                    y: (HEIGHT / 2).into(),
                },
                velocity: Vector2D {
                    x: Number::<10>::from_raw(rng.next()) % 1,
                    y: Number::<10>::from_raw(rng.next()) % 1,
                },
                angular_velocity: Number::<8>::from_raw(rng.next()) % (one_number_8 / 50),
                angle: Number::<8>::from_raw(rng.next()) % 1,
            };
            new_asteroid.object.set_sprite_size(Size::S16x16);
            new_asteroid.object.set_affine_mat(&new_asteroid.matrix);
            new_asteroid.object.set_tile_id(12);
            new_asteroid.object.show();
            new_asteroid.matrix.attributes =
                agb::syscall::affine_matrix(1.into(), 1.into(), 0.into());
            new_asteroid.matrix.commit();

            for ast in asteroids.iter_mut() {
                if ast.is_none() {
                    *ast = Some(new_asteroid);
                    break;
                }
            }
        }

        for asteroid in asteroids.iter_mut().flatten() {
            asteroid.position.x += asteroid.velocity.x;
            asteroid.position.y += asteroid.velocity.y;

            asteroid.angle += asteroid.angular_velocity;
            asteroid.position.wrap_to_bounds(16, screen_bounds);

            asteroid.matrix.attributes = AffineMatrixAttributes {
                p_a: asteroid.angle.cos().to_raw() as i16,
                p_b: -asteroid.angle.sin().to_raw() as i16,
                p_c: asteroid.angle.sin().to_raw() as i16,
                p_d: asteroid.angle.cos().to_raw() as i16,
            };

            asteroid
                .object
                .set_x((asteroid.position.x.floor() - 8) as u16);
            asteroid
                .object
                .set_y((asteroid.position.y.floor() - 8) as u16);
            asteroid.object.commit();
            asteroid.matrix.commit();
        }

        for asteroid in asteroids.iter_mut() {
            if !bullet.present {
                break;
            }
            if let Some(ast) = asteroid {
                if circle_collision(bullet.position, ast.position, (8 + 4).into()) {
                    *asteroid = None;
                    bullet.present = false;

                    explode_sound(agb.sound.noise());
                }
            }
        }

        vblank.wait_for_VBlank();
    }
}

fn axis_aligned_bounding_box_check(
    pos_a: Vector2D,
    pos_b: Vector2D,
    size_a: Vector2D,
    size_b: Vector2D,
) -> bool {
    pos_a.x < pos_b.x + size_b.x
        && pos_a.x + size_a.x > pos_b.x
        && pos_a.y < pos_b.y + size_b.y
        && pos_a.y + size_a.y > pos_b.y
}

fn circle_collision(pos_a: Vector2D, pos_b: Vector2D, r: Number<10>) -> bool {
    let x = pos_a.x - pos_b.x;
    let y = pos_a.y - pos_b.y;

    x * x + y * y < r * r
}

fn shoot_sound(channel1: Channel1) {
    channel1.play_sound(
        1600,
        Some(15),
        &SweepSettings::new(7, SoundDirection::Decrease, 7),
        &EnvelopeSettings::new(5, SoundDirection::Decrease, 15),
        DutyCycle::OneQuarter,
    );
}

fn explode_sound(noise: Noise) {
    noise.play_sound(
        Some(0),
        &EnvelopeSettings::new(5, SoundDirection::Decrease, 5),
        0,
        false,
        7,
    );
}

impl Vector2D {
    fn wrap_to_bounds(&mut self, size: i32, bounds: Vector2D) {
        self.x = (self.x + size / 2).rem_euclid(bounds.x + size) - size / 2;
        self.y = (self.y + size / 2).rem_euclid(bounds.y + size) - size / 2;
    }
}
