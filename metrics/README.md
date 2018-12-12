# Metrics

a POC crate to include metrics.

Currently it only support a Prometheus backend through crate [prometheus](https://crates.io/crates/prometheus).
Therefore trait design is highly tighted to this crate (maybe directly the crate trait for POC), going to something like https://godoc.org/github.com/rcrowley/go-metrics could be an idea.

## design

### call
For ease of use, we will not try to use standard log macro with additional metrics def content.

### config

The configuration (disable enable) is done by rustc `features` that way we got a compiling defined global state (keeping in mind that cargo features are a unique set for all lib inclusion).

The initialisation function will be lazy by default, but for real case (not test case) a feature could make the code skipping lazy loading and a init call will be needed in the main. 
