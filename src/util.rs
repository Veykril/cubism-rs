/// A simple wrapper around a vec that returns the index of newly
/// pushed/inserted elements and allows holes to exist.
pub struct SimpleSlab<T> {
    buf: Vec<Option<T>>,
    last_free: usize,
}

impl<T> SimpleSlab<T> {
    pub fn new() -> Self {
        SimpleSlab {
            buf: Vec::new(),
            last_free: 0,
        }
    }

    pub fn push(&mut self, t: T) -> usize {
        let len = self.buf.len();
        if len <= self.last_free {
            let ret = len;
            self.buf.push(Some(t));
            self.last_free = self.buf.len();
            ret
        } else {
            let ret = self.last_free;
            self.buf[self.last_free].replace(t);
            self.last_free = self.buf[self.last_free..]
                .iter()
                .position(Option::is_none)
                .map(|pos| pos + self.last_free)
                .unwrap_or(len);
            ret
        }
    }

    pub fn take(&mut self, idx: usize) -> Option<T> {
        if idx < self.last_free {
            self.last_free = idx;
        }
        self.buf.get_mut(idx).and_then(Option::take)
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.buf.get(idx).and_then(Option::as_ref)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.buf.get_mut(idx).and_then(Option::as_mut)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
        self.buf.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<T>> {
        self.buf.iter_mut()
    }
}

#[test]
fn simple_slab_push() {
    let mut slab = SimpleSlab::new();
    assert_eq!(0, slab.push(100));
    assert_eq!(1, slab.push(101));
    assert_eq!(slab.last_free, 2);
    assert_eq!(slab.buf.len(), 2);
}

#[test]
fn simple_slab_take() {
    let mut slab = SimpleSlab::new();
    assert_eq!(0, slab.push(100));
    assert_eq!(1, slab.push(101));
    assert_eq!(Some(100), slab.take(0));
    assert_eq!(slab.last_free, 0);
    assert_eq!(slab.buf.len(), 2);
}

#[test]
fn simple_slab_take_push() {
    let mut slab = SimpleSlab::new();
    assert_eq!(0, slab.push(100));
    assert_eq!(1, slab.push(101));
    assert_eq!(2, slab.push(102));
    assert_eq!(Some(101), slab.take(1));
    assert_eq!(slab.last_free, 1);
    assert_eq!(slab.buf.len(), 3);
    assert_eq!(1, slab.push(104));
}
