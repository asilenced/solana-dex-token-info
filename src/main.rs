use actix_web::{get, web, App, HttpResponse, HttpServer, Result, web::ServiceConfig};
use reqwest::Client;
use serde_json::Value;
use chrono::{DateTime, NaiveDate, Utc};
use shuttle_actix_web::ShuttleActixWeb;

const API_KEY: &str = "BQYREb6iuyk839040Wa9SBCeCQzJy5dA";

async fn fetch_data_for_mint_addresses(mint_addresses: Vec<String>) -> Vec<Value> {
    let client = Client::new();
    let mut results = Vec::new();

    for mint_address in mint_addresses {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint_address);

        match client.get(&url).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(pairs) = json.get("pairs")
                    {
                        if !pairs.is_null() {
                            results.push(json);
                        }
                    }
                } else {
                    eprintln!("Failed to parse response for {}", mint_address);
                }
            }
            Err(err) => {
                eprintln!("Error fetching data for {}: {}", mint_address, err);
            }
        }
    }

    results
}

#[get("/raydium/{past_time}/{now}")]
async fn raydium(path: web::Path<(String, String)>) -> Result<HttpResponse> {

    // Define the cutoff time: 2024-11-25 00:00:00 UTC
    let cutoff_time = DateTime::<Utc>::from_utc(
        NaiveDate::from_ymd_opt(2024, 11, 26)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        Utc,
    );

    // Get the current time
    let current_time = Utc::now();

    // Check if current time is before the cutoff
    if current_time >= cutoff_time {
        return Ok(HttpResponse::Forbidden().body("This endpoint is no longer available"));
    }
    
    let info = path.into_inner();
    let past_time = info.0;
    let now = info.1;

    let query = r#"
        query($since: DateTime!, $till: DateTime!) {
            Solana(dataset: archive) {
                DEXTradeByTokens(
                where: {Trade: {Dex: {ProtocolFamily: {is: "Raydium"}}}}
                limit: {count: 10}
                orderBy: {descending: Block_Time}
                ) {
                Block {
                    Time(
                    minimum: Block_Time
                    selectWhere: {since: $since, till: $till}
                    )
                }
                Trade {
                    Dex {
                    ProtocolFamily
                    }
                    Market {
                    MarketAddress
                    }
                    Currency {
                    Symbol
                    MintAddress
                    }
                    Side {
                    Currency {
                        Symbol
                        MintAddress
                    }
                    }
                }
                }
            }
        }
    "#;

    let payload = serde_json::json!({
        "query": query,
        "variables": {
            "since": past_time,
            "till": now
        }
    });

    let client = Client::new();
    let response = client
        .post("https://streaming.bitquery.io/eap") // Replace with the actual GraphQL endpoint
        .header("X-API-KEY", API_KEY)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let body: String = resp.text().await.unwrap_or_else(|_| "Failed to read response".to_string());
            let json: Value = serde_json::from_str(&body).unwrap_or_else(|_| {
                serde_json::json!({
                    "error": "Failed to parse response"
                })
            });

            // Extract and filter MintAddress
            let mut filtered_mint_addresses = Vec::new();

            if let Some(dex_trades) = json
                .get("data")
                .and_then(|data| data.get("Solana"))
                .and_then(|solana| solana.get("DEXTradeByTokens"))
                .and_then(|dex_trade_tokens| dex_trade_tokens.as_array())
            {
                for trade in dex_trades {
                    // Access Trade -> Currency -> MintAddress
                    if let Some(currency_mint) = trade
                        .get("Trade")
                        .and_then(|trade| trade.get("Currency"))
                        .and_then(|currency| currency.get("MintAddress"))
                        .and_then(|mint| mint.as_str())
                    {
                        if currency_mint != "So11111111111111111111111111111111111111112" {
                            filtered_mint_addresses.push(currency_mint.to_string());
                        }
                    }

                    // Access Trade -> Side -> Currency -> MintAddress
                    if let Some(side_mint) = trade
                        .get("Trade")
                        .and_then(|trade| trade.get("Side"))
                        .and_then(|side| side.get("Currency"))
                        .and_then(|currency| currency.get("MintAddress"))
                        .and_then(|mint| mint.as_str())
                    {
                        if side_mint != "So11111111111111111111111111111111111111112" {
                            filtered_mint_addresses.push(side_mint.to_string());
                        }
                    }
                }
            }
            
            let fetched_data = fetch_data_for_mint_addresses(filtered_mint_addresses).await;
            Ok(HttpResponse::Ok().json(fetched_data))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().body(format!("Error fetching data: {}", err))),
    }
}

#[get("/moonshot/{past_time}/{now}")]
async fn moonshot(path: web::Path<(String, String)>) -> Result<HttpResponse> {
    let info = path.into_inner();
    let past_time = info.0;
    let now = info.1;

    // Define the cutoff time: 2024-11-25 00:00:00 UTC
    let cutoff_time = DateTime::<Utc>::from_utc(
        NaiveDate::from_ymd_opt(2024, 11, 26)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        Utc,
    );

    // Get the current time
    let current_time = Utc::now();

    // Check if current time is before the cutoff
    if current_time >= cutoff_time {
        return Ok(HttpResponse::Forbidden().body("This endpoint is no longer available"));
    }

    let query = r#"
        query($since: DateTime!, $till: DateTime!) {
            Solana(dataset: archive) {
                DEXTradeByTokens(
                where: {Trade: {Dex: {ProtocolFamily: {is: "Moonshot"}}}}
                limit: {count: 10}
                orderBy: {descending: Block_Time}
                ) {
                Block {
                    Time(
                    minimum: Block_Time
                    selectWhere: {since: $since, till: $till}
                    )
                }
                Trade {
                    Dex {
                    ProtocolFamily
                    }
                    Market {
                    MarketAddress
                    }
                    Currency {
                    Symbol
                    MintAddress
                    }
                    Side {
                    Currency {
                        Symbol
                        MintAddress
                    }
                    }
                }
                }
            }
        }
    "#;

    let payload = serde_json::json!({
        "query": query,
        "variables": {
            "since": past_time,
            "till": now
        }
    });

    let client = Client::new();
    let response = client
        .post("https://streaming.bitquery.io/eap") // Replace with the actual GraphQL endpoint
        .header("X-API-KEY", API_KEY)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let body: String = resp.text().await.unwrap_or_else(|_| "Failed to read response".to_string());
            let json: Value = serde_json::from_str(&body).unwrap_or_else(|_| {
                serde_json::json!({
                    "error": "Failed to parse response"
                })
            });

            // Extract and filter MintAddress
            let mut filtered_mint_addresses = Vec::new();

            if let Some(dex_trades) = json
                .get("data")
                .and_then(|data| data.get("Solana"))
                .and_then(|solana| solana.get("DEXTradeByTokens"))
                .and_then(|dex_trade_tokens| dex_trade_tokens.as_array())
            {
                for trade in dex_trades {
                    // Access Trade -> Currency -> MintAddress
                    if let Some(currency_mint) = trade
                        .get("Trade")
                        .and_then(|trade| trade.get("Currency"))
                        .and_then(|currency| currency.get("MintAddress"))
                        .and_then(|mint| mint.as_str())
                    {
                        if currency_mint != "So11111111111111111111111111111111111111112" {
                            filtered_mint_addresses.push(currency_mint.to_string());
                        }
                    }

                    // Access Trade -> Side -> Currency -> MintAddress
                    if let Some(side_mint) = trade
                        .get("Trade")
                        .and_then(|trade| trade.get("Side"))
                        .and_then(|side| side.get("Currency"))
                        .and_then(|currency| currency.get("MintAddress"))
                        .and_then(|mint| mint.as_str())
                    {
                        if side_mint != "So11111111111111111111111111111111111111112" {
                            filtered_mint_addresses.push(side_mint.to_string());
                        }
                    }
                }
            }
            
            let fetched_data = fetch_data_for_mint_addresses(filtered_mint_addresses).await;
            Ok(HttpResponse::Ok().json(fetched_data))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().body(format!("Error fetching data: {}", err))),
    }
}

#[get("/pumpfun/{token}/{past_time}")]
async fn pumpfun(path: web::Path<(String, String)>) -> Result<HttpResponse> {
    let info = path.into_inner();
    let token = info.0;
    let past_time = info.1;
    
    // Define the cutoff time: 2024-11-25 00:00:00 UTC
    let cutoff_time = DateTime::<Utc>::from_utc(
        NaiveDate::from_ymd_opt(2024, 11, 26)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        Utc,
    );

    // Get the current time
    let current_time = Utc::now();

    // Check if current time is before the cutoff
    if current_time >= cutoff_time {
        return Ok(HttpResponse::Forbidden().body("This endpoint is no longer available"));
    }

    let query = r#"query($token: String!, $since1: DateTime!) {
  Solana {
    DEXTradeByTokens(
      where: {Transaction: {Result: {Success: true}}, Trade: {Currency: {MintAddress: {is: $token}}}}
    ) {
      Trade {
        Currency {
          Name
          MintAddress
          Symbol
        }
        start: PriceInUSD(minimum: Block_Time)
        min5: PriceInUSD(
          minimum: Block_Time
          if: {Block: {Time: {after: $since1}}}
        )
        end: PriceInUSD(maximum: Block_Time)
        Dex {
          ProtocolName
          ProtocolFamily
          ProgramAddress
        }
        Market {
          MarketAddress
        }
        Side {
          Currency {
            Symbol
            Name
            MintAddress
          }
        }
      }
      trades: count
      trades_5min: count(if:{Block:{Time:{after: $since1}}})
      traded_volume: sum(of: Trade_Side_AmountInUSD)
      traded_volume_5min: sum(of: Trade_Side_AmountInUSD if:{Block:{Time:{after: $since1}}})
      buy_volume: sum(
        of: Trade_Side_AmountInUSD
        if: {Trade: {Side: {Type: {is: buy}}}}
      )
      buy_volume_5min: sum(
        of: Trade_Side_AmountInUSD
        if: {Trade: {Side: {Type: {is: buy}}} Block:{Time:{after:$since1}}}
      )
      sell_volume: sum(
        of: Trade_Side_AmountInUSD
        if: {Trade: {Side: {Type: {is: sell}}}}
      )
      sell_volume_5min: sum(
        of: Trade_Side_AmountInUSD
        if: {Trade: {Side: {Type: {is: sell}}} Block:{Time:{after:$since1}}}
      )
      buys: count(if: {Trade: {Side: {Type: {is: buy}}}})
      buys_5min: count(if: {Trade: {Side: {Type: {is: buy}}} Block:{Time:{after:$since1}}})
      sells: count(if: {Trade: {Side: {Type: {is: sell}}}})
      sells_5min: count(if: {Trade: {Side: {Type: {is: sell}}} Block:{Time:{after:$since1}}})
    }
  }
}

"#;

    let payload = serde_json::json!({
        "query": query,
        "variables": {
            "since1": past_time,
            "token": token
        }
    });

    let client = Client::new();
    let response = client
        .post("https://streaming.bitquery.io/eap") // Replace with the actual GraphQL endpoint
        .header("X-API-KEY", API_KEY)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let body: String = resp.text().await.unwrap_or_else(|_| "Failed to read response".to_string());
            let json: Value = serde_json::from_str(&body).unwrap_or_else(|_| {
                serde_json::json!({
                    "error": "Failed to parse response"
                })
            });
            Ok(HttpResponse::Ok().json(json))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().body(format!("Error fetching data: {}", err))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(raydium)
            .service(moonshot)
            .service(pumpfun)
            .route("/hey", web::get().to(|| async { "Hello there!" }))
    })
    .bind(("127.0.0.1", 8080))?
    .workers(4)
    .run()
    .await
}
