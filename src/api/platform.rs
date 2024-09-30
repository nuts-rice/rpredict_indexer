use serde::Deserialize;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// async fn markets_service(_: )
pub async fn fetch_page(endpoint: &str) -> Result<String> {
    // let host = endpoint.host().expect("uri has no host");
    // let port = endpoint.port_u16().unwrap_or(80);
    // let addr = format!("{}:{}", host, port);

    // let listener = TcpStream::connect(addr).await.unwrap();
    // let io = TokioIo::new(listener);

    // let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
    // tokio::task::spawn(async move {
    //     if let Err(err) = conn.await {
    //         println!("Connection failed: {:?}", err);
    //     }
    // });
    // let authority = endpoint.authority().unwrap().clone();

    // let req = Request::get(endpoint)
    //     // .header("Content-Type", "text/json")
    //     .body(Empty::<Bytes>::new())
    //     .unwrap();
    let res = reqwest::get(endpoint).await?.text().await?;
    // let mut res = sender.send_request(req).await.unwrap();
    tracing::debug!("response: {:?}", res);
    let markets: Vec<Question> = serde_json::from_str(&res)?;
    tracing::debug!("markets: {:?}", markets[0]);
    // while let Some(next) = res.frame().await {
    //     let frame = next?;
    //     if let Some(chunk) = frame.data_ref() {
    //         tracing::debug!("chunk: {:?}", chunk);

    //     }
    // }
    // let markets = serde_json::from_reader(body.reader())?;
    // // let body = Response::new(Full::new(Bytes::from(res)));
    // tracing::debug!("body: {:#?}", markets);
    Ok(res)
    // let body = res.collect().await?.aggregate();
    // let markets : Vec<Question> = serde_json::from_reader(body.reader())?;
    // tracing::debug!("markets: {:?}", markets[0]);
    // Ok(markets)
}

#[derive(Deserialize, Debug)]
pub struct Question {
    question: String,
    id: String,
    created_at: i64,
    cloded_at: i64,
    total_liquidity: i32,
}
