use sha2::Sha256;
use hmac::{Hmac, Mac};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::env;

pub fn check_signature(sig: &str, time: &str, body: &[u8]) -> bool {
    let webhook_key = &env::var("WEBHOOK_KEY").unwrap();
    let mut crypto = Hmac::<Sha256>::new_from_slice(webhook_key.as_bytes()).unwrap();
    let signed_payload = format!("{}.", time);
    crypto.update([signed_payload.as_bytes(), body].concat().as_ref());
    format!("{:02x}", &crypto.finalize().into_bytes()) == sig
}

#[derive(Serialize, Deserialize, Debug)]
struct AutoTax {
    enabled: bool,
    status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PhoneNumberCollection {
    enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct TotalDetails {
    amount_discount: u32,
    amount_shipping: u32,
    amount_tax: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Address {
    city: Option<String>,
    country: Option<String>,
    line1: Option<String>,
    line2: Option<String>,
    postal_code: Option<String>,
    state: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomerDetails {
    address: Address,
    email: Option<String>,
    name: Option<String>,
    phone: Option<String>,
    tax_exempt: Option<String>,
    tax_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    id: String,
    object: String,
    after_expiration: Option<String>,
    allow_promotion_codes: Option<String>,
    amount_subtotal: Option<u32>,
    pub amount_total: Option<u32>,
    automatic_tax: AutoTax,
    billing_address_collection: Option<String>,
    cancel_url: String,
    client_reference_id: Option<String>,
    consent: Option<String>,
    consent_collection: Option<String>,
    currency: Option<String>,
    customer: Option<String>,
    customer_creation: Option<String>,
    customer_details: Option<CustomerDetails>,
    customer_email: Option<String>,
    expires_at: u64,
    livemode: bool,
    locale: Option<String>,
    pub metadata: HashMap<String, String>,
    mode: String,
    payment_intent: String,
    payment_link: Option<String>,
    payment_method_collection: Option<String>,
    payment_method_options: Option<HashMap<String, String>>,
    payment_method_types: Option<Vec<String>>,
    payment_status: String,
    phone_number_collection: Option<PhoneNumberCollection>,
    recovered_from: Option<String>,
    setup_intent: Option<String>,
    shipping_addresss_collection: Option<String>,
    shipping_cost: Option<String>,
    shipping_details: Option<String>,
    shipping_options: Option<Vec<String>>,
    status: String,
    submit_type: Option<String>,
    subscription: Option<String>,
    success_url: String,
    total_details: Option<TotalDetails>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data{
    pub object: Session,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request{
    id: Option<String>,
    idempotency_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    id: String,
    object: String,
    api_version: String,
    created: u64,
    pub data: Data,
    livemode: bool,
    pending_webhooks: u16,
    request: Request,
    r#type: String,
}
