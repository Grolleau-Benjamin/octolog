# Octolog

Multi-serial-port log monitor for the terminal. Octolog connects to multiple
serial devices at once, prints timestamped lines with per-port labels and
colors, and optionally mirrors output to a file.

![image-20260203095128979](./img/screen.png)

## Features

- Read from multiple serial ports concurrently.
- Per-port labels (alias) and deterministic coloring.
- Highlight patterns in output.
- Include or exclude lines by substring.
- Optional file output (same rendered format as stdout).
- Clean Ctrl+C shutdown.

## Requirements

- Access to serial devices on your OS (permissions may be required).

## Build

```bash
cargo build --release
```

## Usage

List available ports:

```bash
cargo run -- --list
```

Monitor one or more ports:

```bash
cargo run -- -p /dev/ttyACM0 -p /dev/ttyACM1
cargo run -- -p /dev/ttyACM0:115200:Sensor -p /dev/ttyUSB0:9600:GPS
```

Write output to a file:

```bash
cargo run -- -p /dev/ttyACM0 --output logs/session.log
```

Filter and highlight:

```bash
cargo run -- -p /dev/ttyACM0 --filter "AT+" --exclude "DEBUG" --highlight ERROR
```

## Port Spec Format

Ports are provided with `-p/--port` and accept:

```
path[:baudrate][:alias]
```

Examples:

- `/dev/ttyACM0`
- `/dev/ttyACM0:115200`
- `/dev/ttyACM0:Sensor` (alias without explicit baudrate)
- `/dev/ttyACM0:115200:Sensor`

If a baudrate is omitted, the `--baud` default is used (115200 by default).

## Output Format

Each log line is printed as:

```
[timestamp] [source] │ message
```

System events (connect/disconnect, warnings) go to stderr with `[SYS]`.

## Notes

- `--list` shows commonly used serial port names for your OS (USB/ACM/COM).
- If a port fails to open, Octolog keeps running as long as at least one port
  is available.
- File output directories are created automatically when needed.

