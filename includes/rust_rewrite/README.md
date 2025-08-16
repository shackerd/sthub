# `mod_rewrite`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/mod_rewrite?label=latest)](https://crates.io/crates/mod_rewrite)
[![Documentation](https://docs.rs/mod_rewrite/badge.svg?version=0.2.0)](https://docs.rs/mod_rewrite/0.2.0)
![Version](https://img.shields.io/badge/rustc-1.72+-ab6000.svg)
![License](https://img.shields.io/crates/l/mod_rewrite.svg)
<br />
[![dependency status](https://deps.rs/crate/mod_rewrite/0.2.0/status.svg)](https://deps.rs/crate/mod_rewrite/0.2.0)
[![Download](https://img.shields.io/crates/d/mod_rewrite.svg)](https://crates.io/crates/mod_rewrite)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Dynamic routing rewrite library inspired by apache
[`mod_rewrite`](https://httpd.apache.org/docs/current/mod/mod_rewrite.html).

## Examples

```rust
use mod_rewrite::Engine;

let mut engine = Engine::default();
engine.add_rules(r#"
  RewriteRule /file/(.*)     /tmp/$1      [L]
  RewriteRule /redirect/(.*) /location/$1 [R=302]
  RewriteRule /blocked/(.*)  -            [F]
"#).expect("failed to process rules");

let uri = "http://localhost/file/my/document.txt";
let result = engine.rewrite(uri).unwrap();
println!("{result:?}");
```

<!-- cargo-rdme end -->
