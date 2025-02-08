use reqwest::Error;
use serde::{Deserialize, Serialize};
use warp::Filter;
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
struct RajaOngkirResponse {
    rajaongkir: RajaOngkirData,
}

#[derive(Deserialize, Serialize, Debug)]
struct RajaOngkirData {
    results: Vec<City>,
}

#[derive(Deserialize, Serialize, Debug)]
struct City {
    city_id: String,
    city_name: String,
    province_id: String,
    province: String,
    #[serde(rename = "type")]
    city_type: String,
    postal_code: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CostRequest {
    origin: String,
    destination: String,
    weight: u32,
    courier: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CostResponse {
    rajaongkir: CostData,
}

#[derive(Deserialize, Serialize, Debug)]
struct CostData {
    results: Vec<CourierCost>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CourierCost {
    code: String,
    name: String,
    costs: Vec<CostDetail>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CostDetail {
    service: String,
    description: String,
    cost: Vec<CostValue>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CostValue {
    value: u32,
    etd: String,
    note: String,
}

async fn get_cities(api_key: &str) -> Result<Vec<City>, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.rajaongkir.com/starter/city")
        .header("key", api_key)
        .send()
        .await?;

    let response: RajaOngkirResponse = res.json().await?;
    Ok(response.rajaongkir.results)
}

async fn calculate_cost(api_key: &str, request: CostRequest) -> Result<Vec<CourierCost>, Error> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.rajaongkir.com/starter/cost")
        .header("key", api_key)
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&[
            ("origin", request.origin),
            ("destination", request.destination),
            ("weight", request.weight.to_string()),
            ("courier", request.courier),
        ])
        .send()
        .await?;

    let response: CostResponse = res.json().await?;
    Ok(response.rajaongkir.results)
}

#[tokio::main]
async fn main() {
    let api_key = Arc::new("9341fe1bc60292d9c9a79d9aabe66110".to_string());

    let cities_route = warp::path("cities")
        .and(warp::get())
        .and(with_api_key(api_key.clone()))
        .and_then(get_cities_handler);

    let cost_route = warp::path("cost")
        .and(warp::post())
        .and(with_api_key(api_key.clone()))
        .and(warp::body::json())
        .and_then(calculate_cost_handler);
    
    let cors = warp::cors()
        .allow_any_origin()  // Mengizinkan semua domain
        .allow_methods(vec!["GET", "POST"]) // Metode yang diizinkan
        .allow_headers(vec!["Content-Type", "key"]); // Header yang diizinkan

    let routes = cities_route.or(cost_route).with(cors);

    warp::serve(routes)
        .run(([127.0.0.1], 3030))
        .await;
}

fn with_api_key(api_key: Arc<String>) -> impl Filter<Extract = (Arc<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || api_key.clone())
}

async fn get_cities_handler(api_key: Arc<String>) -> Result<impl warp::Reply, warp::Rejection> {
    match get_cities(&api_key).await {
        Ok(cities) => Ok(warp::reply::json(&cities)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn calculate_cost_handler(api_key: Arc<String>, request: CostRequest) -> Result<impl warp::Reply, warp::Rejection> {
    match calculate_cost(&api_key, request).await {
        Ok(costs) => Ok(warp::reply::json(&costs)),
        Err(_) => Err(warp::reject::not_found()),
    }
}