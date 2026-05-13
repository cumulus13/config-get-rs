use regex::Regex;
use serde::Deserialize;
use std::process::Command;

use crate::error::Result;
use crate::icons;

#[derive(Debug, Deserialize)]
struct GitHubPR {
    number: u64,
    title: String,
    html_url: String,
    draft: Option<bool>,
    user: GitHubUser,
}

#[derive(Debug, Deserialize)]
struct GitHubIssue {
    number: u64,
    title: String,
    html_url: String,
    pull_request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    description: Option<String>,
    stargazers_count: Option<u64>,
    forks_count: Option<u64>,
    open_issues_count: Option<u64>,
    default_branch: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

pub struct RemoteInfo {
    client: reqwest::Client,
}

impl RemoteInfo {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("gits/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn show(&self, input: &str, cwd: &str) -> Result<()> {
        let (owner, repo) = self.parse_remote(input, cwd)?;

        let repo_slug = format!("{}/{}", owner, repo);
        
        print!("{} ", icons::Icons::REMOTE);
        println!("https://github.com/{}", repo_slug);

        // Fetch repo info
        if let Ok(repo_info) = self.fetch_repo(&repo_slug).await {
            print!("   ★ Stars: {}   ", repo_info.stargazers_count.unwrap_or(0));
            print!("⑂ Forks: {}   ", repo_info.forks_count.unwrap_or(0));
            print!("● Open issues: {}   ", repo_info.open_issues_count.unwrap_or(0));
            println!("Default branch: {}", repo_info.default_branch.as_deref().unwrap_or("main"));
            
            if let Some(desc) = &repo_info.description {
                if !desc.is_empty() {
                    println!("   {}", desc);
                }
            }
        }

        // Fetch PRs
        println!();
        println!("{} Open Pull Requests", icons::Icons::PR);
        if let Ok(prs) = self.fetch_prs(&repo_slug).await {
            if prs.is_empty() {
                println!("   (none)");
            }
            for pr in prs {
                let draft = if pr.draft.unwrap_or(false) { " [draft]" } else { "" };
                println!("   #{} {}{} — @{}", 
                    pr.number, pr.title, draft, 
                    pr.user.login
                );
                println!("      {}", pr.html_url);
            }
        }

        // Fetch Issues
        println!();
        println!("{} Open Issues", icons::Icons::ISSUE);
        if let Ok(issues) = self.fetch_issues(&repo_slug).await {
            let filtered: Vec<_> = issues.iter().filter(|i| i.pull_request.is_none()).collect();
            if filtered.is_empty() {
                println!("   (none)");
            }
            for issue in filtered {
                println!("   #{} {}", issue.number, issue.title);
                println!("      {}", issue.html_url);
            }
        }

        Ok(())
    }

    fn parse_remote(&self, input: &str, cwd: &str) -> Result<(String, String)> {
        // Try HTTPS
        let re = Regex::new(r"https://github\.com/([^/]+)/([^/]+)").unwrap();
        if let Some(caps) = re.captures(input) {
            return Ok((caps[1].to_string(), caps[2].trim_end_matches(".git").to_string()));
        }

        // Try SSH
        let re = Regex::new(r"git@github\.com:([^/]+)/(.+)").unwrap();
        if let Some(caps) = re.captures(input) {
            return Ok((caps[1].to_string(), caps[2].trim_end_matches(".git").to_string()));
        }

        // Try owner/repo
        if input.contains('/') {
            let parts: Vec<&str> = input.splitn(2, '/').collect();
            return Ok((parts[0].to_string(), parts[1].trim_end_matches(".git").to_string()));
        }

        // Resolve via git remote
        let remote_name = if input.is_empty() { "origin" } else { input };
        let output = Command::new("git")
            .args(["remote", "get-url", remote_name])
            .current_dir(cwd)
            .output()?;

        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !url.is_empty() {
                return self.parse_remote(&url, cwd);
            }
        }

        Err(anyhow::anyhow!("Cannot resolve GitHub remote").into())
    }

    async fn fetch_repo(&self, slug: &str) -> Result<GitHubRepo> {
        let url = format!("https://api.github.com/repos/{}", slug);
        let resp = self.client.get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("GitHub API error: HTTP {}", resp.status()).into());
        }

        Ok(resp.json().await?)
    }

    async fn fetch_prs(&self, slug: &str) -> Result<Vec<GitHubPR>> {
        let url = format!("https://api.github.com/repos/{}/pulls?state=open&per_page=10", slug);
        let resp = self.client.get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        Ok(resp.json().await?)
    }

    async fn fetch_issues(&self, slug: &str) -> Result<Vec<GitHubIssue>> {
        let url = format!("https://api.github.com/repos/{}/issues?state=open&per_page=10", slug);
        let resp = self.client.get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        Ok(resp.json().await?)
    }
}

impl Default for RemoteInfo {
    fn default() -> Self {
        Self::new()
    }
}