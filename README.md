tracing-chrome
======

[![Crates.io](https://img.shields.io/crates/v/tracing-chrome)](https://crates.io/crates/tracing-chrome)
[![Documentation](https://docs.rs/tracing-chrome/badge.svg)](https://docs.rs/tracing-chrome/)
![GitHub](https://img.shields.io/github/license/Antigroup/tracing-chrome)
![CI](https://github.com/thoren-d/tracing-chrome/workflows/CI/badge.svg?branch=develop)

# Overview

tracing-chrome is a Layer for [tracing-subscriber](https://crates.io/crates/tracing-subscriber) that outputs traces in Chrome's trace viewer format that can be viewed at `chrome://tracing`.

# Usage

```rust
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{registry::Registry, prelude::*};

let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
tracing_subscriber::registry().with(chrome_layer).init();
```
