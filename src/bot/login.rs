use std::{io, process::Command};

use base64::{engine::general_purpose, Engine};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use json::JsonValue::Null;
use regex::Regex;
use ureq::Agent;

static USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

pub fn post_ubisoft_rememberme(agent: &Agent, ticket: &str) -> Result<String, ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "steam")
        .set("Authorization", &format!("rm_v1 t={}", ticket))
        .set("Content-Type", "application/json")
        .send_string(
            json::stringify(json::object! {
                "rememberMe": true,
            })
            .as_str(),
        )?
        .into_string()?;

    let json = json::parse(&body).unwrap();
    Ok(json["ticket"].to_string())
}

pub fn post_ubisoft_2fa_ticket(
    agent: &Agent,
    ticket: &str,
    token: &str,
) -> Result<String, ureq::Error> {
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Ubi-AppId", "f2f8f582-6b7b-4d87-9a19-c72f07fccf99")
        .set("Ubi-RequestedPlatformType", "steam")
        .set("Ubi-2faCode", &format!("{}", token))
        .set("Authorization", &format!("ubi_2fa_v1 t={}", ticket))
        .set("Content-Type", "application/json")
        .send_string(
            json::stringify(json::object! {
                "rememberMe": true,
                "trustedDevice": Null,
            })
            .as_str(),
        )?
        .into_string()?;

    let json = json::parse(&body).unwrap();
    let ticket = &json["ticket"].to_string();
    if json.has_key("rememberMeTicket") {
        let remember_me_ticket = &json["rememberMeTicket"].to_string();
        let ticket = post_ubisoft_rememberme(&agent, remember_me_ticket)?;
        return Ok(ticket.to_string());
    }
    Ok(ticket.to_string())
}

pub fn get_ubisoft_session(
    agent: &Agent,
    email: &str,
    password: &str,
    code: &str,
) -> Result<String, ureq::Error> {
    let encoded = general_purpose::STANDARD.encode(format!("{}:{}", email, password));
    let body = agent
        .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
        .set("User-Agent", USER_AGENT)
        .set("Authorization", &format!("Basic {}", encoded))
        .set("Ubi-AppId", "afb4b43c-f1f7-41b7-bcef-a635d8c83822")
        .set("Ubi-RequestedPlatformType", "uplay")
        .set("Content-Type", "application/json")
        .send_string(
            json::stringify(json::object! {
                "rememberMe": true,
            })
            .as_str(),
        )?
        .into_string()?;

    let json = json::parse(&body).unwrap();
    if json.has_key("twoFactorAuthenticationTicket")
        && json["twoFactorAuthenticationTicket"] != Null
    {
        let ticket = &json["twoFactorAuthenticationTicket"].to_string();
        let token = rust_otp::make_totp(&code.to_ascii_uppercase(), 30, 0).unwrap();
        let session_ticket = post_ubisoft_2fa_ticket(&agent, ticket, &token.to_string())?;
        return Ok(session_ticket);
    }
    Ok(json["ticket"].to_string())
}

pub fn get_ubisoft_token(email: &str, password: &str, code: &str) -> Result<String, ureq::Error> {
    let agent = ureq::AgentBuilder::new().redirects(5).build();
    let session = match get_ubisoft_session(&agent, email, password, code) {
        Ok(res) => res,
        Err(err) => {
            return Err(err);
        }
    };

    let formated = format!("UbiTicket|{}\nrequestedName|\nf|1\nprotocol|209\ngame_version|4.62\nfz|46297624\nlmode|0\ncbits|1024\nplayer_age|25\nGDPR|1\ncategory|_-5100\ntotalPlaytime|0\nklv|461a6affd0aac154c25c9e867c789ef8c7b5017bbe723d1f86a578ff325b97fe\nhash2|841545814\nmeta|+NlguMhpl2JQ1iP7kyp2Z8W8n9OKDNn57/xI5jJp7/g=\nfhash|-716928004\nrid|020F3BE731F0CF30002CA0AB1843B2A1\nplatformID|13,1,1\ndeviceVersion|0\ncountry|us\nhash|-1829975549\nmac|b4:8c:9d:90:79:cf\nwk|66A6ABCD9753A066E39975DED77852A8\nzf|1390211647", session);
    let body = agent
            .post("https://login.growtopiagame.com/player/login/dashboard?valKey=40db4045f2d8c572efe8c4a060605726")
            .set("cache-control", "max-age=0")
            .set("sec-ch-ua", "\"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Microsoft Edge\";v=\"126\", \"Microsoft Edge WebView2\";v=\"126\"")
            .set("sec-ch-ua-mobile", "?0")
            .set("sec-ch-ua-platform", "\"Windows\"")
            .set("content-type", "application/x-www-form-urlencoded")
            .set("upgrade-insecure-requests", "1")
            .set("user-agent", USER_AGENT)
            .set("origin", "null")
            .set("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .set("sec-fetch-site", "none")
            .set("sec-fetch-mode", "navigate")
            .set("sec-fetch-user", "?1")
            .set("sec-fetch-dest", "document")
            .set("accept-encoding", "gzip, deflate, br, zstd")
            .set("accept-language", "en-US,en;q=0.9")
            .send_string(&formated)?;

    let body = body.into_string()?;
    let json = json::parse(&body).unwrap();
    Ok(json["token"].to_string())
}

pub fn get_apple_token(url: &str) -> Result<String, std::io::Error> {
    println!("Getting apple token");
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/c", "start", "", url])
            .spawn()
            .expect("Failed to open URL on Windows");
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .expect("Failed to open URL on Linux");
    }

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}
#[tokio::main]
pub async fn get_google_token(
    email: &str,
    password: &str,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .args(vec![
                "--excludeSwitches=enable-automation",
                "--disable-blink-features=AutomationControlled",
                "--lang=en-EN",
                "--window-size=1920,1080",
                &format!("--user-agent={}", USER_AGENT),
            ])
            .build()?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = browser.new_page(url).await?;
    page.enable_stealth_mode().await?;
    match page
        .find_xpath(format!("//li/div[@data-identifier='{}']", email))
        .await
    {
        Ok(elem) => {
            elem.click().await?;
        }
        Err(..) => match page.find_xpath("//*[@id=\"identifierId\"]").await {
            Ok(..) => {
                handle_google_login_form(email, password, &page).await?;
            }
            Err(..) => {
                page.find_xpath("//li/div[not(@data-identifier)]")
                    .await?
                    .click()
                    .await?;
                page.wait_for_navigation_response().await?;
                handle_google_login_form(email, password, &page).await?;
            }
        },
    };
    page.wait_for_navigation_response().await?;
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let source = page
        .find_element("body")
        .await?
        .inner_text()
        .await?
        .unwrap();
    if source.contains("too many people") {
        return Err("Too many people trying to login".into());
    }
    let json = json::parse(&source).unwrap();

    browser.close().await?;
    handle.await?;
    Ok(json["token"].to_string())
}

async fn handle_google_login_form(
    email: &str,
    password: &str,
    page: &Page,
) -> Result<(), Box<dyn std::error::Error>> {
    page.find_xpath("//*[@id=\"identifierId\"]")
        .await?
        .type_str(email)
        .await?;

    page.find_xpath("//*[@id=\"identifierNext\"]/div/button/span")
        .await?
        .click()
        .await?;
    page.wait_for_navigation_response().await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    page.find_xpath("//*[@id=\"password\"]/div[1]/div/div[1]/input")
        .await?
        .type_str(password)
        .await?;
    page.find_xpath("//*[@id=\"passwordNext\"]/div/button/span")
        .await?
        .click()
        .await?;
    page.wait_for_navigation_response().await?;

    Ok(())
}

pub fn get_legacy_token(url: &str, username: &str, password: &str) -> Result<String, ureq::Error> {
    let agent = ureq::AgentBuilder::new().build();
    let body = agent
        .get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0")
        .set(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        )
        .set("Accept-Language", "en-US,en;q=0.5")
        .set("Accept-Encoding", "gzip, deflate, br, zstd")
        .set("DNT", "1")
        .set("Sec-GPC", "1")
        .set("Connection", "keep-alive")
        .set("Upgrade-Insecure-Requests", "1")
        .set("Sec-Fetch-Dest", "document")
        .set("Sec-Fetch-Mode", "navigate")
        .set("Sec-Fetch-Site", "none")
        .set("Sec-Fetch-User", "?1")
        .set("Sec-CH-UA-Platform", "Windows")
        .set(
            "Sec-CH-UA",
            "\"Edge\";v=\"120\", \"Chromium\";v=\"120\", \"Not=A?Brand\";v=\"24\"",
        )
        .set("Sec-CH-UA-Mobile", "?0")
        .set("Priority", "u=1")
        .set("TE", "trailers")
        .call()?
        .into_string()?;

    let token = match extract_token_from_html(&body) {
        Some(token) => token,
        None => panic!("Failed to extract token"),
    };
    let req = agent
        .post("https://login.growtopiagame.com/player/growid/login/validate")
        .send_form(&[
            ("_token", &token),
            ("growId", &username),
            ("password", &password),
        ])?;

    let body = req.into_string().unwrap();
    let json = json::parse(&body).unwrap();
    Ok(json["token"].to_string())
}

pub fn extract_token_from_html(body: &str) -> Option<String> {
    let regex = Regex::new(r#"name="_token"\s+type="hidden"\s+value="([^"]*)""#).unwrap();
    regex
        .captures(body)
        .and_then(|cap| cap.get(1).map(|match_| match_.as_str().to_string()))
}
