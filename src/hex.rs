pub struct Hex {
    pub q: i32,
    pub r: i32
}

pub enum Rotation {
    Rot0,
    Rot60Cw,
    Rot60Ccw,
    Rot120Cw,
    Rot120Ccw,
    Rot180,
}

impl Hex {
    fn to_cube(&self) -> HexCube {
        HexCube {
            q: self.q,
            r: self.r,
            s: -self.q - self.r
        }
    }

    pub fn rotate(&self, rotation: &Rotation) -> Self {
        self.to_cube().rotate(rotation).to_hex()
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
    
    pub fn rotate(&self, rotation: &Rotation) -> Self {
        match rotation {
            Rotation::Rot0 => HexCube { q: self.q, r: self.r, s: self.s },
            Rotation::Rot60Cw => HexCube { q: -self.r, r: -self.s, s: -self.q },
            Rotation::Rot60Ccw => HexCube { q: -self.s, r: -self.q, s: -self.r },
            Rotation::Rot120Cw => HexCube { q: self.s, r: self.q, s: self.r },
            Rotation::Rot120Ccw => HexCube { q: self.r, r: self.s, s: self.q },
            Rotation::Rot180 => HexCube { q: -self.q, r: -self.r, s: -self.s }
        }
    }
}