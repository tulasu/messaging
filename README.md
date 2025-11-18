# messaging

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/status-development-yellow.svg?style=for-the-badge)]()

A Rust application for sending messages across multiple messaging platforms including VK, Telegram and MAX.

## Running locally

### Docker Compose

```bash
docker compose up --build
```

The compose stack launches PostgreSQL, NATS JetStream and the API container. Default credentials are defined directly inside `docker-compose.yml`. Adjust them through environment variables if needed.

### Manual run

1. Copy `.env.example` to `.env` and adjust values (database URL, NATS endpoint, JWT secret).
2. Start PostgreSQL and NATS services that match your configuration.
3. Run migrations on startup automatically by launching the API:

```bash
cargo run
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

