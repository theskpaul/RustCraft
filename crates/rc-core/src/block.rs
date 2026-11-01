#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct BlockId(pub u16);

impl BlockId {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const GRASS: Self = Self(3);

    #[inline]
    pub fn is_air(self) -> bool {
        self.0 == 0
    }
}
