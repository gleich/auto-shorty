use std::env;

use anyhow::{bail, Context};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

use crate::api::Social;

#[derive(Deserialize, Debug)]
pub struct Link {
	name: String,
	public: bool,
}

const SHORTY_TOKEN: &'static str = "SHORTY_TOKEN";
const SHORTY_URL: &'static str = "https://links.mattglei.ch";

pub fn get_links(client: &Client) -> anyhow::Result<Vec<Link>> {
	let shorty_token = env::var(SHORTY_TOKEN)?;
	// Making request
	let response = client
		.get(format!("{}/api/link", SHORTY_URL))
		.bearer_auth(&shorty_token)
		.send()
		.context("Failed to send request to get list of links")?;
	let status = response.status();
	if status != StatusCode::OK {
		bail!(
			"Request to get list of links failed with status code of {}",
			status
		);
	}

	// Parsing response
	let shorty_links: Value = serde_json::from_str(
		&response
			.text()
			.context("Failed to get returned response of request")?,
	)
	.context("Failed to parse response")?;

	// Collecting vector of Link
	let mut links: Vec<Link> = Vec::new();
	for link in shorty_links["data"].as_array().unwrap().iter() {
		links.push(
			serde_json::from_value(link.to_owned()).context("Failed to parse a specific link")?,
		)
	}

	Ok(links)
}

pub fn update_social_links(
	client: &Client,
	socials: Vec<Social>,
	links: Vec<Link>,
) -> anyhow::Result<()> {
	let shorty_token = env::var(SHORTY_TOKEN)?;
	for social in socials.iter() {
		// Formulating request based off link existence
		let mut exists = false;
		for link in links.iter() {
			exists = link.public && link.name == social.name;
		}
		let mut request = client.post(format!("{}/api/link", SHORTY_URL)).json(
			&json!({"url": social.url, "description": social.description, "public": true, "name": social.name}),
		);
		if exists {
			request = client
				.patch(format!("{}/api/link/{}", SHORTY_URL, social.name))
				.json(&json!({"url": social.url, "description": social.description}));
		}

		// Making request
		let response = request
			.bearer_auth(&shorty_token)
			.send()
			.context(format!("Failed to update link /{}", social.name))?;

		// Checking response status code
		let status = response.status();
		if status != StatusCode::OK {
			bail!(
				"Failed to update link of {} with status code of {}",
				social.name,
				status
			);
		}

		if exists {
			info!("Updated link for /{}", social.name)
		} else {
			info!("Created link at /{}", social.name)
		}
	}

	Ok(())
}
