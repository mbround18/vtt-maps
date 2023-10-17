use reqwasm::http::Request;

async fn get_txt(path: &str) -> String {
    Request::get(path)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
pub async fn get_readme() -> String {
    get_txt("/README.md").await
}

pub async fn get_catalog() -> String {
    get_txt("/assets/catalog.html").await
}
