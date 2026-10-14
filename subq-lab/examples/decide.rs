use subq_lab::{Choice, Client, Noul, Score};

#[tokio::main]
async fn main() -> subq_lab::Result<()> {
    let subq = Client::from_env()?;

    let d = subq
        .decide("Asthmatic, fever, short of breath walking to the kitchen.")
        .question(
            "escalate",
            Noul::new("Should a nurse see this chat before the patient books anything?")
                .when_true("Breathing trouble, chest pain, or a flare of a known condition")
                .when_false("Stable symptoms the patient can book for themselves"),
        )
        .question(
            "queue",
            Choice::new("Which team should own this chat?")
                .option("nurse_line", "Needs a nurse to assess symptoms today")
                .option("primary_care", "Book with the patient's own doctor")
                .option("pharmacy", "Refill or medication question only")
                .bare_option("front_desk"),
        )
        .question(
            "acuity",
            Score::new(
                "How soon should someone respond?",
                ["Within a week", "Within a day", "Within an hour"],
            ),
        )
        .send()
        .await?;

    let queue = d.choice("queue")?;
    let acuity = d.score("acuity")?;
    println!("escalate: {:.3}", d.noul("escalate")?);
    println!("queue:    {} (confidence {:.2})", queue.choice, queue.confidence);
    println!("acuity:   {:.2} → {:?}", acuity.score, acuity.nearest_label());
    println!("tokens:   {}", d.usage.input_tokens);
    Ok(())
}
