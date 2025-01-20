use jwt_simple::prelude::*;

use crate::User;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_server";

pub struct EncodingKey(Ed25519KeyPair);

#[allow(unused)]
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, jwt_simple::Error> {
        let user: User = user.into();
        let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));

        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);

        self.0.sign(claims)
    }
}

#[allow(unused)]
impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let opts = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };

        let claims = self.0.verify_token::<User>(token, None)?;

        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> Result<(), jwt_simple::Error> {
        let encoding_pem = include_str!("../../fixture/encoding.pem");
        let decoding_pem = include_str!("../../fixture/decoding.pem");

        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "zhangsan", "zhangsan@qq.com");

        /*
        eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3MzY5MTcyNTYsImV4cCI6MTczNzUyMjA1NiwibmJmIjoxNzM2OTE3MjU2LCJpc3MiOiJjaGF0X3NlcnZlciIsImF1ZCI6ImNoYXRfc2VydmVyIiwiaWQiOjEsImZ1bGxuYW1lIjoiemhhbmdzYW4iLCJlbWFpbCI6InpoYW5nc2FuQHFxLmNvbSIsImNyZWF0ZWRfYXQiOiIyMDI1LTAxLTE1VDA1OjAwOjU2LjA5MDEwMDUwMFoifQ.UbrvRKdl7mbM_dYYNRxpb1MAIZlhBWgJbBp87vAO4Y4tpur-FFOjhEP1gheP-HJVQ85EXrpHowHmjf55mrEnAg
         */
        let token = ek.sign(user.clone())?;

        let user2 = dk.verify(&token)?;

        assert_eq!(user, user2);

        Ok(())
    }
}
