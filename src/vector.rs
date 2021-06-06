use agb::number::Number;

#[derive(Clone, Copy)]
pub struct Vector2D {
    pub x: Number<10>,
    pub y: Number<10>,
}

impl Vector2D {
    pub fn wrap_to_bounds(&mut self, size: i32, bounds: Vector2D) {
        self.x = (self.x + size / 2).rem_euclid(bounds.x + size) - size / 2;
        self.y = (self.y + size / 2).rem_euclid(bounds.y + size) - size / 2;
    }
}
