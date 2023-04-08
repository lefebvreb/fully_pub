# Fully Pub

[<img alt="github" src="https://img.shields.io/badge/github-lefebvreb/fully_pub-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/lefebvreb/fully_pub)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fully_pub.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fully_pub)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fully_pub-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fully_pub)

This crate exposes a single attribute macro that can be
used to alleviate some of the verbosity of marking every field of
a Rust item as [`pub`](https://doc.rust-lang.org/std/keyword.pub.html),
by doing it automatically.

```toml
[dependencies]
fully_pub = "0.1"
```

<br>

## Example

```rust
use fully_pub::fully_pub;

#[fully_pub]
struct User {
    name: String,
    age: i32,
    #[fully_pub(exclude)]
    secret: String,
}

#[fully_pub]
impl User {
    fn new(name: String, age: i32, secret: String) -> Self {
        Self { name, age, secret }
    }

    fn happy_birthday(&mut self) {
        self.age += 1;
    }

    #[fully_pub(exclude)]
    fn get_secret(&mut self) -> &str {
        &self.secret
    }
}
```

This macro works on nearly everything in the Rust programming language,
and can even be used on nested modules recursively, if needed.

See the documentation of the macro itself for more details.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>