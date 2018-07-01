use hibitset::BitSetLike;
use specs::storage::{MaskedStorage, TryDefault, UnprotectedStorage};
use specs::world::Index;
use specs::{Component, Join, Storage};
use std::marker::PhantomData;
use std::mem;
use std::ops::DerefMut;

pub trait Retained<C> {
    fn retained(&mut self) -> Vec<C>;
}

impl<'e, T, D> Retained<T> for Storage<'e, T, D>
where
    T: Component,
    T::Storage: Retained<T>,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    fn retained(&mut self) -> Vec<T> {
        unsafe {
            self.open().1.retained()
        }
    }
}

pub struct RetainedStorage<C, T = UnprotectedStorage<C>> {
    retained: Vec<C>,
    storage: T,
    phantom: PhantomData<C>,
}

impl<C, T> Default for RetainedStorage<C, T>
where
    T: TryDefault,
{
    fn default() -> Self {
        RetainedStorage {
            retained: vec![],
            storage: T::try_default().unwrap(),
            phantom: PhantomData,
        }
    }
}

impl<C, T> Retained<C> for RetainedStorage<C, T> {
    fn retained(&mut self) -> Vec<C> {
        mem::replace(&mut self.retained, vec![])
    }
}

impl<C: Clone, T: UnprotectedStorage<C>> UnprotectedStorage<C> for RetainedStorage<C, T> {
    unsafe fn clean<B>(&mut self, has: B)
    where
        B: BitSetLike,
    {
        self.storage.clean(has)
    }

    unsafe fn get(&self, id: Index) -> &C {
        self.storage.get(id)
    }

    unsafe fn get_mut(&mut self, id: Index) -> &mut C {
        self.storage.get_mut(id)
    }

    unsafe fn insert(&mut self, id: Index, comp: C) {
        self.storage.insert(id, comp);
    }

    unsafe fn remove(&mut self, id: Index) -> C {
        let comp = self.storage.remove(id);
        self.retained.push(comp.clone());
        comp
    }
}
