//! A decision about a photo: gate on "is it a cat?" before trusting the score.
//!
//! cargo run -p subq-lab --example image -- path/to/photo.jpg

use subq_lab::{Client, Image, Noul, Score};

#[tokio::main]
async fn main() -> subq_lab::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p subq-lab --example image -- <photo.jpg|png|webp>");
        std::process::exit(2);
    });
    let subq = Client::from_env()?;

    let d = subq
        .decide("Photo entered in a cat photo contest.")
        .image(Image::from_path(&path)?)
        .question("is_cat", Noul::new("Does the photo show a cat?"))
        .question(
            "cuteness",
            Score::new(
                "How cute does the cat look?",
                ["Not cute", "Somewhat cute", "Cute", "Gorgeous"],
            ),
        )
        .send()
        .await?;

    let p_cat = d.noul("is_cat")?;
    let cute = d.score("cuteness")?;

    if p_cat < 0.5 {
        println!("not a cat (p = {p_cat:.2})");
    } else if cute.confidence < 0.5 {
        println!(
            "a cat, but the cuteness verdict is unsure (confidence {:.2})",
            cute.confidence
        );
    } else {
        println!(
            "{:?}: score {:.2}, confidence {:.2}",
            cute.nearest_label(),
            cute.score,
            cute.confidence
        );
    }
    println!("tokens: {}", d.usage.input_tokens);
    Ok(())
}
