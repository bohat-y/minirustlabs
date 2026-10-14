use serde_json::{Value, json};
use subq_lab;

type Error = Box<dyn std::error::Error>;

async fn escalate(client: &reqwest::Client, key: &str, state: &str) -> Result<f64, Error> {
    let res = client
        .post("https://api.subq.ai/v1/systemone")
        .bearer_auth(key)
        .json(&json!({
            "model": "subq-snap-preview",
            "state": state,
            "questions": {
                "escalate": {
                    "type": "noul",
                    "instructions": "Should a nurse see this chat before the patient books anything?",
                    "criteria": {
                        "true": "Breathing trouble, chest pain, or a flare of a known condition",
                        "false": "Stable symptoms the patient can book for themselves"
                    }
                }
            }
        }))
        .send()
        .await?;

    let status = res.status();
    let v: Value = res.json().await?;
    if !status.is_success() {
        return Err(format!("{status}: {}", v["detail"]["message"]).into());
    }

    Ok(v["answers"]["escalate"]["noul"]
        .as_f64()
        .unwrap_or_default())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let key = std::env::var("SUBQ_API_KEY")?;
    let client = reqwest::Client::new();

    let (severe, mild, refill) = tokio::join!(
        escalate(
            &client,
            &key,
            "Asthmatic, fever, short of breath walking to the kitchen."
        ),
        escalate(
            &client,
            &key,
            "Mild cough for 3 days, otherwise feeling fine."
        ),
        escalate(
            &client,
            &key,
            "Wants to renew their hay fever prescription."
        ),
    );

    println!("sever: {:.3}", severe?);
    println!("mild: {:.3}", mild?);
    println!("refill: {:.3}", refill?);
    Ok(())
}
