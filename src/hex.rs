use bevy::ecs::component::Component;

// info about hexagons https://www.redblobgames.com/grids/hexagons/

#[derive(PartialEq, Eq, Hash, Component, Clone, Copy, Debug)]
pub struct Hex {
    pub q: i32,
    pub r: i32
}

#[derive(Copy, Clone)]
pub enum Rotation {
    Rot0,
    Rot60Cw,
    Rot60Ccw,
    Rot120Cw,
    Rot120Ccw,
    Rot180,
}

impl Hex {
    pub const ZERO: Self = Self { q: 0, r: 0 };

    fn to_cube(&self) -> HexCube {
        HexCube {
            q: self.q,
            r: self.r,
            s: -self.q - self.r
        }
    }

    pub fn add(&self, rhs: &Hex) -> Hex {
        Self { q: self.q + rhs.q, r: self.r + rhs.r }
    }

    pub fn sub(&self, rhs: &Hex) -> Hex {
        Self { q: self.q - rhs.q, r: self.r - rhs.r }
    }

    pub fn rotate(&self, rotation: Rotation) -> Self {
        self.to_cube().rotate(rotation).to_hex()
    }

    pub fn from_fraction(q: f32, r: f32) -> Self {
        let s = -q - r;
        HexCube::from_fraction(q, r, s).to_hex()
    }
}

impl std::ops::Add<Hex> for Hex {
    type Output = Hex;

    fn add(self, rhs: Hex) -> Hex {
        Self { q: self.q + rhs.q, r: self.r + rhs.r }
    }
}

impl std::ops::Sub<Hex> for Hex {
    type Output = Hex;

    fn sub(self, rhs: Hex) -> Hex {
        Self { q: self.q - rhs.q, r: self.r - rhs.r }
    }
}

struct HexCube {
    pub q: i32,
    pub r: i32,
    pub s: i32
}

impl HexCube {
    pub fn to_hex(&self) -> Hex {
        Hex {
            q: self.q,
            r: self.r
        }
    }

    pub fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::Rot0 => HexCube { q: self.q, r: self.r, s: self.s },
            Rotation::Rot60Cw => HexCube { q: -self.r, r: -self.s, s: -self.q },
            Rotation::Rot60Ccw => HexCube { q: -self.s, r: -self.q, s: -self.r },
            Rotation::Rot120Cw => HexCube { q: self.s, r: self.q, s: self.r },
            Rotation::Rot120Ccw => HexCube { q: self.r, r: self.s, s: self.q },
            Rotation::Rot180 => HexCube { q: -self.q, r: -self.r, s: -self.s }
        }
    }

    pub fn from_fraction(frac_q: f32, frac_r: f32, frac_s: f32) -> Self {
        let mut q = frac_q.round() as i32;
        let mut r = frac_r.round() as i32;
        let mut s = frac_s.round() as i32;

        let q_diff = (q as f32 - frac_q).round();
        let r_diff = (r as f32 - frac_r).round();
        let s_diff = (s as f32 - frac_s).round();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r-s
        } else if r_diff > s_diff {
            r = -q-s
        } else {
            s = -q-r
        }

        Self { q, r, s }
    }
}