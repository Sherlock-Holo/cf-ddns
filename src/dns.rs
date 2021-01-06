use std::str::FromStr;

use anyhow::{Context, Error, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::config::Config;

#[derive(Debug, Serialize, Copy, Clone, Deserialize)]
pub enum DnsType {
    A,
    AAAA,
    /*    CNAME,
    TXT,
    SRV,
    LOC,
    MX,
    NS,
    SPF,
    CERT,
    DNSKEY,
    DS,
    NAPTR,
    SMIMEA,
    SSHFP,
    TLSA,
    URI,*/
}

impl FromStr for DnsType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DnsType::*;

        match s {
            "A" => Ok(A),
            "AAAA" => Ok(AAAA),
            /*            "CNAME" => Ok(CNAME),
            "TXT" => Ok(TXT),
            "SRV" => Ok(SRV),
            "LOC" => Ok(LOC),
            "MX" => Ok(MX),
            "NS" => Ok(NS),
            "SPF" => Ok(SPF),
            "CERT" => Ok(CERT),
            "DNSKEY" => Ok(DNSKEY),
            "DS" => Ok(DS),
            "NAPTR" => Ok(NAPTR),
            "SMIMEA" => Ok(SMIMEA),
            "SSHFP" => Ok(SSHFP),
            "TLSA" => Ok(TLSA),
            "URI" => Ok(URI),*/
            _ => Err(anyhow::format_err!("unsupported dns type {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDnsRecordsResponse {
    success: bool,
    errors: Vec<JsonValue>,
    messages: Vec<JsonValue>,
    result: Vec<DnsRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnsRecord {
    id: String,
    #[serde(rename = "type")]
    type_field: String,
    name: String,
    content: String,
    proxiable: bool,
    proxied: bool,
    ttl: i64,
    locked: bool,
    zone_id: String,
    zone_name: String,
    created_on: String,
    modified_on: String,
    meta: DnsMeta,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateOrUpdateDnsRecordResponse {
    success: bool,
    errors: Vec<JsonValue>,
    messages: Vec<JsonValue>,
    result: Option<DnsRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnsMeta {
    auto_added: bool,
}

#[derive(Debug, Deserialize)]
struct ListZonesResponse {
    success: bool,
    errors: Vec<JsonValue>,
    messages: Vec<JsonValue>,
    result: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
    name: String,
    account: Account,
    betas: Option<Vec<String>>,
    created_on: String,
    deactivation_reason: Option<String>,
    development_mode: i64,
    host: Option<HostingPartner>,
    meta: ZoneMeta,
    modified_on: String,
    name_servers: Vec<String>,
    original_dnshost: Option<String>,
    original_name_servers: Option<Vec<String>>,
    original_registrar: Option<String>,
    owner: Owner,
    paused: bool,
    permissions: Vec<String>,
    plan: Option<Plan>,
    plan_pending: Option<Plan>,
    status: Status,
    activated_on: String,
    #[serde(rename = "type")]
    zone_type: ZoneType,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "status", rename_all = "lowercase")]
enum Status {
    Active,
    Pending,
    Initializing,
    Moved,
    Deleted,
    Deactivated,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ZoneType {
    Full,
    Partial,
}

#[derive(Deserialize, Debug)]
struct HostingPartner {
    /// Host company name
    name: String,
    /// The host's website URL
    website: String,
}

#[derive(Deserialize, Debug)]
struct ZoneMeta {
    /// Maximum custom certificates that can be uploaded/used.
    custom_certificate_quota: u32,
    /// Maximum page rules that can be created.
    page_rule_quota: u32,
    /// Indicates whether wildcard DNS records can receive Cloudflare security and performance
    /// features
    wildcard_proxiable: bool,
    /// Indicates if URLs on the zone have been identified as hosting phishing content.
    phishing_detected: bool,
    /// Indicates whether the zone is allowed to be connected to multiple Railguns at once
    multiple_railguns_allowed: bool,
}

#[derive(Debug, Deserialize)]
struct Owner {
    id: String,
    email: String,
    #[serde(rename = "type")]
    type_field: String,
}

#[derive(Debug, Deserialize)]
struct Account {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Plan {
    id: String,
    name: String,
    price: i64,
    currency: String,
    frequency: String,
    legacy_id: String,
    is_subscribed: bool,
    can_subscribe: bool,
}

#[derive(Serialize, Debug, Deserialize)]
struct CreateOrUpdateDnsRecordRequest {
    #[serde(rename = "type")]
    dns_type: DnsType,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

impl CreateOrUpdateDnsRecordRequest {
    fn new(
        dns_type: DnsType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> Self {
        Self {
            dns_type,
            name,
            content,
            ttl: match ttl {
                Some(ttl) => ttl,
                None => 1,
            },
            proxied: match proxied {
                Some(proxied) => proxied,
                None => false,
            },
        }
    }
}

pub struct Client {
    client: reqwest::Client,
    cfg: Config,
}

impl Client {
    pub fn new(cfg: Config) -> Self {
        Client {
            client: reqwest::Client::new(),
            cfg,
        }
    }

    pub async fn run(&self, content: &str) -> Result<()> {
        let domain_zone_id = if let Some(domain_zone_id) = self.get_domain_zone_id().await? {
            domain_zone_id
        } else {
            return Err(anyhow::anyhow!("domain {} not found", self.cfg.domain));
        };

        match self.get_dns_record_id(&domain_zone_id).await? {
            Some(dns_record_id) => {
                self.update_dns_record(&domain_zone_id, &dns_record_id, content)
                    .await
            }

            None => self.create_dns_record(&domain_zone_id, content).await,
        }
    }

    async fn get_domain_zone_id(&self) -> Result<Option<String>> {
        const LIST_ZONE_API: &str = "https://api.cloudflare.com/client/v4/zones";

        let resp = self
            .client
            .get(LIST_ZONE_API)
            .header("X-Auth-Email", &self.cfg.email)
            .header("X-Auth-Key", &self.cfg.auth_key)
            .send()
            .await
            .context("send get domain zone id request failed")?;

        check_status_code(resp.status())?;

        let resp: ListZonesResponse = resp
            .json()
            .await
            .context("get domain zone id response failed")?;

        if !resp.success {
            return Err(anyhow::anyhow!(
                "get domain zone id failed: {:?}",
                resp.errors
            ));
        }

        Ok(resp.result.into_iter().find_map(|result| {
            if &self.cfg.domain == &result.name {
                Some(result.id)
            } else {
                None
            }
        }))
    }

    async fn get_dns_record_id(&self, domain_zone_id: &str) -> Result<Option<String>> {
        const GET_DNS_RECORD_API: &str =
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records";

        let url = GET_DNS_RECORD_API.replace("{}", domain_zone_id);

        let resp = self
            .client
            .get(&url)
            .header("X-Auth-Email", &self.cfg.email)
            .header("X-Auth-Key", &self.cfg.auth_key)
            .query(&vec![
                ("type", format!("{:?}", self.cfg.dns_type)),
                ("name", self.cfg.name.to_string()),
            ])
            .send()
            .await
            .context("send get dns record id request failed")?;

        check_status_code(resp.status())?;

        let dns_records: ListDnsRecordsResponse = resp
            .json()
            .await
            .context("get dns record id response failed")?;

        if !dns_records.success {
            return Err(anyhow::anyhow!(
                "get dns record id failed: {:?}",
                dns_records.errors
            ));
        }

        match dns_records
            .result
            .iter()
            .find(|result| result.name == self.cfg.name)
        {
            None => Ok(None),
            Some(result) => Ok(Some(result.id.to_string())),
        }
    }

    async fn create_dns_record(&self, domain_zone_id: &str, content: &str) -> Result<()> {
        const CREATE_DNS_RECORD_API: &str =
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records";

        let url = CREATE_DNS_RECORD_API.replace("{}", domain_zone_id);

        let resp = self
            .client
            .post(&url)
            .header("X-Auth-Email", &self.cfg.email)
            .header("X-Auth-Key", &self.cfg.auth_key)
            .json(&CreateOrUpdateDnsRecordRequest::new(
                self.cfg.dns_type,
                self.cfg.name.to_string(),
                content.to_string(),
                self.cfg.ttl,
                self.cfg.proxied,
            ))
            .send()
            .await
            .context("send create dns record id request failed")?;

        check_status_code(resp.status())?;

        let resp: CreateOrUpdateDnsRecordResponse = resp
            .json()
            .await
            .context("get create record id response failed")?;

        if !resp.success {
            return Err(anyhow::anyhow!(
                "create dns record id failed: {:?}",
                resp.errors
            ));
        }

        match resp.result {
            None => Err(anyhow::anyhow!("no dns record found")),
            Some(_result) => Ok(()),
        }
    }

    async fn update_dns_record(
        &self,
        domain_zone_id: &str,
        dns_record_id: &str,
        content: &str,
    ) -> Result<()> {
        const UPDATE_DNS_RECORD_API: &str =
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{dns_record_id}";

        let url = UPDATE_DNS_RECORD_API
            .replace("{zone_id}", domain_zone_id)
            .replace("{dns_record_id}", dns_record_id);

        let resp = self
            .client
            .put(&url)
            .header("X-Auth-Email", &self.cfg.email)
            .header("X-Auth-Key", &self.cfg.auth_key)
            .json(&CreateOrUpdateDnsRecordRequest::new(
                self.cfg.dns_type,
                self.cfg.name.to_string(),
                content.to_string(),
                self.cfg.ttl,
                self.cfg.proxied,
            ))
            .send()
            .await
            .context("send update dns record id request failed")?;

        check_status_code(resp.status())?;

        let resp: CreateOrUpdateDnsRecordResponse = resp
            .json()
            .await
            .context("get update record id response failed")?;

        if !resp.success {
            return Err(anyhow::anyhow!(
                "update dns record id failed: {:?}",
                resp.errors
            ));
        }

        Ok(())
    }
}

fn check_status_code(status: StatusCode) -> Result<()> {
    if status != StatusCode::OK {
        let err = anyhow::anyhow!(
            "response status code is {}, expect {}",
            status,
            StatusCode::OK
        );
        log::error!("{}", err);
        return Err(err);
    }

    Ok(())
}
