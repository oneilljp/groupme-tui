# GroupMe TUI

A GroupMe client with basic functionality written in Rust.

## Requirements
- This client partially uses native-tls for websocket communication. Windows 10 and macOS users should be fine as far as I know, but Linux users will need to ensure that they have either OpenSSL (1.0.1 - 1.1.1) or LibreSSL (2.5 - 2.8) installed on their systems in order to compile.
- GroupMe Account and API Key, the latter can be obtained [here](https://dev.groupme.com) by logging in to your account and locating your "Access Token"

## Installation

### Cargo

Ensure that you have [Rust](https://www.rust-lang.org/tools/install) installed, and clone this repository. Then, run the following from the command line:

```bash
cargo install --path /path/to/repository
```

If need be, this application can be uninstalled by running

```bash
cargo uninstall groupme-tui
```

## Configuration

Upon first running groupme-tui, you will be prompted to input your API Key, which will be stored in a config file called ```config.toml``` (along with other potential configuration options in the future).

**IMPORTANT:** Ensure that your API Key stored in ```config.toml``` is correct, or the application will break

### config.toml Directory

If the environmental variable ```GMTUI_CONFIG``` is set, groupme-tui will look through that directory (or create it if necessary) for the config file. Otherwise, the following OS specific directories will be used:

#### Linux
Either ```${XDG_CONFIG_HOME}/groupme-tui``` or ```${HOME}/.config/groupme-tui```

#### macOS
```${HOME}/Library/Application Support/Groupme-tui```

#### Windows
```C:\Users\<username>\AppData\Roaming\Groupme-tui```

## Running

groupme-tui can be run through the commandline via the command ```gmtui```

## Roadmap
- [x] Send and Recieve Group and Direct Messages
- [x] Like/Unlike Messages
- [x] Cross platform config file standard for ease of use
	- Currently just stores api key
- [ ] Group Management
- [ ] More customization through config file
- [ ] Abstract away api calls into separate library
