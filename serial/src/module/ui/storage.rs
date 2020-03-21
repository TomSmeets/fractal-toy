use serde::{Deserialize, Serialize};

use std::{any::Any, collections::HashMap};

#[derive(Serialize, Deserialize)]
pub struct Storage {
    #[serde(skip)]
    pub values: HashMap<Vec<String>, Box<dyn Any>>,
    pub id_stack: Vec<String>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            values: HashMap::new(),
            id_stack: Vec::new(),
        }
    }

    #[inline]
    pub fn get_mut<'a, T>(&mut self, key: &str) -> Option<&mut T>
    where
        T: Any + Serialize + Deserialize<'a>,
    {
        let mut i2 = self.id_stack.clone();
        i2.push(key.to_string());
        let v = self.values.get_mut(&i2)?;
        let v = v
            .downcast_mut()
            .expect("lookup for value of a different type");
        Some(v)
    }

    pub fn get_mut_or<'a, T, F: FnOnce() -> T>(&mut self, key: &str, v: F) -> &mut T
    where
        T: Any + Serialize + Deserialize<'a>,
    {
        let v = self.get_mut(key);
        match v {
            Some(v) => v,
            None => panic!(""),
        }
    }

    pub fn push_id(&mut self, id: &str) {
        self.id_stack.push(id.to_string());
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }
}
