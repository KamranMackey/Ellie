use serde::Deserialize;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message
};

use crate::data::ReqwestContainer;

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "list")]
    definitions: Vec<Definition>
}

#[derive(Debug, Deserialize)]
pub struct Definition {
    #[serde(rename = "definition")]
    description: String,
    example: String,
    word: String,
    thumbs_up: usize,
    thumbs_down: usize,
    permalink: String
}

#[command]
#[description = "Looks up a definition from the Urban Dictionary."]
#[usage = "<name of word>"]
#[aliases("urbandict", "ud", "urban", "define")]
async fn urban(context: &Context, message: &Message, arguments: Args) -> CommandResult {
    if arguments.rest().is_empty() {
        message.channel_id.say(context, "You did not provide a word to look up. Please provide one.").await?;
        return Ok(());
    }

    let term = arguments.rest();

    let client = context.data.read().await.get::<ReqwestContainer>().cloned().unwrap();
    let request = client.get("https://api.urbandictionary.com/v0/define").query(&[("term", term)]).send().await?;
    let response: Response = request.json().await?;

    if response.definitions.is_empty() {
        message.channel_id.say(context, format!("No definitions found for `{}`. Try a different word.", term)).await?;
        return Ok(());
    }

    let definition = response.definitions.get(0).unwrap();

    let word = &definition.word;
    let description = &definition.description;
    let example = &definition.example;
    let permalink = &definition.permalink;
    let thumbs_up = &definition.thumbs_up;
    let thumbs_down = &definition.thumbs_down;

    let rating = format!("{} 👍 | {} 👎", thumbs_up, thumbs_down);

    message
        .channel_id
        .send_message(&context, |message| {
            message.embed(|embed| {
                embed.author(|a| a.name(word).url(permalink));
                embed.color(0x00EF_FF00);
                embed.description(format!("*{description}*\n\n{example}\n\n**Ratings**: {rating}"));
                embed.footer(|f| f.text("Powered by the Urban Dictionary."))
            })
        })
        .await?;

    Ok(())
}

#[command]
#[description = "Gets a random definition from the Urban Dictionary."]
async fn randefine(context: &Context, message: &Message) -> CommandResult {
    let client = context.data.read().await.get::<ReqwestContainer>().cloned().unwrap();
    let request = client.get("http://api.urbandictionary.com/v0/random").send().await?;
    let response: Response = request.json().await?;
    let definition = response.definitions.get(0).unwrap();

    let word = &definition.word;
    let description = &definition.description;
    let example = &definition.example;
    let permalink = &definition.permalink;
    let thumbs_up = &definition.thumbs_up;
    let thumbs_down = &definition.thumbs_down;

    let rating = format!("{} 👍 | {} 👎", thumbs_up, thumbs_down);

    message
        .channel_id
        .send_message(&context, |message| {
            message.embed(|embed| {
                embed.author(|a| a.name(word).url(permalink));
                embed.color(0x00EF_FF00);
                embed.description(format!("*{description}*\n\n{example}\n\n**Ratings**: {rating}"));
                embed.footer(|f| f.text("Powered by the Urban Dictionary."))
            })
        })
        .await?;

    Ok(())
}
