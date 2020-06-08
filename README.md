# proper\[ty\]watcher

The proper\[ty\]watcher is a lightweight Rust application that can monitor different property website queries. Once new properties are found on any of the sources, these changes can be forwarded to different sinks. So far a firebase database and/or a telegram chat can be configured.

## Features

Since the tool is written in Rust it comes with a very low memory and cpu footprint. When only a few sources are being watched, memory will reside within 10-20 mb. Thus, you can have many instances running at the same time, watching different property queries and publishing results to different output channels.

- **lightweight**: low on memory and cpu
- **easy to setup**: configure custom searches on the propery portals and then use resulting URLs
- **be the first to know**: define notifications and know about new properties immediately once they are available.

### Supported Websites

- ImmobilienScout24
- Wohnungsboerse
- Immobilienmarkt Süddeutsche Zeitung
- WG Gesucht
- ImmoWelt

### Supported Sinks

- Firebase: Cloud Firestore
- Telegram: Sends messages to any Telegram chat
- Mail: Sends mails via SMTP

## Usage

### Configuration File

Settings and property queries that should be watched have to be defined in a configuration file. propertywatcher by default looks for a file called `config.toml` that resides in the same directory as the tool is run from. A sample configuration file can be found in this repository and is named [config.sample.toml](/config.sample.toml). You can create a copy and adjust it to your needs. Pay special attention to the [`watcher` section](config.sample.toml#L21). This section can be given multiple times and will tell properwatcher, where to look for new flats/houses.

### via Docker

Once you have created a valid configuration file, you can run properwatcher via the provided docker image. The properwatcher command takes an optional location for a configuration file as first parameter. We'll use this fact in the following command to refer to our mounted config file that locally resides in `/home/flo/config.toml`.

```bash
docker run -it -v /home/flo/config.toml:/opt/properwatcher.toml --name properwatcher floschnell/properwatcher /opt/properwatcher.toml
```