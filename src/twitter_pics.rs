use regex::Regex;

pub fn get_image_urls(url: &str) -> Vec<String> {
    let html = reqwest::get(url).unwrap().text().unwrap();
    
    lazy_static! {
        static ref twitter_link_re: Regex =
            Regex::new("https?://(?:www\\.)?twitter\\.com/(?:[\\w\\d]+)/status/(\\d+)")
            .unwrap();
        static ref twitter_id_re: Regex = 
            Regex::new("data-tweet-id=\"(\\d*)\"")
            .unwrap();
        static ref twitter_pic_re: Regex =
            Regex::new("data-image-url=\"(.*)\"")
            .unwrap();
    }
    
    let id_str = twitter_link_re.captures(url)
                                .unwrap()[1]
                                .to_string();
    
    let mut tweet_start: usize = 0;
    let mut tweet_end: usize = html.len();
    for cap in twitter_id_re.captures_iter(&html[..]) {
        if cap[1] == id_str {
            tweet_start = cap.get(0).unwrap().end();
        }
        else if tweet_start != 0 {
            tweet_end = cap.get(0).unwrap().start();
            break;
        }
    }
    
    twitter_pic_re.captures_iter(&html[tweet_start..tweet_end])
                  .map(|cap| cap[1].to_string())
                  .collect()
}
