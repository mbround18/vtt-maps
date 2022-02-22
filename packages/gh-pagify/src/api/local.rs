use reqwasm::http::Request;

pub async fn get_readme() -> String {
     Request::get("/README.md")
        .send()
        .await
        .unwrap()
        .text()
         .await
        .unwrap()
}
