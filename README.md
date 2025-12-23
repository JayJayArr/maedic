# Maedic

Service to monitor a PW-installation for its health.

Currently featuring the following PW specific checks:

- Spool Files
- Hi_Queue Size
- PW main Service monitoring
- Sysinfo checks incl. CPU/RAM

All Health Checks are available via a configurable REST endpoint

There are multiple options to install:

## Docker

> [!WARNING]
> When using Docker the local Service can not be checked, disable the check via `check_local_service: true` in the base.yaml file.

### Prerequisites

- A Docker installation, if you need help installing please see [here](https://docs.docker.com/engine/install/)
- The Rust toolchain installed, if you need help installing please see [here](https://rust-lang.org/learn/get-started/)

```bash
# Clone the repository
git clone https://github.com/JayJayArr/maedic
cd maedic
# Run the container
docker compose up -d
```

## Windows

For Windows an installation using [pm2](https://github.com/jessety/pm2-installer) is recommended.

For a complete Installation with pm2 the following Files are required(please check the [releases page](https://github.com/JayJayArr/maedic/releases/latest)):

- the compiled binary
- base.yaml as a config file
- ecosystem.config.js

Assuming a node js environment installed follow these steps to setup pm2:

```bash
# configure node prefix and cache
npm run configure

# configure execution policy
npm run configure-policy

# Setup pm2
npm run setup

```

After setting up pm2 follow these steps to install maedic as a service:

```bash
# Assuming all files are located in C:\maedic
pm2 start ecosystem.config.js

# Save the pm2 config for automatic restarts
pm2 save

```
