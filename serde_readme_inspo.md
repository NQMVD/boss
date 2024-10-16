# Serde JSON &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Rustc Version 1.36+]][rustc]

[Build Status]: https://img.shields.io/github/actions/workflow/status/serde-rs/json/ci.yml?branch=master
[actions]: https://github.com/serde-rs/json/actions?query=branch%3Amaster
[Latest Version]: https://img.shields.io/crates/v/serde_json.svg
[crates.io]: https://crates.io/crates/serde\_json
[Rustc Version 1.36+]: https://img.shields.io/badge/rustc-1.36+-lightgray.svg
[rustc]: https://blog.rust-lang.org/2019/07/04/Rust-1.36.0.html

- [JSON API documentation](https://docs.rs/serde_json)
- [Serde API documentation](https://docs.rs/serde)
- [Detailed documentation about Serde](https://serde.rs/)
- [Setting up `#[derive(Serialize, Deserialize)]`](https://serde.rs/derive.html)
- [Release notes](https://github.com/serde-rs/json/releases)

```rust
enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

A string of JSON data can be parsed into a `serde_json::Value` by the
[`serde_json::from_str`][from_str] function. There is also
[`from_slice`][from_slice] for parsing from a byte slice `&[u8]` and
[`from_reader`][from_reader] for parsing from any `io::Read` like a File or a
TCP stream.

<div align="right">
<a href="https://play.rust-lang.org/?edition=2018&gist=d69d8e3156d4bb81c4461b60b772ab72" target="_blank">
<img align="center" width="85" src="https://raw.githubusercontent.com/serde-rs/serde-rs.github.io/master/img/runtab.png">
</a>
</div>

```rust
use serde_json::{Result, Value};

fn untyped_example() -> Result<()> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(data)?;

    // Access parts of the data by indexing with square brackets.
    println!("Please call {} at the number {}", v["name"], v["phones"][0]);

    Ok(())
}
```


## Getting help

Serde is one of the most widely used Rust libraries, so any place that
Rustaceans congregate will be able to help you out. For chat, consider trying
the [#rust-questions] or [#rust-beginners] channels of the unofficial community
Discord (invite: <https://discord.gg/rust-lang-community>), the [#rust-usage] or
[#beginners] channels of the official Rust Project Discord (invite:
<https://discord.gg/rust-lang>), or the [#general][zulip] stream in Zulip. For
asynchronous, consider the [\[rust\] tag on StackOverflow][stackoverflow], the
[/r/rust] subreddit which has a pinned weekly easy questions post, or the Rust
[Discourse forum][discourse]. It's acceptable to file a support issue in this
repo, but they tend not to get as many eyes as any of the above and may get
closed without a response after some time.

[#rust-questions]: https://discord.com/channels/273534239310479360/274215136414400513
[#rust-beginners]: https://discord.com/channels/273534239310479360/273541522815713281
[#rust-usage]: https://discord.com/channels/442252698964721669/443150878111694848
[#beginners]: https://discord.com/channels/442252698964721669/448238009733742612
[zulip]: https://rust-lang.zulipchat.com/#narrow/stream/122651-general
[stackoverflow]: https://stackoverflow.com/questions/tagged/rust
[/r/rust]: https://www.reddit.com/r/rust
[discourse]: https://users.rust-lang.org

## No-std support

As long as there is a memory allocator, it is possible to use serde_json without
the rest of the Rust standard library. Disable the default "std" feature and
enable the "alloc" feature:

```toml
[dependencies]
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
```

For JSON support in Serde without a memory allocator, please see the
[`serde-json-core`] crate.

[`serde-json-core`]: https://github.com/rust-embedded-community/serde-json-core

[value]: https://docs.rs/serde_json/1/serde_json/value/enum.Value.html
[from_str]: https://docs.rs/serde_json/1/serde_json/de/fn.from_str.html
[from_slice]: https://docs.rs/serde_json/1/serde_json/de/fn.from_slice.html
[from_reader]: https://docs.rs/serde_json/1/serde_json/de/fn.from_reader.html
[to_string]: https://docs.rs/serde_json/1/serde_json/ser/fn.to_string.html
[to_vec]: https://docs.rs/serde_json/1/serde_json/ser/fn.to_vec.html
[to_writer]: https://docs.rs/serde_json/1/serde_json/ser/fn.to_writer.html
[macro]: https://docs.rs/serde_json/1/serde_json/macro.json.html
