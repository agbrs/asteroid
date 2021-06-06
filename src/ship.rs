use agb::{
    display::{
        object::{AffineMatrix, AffineMatrixAttributes, ObjectAffine, ObjectControl, Size},
        HEIGHT, WIDTH,
    },
    number::{change_base, Number},
};

use crate::Vector2D;

pub struct Ship<'a> {
    object: ObjectAffine<'a>,
    matrix: AffineMatrix<'a>,
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub angle: Number<8>,
}

impl<'a> Ship<'a> {
    pub fn new(control: &'a ObjectControl) -> Self {
        let mut character = Ship {
            object: control.get_object_affine(),
            matrix: control.get_affine(),
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

        character
    }
    pub fn update_angle(&mut self, angle_diff: Number<8>) {
        self.angle += angle_diff;
    }

    pub fn commit(&mut self) {
        self.matrix.attributes = AffineMatrixAttributes {
            p_a: self.angle.cos().to_raw() as i16,
            p_b: -self.angle.sin().to_raw() as i16,
            p_c: self.angle.sin().to_raw() as i16,
            p_d: self.angle.cos().to_raw() as i16,
        };
        self.matrix.commit();

        self.object.set_x((self.position.x.floor() - 8) as u16);
        self.object.set_y((self.position.y.floor() - 8) as u16);

        self.object.commit();
    }

    pub fn accelerate(&mut self, acceleration: Number<10>) {
        if acceleration != 0.into() {
            self.object.set_tile_id(4);
        } else {
            self.object.set_tile_id(0);
        }

        self.velocity.x += change_base(self.angle.cos()) / 40 * acceleration;
        self.velocity.y += -change_base(self.angle.sin()) / 40 * acceleration;

        self.velocity.x = self.velocity.x * 120 / 121;
        self.velocity.y = self.velocity.y * 120 / 121;

        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
    }
}
