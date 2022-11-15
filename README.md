tracing-chrome
======

[![Crates.io](https://img.shields.io/crates/v/tracing-chrome)](https://crates.io/crates/tracing-chrome)
[![Documentation](https://docs.rs/tracing-chrome/badge.svg)](https://docs.rs/tracing-chrome/)
![GitHub](https://img.shields.io/github/license/Antigroup/tracing-chrome)
![CI](https://github.com/thoren-d/tracing-chrome/workflows/CI/badge.svg?branch=develop)

# Overview

tracing-chrome is a Layer for [tracing-subscriber](https://crates.io/crates/tracing-subscriber) that outputs traces in Chrome's trace viewer format that can be viewed with `chrome://tracing` or [ui.perfetto.dev](https://ui.perfetto.dev).

# Usage

Add this near the beginning of `main`:
```rust
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{registry::Registry, prelude::*};

let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
tracing_subscriber::registry().with(chrome_layer).init();
```

When `_guard` is dropped, your trace will be in a file like `trace-1668480819035032.json`.

Open that file with [ui.perfetto.dev](https://ui.perfetto.dev) (or `chrome://tracing`) and take a look at your pretty trace.

![](https://github.com/thoren-d/tracing-chrome/raw/develop/doc/images/perfetto-screenshot.png)

# License

Licensed under the [MIT license](http://opensource.org/licenses/MIT)

## Contributions

Unless you state otherwise, any contribution intentionally submitted for inclusion in the work shall be licensed as above.
