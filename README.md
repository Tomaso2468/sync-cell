# Sync Cell

A crate containing easier to use thread-safe types for the creation of larger thread safe systems.

## Included Types
- `SyncCell<T>` - A replacement for `std::cell::RefCell` and `std::cell::Cell` with an easier to use API than `std::sync::RwLock`.
- `HeldSyncCell<T>` - A cell that maintains a previous value until the `update` method is called at which point any changes to the value are applied.

## Documentation
You can read the documentation at https://docs.rs/sync-cell/0.2.0/sync_cell/

## Usage
To use `SyncCell` and `HeldSyncCell` add the following to the `[dependencies]` section of your `Cargo.toml`.
```toml
[dependencies]
sync-cell = "0.2.0"
```
