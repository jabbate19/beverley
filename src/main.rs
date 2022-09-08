mod stripe;

use actix_web::*;
use std::str;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::collections::HashMap;
use stripe::{Session, check_signature};
use serde::Deserialize;
use reqwest;


#[post("/hook")]
async fn index(request: HttpRequest, body: web::Bytes) -> impl Responder {    
    let sig_header = request.headers().get("stripe-signature").unwrap().to_str().unwrap();
    let mut sig_values = HashMap::<&str, &str>::new();
    sig_header.split(',').for_each(|value| {
        let mut key_val = value.split('=');
        sig_values.insert(key_val.next().unwrap(), key_val.next().unwrap());
    });
    format!("{}", check_signature(sig_values.get("v1").unwrap(), sig_values.get("t").unwrap(), body.as_ref()))
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
    let mut form_data = HashMap::new();
    form_data.insert("success_url","https://anakin.csh.rit.edu/success");
    form_data.insert("cancel_url","https://anakin.csh.rit.edu/cancel");
    form_data.insert("line_items[0][price]","");
    form_data.insert("line_items[0][quantity]", &f.amount);
    form_data.insert("mode", "payment");
    form_data.insert("metadata[username]", &f.username);
    let req = client.post("https://api.stripe.com/v1/checkout/sessions")
        .basic_auth("", None::<&str>)
        .form(&form_data)
        .send()
        .await
        .unwrap();
    println!("{}", &req.status());
    let text = req.text().await.unwrap();
    println!("{}", text);
    let session: Session = serde_json::from_str(&text).unwrap();
    println!("{:?}", session);
    let url = session.url.unwrap();
    HttpResponse::Found()
        .body(url)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(checkout)
    })
    .bind(("0.0.0.0", 8080))?
    .bind_openssl("0.0.0.0:8081", builder)?
    .run()
    .await
}
