- Make `Mapper::map_to` and `Mapper::identity_map` unsafe because it is possible to break memory safety by passing invalid arguments.

# 0.3.6

- Add a `SIZE` constant to the `Page` type
- Add two interrupt tests to the `testing` sub-crate
