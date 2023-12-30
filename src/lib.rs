//! A module containing easier to use thread-safe types for the creation of larger thread safe
//! systems.
//!
//! ## Included Types
//! - `SyncCell` - A replacement for `std::cell::RefCell` and `std::cell::Cell` with an easier to
//! use API than `std::sync::RwLock`.
//! - `HeldSyncCell` - A cell that maintains a previous value until the `update` method is called
//! at which point any changes to the value are applied.

use std::{sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}, cmp::Ordering, hash::{Hash, Hasher}, mem::swap};

/// A mutable memory location that can be modified safely from multiple threads.
/// This structure is similar to `std::cell::Cell` or `std::cell::RefCell`
/// while being thread-safe.
/// It functions as a thin wrapper around `std::sync::RwLock` while assuming that poisoned locks
/// indicate an unrecoverable error. This makes it more ergonomic to use than `RwLock` at the cost
/// of some stability.
/// 
/// # As a `Cell` replacement.
/// `SyncCell` can be used to replace the functionality of a `std::cell::Cell` in contexts where
/// data need to mutably accessed across multiple threads.
/// ## Using `std::cell::Cell`
/// ```
/// use std::cell::Cell;
///
/// let cell = Cell::new(0);
///
/// cell.set(1);
///
/// println!("{}", cell.get());
/// ```
/// ## Using `sync_cell::SyncCell`
/// ```
/// use sync_cell::SyncCell;
///
/// let cell = SyncCell::new(0);
///
/// cell.set(1);
///
/// println!("{}", cell.get());
/// ```
///
/// # As a `RefCell` replacement.
/// `SyncCell` can also be used to replace usages of `RefCell`.
/// ## Using `std::cell::RefCell`
/// ```
/// use std::cell::RefCell;
///
/// let cell = RefCell::new((0, 1));
///
/// let borrowed = cell.borrow();
/// println!("{}", borrowed.0);
/// drop(borrowed);
///
/// let mut mutable_borrow = cell.borrow_mut();
/// mutable_borrow.1 = 2;
/// drop(mutable_borrow);
///
/// let borrowed = cell.borrow();
/// println!("{:?}", borrowed);
/// ```
/// ## Using `sync_cell::SyncCell`
/// ```
/// use sync_cell::SyncCell;
///
/// let cell = SyncCell::new((0, 1));
///
/// let borrowed = cell.borrow();
/// println!("{}", borrowed.0);
/// drop(borrowed);
///
/// let mut mutable_borrow = cell.borrow_mut();
/// mutable_borrow.1 = 2;
/// drop(mutable_borrow);
///
/// let borrowed = cell.borrow();
/// println!("{:?}", borrowed);
/// ```
///
/// # Panicking
/// Unlike `std::sync::RwLock`, `SyncCell` will panic rather than return an error when the lock
/// becomes poisoned.
#[derive(Debug)]
pub struct SyncCell<T: ?Sized> {
    /// The internal lock holding the data of this cell.
    data: RwLock<T>,
}

impl <T> SyncCell<T> {
    /// Creates a new `SyncCell`.
    ///
    /// - `data` - The initial value of the `SyncCell`.
    pub const fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data)
        }
    }

    /// Sets the value contained in this cell.
    ///
    /// - `value` - The new value of the cell.
    ///
    /// # Panicking
    /// This method will panic if the lock becomes poisoned.
    pub fn set(&self, value: T) {
        match self.data.write() {
            Ok(mut data) => *data = value,
            Err(err) => panic!("Failed to set cell value. Lock was poisoned: {}", err),
        }
    }

    /// Retrieves the inner value stored in this `SyncCell`. 
    ///
    /// # Panicking
    /// This method will panic if the lock becomes poisoned.
    pub fn into_inner(self) -> T {
        match self.data.into_inner() {
            Ok(data) => data,
            Err(err) => panic!("Failed to get cell value. Lock was poisoned: {}", err),
        }
    }

    /// Replaces the internal value contained in this cell.
    /// The previous value is returned.
    ///
    /// - `value` - The new value of the cell.
    ///
    /// # Panicking
    /// This method will panic if the lock becomes poisoned.
    pub fn replace(&self, mut value: T) -> T {
        match self.data.write() {
            Ok(mut data) => {
                swap(&mut *data, &mut value);
                value
            },
            Err(err) => panic!("Failed to set cell value. Lock was poisoned: {}", err),
        }
    }
}

impl <T: ?Sized> SyncCell<T> {
    /// Borrows a immutable reference to the data stored in this cell.
    ///
    /// # Panicking
    /// This method will panic if the lock becomes poisoned.
    pub fn borrow(&self) -> RwLockReadGuard<T> {
        match self.data.read() {
            Ok(data) => data,
            Err(err) => panic!("Failed to get cell value. Lock was poisoned: {}", err),
        }
    }
    
    /// Borrows a mutable reference to the data stored in this cell.
    ///
    /// # Panicking
    /// This method will panic if the lock becomes poisoned.
    pub fn borrow_mut(&self) -> RwLockWriteGuard<T> {
        match self.data.write() {
            Ok(data) => data,
            Err(err) => panic!("Failed to get cell value. Lock was poisoned: {}", err),
        }
    }
}

impl <T: Clone> SyncCell<T> {
    /// Gets the value contained in this cell.
    ///
    /// # Panicking
    /// This method will panic if the lock becomes poisoned.
    pub fn get(&self) -> T {
        match self.data.read() {
            Ok(data) => data.clone(),
            Err(err) => panic!("Failed to get cell value. Lock was poisoned: {}", err),
        }
    }
}

impl <T: Clone> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl <T: Default> Default for SyncCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl <T: PartialEq + ?Sized> PartialEq for SyncCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().eq(&*other.borrow())
    }
}

impl <T: Eq + ?Sized> Eq for SyncCell<T> {
}

impl <T: PartialOrd + ?Sized> PartialOrd for SyncCell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }
}

impl <T: Ord + ?Sized> Ord for SyncCell<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrow().cmp(&*other.borrow())
    }
}

impl <T: Hash + ?Sized> Hash for SyncCell<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().hash(state)
    }
}

impl <T> From<T> for SyncCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// A cell that holds a value until any changes made are applied by use of the `update` method.
/// Getting the value or obtaining a reference to the value in this cell will return the value
/// immediately following the last call to `update`. This allows for mutably altering a value while
/// keeping a reference for a short amount of time to the old value.
/// This is useful when you want a method in a structure to be able to modify the structure it is
/// being called from such as when changing the scene in a game engine.
///
/// # Usage
/// ```
/// use sync_cell::HeldSyncCell;
///
/// let cell = HeldSyncCell::new(0);
///
/// // Set the next value of the cell.
/// cell.set(1);
///
/// // Cell continues to hold a value of 0 until the `update` method is called.
/// assert_eq!(0, cell.get());
///
/// cell.update();
/// assert_eq!(1, cell.get());
/// ```
pub struct HeldSyncCell<T> {
    /// The current value that is made available.
    current_value: SyncCell<T>,
    /// The value to use next.
    next_value: SyncCell<Option<T>>,
}

impl <T> HeldSyncCell<T> {
    /// Creates a new `HeldSyncCell`.
    ///
    /// - `data` - The initial value of the `HeldSyncCell`.
    pub const fn new(data: T) -> Self {
        Self {
            current_value: SyncCell::new(data),
            next_value: SyncCell::new(None),
        }
    }

    /// Sets the value contained in this cell.
    /// This value will only become available once the `update` method is called.
    /// 
    /// In the case that multiple threads call this method simultaniously,
    /// the order in which the calls are processed is not defined. However, the final result will
    /// be the value specified by one of the method calls.
    ///
    /// - `value` - The new value of the cell.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn set(&self, value: T) {
        self.next_value.set(Some(value))
    }

    /// Retrieves the inner value stored in this `HeldSyncCell`. 
    /// This will return the most up-to-date value even if `update` has not been called.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn into_inner(self) -> T {
        self.next_value.into_inner()
            .unwrap_or(self.current_value.into_inner())
    }

    /// Borrows a immutable reference to the data stored in this cell.
    /// This is a reference to the current value of the cell.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn borrow(&self) -> RwLockReadGuard<T> {
        self.current_value.borrow()
    }
    
    /// Borrows a mutable reference to the data stored in this cell.
    /// This is a reference to the current value of the cell not the incoming value. Any changes to
    /// the value will update the current value.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn borrow_mut(&self) -> RwLockWriteGuard<T> {
        self.current_value.borrow_mut()
    }

    /// Checks if a new nalue is available that can be applied by calling `update`.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn has_update(&self) -> bool {
        self.next_value.borrow().is_some()
    }

    /// Updates the internal value of this cell.
    /// This involves replacing the current value with the incoming value if it is available.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn update(&self) {
        if let Some(next) = self.next_value.replace(None) {
            self.current_value.set(next);
        }
    } 
}

impl <T: Clone> HeldSyncCell<T> {
    /// Gets the value contained in this cell.
    ///
    /// # Panicking
    /// This method will panic if any of the locks become poisoned.
    pub fn get(&self) -> T {
        self.current_value.get()
    }
}

impl <T: Clone> Clone for HeldSyncCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl <T: Default> Default for HeldSyncCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl <T: PartialEq> PartialEq for HeldSyncCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().eq(&*other.borrow())
    }
}

impl <T: Eq> Eq for HeldSyncCell<T> {
}

impl <T: PartialOrd> PartialOrd for HeldSyncCell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }
}

impl <T: Ord> Ord for HeldSyncCell<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrow().cmp(&*other.borrow())
    }
}

impl <T: Hash> Hash for HeldSyncCell<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().hash(state)
    }
}

impl <T> From<T> for HeldSyncCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{thread, sync::Arc};

    use crate::{SyncCell, HeldSyncCell};

    #[test]
    pub fn test_sync_cell_new() {
        let _cell = SyncCell::new(1);
    } 

    #[test]
    pub fn test_sync_cell_set() {
        let cell = SyncCell::new(2);

        cell.set(3);

        assert_eq!(3, cell.get())
    }
    
    #[test]
    pub fn test_sync_cell_get() {
        let cell = SyncCell::new(4);

        assert_eq!(4, cell.get())
    }
    
    #[test]
    pub fn test_sync_cell_replace() {
        let cell = SyncCell::new(2);

        let old = cell.replace(3);
        
        assert_eq!(2, old);
        assert_eq!(3, cell.get())
    }
    
    #[test]
    #[should_panic]
    pub fn test_sync_cell_replace_poisoned() {
        let cell = Arc::new(SyncCell::new(4));

        let cell2 = cell.clone();

        let _ = thread::spawn(move || {
            let _borrow = cell2.borrow();

            panic!("Intentional panic.");
        }).join();

        let old = cell.replace(3);
        
        assert_ne!(2, old);
        assert_ne!(3, cell.get())
    }
    
    #[test]
    pub fn test_sync_cell_into_inner() {
        let cell = SyncCell::new(4);

        assert_eq!(4, cell.into_inner())
    }
    
    #[test]
    pub fn test_sync_cell_mutable_borrow() {
        let cell = SyncCell::new(4);

        let mut borrow = cell.borrow_mut();

        *borrow = 5;

        drop(borrow);

        assert_eq!(5, cell.get())
    }
    
    #[test]
    #[should_panic]
    pub fn test_sync_cell_mutable_borrow_poisoned() {
        let cell = Arc::new(SyncCell::new(4));

        let cell2 = cell.clone();

        let _ = thread::spawn(move || {
            let _borrow = cell2.borrow();

            panic!("Intentional panic.");
        }).join();

        let mut borrow = cell.borrow_mut();

        *borrow = 5;

        drop(borrow);

        assert_ne!(5, cell.get())
    }
    
    #[test]
    #[should_panic]
    pub fn test_sync_cell_get_poisoned() {
        let cell = Arc::new(SyncCell::new(4));

        let cell2 = cell.clone();

        let _ = thread::spawn(move || {
            let _borrow = cell2.borrow();

            panic!("Intentional panic.");
        }).join();

        assert_ne!(4, cell.get())
    }
    
    #[test]
    #[should_panic]
    pub fn test_sync_cell_set_poisoned() {
        let cell = Arc::new(SyncCell::new(4));

        let cell2 = cell.clone();

        let _ = thread::spawn(move || {
            let _borrow = cell2.borrow();

            panic!("Intentional panic.");
        }).join();

        cell.set(5);

        assert_ne!(5, cell.get());
    }

    #[test]
    pub fn test_held_sync_cell_new() {
        let _cell = HeldSyncCell::new(0);
    }
    
    #[test]
    pub fn test_held_sync_cell_get() {
        let cell = HeldSyncCell::new(1);

        assert_eq!(1, cell.get())
    }
    
    #[test]
    pub fn test_held_sync_cell_set_no_update() {
        let cell = HeldSyncCell::new(1);

        cell.set(2);

        assert_eq!(true, cell.has_update());
        assert_eq!(1, cell.get())
    }
    
    #[test]
    pub fn test_held_sync_cell_set_update() {
        let cell = HeldSyncCell::new(1);

        cell.set(2);
        cell.update();

        assert_eq!(false, cell.has_update());
        assert_eq!(2, cell.get())
    }
    
    #[test]
    pub fn test_held_sync_cell_set_double_update() {
        let cell = HeldSyncCell::new(1);

        cell.set(2);
        cell.update();
        cell.update();

        assert_eq!(false, cell.has_update());
        assert_eq!(2, cell.get())
    }

    #[test]
    pub fn test_held_sync_cell_no_set_update() {
        let cell = HeldSyncCell::new(1);

        cell.update();

        assert_eq!(false, cell.has_update());
        assert_eq!(1, cell.get())
    }

    #[test]
    pub fn test_held_sync_cell_no_set() {
        let cell = HeldSyncCell::new(1);

        assert_eq!(false, cell.has_update());
        assert_eq!(1, cell.get())
    }
    
    #[test]
    pub fn test_held_sync_cell_into_inner() {
        let cell = HeldSyncCell::new(4);

        assert_eq!(4, cell.into_inner())
    }
    
    #[test]
    pub fn test_held_sync_cell_set_into_inner() {
        let cell = HeldSyncCell::new(4);

        cell.set(5);

        assert_eq!(5, cell.into_inner())
    }
    
    #[test]
    pub fn test_held_sync_cell_set_update_into_inner() {
        let cell = HeldSyncCell::new(4);

        cell.set(5);
        cell.update();

        assert_eq!(5, cell.into_inner())
    }
    
    #[test]
    pub fn test_held_sync_cell_mutable_borrow() {
        let cell = HeldSyncCell::new(4);

        let mut borrow = cell.borrow_mut();

        *borrow = 5;

        drop(borrow);

        assert_eq!(5, cell.get())
    }
    
    #[test]
    pub fn test_held_sync_cell_mutable_borrow_set() {
        let cell = HeldSyncCell::new(4);

        let mut borrow = cell.borrow_mut();

        *borrow = 5;

        cell.set(6);

        drop(borrow);

        assert_eq!(5, cell.get());
        cell.update();
        assert_eq!(6, cell.get());
    }
}

