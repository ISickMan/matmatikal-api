use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use jsonwebtoken::{
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;

const JWKS_REPLY: &str = r#"{
  "keys": [
    {
      "kid": "ac3e3e558111c7c7a75c5b65134d22f63ee006d0",
      "use": "sig",
      "alg": "RS256",
      "kty": "RSA",
      "e": "AQAB",
      "n": "puQJMii881LWwQ_OY2pOZx9RJTtpmUhAn2Z4_zrbQ9WmQqld0ufKesvwIAmuFIswzfOWxv1-ijZWwWrVafZ3MOnoB_UJFgjCPwJyfQiwwNMK80MfEm7mDO0qFlvrmLhhrYZCNFXYKDRibujCPF6wsEKcb3xFwBCH4UFaGmzsO0iJiqD2qay5rqYlucV4-kAIj4A6yrQyXUWWTlYwedbM5XhpuP1WxqO2rjHVLmwECUWqEScdktVhXXQ2CW6zvvyzbuaX3RBkr1w-J2U07vLZF5-RgnNjLv6WUNUwMuh-JbDU3tvmAahnVNyIcPRCnUjMk03kTqbSkZfu6sxWF0qNgw"
    },
    {
      "e": "AQAB",
      "use": "sig",
      "n": "uBHF-esPKiNlFaAvpdpejD4vpONW9FL0rgLDg1z8Q-x_CiHCvJCpiSehD41zmDOhzXP_fbMMSGpGL7R3duiz01nK5r_YmRw3RXeB0kcS7Z9H8MN6IJcde9MWbqkMabCDduFgdr6gvH0QbTipLB1qJK_oI_IBfRgjk6G0bGrKz3PniQw5TZ92r0u1LM-1XdBIb3aTYTGDW9KlOsrTTuKq0nj-anW5TXhecuxqSveFM4Hwlw7pw34ydBunFjFWDx4VVJqGNSqWCfcERxOulizIFruZIHJGkgunZnB4DF7mCZOttx2dwT9j7s3GfLJf0xoGumqpOMvecuipfTPeIdAzcQ",
      "alg": "RS256",
      "kid": "a3b762f871cdb3bae0044c649622fc1396eda3e3",
      "kty": "RSA"
    }
  ]
}"#;
const CLIENT_ID : &'static str = "794981933073-c59qh87r995625mjrk4iph5m89cd9s03.apps.googleusercontent.com";

#[derive(Debug, Deserialize)]
struct GoogleLoginJwt {
    email: String,
    name: String,
}

#[post("/google-login")]
pub async fn google_login(req_body: String) -> impl Responder {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://accounts.google.com"]); // Issuer for Google One Tap
    validation.set_audience(&[
       CLIENT_ID,
    ]);

    let credential = req_body;
    let header = jsonwebtoken::decode_header(&credential).unwrap();

    let Some(kid) = header.kid else {
        return panic!("Token doesn't have a `kid` header field");
    };

    let jwks: JwkSet = serde_json::from_str(JWKS_REPLY).unwrap();
    let Some(jwk) = jwks.find(&kid) else {
        return panic!("No matching JWK found for the given kid");
    };

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap(),
        _ => unreachable!("algorithm should be a RSA in this example"),
    };

    let data =
        jsonwebtoken::decode::<GoogleLoginJwt>(&credential, &decoding_key, &validation)
            .unwrap();

    println!("body: {:#?}", data);
    HttpResponse::Ok().body(credential.clone())
}