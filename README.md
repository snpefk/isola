# isola

TUI dashboard for monitoring GitLab Runners.

## Build

Nothing extra, for standalone binary at `target/release/` just run

```shell
cargo build --release
```

or to run immediately

```shell
cargo run -- <HOST> <TOKEN>
```

## Running

To start `isola` you need to specify GitLab host and [personal access token](https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html). You can use these options:

- Passing values as arguments: `isola <HOST> <TOKEN>`
- Specify values through environemt variables: `export ISOLA_HOST=<HOST>` and `export ISOLA_TOKEN=<TOKEN>`

## Keybindings

| Key        | Action |
| ---------- | ------ |
| `q`        | exit   |
| Down / `j` | down   |
| Up / `k`   | up     |
