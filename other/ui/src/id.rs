use crc32fast::Hasher;

#[derive(Clone, Eq, Copy, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Id(u32);

impl Id {
    pub fn root() -> Id {
        Id(0)
    }

    pub fn new(data: &str, seed: Id) -> Self {
        let mut hasher = Hasher::new_with_initial(seed.0);
        hasher.update(&[0]);
        hasher.update(data.as_bytes());
        let checksum = hasher.finalize();
        Id(checksum)
    }
}

#[test]
fn test_id() {
    let i0 = Id::root();
    let i1 = Id::new("hello", i0);
    let i2 = Id::new("world", i0);
    assert_ne!(i0, i1);
    assert_ne!(i1, i2);
}

#[test]
fn test_id_empty() {
    let i0 = Id::root();
    let i1 = Id::new("", i0);
    assert_ne!(i0, i1);
}

#[test]
fn test_id_nest() {
    let i0 = Id::root();
    let i1 = Id::new("helloworld", i0);
    let i2 = Id::new("hello", Id::new("world", i0));
    let i3 = Id::new("world", Id::new("hello", i0));
    assert_ne!(i1, i2);
    assert_ne!(i1, i3);
}
