# Maedic

Service to monitor a PW-installation for its health.

Currently featuring:

- Spool Files
- Hi_Queue Size

All Health Checks are available via a configurable REST endpoint

There are multiple options to install, but for windows services [pm2](https://github.com/jessety/pm2-installer) is recommended.

For a complete Installation with pm2 following Files are required:

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
