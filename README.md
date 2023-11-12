# GPIO REST API written in Rust

This is a personal project to connect Raspberry PI GPIO to a home assistant instance through a REST API.
I made this because of a bug in the remote_rpi_gpio platform of home assistant (wasn't reading the gpio correctly...)

To use it, it's not necessary to modify code, I made it config-based and changing [config.json](config.json) should be enough.

Config before:

```yaml
binary_sensor:
  - platform: remote_rpi_gpio
    host: xxx.xxx.x.x
    ports:
      24: garage_door
    pull_mode: DOWN
```

Config after:

```yaml
garage_door_sensor:
  - platform: rest
    resource: https://localhost:5679/?id=24
    scan_interval: 1
    value_template: "{{value_json.value}}"
```

## TO-DO

- Sending output commands
