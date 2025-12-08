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
    const NAME: &str = "https://www.youtube.com/watch?v=2JpkMXinO1M&list=RDGMEMhCgTQvcskbGUxqI4Sn2QYwVM2JpkMXinO1M&start_radio=1";
    let (_map , other )= proccess_scrape_request(NAME.to_string(),2).await.unwrap();
    
    dbg!("other urls\t{:?}",other);
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
use std::{ collections::{BinaryHeap,HashMap, HashSet}};
// assume protocol is http or https
async fn proccess_scrape_request(url:String,max_depth:u8) -> Result<(HashMap<String,Vec<String>>,BinaryHeap<String>),String> {
    let str_repr = url.as_str();
    if ! str_repr.starts_with("http://") && !str_repr.starts_with("https://"){
        return Err(format!("Invalid string {} passed as starting point for webscrape, does not start with http or https",url))
    }
    let url_pieces: Vec<&str> = url.split("/").collect();
    if url_pieces.len() < 3{
        return Err(format!("input url:{} is not a valid string, doesnt consist of protocol//domain pattern and as such has less than 3 pieces",url))
    }
    // ex  ["https:", "", "www.w3schools.com", "tags", "tag_cite.asp"]
    // idx:2 -> domain
    Ok(begin_scape(url.clone(), url_pieces[2].to_string(),max_depth).await)
    
}
async fn begin_scape(url:String,domain:String,max_depth:u8) -> (HashMap<String,Vec<String>>,BinaryHeap<String>){
    let mut cur_depth = 0;
    let mut heap = BinaryHeap::new();
    let mut inverted_index: HashMap<String, Vec<String>> = HashMap::new();
    let base_p1 = format!("http://{}",domain);
    let base_p2 = format!("https://{}",domain);
    let mut seen_links: HashSet<String> = HashSet::new();
    heap.push(url);
    loop {
        if cur_depth == max_depth || heap.is_empty(){
            break
        }
        cur_depth += 1;
        let new_site = heap.pop().unwrap();
        let (new_links,site_content) = parse_html_content(request_site(new_site.clone()).await).unwrap();
        let site_content = to_words(site_content);
        let new_links = new_links.into_iter().filter(|l| l.starts_with(&base_p1)|| l.starts_with(&base_p2)).collect::<Vec<_>>();
        for link in new_links{
            if seen_links.insert(link.clone()){
                heap.push(link);
                
            }
        }
        update_mapping(&mut inverted_index, new_site.clone(), site_content.clone());

    }
   (inverted_index,heap )
}
fn update_mapping(input_map: &mut HashMap<String, Vec<String>>, source: String, words: Vec<String>) {
    //input_map.insert(, v)
    // table => [page1,page2]
    for word in words{
        if let Some(prev_map) = input_map.get_mut(&word) {
            if !prev_map.contains(&source){
            prev_map.push(source.clone());

            }
        } else {
            input_map.insert(word, vec![source.clone()]);
        }
    }
}

fn to_words(sent: Vec<String>) -> Vec<String>{
    sent.into_iter()
        .flat_map(|s| {
            s.split_whitespace()
                .map(|word| word.to_string())
                .collect::<Vec<String>>()
        })
        .collect()
}
/* 
fn remove_duplicates(input_map: &mut HashMap<String, Vec<String>>) {
    use std::collections::HashSet;
    for (_key, value) in input_map.iter_mut() {
        let unique: HashSet<_> = value.drain(..).collect();
        *value = unique.into_iter().collect();
    }
}
    */

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

fn parse_html_content(content: String) -> Result<(Vec<String>,Vec<String>),String> {
    let document = scraper::Html::parse_document(&content);
    let selector = generate_parser("a[href]");

    let links = document.select(&selector);
    let valid_links: Vec<String> = links.into_iter().filter_map(|link| link.value().attr("href").map(|s| s.to_string())).collect();
    let valid_links: Vec<String> = valid_links.into_iter().filter(|link| link.starts_with("https://")|| link.starts_with("http://")).collect();
    
    let mut extracted_text: Vec<String> = Vec::new();
    
    // Extract text from h1-h6 tags
    for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        let selector = generate_parser(tag);
        let elements = document.select(&selector);
        extracted_text.extend(elements.into_iter().filter_map(|el| el.text().next().map(|s| s.trim().to_string()).map(|s| s.replace("\n", ""))));
    }
    
    // Extract text from p tags
    let p_selector = generate_parser("p");
    let p_elements = document.select(&p_selector);
    extracted_text.extend(p_elements.into_iter().filter_map(|el| el.text().next().map(|s| s.trim().to_string()).map(|s| s.replace("\n", ""))));
    
    // Extract text from strong tags
    let strong_selector = generate_parser("strong");
    let strong_elements = document.select(&strong_selector);
    extracted_text.extend(strong_elements.into_iter().filter_map(|el| el.text().next().map(|s| s.trim().to_string()).map(|s| s.replace("\n", ""))));
    
    // Extract text from em tags
    let em_selector = generate_parser("em");
    let em_elements = document.select(&em_selector);
    extracted_text.extend(em_elements.into_iter().filter_map(|el| el.text().next().map(|s| s.trim().to_string()).map(|s| s.replace("\n", ""))));
    
    // Extract text from b tags
    let b_selector = generate_parser("b");
    let b_elements = document.select(&b_selector);
    extracted_text.extend(b_elements.into_iter().filter_map(|el| el.text().next().map(|s| s.trim().to_string()).map(|s| s.replace("\n", ""))));
    
    // Extract text from i tags
    let i_selector = generate_parser("i");
    let i_elements = document.select(&i_selector);
    extracted_text.extend(i_elements.into_iter().filter_map(|el| el.text().next().map(|s| s.trim().to_string()).map(|s| s.replace("\n", ""))));

    Ok((valid_links, extracted_text))
}
fn generate_parser(element: &str) -> scraper::Selector {
   scraper::Selector::parse(element).unwrap()

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
(3) done
working with json in rust
(https://github.com/serde-rs/json)
(https://reintech.io/blog/working-with-json-in-rust)
*/


/*
Html tag stuff
<h1> -> <h6> tags
<p> 
<strong>
<em>
<b>
<i>



*/