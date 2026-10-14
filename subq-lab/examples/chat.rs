use subq_lab::Client;
use subq_lab::async_openai::types::chat::{
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subq = Client::from_env()?;

    let models = subq.openai().models().list().await?;
    for m in &models.data {
        println!("model: {}", m.id);
    }

    let request = CreateChatCompletionRequestArgs::default()
        .model("subq-preview")
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content("Say hello in five words.")
            .build()?
            .into()])
        .build()?;

    let response = subq.openai().chat().create(request).await?;
    if let Some(choice) = response.choices.first() {
        println!("{}", choice.message.content.as_deref().unwrap_or(""));
    }
    Ok(())
}
