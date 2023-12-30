use std::{sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}, cmp::Ordering, hash::{Hash, Hasher}};

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
    /// Creates a new `ACell`.
    ///
    /// - `data` - The initial value of the `ACell`.
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

