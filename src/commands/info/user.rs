use itertools::Itertools;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        gateway::{Activity, ActivityType},
        prelude::Message,
        user::OnlineStatus
    },
    utils::Colour
};

use tracing::info;

use crate::utils::parsing_utils::parse_user;

#[command]
#[description = "Shows various information about a user."]
#[usage = "<user> or <blank>"]
#[aliases("user", "userinfo", "uinfo", "u")]
#[only_in("guilds")]
async fn user(context: &Context, message: &Message, args: Args) -> CommandResult {
    let cache = &context.cache;
    let guild_id = message.guild_id.ok_or("Failed to get GuildID from Message.")?;
    let cached_guild = cache.guild(guild_id).await.ok_or("Unable to retrieve guild")?;
    let member = if message.mentions.is_empty() {
        if args.is_empty() {
            message.member(&context).await.map_err(|_| "Could not find member.")?
        } else {
            match parse_user(args.rest(), guild_id, context).await {
                Some(i) => guild_id.member(&context, i).await?,
                None => return Ok(())
            }
        }
    } else {
        guild_id.member(&context, message.mentions.first().ok_or("Failed to get user mentioned.")?).await?
    };

    let user = &member.user;

    let mut track_art: String = "".to_string();
    let mut activities: String = String::new();
    let mut active_status: String = String::new();

    if !cached_guild.presences.get(&user.id).is_none() {
        let presence = cached_guild.presences.get(&user.id).unwrap();

        activities = presence
            .activities
            .iter()
            .filter(|a| a.kind != ActivityType::Custom)
            .map(|activity: &Activity| {
                let mut activity_name = activity.name.as_str();
                let activity_kind = match activity.kind {
                    ActivityType::Listening => {
                        if activity_name == "Spotify" {
                            let assets = activity.assets.as_ref().unwrap();
                            let song = activity.details.as_ref().unwrap();
                            let artists = activity.state.as_ref().unwrap();
                            let album = assets.large_text.as_ref().unwrap();
                            let uri = activity.sync_id.as_ref().unwrap();
                            let url = format!("https://open.spotify.com/track/{}", uri);
                            let mut artist_string = artists.to_string();

                            if artists.contains(';') {
                                let replacer = artist_string.replace(";", ",");
                                let commas = replacer.matches(", ").count();
                                let rfind = artist_string.rfind(';').unwrap();
                                let (left, right) = replacer.split_at(rfind);

                                let format_string = if commas >= 2 {
                                    format!("{}{}", left, right.replace(",", ", &"))
                                } else {
                                    format!("{} {}", left, right.replace(",", "&"))
                                };

                                artist_string.clear();
                                artist_string.push_str(&format_string);
                            }

                            let artwork = assets.large_image.as_ref().unwrap().replace("spotify:", "");
                            let artwork_url = format!("https://i.scdn.co/image/{}", artwork);

                            track_art = artwork_url;

                            format!("listening to **[{}]({})** on **{}** by **{}** on", song, url, album, artist_string)
                        } else {
                            "listening to".to_owned()
                        }
                    }
                    ActivityType::Playing => {
                        if activity_name == "Visual Studio Code" {
                            let mut task = activity.details.as_ref().unwrap().clone();
                            let mut project = activity.state.as_ref().unwrap().clone();
                            let app = activity.assets.as_ref().unwrap().small_text.as_ref().unwrap();
                            activity_name = app;
                            if task.contains("Editing ") {
                                task = task.replace("Editing ", "");
                                project = project.replace("Workspace: ", "");
                                format!("editing the file **{}** in the project **{}** with", task, project)
                            } else {
                                task = task.replace("Debugging ", "");
                                project = project.replace("Debugging: ", "");
                                format!("debugging the file **{}** in the project **{}** with", task, project)
                            }
                        } else {
                            "playing".to_owned()
                        }
                    }
                    ActivityType::Competing => "competing in".to_owned(),
                    ActivityType::Streaming => "streaming on".to_owned(),
                    _ => "".to_owned()
                };

                format!("{} **{}**", activity_kind, activity_name)
            })
            .join(" and ");

        let currently_status: String = format!("{} is currently ", user.name);

        active_status.push_str(currently_status.as_str());

        let status = match presence.status {
            OnlineStatus::Online => "Online",
            OnlineStatus::Idle => "Idle",
            OnlineStatus::DoNotDisturb => "Do Not Disturb",
            OnlineStatus::Invisible => "Invisible",
            _ => "Offline"
        };

        let client_status = match &presence.client_status {
            Some(status) => {
                if !status.desktop.is_none() && status.mobile.is_none() && status.web.is_none() {
                    "Desktop"
                } else if !status.mobile.is_none() && status.desktop.is_none() && status.web.is_none() {
                    "Mobile"
                } else if !status.web.is_none() && status.desktop.is_none() && status.mobile.is_none() {
                    "Web"
                } else if !status.desktop.is_none() && !status.mobile.is_none() && status.web.is_none() {
                    "Desktop and Mobile"
                } else if !status.desktop.is_none() && !status.mobile.is_none() && !status.web.is_none() {
                    "Desktop, Mobile, and Web"
                } else if !status.mobile.is_none() && !status.web.is_none() && status.desktop.is_none() {
                    "Mobile and Web"
                } else {
                    "Desktop and Web"
                }
            }
            None => ""
        };

        if status != "Do Not Disturb" {
            active_status.push_str("**");
            active_status.push_str(status);
            active_status.push_str("**");
        } else {
            active_status.push_str("in **Do Not Disturb** mode");
        }

        if !client_status.is_empty() {
            active_status.push_str(" on ");
            active_status.push_str("**");
            active_status.push_str(client_status);
            active_status.push_str("**")
        }

        if activities.is_empty() {
            active_status.push_str(".\n\n")
        } else {
            activities = format!(", {}.\n\n", activities);
        }
    };

    let account_type = if user.bot { "Bot" } else { "User" };

    let created = user.created_at().format("%A, %B %e, %Y @ %l:%M %P");
    let tag = user.tag();
    let id = user.id;
    let color: Colour;
    let hex: String;

    if member.colour(cache).await.is_none() {
        color = Colour::new(0x00FF_FFFF);
        hex = "No display color available.".to_owned()
    } else {
        color = member.colour(cache).await.unwrap();
        hex = format!("#{}", color.hex().to_lowercase());
    }

    let mut roles = String::new();
    let mut role_count = 0;

    if !member.roles(&cache).await.is_none() {
        let cached_roles = member.roles(&cache).await.unwrap();
        let cached_roles_sorted = cached_roles.iter().sorted_by_key(|r| -r.position);
        roles = cached_roles_sorted.map(|r| format!("<@&{}>", &r.id.as_u64())).join(" / ");
        role_count = cached_roles.len();
        if roles.is_empty() {
            roles = "No roles available.".to_owned();
        }
    }

    let main_role = if member.highest_role_info(&cache).await.is_none() {
        info!("Cannot get role information.");
        "No main role available.".to_owned()
    } else {
        let hoist_role_id = member.highest_role_info(&cache).await.ok_or("cannot get role id")?.0;
        let hoist_role = cached_guild.roles.get(&hoist_role_id).ok_or("Cannot get role")?.id.as_u64();
        format!("<@&{}>", hoist_role)
    };

    let nickname = member.nick.map_or("No nickname has been set.".to_owned(), |nick| nick);
    let joined = member.joined_at.unwrap().format("%A, %B %e, %Y @ %l:%M %P");

    message
        .channel_id
        .send_message(&context, |message| {
            message.embed(|embed| {
                embed.author(|a| a.name(&user.name).icon_url(&user.face()));
                embed.thumbnail(track_art);
                embed.colour(color);
                embed.description(format!(
                    "{}{}\
                    **__User Information__**:\n\
                    **Type**: {}\n\
                    **Profile**: <@{}>\n\
                    **Tag**: {}\n\
                    **ID**: {}\n\
                    **Creation Date**: {}\n\n\
                    **__Guild-related Information__**:\n\
                    **Join Date**: {}\n\
                    **Nickname**: {}\n\
                    **Display Color**: {}\n\
                    **Main Role**: {}\n\
                    **Roles ({})**: {}",
                    active_status, activities, account_type, id, tag, id, created, joined, nickname, hex, main_role, role_count, roles
                ))
            })
        })
        .await?;

    Ok(())
}
