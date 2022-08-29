# JSnom

[![crates.io](https://img.shields.io/crates/v/jsnom)](https://crates.io/crates/jsnom)
[![documentation](https://img.shields.io/docsrs/jsnom/latest)](https://docs.rs/newdoku/latest/jsnom/)
[![license](https://img.shields.io/crates/l/jsnom)](https://crates.io/crates/jsnom)
[![stargazers](https://img.shields.io/github/stars/Piturnah/jsnom?style=social)](https://github.com/Piturnah/jsnom/stargazers)

JSON parser, with a focus on small size and ergonomics.

## Example
<details>
<summary>
Click to show Cargo.toml.
</summary>

```toml
[dependencies]
jsnom = "1.0"
```

</details>

```rust
use jsnom::JsonValue;

fn main() {
    assert_eq!(
        JsonValue::from_str("[null, null, true]"),
        Ok(JsonValue::Array(vec![
            JsonValue::Null,
            JsonValue::Null,
            JsonValue::Bool(true)
        ]))
    );
}
```

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
