# subq-lab

Unofficial Rust client for the [SubQ](https://docs.subq.ai/) Decisions API.

> Not affiliated with Subquadratic. The SubQ API is in preview, so this crate may change.

## Install

```sh
cargo add subq-lab
cargo add tokio --features macros,rt-multi-thread
```

## Usage

Set `SUBQ_API_KEY` (from [console.subq.ai](https://console.subq.ai)), then:

```rust
use subq_lab::{Client, Noul};

#[tokio::main]
async fn main() -> subq_lab::Result<()> {
    let subq = Client::from_env()?;

    let decision = subq
        .decide("Patient with asthma reports shortness of breath.")
        .question(
            "escalate",
            Noul::new("Should a nurse see this chat?")
                .when_true("Breathing trouble or a flare of a known condition")
                .when_false("Stable symptoms"),
        )
        .send()
        .await?;

    println!("P(escalate) = {:.2}", decision.noul("escalate")?);
    Ok(())
}
```

Questions can be `Noul` (yes/no), `Choice` (pick one option) or `Score` (ordered levels).
See [`examples/decide.rs`](examples/decide.rs) for all three.

## OpenAI-compatible endpoints

Enable the `openai` feature to get an [`async-openai`](https://crates.io/crates/async-openai)
client pointed at SubQ (chat, responses, files, models):

```toml
subq-lab = { version = "0.1", features = ["openai"] }
```

```rust
let chat = subq.openai().chat().create(request).await?;
```

## License

MIT OR Apache-2.0
