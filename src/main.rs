//#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use]
extern crate url;
use url::percent_encoding::{utf8_percent_encode, SIMPLE_ENCODE_SET};
define_encode_set! {
    /// This encode set is used in the URL parser for query strings.
    pub QUERY_ENCODE_SET = [SIMPLE_ENCODE_SET] | {' ', '"', '#', '<', '>', '}', '{'}
}

extern crate serde;
extern crate serde_json;

use serde_json::Value;
use std::fmt;

#[macro_use]
extern crate serde_derive;

extern crate reqwest;

#[derive(Serialize, Deserialize)]
struct SgId {
    id: u32,
    stype: String,
}

#[derive(Serialize, Deserialize)]
struct SgResponse<T> {
    errcode: u32,
    err: String,
    result: Vec<T>,
}

#[derive(Debug)]
struct SgError {
    details: String,
}

impl SgError {
    fn new(msg: &str) -> SgError {
        SgError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for SgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for SgError {
    fn description(&self) -> &str {
        &self.details
    }
}

fn build_py_list(vv: &Vec<Vec<&str>>) -> String {
    let mut r_str = "[".to_owned();

    for i in 0..vv.len() {
        r_str += "[";
        for j in 0..vv[i].len() {
            r_str += vv[i][j];
            if j != vv[i].len() - 1 {
                r_str += ",";
            }
        }
        r_str += "]";
    }

    r_str += "]";
    r_str
}

fn sg_find_one(
    stype: &str,
    vv: &Vec<Vec<&str>>,
) -> Result<SgResponse<SgId>, Box<std::error::Error>> {
    return sg_find(stype, vv, 1);
}

fn sg_find(
    stype: &str,
    vv: &Vec<Vec<&str>>,
    limit: u32,
) -> Result<SgResponse<SgId>, Box<std::error::Error>> {
    return sg_find_verbose(stype, vv, "", limit, false, 0, true);
}

//entity_type, filters, {fields=None}, {order=None}, filter_operator=None, limit=0, retired_only=False, page=0, include_archived_projects=True, additional_filter_presets=None
fn sg_find_verbose(
    stype: &str,
    vv: &Vec<Vec<&str>>,
    filter_operator: &str,
    limit: u32,
    retired_only: bool,
    page: u32,
    include_archived_projects: bool,
) -> Result<SgResponse<SgId>, Box<std::error::Error>> {
    let filts: String = build_py_list(vv);
    let mut url = "http://192.168.250.85:8001/?req=find".to_owned()
        + "&limit="
        + &*limit.to_string()
        + "&type="
        + stype
        + "&filters="
        + &filts;

    if filter_operator != "" {
        url.push_str(&*("&filteroperator=".to_owned() + filter_operator));
    } else if retired_only {
        url.push_str("&retiredonly=true");
    } else if page != 0 {
        url.push_str(&*("&page=".to_owned() + &*page.to_string()));
    } else if include_archived_projects {
        url.push_str("&includearchived=true");
    }

    url = utf8_percent_encode(&*url, QUERY_ENCODE_SET).collect::<String>();
    let mut res = reqwest::get(&url)?;
    println!("GET {:0}", url);

    let pos_text = &res.text()?;
    let pval: Value = serde_json::from_str(pos_text)?;

    if pval["errcode"] == 200 {
        let sgr: SgResponse<SgId> = serde_json::from_str(pos_text)?;
        return Ok(sgr);
    } else {
        let err_str = pval["err"].as_str().unwrap();
        return Err(Box::new(SgError::new(err_str)));
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    env_logger::init();

    let filters: Vec<Vec<&str>> = vec![vec![
        "\"project\"",
        "\"is\"",
        "{\"id\": 182,\"type\":\"Project\"}",
    ]];
    let sgr = sg_find_one("Shot", &filters);
    match sgr {
        Ok(v) => {
            println!(
                "Returned successfully\nERRCODE: {:0}   ERR: {:1}",
                v.errcode, v.err
            );
            for i in 0..v.result.len() {
                println!("TYPE: {:0}, ID: {:1}", v.result[i].stype, v.result[i].id);
            }
        }
        Err(e) => println!("rError: {:0}", e),
    }

    Ok(())
}
