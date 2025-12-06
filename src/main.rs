use axum::{
    Json, Router, extract::Query, routing::{get,post}
};
use serde::{Deserialize,Serialize};
use reqwest::{Client};
#[derive(Deserialize)]
struct SearchQuery {
    search_term: String,
}
#[derive(Deserialize,Debug)]
struct IndexRequest{
    site: String,
    max_depth: u8
}

#[derive(Deserialize, Serialize)]
struct IndexRepresentation{
    word: String, // word/search term
    pages: Vec<String> // urls where this word appears 
}
#[derive(Deserialize, Serialize)]
struct AllIndexResponse{
    meessage: String,
    data: Vec<IndexRepresentation>
}
#[derive(Deserialize, Serialize)]
struct SingleIndexResponse{
    message: String,
    data: IndexRepresentation
}
fn build_index_repr(word:String, pages:Vec<String> ) -> IndexRepresentation {
    IndexRepresentation { word, pages }
}
#[tokio::main]
async fn main() {
    // build our application with a single route
    let _router = new_router().await;
    const NAME: &str = "https://www.w3schools.com/tags/tag_cite.asp";
   let content= request_site(NAME.to_string()).await;
   parse_html_content(content);
    // run our app with hyper, listening globally on port 3000
    //let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    //axum::serve(listener, router).await.unwrap();
}
async fn new_router() -> Router<>{
    Router::new()
    .route("/api/v1/index", post(post_link))
    .route("/api/v1/index", get(get_all_links))
    .route("/api/v1", get(find_word_index))
}
async fn post_link(Json(body): Json<IndexRequest>) -> String {
    println!("creating an inverted index starting at: {} max depth: {}\n",body.site,body.max_depth);
    // TODO: add logic for actaully web scrapping this term
    "OK".to_string()
}
use axum::response::IntoResponse;

async fn get_all_links() -> impl IntoResponse {
    println!("Get all links route\n");
    let example_resp_body = dummy_response();
    Json(
        AllIndexResponse{
            meessage: "ok".to_string(),
            data:example_resp_body
        }
    )

}
async fn find_word_index(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    println!("You searched for: {}\n", query.search_term);
    let ex = dummy_single_response(query.search_term);
    Json(
        SingleIndexResponse{
            message:"ok".to_string(),
            data:ex
        }
    )
}
fn dummy_response() -> Vec<IndexRepresentation>{
    let mut base = vec![build_index_repr("this".to_string(), vec!["http.example1".to_string(),"http.example2".to_string(),"http.example3".to_string(),])];
    base.push(build_index_repr("different_term".to_string(), vec!["http.ruid1".to_string()]));
    base
}
fn dummy_single_response(term: String) -> IndexRepresentation{
    build_index_repr(term.to_string(), vec!["page1".to_string(),"page2".to_string()])
}

async fn request_site(url:String) -> String{
    let client = Client::new();

    let response = client
    .get(url)
    .send()
    .await
    .unwrap();
    let resp_body = response.text().await.unwrap();
    resp_body

}
fn parse_html_content(content: String) -> String {
    let document = scraper::Html::parse_document(&content);
    let selector = match scraper::Selector::parse("a[href]") {
        Ok(v) => v,
        Err(_) => return "error occurred generating parse".to_string(),
    };

    let links = document.select(&selector);
    for link in links {
        if let Some(href) = link.value().attr("href") {
            println!("Found link: {}", href);
        }
    }

    "ok".to_string()
}
    
/*
First draw out the system diagram (Done)
(1) Done
setting up a basic web server in rust
(https://docs.rs/axum/latest/axum/)
(1.5) Done
web scraping in rust
(https://www.zenrows.com/blog/rust-web-scraping#extract-html-data)
(https://www.scrapingbee.com/blog/web-scraping-rust/)
(2)
How are we going to structure data (write to database?) prob redis since the data is pretty simple
(2.5)how to store data in hash tables in rust
(https://doc.rust-lang.org/std/collections/struct.HashMap.html)
(3)
working with json in rust
(https://github.com/serde-rs/json)
(https://reintech.io/blog/working-with-json-in-rust)
(4)
how to read and write files in rust
(https://medium.com/@bayounm95.eng/introduction-to-i-o-in-rust-88a247e0f156)
*/