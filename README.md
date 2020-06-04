# proper\[ty\]watcher

The proper\[ty\]watcher is a lightweight Rust application that can monitor different property website queries. Once new properties are found on any of the sources, these changes can be forwarded to different sinks. So far a firebase database and/or a telegram chat can be configured.

## Key Ideas

Since the tool is written in Rust it comes with a very low memory and cpu footprint. When only a few sources are being watched, memory will reside within 10-20 mb. Thus, you can have many instances running at the same time, watching different property queries and publishing results to different sinks.

- lightweight: low on memory and cpu
- easy to setup: configure custom searches on the propery portals and then use resulting URLs
- be the first to know: define sinks and get instant notifications (i.e. via Telegram)

## Usage

TBD