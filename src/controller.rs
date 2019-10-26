//! Controller definitions.
use fixedbitset::FixedBitSet;
use fxhash::FxHashMap;

use cubism_core::Model;

mod eye_blink;
pub use self::eye_blink::EyeBlink;

/// Priorities used by the standard controllers of this crate.
pub mod default_priorities {
    /// The eyeblink controller priority.
    pub const EYE_BLINK: usize = 100;
}

/// The controller trait. A controller is an object that modifies a models
/// parameter and part values in a defined fashion.
pub trait Controller {
    /// Run the controller on the passed [`Model`].
    fn update_parameters(&mut self, model: &mut Model, delta: f32);
    /// The execution priority of this controller. The smallest value has the
    /// highest priority.
    fn priority(&self) -> usize;
}

/// A simple wrapper around a vec that returns the index of newly
/// pushed/inserted elements and allows holes to exist.
struct SimpleSlab {
    buf: Vec<Option<Box<dyn Controller>>>,
    last_free: usize,
}

impl SimpleSlab {
    fn push(&mut self, c: Box<dyn Controller + 'static>) -> usize {
        let len = self.buf.len();
        if len <= self.last_free {
            let ret = self.buf.len();
            self.buf.push(Some(c));
            self.last_free = len;
            ret
        } else {
            let ret = self.last_free;
            self.buf[self.last_free].replace(c);
            self.last_free = self.buf[self.last_free..]
                .iter()
                .position(|c| c.is_none())
                .map(|pos| pos + self.last_free)
                .unwrap_or(len);
            ret
        }
    }

    fn take(&mut self, idx: usize) -> Option<Box<dyn Controller + 'static>> {
        if idx < self.last_free {
            self.last_free = idx;
        }
        self.buf.get_mut(idx).and_then(Option::take)
    }

    fn get(&self, idx: usize) -> Option<&dyn Controller> {
        self.buf.get(idx).and_then(|c| c.as_ref().map(|c| &**c))
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut (dyn Controller + 'static)> {
        self.buf
            .get_mut(idx)
            .and_then(|c| c.as_mut().map(|c| &mut **c))
    }

    fn iter(&self) -> impl Iterator<Item = &dyn Controller> {
        self.buf
            .iter()
            .flat_map(|con| con.as_ref().map(|con| &**con))
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut (dyn Controller + 'static)> {
        self.buf
            .iter_mut()
            .flat_map(|con| con.as_mut().map(|con| &mut **con))
    }
}

/// A ControllerMap maps names to [`Controller`]s and tracking their enabled
/// state.
pub struct ControllerMap {
    controllers: SimpleSlab,
    name_map: FxHashMap<String, usize>,
    enabled: FixedBitSet,
}

impl ControllerMap {
    /// Creates a new empty controller map.
    pub fn new() -> Self {
        ControllerMap {
            controllers: SimpleSlab {
                buf: Vec::new(),
                last_free: 0,
            },
            name_map: FxHashMap::with_hasher(Default::default()),
            enabled: FixedBitSet::with_capacity(0),
        }
    }

    /// Inserts a new controller into the map with the name, unregistering
    /// and returning back the previous controller under the same name if it
    /// exists.
    pub fn insert<C: Controller + 'static>(
        &mut self,
        name: impl Into<String>,
        controller: C,
    ) -> Option<Box<dyn Controller>> {
        let index = self.controllers.push(Box::new(controller));
        if self.enabled.len() <= index {
            self.enabled.grow(self.enabled.len() + 1);
        }
        self.enabled.set(index, true);
        self.name_map
            .insert(name.into(), index)
            .and_then(|old| self.controllers.take(old))
    }

    /// Removes and returns the controller corresponding to the name.
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn Controller>> {
        self.name_map
            .remove(name)
            .and_then(|old| self.controllers.take(old))
    }

    /// Returns a reference to the controller corresponding to the name.
    pub fn get(&self, name: &str) -> Option<&dyn Controller> {
        self.name_map
            .get(name)
            .and_then(|idx| self.controllers.get(*idx))
    }

    /// Returns a mutable reference to the controller corresponding to the name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut (dyn Controller + 'static)> {
        self.name_map
            .get(name)
            .copied()
            .and_then(move |idx| self.controllers.get_mut(idx))
    }

    /// Enables or disables the controller corresponding to the name.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(&pos) = self.name_map.get(name) {
            self.enabled.set(pos, enabled);
        }
    }

    /// Returns true if the controller corresponding to the name is enabled,
    /// false otherwise.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.name_map
            .get(name)
            .map(|idx| self.enabled[*idx])
            .unwrap_or(false)
    }

    /// Returns an iterator over the enabled controllers.
    pub fn enabled_controllers<'this>(
        &'this self,
    ) -> impl Iterator<Item = &'this dyn Controller> + 'this {
        self.controllers
            .iter()
            .zip(BitIter::from_bitset(&self.enabled))
            .filter_map(|(con, enab)| if enab { Some(con) } else { None })
    }

    /// Returns an iterator over the enabled controllers.
    pub fn enabled_controllers_mut<'this>(
        &'this mut self,
    ) -> impl Iterator<Item = &'this mut (dyn Controller + 'static)> + 'this {
        self.controllers
            .iter_mut()
            .zip(BitIter::from_bitset(&self.enabled))
            .filter_map(|(con, enab)| if enab { Some(con) } else { None })
    }

    /// Calls [`update_parameters`] on every enabled controller in the order of
    /// their priority.
    pub fn update_enabled_controllers(&mut self, model: &mut Model, delta: f32) {
        let mut controllers = self.enabled_controllers_mut().collect::<Vec<_>>();
        controllers.sort_unstable_by_key(|c| c.priority());
        for con in controllers {
            con.update_parameters(model, delta);
        }
    }
}

impl Default for ControllerMap {
    fn default() -> Self {
        Self::new()
    }
}

const BITS: usize = 32;

struct BitIter<'a> {
    last_block_len: usize,
    current_block_shift: usize,
    remaining_blocks: &'a [u32],
    current_block: u32,
}

impl<'a> BitIter<'a> {
    fn from_bitset(set: &'a FixedBitSet) -> Self {
        let (&current_block, remaining_blocks) = set.as_slice().split_first().unwrap_or((&0, &[]));
        BitIter {
            last_block_len: set.len() % BITS,
            current_block_shift: 0,
            current_block,
            remaining_blocks,
        }
    }
}

impl<'a> Iterator for BitIter<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_blocks.is_empty() && self.current_block_shift == self.last_block_len {
            return None;
        } else if self.current_block_shift == BITS {
            match self.remaining_blocks.split_first() {
                Some((&next_block, rest)) => {
                    self.remaining_blocks = rest;
                    self.current_block_shift = 0;
                    self.current_block = next_block;
                },
                None => unreachable!(),
            }
        }
        let ret = (self.current_block & 1) == 1;
        self.current_block >>= 1;
        self.current_block_shift += 1;
        Some(ret)
    }
}
