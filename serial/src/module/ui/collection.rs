use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Item<T> {
    pub id: String,
    pub active: bool,
    pub value: T,
}

// TODO: clean up items that have been inactive for a whole frame
#[derive(Serialize, Deserialize)]
pub struct Collection<T> {
    pub index: usize,
    pub content: Vec<Item<T>>,
}

impl<T> Default for Collection<T> {
    fn default() -> Self {
        Collection::new()
    }
}

impl<T> Collection<T> {
    pub fn new() -> Self {
        Collection {
            index: 0,
            content: Vec::new(),
        }
    }

    pub fn item_index(&mut self, id: &str) -> Option<usize> {
        // we always start at the current index with searching
        // the item we are searching is almost always the next one
        // TODO: figure out if this matters for performance
        // TODO: if we do index+1 we have to mod it with the content length
        let (pre, post) = self.content.split_at(self.index);

        for (offset, e) in post.iter().enumerate() {
            if e.id == id {
                return Some(self.index + offset);
            }
        }

        for (offset, e) in pre.iter().enumerate() {
            if e.id == id {
                return Some(offset);
            }
        }
        return None;
    }

    pub fn begin(&mut self) {
        for i in self.content.iter_mut() {
            i.active = false;
        }
    }

    pub fn item<F: FnOnce() -> T>(&mut self, id: &str, def: F) -> &mut T {
        let idx = self.item_index(id);
        let idx = match idx {
            Some(i) => i,
            None => {
                let def = def();
                self.content.push(Item {
                    id: id.to_string(),
                    active: true,
                    value: def,
                });
                self.content.len() - 1
            },
        };

        self.index = idx;

        let item = &mut self.content[idx];
        item.active = true;
        &mut item.value
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.content
            .iter()
            .filter(|i| i.active)
            .map(|i| (i.id.as_ref(), &i.value))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut T)> {
        self.content
            .iter_mut()
            .filter(|i| i.active)
            .map(|i| (i.id.as_ref(), &mut i.value))
    }
}
