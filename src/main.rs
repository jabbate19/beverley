mod stripe;
mod oidc;

use actix_web::*;
use std::{str, env};
use std::collections::HashMap;
use stripe::{Session, Event, check_signature};
use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    uid: String,
    cn: String,
    pub drinkBalance: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Credits {
    message: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreditUpdate {
    uid: String,
    drinkBalance: i64,
}

#[post("/hook")]
async fn index(request: HttpRequest, body: web::Bytes) -> impl Responder {    
    let sig_header = request.headers().get("stripe-signature").unwrap().to_str().unwrap();
    let mut sig_values = HashMap::<&str, &str>::new();
    sig_header.split(',').for_each(|value| {
        let mut key_val = value.split('=');
        sig_values.insert(key_val.next().unwrap(), key_val.next().unwrap());
    });
    match check_signature(sig_values.get("v1").unwrap(), sig_values.get("t").unwrap(), body.as_ref()) {
        true => {
            let event: Event = serde_json::from_str(str::from_utf8(&body).unwrap()).unwrap();
            let credits = event.data.object.amount_total.unwrap() as i64;
            let user = event.data.object.metadata.get("username").unwrap();
            match oidc::get_token (
                "https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/token",
                "beverley",
                &env::var("BEVERLEY_ACCOUNT_SECRET").unwrap()
            ).await {
                Ok(keycloak_token) => {
                    let client = reqwest::Client::new();
                    let get_req = client.get(format!("https://drink.csh.rit.edu/users/credits?uid={}", user))
                    .bearer_auth(&keycloak_token)
                    .send()
                    .await
                    .unwrap();
                    if get_req.status().is_success() {
                        let user_data: Credits = get_req.json().await.unwrap();
                        let new_data = CreditUpdate{
                            uid: user.to_owned(),
                            drinkBalance: user_data.user.drinkBalance.parse::<i64>().unwrap() + credits
                        };
                        let put_req = client.put("https://drink.csh.rit.edu/users/credits")
                        .bearer_auth(&keycloak_token)
                        .json(&new_data)
                        .send()
                        .await
                        .unwrap();
                        if put_req.status().is_success() {
                            return HttpResponse::Ok().finish();
                        } else {
                            let error = format!("Error on put request\n{}\n\n{}", put_req.status(), put_req.text().await.unwrap());
                            return HttpResponse::InternalServerError().body(error);
                        }
                    } else {
                        let error = format!("Error on get request\n{}\n\n{}", get_req.status(), get_req.text().await.unwrap());
                        return HttpResponse::InternalServerError().body(error);            
                    }
                },
                Err(_) => HttpResponse::InternalServerError().finish()
            }
        }
        false => HttpResponse::BadRequest().finish()
    }
}

#[derive(Deserialize)]
struct CheckoutData {
    amount: String,
    username: String
}

#[post("/stripe")]
async fn checkout(form: web::Form<CheckoutData>) -> impl Responder {
    let f = form.into_inner();
    let client = reqwest::Client::new();
    let server_name = env::var("SERVER_NAME").unwrap();
    let mut form_data = HashMap::from([
        ("success_url", format!("{}/success", server_name)),
        ("cancel_url", format!("{}/cancel", server_name)),
        ("line_items[0][price]", "price_1LfodZJNNb1UiFlPIf6rAxLi".to_string()),
        ("line_items[0][quantity]", f.amount),
        ("mode", "payment".to_string()),
        ("metadata[username]", f.username)
    ]);
    let req = client.post("https://api.stripe.com/v1/checkout/sessions")
        .basic_auth(&env::var("STRIPE_SECRET_KEY").unwrap(), None::<&str>)
        .form(&form_data)
        .send()
        .await
        .unwrap();
    let text = req.text().await.unwrap();
    let session: Session = serde_json::from_str(&text).unwrap();
    let url = session.url.unwrap();
    let mut response = HttpResponse::Found();
    response.insert_header(("LOCATION", url));
    response
        
}

#[get("/")]
async fn welcome() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/index.html"))
}


#[get("/success")]
async fn success() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/success.html"))
}


#[get("/cancel")]
async fn cancel() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/cancel.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(checkout)
            .service(welcome)
            .service(success)
            .service(cancel)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

