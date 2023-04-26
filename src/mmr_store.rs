use crate::{vec::Vec, Result};
use core::marker::PhantomData;

#[derive(Default)]
pub struct MMRBatch<Elem, Store, ForkUnique> {
    memory_batch: Vec<(u64, Vec<Elem>)>,
    store: Store,
    _marker: PhantomData<ForkUnique>,
}

impl<Elem, Store, ForkUnique> MMRBatch<Elem, Store, ForkUnique> {
    pub fn new(store: Store) -> Self {
        MMRBatch {
            memory_batch: Vec::new(),
            store,
            _marker: PhantomData,
        }
    }

    pub fn append(&mut self, pos: u64, elems: Vec<Elem>) {
        self.memory_batch.push((pos, elems));
    }

    pub fn store(&self) -> &Store {
        &self.store
    }
}

impl<Elem: Clone, Store: MMRStoreReadOps<Elem>, ForkUnique> MMRBatch<Elem, Store, ForkUnique> {
    pub fn get_elem(&self, pos: u64) -> Result<Option<Elem>> {
        for (start_pos, elems) in self.memory_batch.iter().rev() {
            if pos < *start_pos {
                continue;
            } else if pos < start_pos + elems.len() as u64 {
                return Ok(elems.get((pos - start_pos) as usize).cloned());
            } else {
                break;
            }
        }
        self.store.get_elem(pos)
    }
}

impl<Elem, Store: MMRStoreWriteOps<Elem, ForkUnique>, ForkUnique: Clone>
    MMRBatch<Elem, Store, ForkUnique>
{
    pub fn commit(&mut self, unique: &ForkUnique) -> Result<()> {
        for (pos, elems) in self.memory_batch.drain(..) {
            self.store.append(pos, elems, unique.clone())?;
        }
        Ok(())
    }
}

impl<Elem, Store, ForkUnique> IntoIterator for MMRBatch<Elem, Store, ForkUnique> {
    type Item = (u64, Vec<Elem>);
    type IntoIter = crate::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.memory_batch.into_iter()
    }
}

pub trait MMRStoreReadOps<Elem> {
    fn get_elem(&self, pos: u64) -> Result<Option<Elem>>;
}

pub trait MMRStoreWriteOps<Elem, ForkUnique> {
    fn append(&mut self, pos: u64, elems: Vec<Elem>, unique: ForkUnique) -> Result<()>;
}
