use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
    Captcha,
}

impl TokenType {
    fn as_str(self) -> &'static str {
        match self {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
            TokenType::Captcha => "captcha",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "access" => Some(TokenType::Access),
            "refresh" => Some(TokenType::Refresh),
            "captcha" => Some(TokenType::Captcha),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub jti: Option<String>,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct ParsedToken {
    pub user_id: Uuid,
    pub role: String,
    pub kind: TokenType,
    pub jti: Option<String>,
    pub exp: i64,
}

pub struct JwtManager {
    encoding: EncodingKey,
    decoding: DecodingKey,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl JwtManager {
    pub fn new(secret: String, access_ttl: Duration, refresh_ttl: Duration) -> Self {
        let bytes = secret.into_bytes();
        Self {
            encoding: EncodingKey::from_secret(&bytes),
            decoding: DecodingKey::from_secret(&bytes),
            access_ttl,
            refresh_ttl,
        }
    }

    pub fn refresh_ttl(&self) -> Duration {
        self.refresh_ttl
    }

    pub fn generate_access_token(&self, user_id: Uuid, role: &str) -> Result<(String, i64)> {
        let now = Utc::now();
        let exp = now + chrono::Duration::from_std(self.access_ttl)?;
        let claims = Claims {
            sub: user_id.to_string(),
            role: Some(role.to_owned()),
            kind: TokenType::Access.as_str().to_owned(),
            jti: None,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)?;
        Ok((token, self.access_ttl.as_secs() as i64))
    }

    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<(String, String, i64)> {
        let now = Utc::now();
        let exp = now + chrono::Duration::from_std(self.refresh_ttl)?;
        let jti = Uuid::new_v4().to_string();
        let claims = Claims {
            sub: user_id.to_string(),
            role: None,
            kind: TokenType::Refresh.as_str().to_owned(),
            jti: Some(jti.clone()),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)?;
        Ok((token, jti, exp.timestamp()))
    }

    pub fn generate_captcha_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = now + chrono::Duration::minutes(5);
        let claims = Claims {
            sub: user_id.to_string(),
            role: None,
            kind: TokenType::Captcha.as_str().to_owned(),
            jti: None,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        Ok(encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)?)
    }

    pub fn parse_token(&self, token: &str) -> Result<ParsedToken> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["sub", "exp"]);
        validation.leeway = 5;
        let data = decode::<Claims>(token, &self.decoding, &validation)?;
        let user_id = Uuid::parse_str(&data.claims.sub)
            .map_err(|e| anyhow!("invalid sub claim: {e}"))?;
        let kind = TokenType::from_str(&data.claims.kind)
            .ok_or_else(|| anyhow!("invalid token type"))?;
        Ok(ParsedToken {
            user_id,
            role: data.claims.role.unwrap_or_default(),
            kind,
            jti: data.claims.jti,
            exp: data.claims.exp,
        })
    }

    pub fn parse_captcha_token(&self, token: &str) -> Result<Uuid> {
        let parsed = self.parse_token(token)?;
        if parsed.kind != TokenType::Captcha {
            return Err(anyhow!("invalid token type"));
        }
        Ok(parsed.user_id)
    }
}
