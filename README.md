# maedic

![Tests](https://img.shields.io/github/actions/workflow/status/JayJayArr/maedic/ci.yml)

Service to monitor a PW-installation for its health, supporting the Following Versions:

- `6.5.2 SP1`
- `6.6 SP1`

The following endpoints are available:

- `/v1/health` for the health of PW, checking:
  - Spool Files
  - Hi_Queue Size
  - PW main Service availability
  - System info checks e.g. CPU/RAM
  - db Connection
- `/metrics` exposes an endpoint for `Prometheus` Style Metrics:
  - Version and Build Number as denoted in the db
  - Number of entries for the most important database tables(Events, Panel, Channels, Subpanels, Readers, Badges, Cards, Hi_Queue, Unackknowledged Alarms, Users)
  - Status of Cards
  - HI_Queue Size => Queued Actions per Channel
  - Spool_Files => Spool Files waiting for Download for each Channel
  - Installation Status of the Panels
  - Firmware Versions of the Panels
- `/v1/config` to check the configured limits and options

## Installation

There are multiple options to install:

### Docker

> [!WARNING]
> When using Docker the local Service can not be checked, disable the check via `check_local_service: false` in the `./settings/base.yaml` file.

Assuming you have Docker installed, clone the repo and build:

```bash
# Clone the repository
git clone https://github.com/JayJayArr/maedic
cd maedic
# Run the container
docker compose up -d
```

### Windows

For Windows an installation using [pm2](https://github.com/jessety/pm2-installer) is recommended. Please follow the Installation instructions for windows carefully.

For a complete Installation with pm2 the following Files are required
(please check the [releases page](https://github.com/JayJayArr/maedic/releases/latest)):

- the compiled binary
- ./settings/base.yaml as a config file
- ecosystem.config.js

`maedic` can be started & installed using:

```bash
pm2 start ecosystem.config.js

# Save the pm2 config for automatic restarts
pm2 save

```

## Licenses

maedic is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](https://github.com/lycheeverse/lychee/blob/master/LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](https://github.com/lycheeverse/lychee/blob/master/LICENSE-MIT) or https://opensource.org/license/MIT)

at your option.
