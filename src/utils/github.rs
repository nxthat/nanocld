use thiserror::Error;
use url::{ParseError, Url};
use serde::{Deserialize, Serialize};
use ntex::http::client::{Client, ClientRequest};

use crate::models::{GitRepositoryPartial, GitRepositoryItem};

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubRepoBranchCommit {
  pub(crate) sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubRepoBranch {
  pub(crate) name: String,
  pub(crate) commit: GithubRepoBranchCommit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubRepo {
  pub(crate) name: String,
  pub(crate) private: bool,
  pub(crate) full_name: String,
  pub(crate) default_branch: String,
}

#[derive(Debug)]
pub struct GitDesc {
  #[allow(dead_code)]
  pub(crate) host: String,
  pub(crate) path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubApiError {
  pub(crate) message: String,
}

#[derive(Error, Debug)]
pub enum GithubError {
  #[error("response error from api")]
  Errorgithubapi(GithubApiError),
}

pub fn parse_git_url(url: &str) -> Result<GitDesc, ParseError> {
  let url_parsed = Url::parse(url)?;

  let host = match url_parsed.host_str() {
    Some(host) => host,
    None => return Err(ParseError::EmptyHost),
  };

  let path = url_parsed.path();

  let result = GitDesc {
    host: host.to_string(),
    path: path.to_string(),
  };

  Ok(result)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BasicCredential {
  pub(crate) username: String,
  pub(crate) password: String,
}

#[derive(Clone)]
pub struct GithubApi {
  client: Client,
  base_url: String,
  pub credential: BasicCredential,
}

impl GithubApi {
  pub fn new(github_user: &str, github_token: &str) -> Self {
    log::info!("creating github api");
    let credential = BasicCredential {
      username: github_user.to_owned(),
      password: github_token.to_owned(),
    };
    let client = Client::build()
      .basic_auth(&credential.username, Some(&credential.password))
      .header("Accept", "application/vnd.github.v3+json")
      .header("User-Agent", "nanocl")
      .finish();
    GithubApi {
      client,
      credential,
      base_url: String::from("https://api.github.com"),
    }
  }

  fn gen_url(&self, url: String) -> String {
    self.base_url.to_owned() + &url
  }

  pub fn get(&self, url: String) -> ClientRequest {
    self.client.get(self.gen_url(url))
  }

  #[allow(dead_code)]
  pub fn post(&self, url: String) -> ClientRequest {
    self.client.post(self.gen_url(url))
  }

  pub async fn sync_repo(
    &self,
    item: &GitRepositoryPartial,
  ) -> Result<GithubRepo, Box<dyn std::error::Error + 'static>> {
    log::info!("syncing github repository {} {}", item.name, item.url);
    let git_desc = parse_git_url(&item.url)?;
    let url = "/repos".to_owned() + &git_desc.path;

    let mut res = self.get(url).send().await?;

    if res.status().is_client_error() || res.status().is_server_error() {
      let err = res.json::<GithubApiError>().await?;
      return Err(Box::new(GithubError::Errorgithubapi(err)));
    }

    let repo = res.json::<GithubRepo>().await.unwrap();

    Ok(repo)
  }

  pub async fn list_branches(
    &self,
    item: &GitRepositoryPartial,
  ) -> Result<Vec<GithubRepoBranch>, Box<dyn std::error::Error + 'static>> {
    let git_desc = parse_git_url(&item.url)?;

    let url = "/repos".to_owned() + &git_desc.path + "/branches";

    let mut res = self.get(url).send().await?;

    if res.status().is_client_error() {
      let err = res.json::<GithubApiError>().await?;
      return Err(Box::new(GithubError::Errorgithubapi(err)));
    }

    let body = res.json::<Vec<GithubRepoBranch>>().await?;
    Ok(body)
  }

  pub async fn inspect_branch(
    &self,
    item: &GitRepositoryItem,
    branch: &str,
  ) -> Result<GithubRepoBranch, Box<dyn std::error::Error + 'static>> {
    let git_desc = parse_git_url(&item.url)?;

    let url = "/repos".to_owned() + &git_desc.path + "/branches/" + branch;
    let mut res = self.get(url).send().await?;
    if res.status().is_client_error() {
      let err = res.json::<GithubApiError>().await?;
      return Err(Box::new(GithubError::Errorgithubapi(err)));
    }
    let body = res.json::<GithubRepoBranch>().await?;
    Ok(body)
  }
}

#[cfg(test)]
mod test_github {
  use crate::utils::test::*;
  use crate::models::GitRepositoryPartial;

  use super::*;

  // #[ntex::test]
  async fn list_repository_branches() -> TestReturn {
    let github_api = GithubApi::new("", "");
    let item = GitRepositoryPartial {
      name: String::from("express-test-deploy"),
      url: String::from("https://github.com/leon3s/express-test-deploy"),
    };
    let _branches = github_api.list_branches(&item).await?;
    let _ = github_api.sync_repo(&item).await?;
    Ok(())
  }
}
