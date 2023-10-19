use std::{
    ops::{
        Index,
        IndexMut,
    },
    marker::PhantomData,
};


pub trait Key {
    fn from_id(id: usize)->Self;
    fn get_id(&self)->usize;
}


/// A simple keyed list of data. Removal is not possible. Basically a `Vec<T>`, but avoids the
/// hassle of using raw `usize` to index a `Vec`
pub struct KeyedVec<K: Key, T> {
    inner: Vec<T>,
    _phantom: PhantomData<K>,
}
impl<K: Key, T> KeyedVec<K, T> {
    pub fn new()->Self {
        KeyedVec {
            inner: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn insert(&mut self, data: T)->K {
        let key = K::from_id(self.inner.len());
        self.inner.push(data);
        return key;
    }

    pub fn get(&self, key: K)->&T {
        let id = key.get_id();
        assert!(id < self.inner.len());
        return &self.inner[id];
    }

    pub fn get_mut(&mut self, key: K)->&mut T {
        let id = key.get_id();
        assert!(id < self.inner.len());
        return &mut self.inner[id];
    }
}
impl<K: Key, T> Index<K> for KeyedVec<K, T> {
    type Output = T;
    #[inline]
    fn index(&self, key: K)->&T {
        self.get(key)
    }
}
impl<K: Key, T> IndexMut<K> for KeyedVec<K, T> {
    #[inline]
    fn index_mut(&mut self, key: K)->&mut T {
        self.get_mut(key)
    }
}

/// A simple map of key:value that reuses old keys that are removed. DOES NOT solve the ABA
/// problem. The user (me) assumes all responsibility to ensure all keys are used properly.
pub struct SlotMap<K: Key, T> {
    inner: Vec<Option<T>>,
    free: Vec<K>,
}
impl<K: Key, T> SlotMap<K, T> {
    pub fn new()->Self {
        SlotMap {
            inner: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn insert(&mut self, data: T)->K {
        let key = self.free.pop().unwrap_or(K::from_id(self.inner.len()));
        self.inner[key.get_id()] = Some(data);

        return key;
    }

    /// assumes the key is valid
    pub fn get(&self, key: K)->&T {
        let id = key.get_id();
        assert!(id < self.inner.len());
        assert!(self.inner[id].is_some());

        return self.inner[id].as_ref().unwrap();
    }

    /// assumes the key is valid
    pub fn get_mut(&mut self, key: K)->&mut T {
        let id = key.get_id();
        assert!(id < self.inner.len());
        assert!(self.inner[id].is_some());

        return self.inner[id].as_mut().unwrap();
    }

    /// assumes the key is valid
    pub fn remove(&mut self, key: K)->T {
        let id = key.get_id();
        assert!(id < self.inner.len());
        assert!(self.inner[id].is_some());

        return self.inner[id].take().unwrap();
    }
}
impl<K: Key, T> Index<K> for SlotMap<K, T> {
    type Output = T;
    #[inline]
    fn index(&self, key: K)->&T {
        self.get(key)
    }
}
impl<K: Key, T> IndexMut<K> for SlotMap<K, T> {
    #[inline]
    fn index_mut(&mut self, key: K)->&mut T {
        self.get_mut(key)
    }
}
