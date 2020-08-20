chrome-tracing
======

![Check Commit](https://github.com/Antigroup/chrome-tracing/workflows/Check%20Commit/badge.svg?branch=develop)

# Overview

chrome-tracing is a Layer for [tracing-subscriber](https://crates.io/crates/tracing-subscriber) that outputs traces in Chrome's trace viewer format that can be viewed at `chrome://tracing`.

# Usage

```rust
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{registry::Registry, prelude::*};

let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
tracing_subscriber::registry().with(chrome_layer).init();

```