#[macro_use] extern crate lazy_static;
extern crate serenity;
extern crate regex;
extern crate reqwest;

mod twitter_pics;

use std::env;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use regex::Regex;

use serenity::{
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
    utils::{MessageBuilder, Colour},
};

struct Handler {
    current_links: Arc<Mutex<HashMap<ChannelId, String>>>,
}

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {

        if msg.content == "!ping" {
            let channel = match msg.channel_id.to_channel() {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                },
            };

            // The message builder allows for creating a message by
            // mentioning users dynamically, pushing "safe" versions of
            // content (such as bolding normalized content), displaying
            // emojis, and more.
            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(msg.author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(why) = msg.channel_id.say(&response) {
                println!("Error sending message: {:?}", why);
            }
        }

        if msg.content.starts_with("!lastLink") {
            let link_data = Arc::clone(&self.current_links);
            let current_links = link_data.lock().unwrap();
            
            let response = 
                match current_links.get(&msg.channel_id) {
                    Some(link) => 
                        MessageBuilder::new()
                        .push("Current link:")
                        .push_codeblock(link, None)
                        .build(),
                    None => 
                        "No link found for current channel".to_string(),
                };
            
            if let Err(why) = msg.channel_id.say(&response) {
                println!("Error sending message: {:?}", why);
            }
        }
        
        lazy_static! {
            static ref twitter_link_re: Regex =
            Regex::new("https?://(?:www\\.)?twitter\\.com/(?:[\\w\\d]+)/status/(\\d+)")
            .unwrap();
        }
        
        if let Some(last_match) = twitter_link_re.captures_iter(&msg.content[..]).last() {
            let link_data = Arc::clone(&self.current_links);
            let mut current_links = link_data.lock().unwrap();
            
            let url = last_match[0].to_string();
            current_links.insert(msg.channel_id, url);
        }

        if msg.content.starts_with("!pics") {
            let link_data = Arc::clone(&self.current_links);
            let current_links = link_data.lock().unwrap();
            
            match current_links.get(&msg.channel_id) {
                Some(link) => {
                    for pic_url in twitter_pics::get_image_urls(link).iter().skip(1) {
                        msg.channel_id.send_message(|m| m
                            .embed(|e| e
                                .title("Additional tweet pics")
                                .image(pic_url)
                                .colour(Colour::from_rgb(0, 172, 237))
                        ));
                    }
                } 
                None =>
                {
                    msg.channel_id.send_message(|m| m
                        .content("No link found for current channel".to_string())
                    );
                }
            };
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler {
        current_links: Arc::new(Mutex::new(HashMap::new()))
    }).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
