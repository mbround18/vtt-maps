use reqwasm::http::Request;

pub async fn get_markdown(url: &str) -> String {
    Request::get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
