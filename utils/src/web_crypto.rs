#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Boolean, Object, Uint8Array, JSON};

use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

use wasm_bindgen_futures::JsFuture;

use web_sys::{window, CryptoKey, CryptoKeyPair, SubtleCrypto};

use rexie::{ObjectStore, Rexie, TransactionMode};

#[derive(Clone)]
pub struct WebCryptoContext {
    subtle: SubtleCrypto,
    db: Rexie,
    store_name: String,
    db_key: JsValue,
}

impl WebCryptoContext {
    /// Create a new CryptoKey in the Browser and then save it to indexedDB.
    pub async fn new(store_name: String) -> Self {
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
            .add_object_store(
                ObjectStore::new(&store_name)
                    .key_path("name")
                    .auto_increment(true),
            )
            .build()
            .await
            .unwrap_throw();

        let transaction = rexie
            .transaction(&[&store_name], TransactionMode::ReadWrite)
            .unwrap_throw();

        let db_store = transaction.store(&store_name).unwrap_throw();

        let db_key = db_store.add(&key_pair, None).await.unwrap_throw();

        transaction.done().await.unwrap_throw();

        Self {
            subtle,
            db: rexie,
            store_name,
            db_key,
        }
    }

    async fn get_key_pair(&self) -> CryptoKeyPair {
        let transaction = self
            .db
            .transaction(&[&self.store_name], TransactionMode::ReadOnly)
            .unwrap_throw();

        let db_store = transaction.store(&self.store_name).unwrap_throw();

        db_store
            .get((&self.db_key).into())
            .await
            .unwrap_throw()
            .unchecked_into()
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

    /// Sign the message then return the signature
    pub async fn sign(&self, mut msg: Vec<u8>) -> Vec<u8> {
        let algorithm = self.ecdsa_params();

        let key_pair = self.get_key_pair().await;
        let private_key = self.get_privkey(&key_pair);

        let promise = self
            .subtle
            .sign_with_object_and_u8_array(&algorithm, &private_key, &mut msg)
            .unwrap_throw();

        let result = JsFuture::from(promise).await.unwrap_throw();
        let buffer: Uint8Array = result.unchecked_into();

        //use p256::ecdsa::signature::Signature;
        //Signature::from_bytes(&vec)

        buffer.to_vec()
    }

    /// Verify that the signature match the message.
    pub async fn verify(&self, mut msg: Vec<u8>, mut sig: Vec<u8>) -> bool {
        let algorithm = self.ecdsa_params();

        let key_pair = self.get_key_pair().await;
        let public_key = self.get_pubkey(&key_pair);

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
}

/* #[async_trait]
impl AsyncBlockSigner<Signature> for WebSigner {
    fn algorithm(&self) -> AlgorithmType {
        AlgorithmType::ES256
    }

    async fn web_key(&self) -> JsonWebKey {
        let window = window().unwrap_throw();
        let crypto = window.crypto().unwrap_throw();
        let subtle = crypto.subtle();

        let pubkey = self.get_pubkey(self.get_key_pair());

        let promise = subtle.export_key("jwk", &pubkey).unwrap_throw();
        let result = JsFuture::from(promise).await.unwrap_throw();

        let js_string = JSON::stringify(&result).unwrap_throw();

        serde_json::from_str(&js_string.into()).unwrap()
    }
} */
