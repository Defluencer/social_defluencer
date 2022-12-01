#![cfg(target_arch = "wasm32")]

//use dag_jose::{AlgorithmType, JsonWebKey};

use js_sys::{Array, Boolean, Object, Uint8Array, JSON};

use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

use wasm_bindgen_futures::JsFuture;

use web_sys::{window, CryptoKey, CryptoKeyPair, SubtleCrypto};

use rexie::{ObjectStore, Rexie, TransactionMode};

const STORE_NAME: &str = "key_pairs";

#[derive(Clone)]
pub struct WebCryptoContext {
    pub subtle: SubtleCrypto,
    pub db_key: String,
    pub key_pair: CryptoKeyPair,
}

impl WebCryptoContext {
    /// Load a CryptoKeyPair from the Browser saved in indexedDB under the key db_key.
    pub async fn load(db_key: String) -> Self {
        let window = window().unwrap_throw();
        let crypto = window.crypto().unwrap_throw();
        let subtle = crypto.subtle();

        let rexie = Rexie::builder("defluencer")
            .version(1)
            .add_object_store(ObjectStore::new(STORE_NAME).key_path("name"))
            .build()
            .await
            .unwrap_throw();

        let transaction = rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .unwrap_throw();

        let db_store = transaction.store(STORE_NAME).unwrap_throw();

        let key_pair: CryptoKeyPair = db_store
            .get((&JsValue::from(db_key.clone())).into())
            .await
            .unwrap_throw()
            .unchecked_into();

        Self {
            subtle,
            db_key,
            key_pair,
        }
    }

    /// Create a new CryptoKey in the Browser and then save it to indexedDB under the key db_key.
    pub async fn new(db_key: String) -> Self {
        let window = window().unwrap_throw();
        let crypto = window.crypto().unwrap_throw();
        let subtle = crypto.subtle();

        let algorithm = JSON::parse(r#"{ name: "ECDSA", namedCurve: "P-256" }"#).unwrap_throw();
        let algorithm = Object::from(algorithm);

        let key_usages = Array::of2(&JsValue::from_str("sign"), &JsValue::from_str("verify"));

        let promise = subtle
            .generate_key_with_object(&algorithm, false, &key_usages)
            .unwrap_throw();

        let key_pair: CryptoKeyPair = JsFuture::from(promise)
            .await
            .unwrap_throw()
            .unchecked_into();

        let rexie = Rexie::builder("defluencer")
            .version(1)
            .add_object_store(ObjectStore::new(STORE_NAME).key_path("name"))
            .build()
            .await
            .unwrap_throw();

        let transaction = rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .unwrap_throw();

        let db_store = transaction.store(STORE_NAME).unwrap_throw();

        db_store
            .add(&key_pair, Some(&JsValue::from(db_key.clone())))
            .await
            .unwrap_throw();

        transaction.done().await.unwrap_throw();

        Self {
            subtle,
            db_key,
            key_pair,
        }
    }

    fn get_pubkey(&self, key_pair: &CryptoKeyPair) -> CryptoKey {
        js_sys::Reflect::get(&key_pair, &JsValue::from("publicKey"))
            .unwrap_throw()
            .unchecked_into()
    }

    fn get_privkey(&self, key_pair: &CryptoKeyPair) -> CryptoKey {
        js_sys::Reflect::get(&key_pair, &JsValue::from("privateKey"))
            .unwrap_throw()
            .unchecked_into()
    }

    fn ecdsa_params(&self) -> Object {
        let algorithm = JSON::parse(r#"{ name: "ECDSA", hash: {name: "SHA-256"} }"#).unwrap_throw();

        Object::from(algorithm)
    }

    /// Hash then sign the message and return the signature.
    pub async fn sign(&self, mut msg: Vec<u8>) -> Vec<u8> {
        let algorithm = self.ecdsa_params();

        let private_key = self.get_privkey(&self.key_pair);

        let promise = self
            .subtle
            .sign_with_object_and_u8_array(&algorithm, &private_key, &mut msg)
            .unwrap_throw();

        let result = JsFuture::from(promise).await.unwrap_throw();
        let buffer: Uint8Array = result.unchecked_into();

        buffer.to_vec()
    }

    /// Verify that the signature match the message.
    pub async fn verify(&self, mut msg: Vec<u8>, mut sig: Vec<u8>) -> bool {
        let algorithm = self.ecdsa_params();

        let public_key = self.get_pubkey(&self.key_pair);

        let promise = self
            .subtle
            .verify_with_object_and_u8_array_and_u8_array(
                &algorithm,
                &public_key,
                &mut sig,
                &mut msg,
            )
            .unwrap_throw();
        let result = JsFuture::from(promise).await.unwrap_throw();

        Boolean::from(result).value_of()
    }

    /*  fn algorithm(&self) -> AlgorithmType {
        AlgorithmType::ES256
    } */

    /* async fn web_key(&self) -> JsonWebKey {
        let pubkey = self.get_pubkey(&self.key_pair);

        let promise = self.subtle.export_key("jwk", &pubkey).unwrap_throw();
        let result = JsFuture::from(promise).await.unwrap_throw();

        let js_string = JSON::stringify(&result).unwrap_throw();
        let string: String = (&js_string).into();

        serde_json::from_str(&string).unwrap()
    } */
}
