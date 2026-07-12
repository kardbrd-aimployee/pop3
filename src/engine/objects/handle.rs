/// Stable runtime handle into the object pool.
///
/// Runtime handles include a generation so a destroyed reference cannot
/// resolve to a different object after its slot is recycled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectHandle {
    slot: u16,
    generation: u32,
}

impl ObjectHandle {
    pub(crate) const fn new(slot: u16, generation: u32) -> Self {
        Self { slot, generation }
    }

    pub const fn slot(self) -> u16 {
        self.slot
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }

    pub(crate) const fn index(self) -> usize {
        self.slot as usize
    }
}

impl std::fmt::Display for ObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.slot, self.generation)
    }
}
