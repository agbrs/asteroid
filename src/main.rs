#![no_std]
#![no_main]

const SHOOT_SOUND: &'static [u8] = include_bytes!("../sfx/shoot.raw");
const EXPLODE_SOUND: &'static [u8] = include_bytes!("../sfx/explode.raw");
const BACKGROUND_MUSIC: &'static [u8] = include_bytes!("../sfx/background_music.raw");

use agb::display::tiled0::Background;
use agb::display::{
    object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, ObjectStandard, Size},
    HEIGHT, WIDTH,
};

use agb::sound::mixer::{Mixer, SoundChannel};

use agb::number::{FixedNum, Rect};

type Vector2D = agb::number::Vector2D<FixedNum<10>>;

struct RandomNumberGenerator {
    state: [u32; 4],
}

const DUST_TTL: i32 = 120;

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
    angle: FixedNum<8>,
}

struct Bullet<'a> {
    object: ObjectStandard<'a>,
    position: Vector2D,
    velocity: Vector2D,
    present: bool,
}
struct Asteroid<'a> {
    object: ObjectAffine<'a>,
    matrix: AffineMatrix<'a>,
    position: Vector2D,
    velocity: Vector2D,
    angle: FixedNum<8>,
    angular_velocity: FixedNum<8>,
}

struct Dust<'a> {
    object: ObjectAffine<'a>,
    position: Vector2D,
    velocity: Vector2D,
}

struct DustParticles<'a> {
    matrix: AffineMatrix<'a>,
    dusts: [Dust<'a>; 4],
    angle: FixedNum<8>,
    angular_velocity: FixedNum<8>,
    ttl: i32,
}

mod sprite_sheet {
    include!(concat!(env!("OUT_DIR"), "/sprite_sheet.rs"));
}

mod background_sheet {
    include!(concat!(env!("OUT_DIR"), "/background_sheet.rs"));
}

fn num_digits_iter(mut n: u32) -> impl core::iter::Iterator<Item = u8> {
    let mut length = 0;
    core::iter::from_fn(move || {
        if n == 0 {
            length += 1;
            if length <= 1 {
                Some(0)
            } else {
                None
            }
        } else {
            length += 1;
            let c = n % 10;
            n /= 10;
            Some(c as u8)
        }
    })
}

struct ScoreDisplay {
    map: [u16; 10],
    background: Background,
    score: u32,
}

impl ScoreDisplay {
    fn new(background: Background) -> ScoreDisplay {
        ScoreDisplay {
            map: Default::default(),
            background,
            score: 0,
        }
    }
    fn set_score(&mut self, score: u32) {
        if score == self.score {
            return;
        }
        self.score = score;
        let length = num_digits_iter(score).count();
        for (index, digit) in num_digits_iter(score).enumerate() {
            self.map[length - index - 1] = (digit + 1) as u16;
        }
        self.background.draw_area(
            &self.map,
            (10, 1).into(),
            Rect::new((0, 0).into(), (length as i32, 1).into()),
        );
        self.background.show();
    }
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();
    let mut mixer = agb.mixer.mixer();

    mixer.enable();

    mixer.play_sound(SoundChannel::new(BACKGROUND_MUSIC).should_loop());

    let images = sprite_sheet::TILE_DATA;
    let palette = sprite_sheet::PALETTE_DATA;

    let mut gfx = agb.display.video.tiled0();
    gfx.set_sprite_palettes(palette);
    gfx.set_sprite_tilemap(images);

    gfx.set_background_palettes(background_sheet::PALETTE_DATA);
    gfx.set_background_tilemap(0, background_sheet::TILE_DATA);

    let mut score_display = ScoreDisplay::new(gfx.get_background().unwrap());

    let vblank = agb.display.vblank.get();
    let mut objs = agb.display.object.get();
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
        state: [1014776995, 476057059, 3301633994, 706340607],
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
    let mut asteroids: [Option<Asteroid>; 28] = Default::default();
    let mut dust_particles: [Option<DustParticles>; 8] = Default::default();

    let one_number_8: FixedNum<8> = 1.into();

    loop {
        game_frame_count += 1;
        score_display.set_score(game_frame_count / 60);

        if !bullet.present {
            input.update();
        }

        character.angle -= one_number_8 * (input.x_tri() as i32) / 100;
        character.matrix.attributes = AffineMatrixAttributes {
            p_a: character.angle.cos().to_raw() as i16,
            p_b: character.angle.sin().to_raw() as i16,
            p_c: -character.angle.sin().to_raw() as i16,
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

        character.velocity +=
            Vector2D::new_from_angle(character.angle.change_base()) / 40 * acceleration;

        character.velocity = character.velocity * 120 / 121;

        character.position += character.velocity;

        wrap_to_bounds(&mut character.position, 16, screen_bounds);

        character
            .object
            .set_position(character.position.floor() - (8, 8).into());

        character.object.commit();

        if input.is_just_pressed(agb::input::Button::B) {
            input.update();
            bullet.position = character.position;
            bullet.velocity = character.velocity;
            bullet.velocity += Vector2D::new_from_angle(character.angle.change_base()) * 2;
            bullet.present = true;
            shoot_sound(&mut mixer)
        }

        if bullet.present {
            bullet.position += bullet.velocity;
            wrap_to_bounds(&mut bullet.position, 8, screen_bounds);
            bullet
                .object
                .set_position(bullet.position.floor() - (4, 4).into());
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
                    x: FixedNum::<10>::from_raw(rng.next()) % 1,
                    y: FixedNum::<10>::from_raw(rng.next()) % 1,
                },
                angular_velocity: FixedNum::<8>::from_raw(rng.next()) % (one_number_8 / 50),
                angle: FixedNum::<8>::from_raw(rng.next()) % 1,
            };
            new_asteroid.object.set_sprite_size(Size::S16x16);
            new_asteroid.object.set_affine_mat(&new_asteroid.matrix);

            let tile_id = if rng.next() % 2 == 0 { 12 } else { 16 };

            new_asteroid.object.set_tile_id(tile_id);
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
            asteroid.position += asteroid.velocity;

            asteroid.angle += asteroid.angular_velocity;
            wrap_to_bounds(&mut asteroid.position, 16, screen_bounds);

            asteroid.matrix.attributes = AffineMatrixAttributes {
                p_a: asteroid.angle.cos().to_raw() as i16,
                p_b: asteroid.angle.sin().to_raw() as i16,
                p_c: -asteroid.angle.sin().to_raw() as i16,
                p_d: asteroid.angle.cos().to_raw() as i16,
            };

            asteroid
                .object
                .set_position(asteroid.position.floor() - (8, 8).into());
            asteroid.object.commit();
            asteroid.matrix.commit();
        }

        for asteroid in asteroids.iter_mut() {
            if !bullet.present {
                break;
            }
            if let Some(ast) = asteroid {
                if circle_collision(bullet.position, ast.position, (8 + 4).into()) {
                    let affine = objs.get_affine();
                    let new_dust_particles = [
                        create_dust_particle(objs.get_object_affine(), &ast, &affine, &mut rng),
                        create_dust_particle(objs.get_object_affine(), &ast, &affine, &mut rng),
                        create_dust_particle(objs.get_object_affine(), &ast, &affine, &mut rng),
                        create_dust_particle(objs.get_object_affine(), &ast, &affine, &mut rng),
                    ];

                    for dust_group in dust_particles.iter_mut() {
                        if dust_group.is_none() {
                            *dust_group = Some(DustParticles {
                                angular_velocity: FixedNum::<8>::from_raw(rng.next())
                                    % (one_number_8 / 50),
                                angle: FixedNum::<8>::from_raw(rng.next()) % 1,
                                dusts: new_dust_particles,
                                matrix: affine,
                                ttl: DUST_TTL,
                            });

                            break;
                        }
                    }

                    *asteroid = None;
                    bullet.present = false;

                    explode_sound(&mut mixer);
                }
            }
        }

        for dust_particle_group in dust_particles.iter_mut().flatten() {
            let ttl = dust_particle_group.ttl;
            dust_particle_group.ttl -= 1;

            dust_particle_group.angle += dust_particle_group.angular_velocity;
            let scaling_factor = FixedNum::<8>::new(DUST_TTL) / ttl;

            dust_particle_group.matrix.attributes = AffineMatrixAttributes {
                p_a: (dust_particle_group.angle.cos() * scaling_factor).to_raw() as i16,
                p_b: (dust_particle_group.angle.sin() * scaling_factor).to_raw() as i16,
                p_c: (-dust_particle_group.angle.sin() * scaling_factor).to_raw() as i16,
                p_d: (dust_particle_group.angle.cos() * scaling_factor).to_raw() as i16,
            };

            dust_particle_group.matrix.commit();

            for dust_particle in dust_particle_group.dusts.iter_mut() {
                dust_particle.position += dust_particle.velocity;
                wrap_to_bounds(&mut dust_particle.position, 8, screen_bounds);

                dust_particle
                    .object
                    .set_position(dust_particle.position.floor() - (4, 4).into());

                dust_particle.object.commit();
            }
        }

        for dust_particle_group in dust_particles.iter_mut() {
            if let Some(some_dust_particle_group) = dust_particle_group {
                if some_dust_particle_group.ttl == 0 {
                    *dust_particle_group = None
                }
            }
        }

        vblank.wait_for_VBlank();
        mixer.vblank();
    }
}

fn create_dust_particle<'a>(
    mut obj: ObjectAffine<'a>,
    ast: &Asteroid<'a>,
    affine: &AffineMatrix<'a>,
    rng: &mut RandomNumberGenerator,
) -> Dust<'a> {
    obj.set_affine_mat(affine);
    obj.set_tile_id((20 + rng.next() % 4) as u16);
    obj.show();

    Dust {
        object: obj,
        position: ast.position,
        velocity: ast.velocity
            + Vector2D {
                x: FixedNum::<10>::from_raw(rng.next()) % 1,
                y: FixedNum::<10>::from_raw(rng.next()) % 1,
            },
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

fn circle_collision(pos_a: Vector2D, pos_b: Vector2D, r: FixedNum<10>) -> bool {
    let x = pos_a.x - pos_b.x;
    let y = pos_a.y - pos_b.y;

    x * x + y * y < r * r
}

fn shoot_sound(mixer: &mut Mixer) {
    mixer.play_sound(SoundChannel::new(SHOOT_SOUND));
}

fn explode_sound(mixer: &mut Mixer) {
    mixer.play_sound(SoundChannel::new(EXPLODE_SOUND));
}

fn wrap_to_bounds(v: &mut Vector2D, size: i32, bounds: Vector2D) {
    v.x = (v.x + size / 2).rem_euclid(bounds.x + size) - size / 2;
    v.y = (v.y + size / 2).rem_euclid(bounds.y + size) - size / 2;
}
