use network::request;

#[tokio::main]
async fn main() {
    let result = request::fetch_page("http://localhost/api/v1/health").await;

    match result {
        Ok(response_result) => {
            let response = &response_result.0;
            println!("Status: {}", response.status);
            println!("Headers: {:?}", response.headers);
            println!("Size: {} bytes", response.size);
            println!("Body: {}", response.body);
        }
        Err(_) => {
            println!("Failed to fetch the URL.");
        }
    }
}
