use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct SearchQuery {
    search_term: String,
}
#[derive(Deserialize, Debug)]
struct IndexRequest {
    site: String,
    max_depth: u8,
}

#[derive(Deserialize, Serialize)]
struct IndexRepresentation {
    word: String,       // word/search term
    pages: Vec<String>, // urls where this word appears
}
#[derive(Deserialize, Serialize)]
struct AllIndexResponse {
    meessage: String,
    data: Vec<IndexRepresentation>,
}
#[derive(Deserialize, Serialize)]
struct SingleIndexResponse {
    message: String,
    data: IndexRepresentation,
}
fn build_index_repr(word: String, pages: Vec<String>) -> IndexRepresentation {
    IndexRepresentation { word, pages }
}
#[tokio::main]
async fn main() {
    // build our application with a single route
    let router = new_router().await;
    println!("=============\trunning new web scraper service\t===================\n");
    //const NAME: &str = "https://www.tutorialspoint.com/redis/lists_lrange.htm";

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
async fn new_router() -> Router {
    Router::new()
        .route("/api/v1/index", post(post_link))
        .route("/api/v1/index", get(get_all_links))
        .route("/api/v1", get(find_word_index))
}
async fn post_link(Json(body): Json<IndexRequest>) -> String {
    println!(
        "creating an inverted index starting at: {} max depth: {}\n",
        body.site.clone(),
        body.max_depth
    );
    match proccess_scrape_request(body.site.clone(), body.max_depth).await {
        Ok(code) => format!("status code:\t{}", code),
        Err(err) => format!(
            "request to scrape {:?} failed with this error {}",
            body.site, err
        ),
    }
}
use axum::response::IntoResponse;

async fn get_all_links() -> impl IntoResponse {
    println!("Get all links route\n");
    let mut redis_instance = new_redis_instance().unwrap();
    let all_keys: Vec<String> = redis::cmd("KEYS")
        .arg("*")
        .query(&mut redis_instance)
        .unwrap();
    let mut output_index = vec![];
    for key_name in all_keys {
        let result_index: redis::RedisResult<Vec<String>> =
            redis_instance.lrange(key_name.clone(), 0, -1);
        let idx_repr = IndexRepresentation {
            word: key_name,
            pages: result_index.unwrap(),
        };
        output_index.push(idx_repr);
    }
    Json(AllIndexResponse {
        meessage: "ok".to_string(),
        data: output_index,
    })
}
async fn find_word_index(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    println!("You searched for: {}\n", query.search_term);
    let mut redis_instance = new_redis_instance().unwrap();
    let result_index: redis::RedisResult<Vec<String>> =
        redis_instance.lrange(query.search_term.clone(), 0, -1);
    let idx_repr = IndexRepresentation {
        word: query.search_term.clone(),
        pages: result_index.unwrap(),
    };
    Json(SingleIndexResponse {
        message: "ok".to_string(),
        data: idx_repr,
    })
}
fn dummy_response() -> Vec<IndexRepresentation> {
    let mut base = vec![build_index_repr(
        "this".to_string(),
        vec![
            "http.example1".to_string(),
            "http.example2".to_string(),
            "http.example3".to_string(),
        ],
    )];
    base.push(build_index_repr(
        "different_term".to_string(),
        vec!["http.ruid1".to_string()],
    ));
    base
}
fn dummy_single_response(term: String) -> IndexRepresentation {
    build_index_repr(
        term.to_string(),
        vec!["page1".to_string(), "page2".to_string()],
    )
}
use std::collections::{BinaryHeap, HashMap, HashSet};
// assume protocol is http or https
async fn proccess_scrape_request(url: String, max_depth: u8) -> Result<u8, String> {
    let str_repr = url.as_str();
    if !str_repr.starts_with("http://") && !str_repr.starts_with("https://") {
        return Err(format!("Invalid string {} passed as starting point for webscrape, does not start with http or https",url));
    }
    let url_pieces: Vec<&str> = url.split("/").collect();
    if url_pieces.len() < 3 {
        return Err(format!("input url:{} is not a valid string, doesnt consist of protocol//domain pattern and as such has less than 3 pieces",url));
    }
    // ex  ["https:", "", "www.w3schools.com", "tags", "tag_cite.asp"]
    // idx:2 -> domain
    let (new_map, _) = begin_scape(url.clone(), url_pieces[2].to_string(), max_depth).await;
    let mut r = new_redis_instance().unwrap();
    update_cache(&mut r, &new_map);
    Ok(200)
}
async fn begin_scape(
    url: String,
    domain: String,
    max_depth: u8,
) -> (HashMap<String, Vec<String>>, BinaryHeap<String>) {
    let mut cur_depth = 0;
    let mut heap = BinaryHeap::new();
    let mut inverted_index: HashMap<String, Vec<String>> = HashMap::new();
    let base_p1 = format!("http://{}", domain);
    let base_p2 = format!("https://{}", domain);
    let mut seen_links: HashSet<String> = HashSet::new();
    heap.push(url);
    loop {
        if cur_depth == max_depth || heap.is_empty() {
            break;
        }
        cur_depth += 1;
        let new_site = heap.pop().unwrap();
        let (new_links, site_content) =
            parse_html_content(request_site(new_site.clone()).await).unwrap();
        let site_content = to_words(site_content);
        let new_links = new_links
            .into_iter()
            .filter(|l| l.starts_with(&base_p1) || l.starts_with(&base_p2))
            .collect::<Vec<_>>();
        for link in new_links {
            if seen_links.insert(link.clone()) {
                heap.push(link);
            }
        }
        update_mapping(&mut inverted_index, new_site.clone(), site_content.clone());
    }
    (inverted_index, heap)
}
fn update_mapping(
    input_map: &mut HashMap<String, Vec<String>>,
    source: String,
    words: Vec<String>,
) {
    //input_map.insert(, v)
    // table => [page1,page2]
    for word in words {
        if let Some(prev_map) = input_map.get_mut(&word) {
            if !prev_map.contains(&source) {
                prev_map.push(source.clone());
            }
        } else {
            input_map.insert(word, vec![source.clone()]);
        }
    }
}

fn to_words(sent: Vec<String>) -> Vec<String> {
    sent.into_iter()
        .flat_map(|s| {
            s.split_whitespace()
                .map(|word| word.to_string())
                .collect::<Vec<String>>()
        })
        .collect()
}

fn remove_duplicates(input: &mut Vec<String>) {
    use std::collections::HashSet;
    let unique: HashSet<_> = input.drain(..).collect();
    *input = unique.into_iter().collect();
}

async fn request_site(url: String) -> String {
    let client = Client::new();

    let response = client.get(url).send().await.unwrap();
    let resp_body = response.text().await.unwrap();
    resp_body
}

fn parse_html_content(content: String) -> Result<(Vec<String>, Vec<String>), String> {
    let document = scraper::Html::parse_document(&content);
    let selector = generate_parser("a[href]");

    let links = document.select(&selector);
    let valid_links: Vec<String> = links
        .into_iter()
        .filter_map(|link| link.value().attr("href").map(|s| s.to_string()))
        .collect();
    let valid_links: Vec<String> = valid_links
        .into_iter()
        .filter(|link| link.starts_with("https://") || link.starts_with("http://"))
        .collect();

    let mut extracted_text: Vec<String> = Vec::new();

    // Extract text from h1-h6 tags
    for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        let selector = generate_parser(tag);
        let elements = document.select(&selector);
        extracted_text.extend(elements.into_iter().filter_map(|el| {
            el.text()
                .next()
                .map(|s| s.trim().to_string())
                .map(|s| s.replace("\n", ""))
        }));
    }

    // Extract text from p tags
    let p_selector = generate_parser("p");
    let p_elements = document.select(&p_selector);
    extracted_text.extend(p_elements.into_iter().filter_map(|el| {
        el.text()
            .next()
            .map(|s| s.trim().to_string())
            .map(|s| s.replace("\n", ""))
    }));

    // Extract text from strong tags
    let strong_selector = generate_parser("strong");
    let strong_elements = document.select(&strong_selector);
    extracted_text.extend(strong_elements.into_iter().filter_map(|el| {
        el.text()
            .next()
            .map(|s| s.trim().to_string())
            .map(|s| s.replace("\n", ""))
    }));

    // Extract text from em tags
    let em_selector = generate_parser("em");
    let em_elements = document.select(&em_selector);
    extracted_text.extend(em_elements.into_iter().filter_map(|el| {
        el.text()
            .next()
            .map(|s| s.trim().to_string())
            .map(|s| s.replace("\n", ""))
    }));

    // Extract text from b tags
    let b_selector = generate_parser("b");
    let b_elements = document.select(&b_selector);
    extracted_text.extend(b_elements.into_iter().filter_map(|el| {
        el.text()
            .next()
            .map(|s| s.trim().to_string())
            .map(|s| s.replace("\n", ""))
    }));

    // Extract text from i tags
    let i_selector = generate_parser("i");
    let i_elements = document.select(&i_selector);
    extracted_text.extend(i_elements.into_iter().filter_map(|el| {
        el.text()
            .next()
            .map(|s| s.trim().to_string())
            .map(|s| s.replace("\n", ""))
    }));

    Ok((valid_links, extracted_text))
}
fn generate_parser(element: &str) -> scraper::Selector {
    scraper::Selector::parse(element).unwrap()
}
use redis::Commands;
fn new_redis_instance() -> Result<redis::Connection, String> {
    match redis::Client::open("redis://157.245.120.23:6379") {
        Ok(client) => match client.get_connection() {
            Ok(conn) => return Ok(conn),
            Err(e) => Err(format!("{:?}", e.to_string())),
        },
        Err(e) => Err(format!("{:?}", e.to_string())),
    }
}
fn update_cache(redis: &mut redis::Connection, hash_table: &HashMap<String, Vec<String>>) {
    // for each key (word) grab the existing index mapping
    // combine it with what we have in memory, deduplicate it and store again
    for (word, index_map) in hash_table {
        let result: redis::RedisResult<Vec<String>> = redis.lrange(word, 0, -1);
        match result {
            Ok(mut v) => {
                v.append(&mut index_map.clone());
                println!("(1)storing {:?}:\t{:?}", word.clone(), v.clone());
                remove_duplicates(&mut v);
                println!("(2)storing {:?}:\t{:?}", word.clone(), v.clone());
                // now set this in redis
                match redis.lpush::<_, _, i32>(word, v.clone()) {
                    Ok(_) => {}
                    Err(err) => {
                        println!(
                            "setting {} index to {:?} failed with this error {:?}",
                            word,
                            v.clone(),
                            err
                        );
                    }
                }
            }
            Err(k) => {
                println!("grabing {} failed with this error {:?}", word, k);
                continue;
            }
        }
    }
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
