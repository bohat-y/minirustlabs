use subq_lab::{Choice, Client, Image, Noul, Score};

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

    let c = subq
        .decide("Cat contest")
        .question(
            "cuteness",
            Score::new(
                "How cute does the cat look?",
                [
                    "Ew, disgusting!",
                    "That's not even a cat ",
                    "Cute",
                    "Gorgeous",
                ],
            ),
        )
        .image(Image::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/not-a-cat.jpg"
        ))?)
        .send()
        .await?;

    let cute = c.score("cuteness")?;
    println!(
        "cuteness: {:.2} → {:?} (confidence {:.2})",
        cute.score,
        cute.nearest_label(),
        cute.confidence
    );
    println!("tokens:   {}", c.usage.input_tokens);
    Ok(())
}
