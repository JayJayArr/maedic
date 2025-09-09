module.exports = {
  apps: [
    {
      name: "maedic",
      script: "./maedic.exe",
      interpreter_args: "--max-old-space-size=4096",
      env_production: {},
      env_development: {},
      instances: 1,
      exec_mode: "fork",
    },
  ],
};
