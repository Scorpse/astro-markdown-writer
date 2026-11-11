use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::{blocking::Client, Method, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

const POST_ROOT: &str = "src/content/posts";
const WORKFLOW: &str = "actions/workflows/site.yml";
const IMAGE_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct Connection {
    client: Client,
    token: String,
    pub repo: String,
    pub branch: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteSetup {
    pub site_url: Option<String>,
    pub message: String,
}

#[derive(Deserialize)]
pub struct Attachment {
    name: String,
    data: String,
}

#[derive(Deserialize)]
pub struct Post {
    slug: String,
    title: String,
    description: String,
    date: String,
    tags: Vec<String>,
    body: String,
    images: Vec<Attachment>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub commit_url: String,
    pub slug: String,
}

struct Failure {
    status: Option<StatusCode>,
    message: String,
}

impl From<Failure> for String {
    fn from(failure: Failure) -> String {
        failure.message
    }
}

fn repository_name(input: &str) -> Result<String, String> {
    let url = Url::parse(input.trim()).map_err(|_| "Enter a GitHub repository URL.")?;
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Use https://github.com/OWNER/REPOSITORY.".into());
    }
    let mut parts = url
        .path()
        .trim_end_matches('/')
        .trim_start_matches('/')
        .split('/');
    let owner = parts.next().unwrap_or_default();
    let repo = parts.next().unwrap_or_default().trim_end_matches(".git");
    let valid = |part: &str| {
        !part.is_empty()
            && part != "."
            && part != ".."
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    };
    if !valid(owner) || !valid(repo) || parts.next().is_some() {
        return Err("Use https://github.com/OWNER/REPOSITORY.".into());
    }
    Ok(format!("{owner}/{repo}"))
}

fn field<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("GitHub returned an incomplete response ({pointer})."))
}

fn markdown(post: &Post) -> Result<String, String> {
    let valid_slug = !post.slug.is_empty()
        && post.slug.len() <= 100
        && post.slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        });
    if !valid_slug {
        return Err("Use lowercase letters, digits, and single hyphens in the slug.".into());
    }
    if post.title.trim().is_empty()
        || post.description.trim().is_empty()
        || post.body.trim().is_empty()
    {
        return Err("A title, description, and Markdown body are required.".into());
    }
    let date = post.date.as_bytes();
    if date.len() != 10
        || date[4] != b'-'
        || date[7] != b'-'
        || date
            .iter()
            .enumerate()
            .any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit())
    {
        return Err("Choose a date in YYYY-MM-DD format.".into());
    }
    Ok(format!(
        "---\ntitle: {}\ndescription: {}\npubDate: {}\ntags: {}\n---\n\n{}\n",
        json!(post.title.trim()),
        json!(post.description.trim()),
        post.date,
        json!(post.tags),
        post.body.trim_end()
    ))
}

fn image_bytes(image: &Attachment) -> Result<Vec<u8>, String> {
    let (stem, ext) = image
        .name
        .rsplit_once('.')
        .ok_or("An image needs a file extension.")?;
    if stem.is_empty()
        || stem.len() > 100
        || !stem.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || image.data.len() > IMAGE_LIMIT.div_ceil(3) * 4
    {
        return Err("An image has an invalid name or is too large.".into());
    }
    let data = STANDARD
        .decode(&image.data)
        .map_err(|_| "An image could not be read.")?;
    let valid = match ext {
        "png" => data.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" => data.starts_with(b"\xff\xd8\xff"),
        "webp" => data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP"),
        "gif" => data.starts_with(b"GIF89a") || data.starts_with(b"GIF87a"),
        _ => false,
    };
    if !valid {
        return Err("Use PNG, JPEG, WebP, or GIF images.".into());
    }
    Ok(data)
}

impl Connection {
    pub fn connect(url: &str, token: String) -> Result<Self, String> {
        let repo = repository_name(url)?;
        let token = token.trim().to_owned();
        if token.is_empty() || token.len() > 1024 || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err("Enter a GitHub personal access token.".into());
        }
        let client = Client::builder()
            .user_agent("Astro-Markdown-Writer")
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(8))
            .timeout(Duration::from_secs(25))
            .build()
            .map_err(|_| "Could not start the GitHub connection.")?;
        let mut connection = Self {
            client,
            token,
            repo,
            branch: String::new(),
        };
        connection.request(Method::GET, "/user", None)?;
        let repository = connection.get("")?;
        connection.repo = field(&repository, "/full_name")?.to_owned();
        connection.branch = field(&repository, "/default_branch")?.to_owned();
        connection.get("/contents/.github/workflows/site.yml").map_err(|error| {
            if error.status == Some(StatusCode::NOT_FOUND) {
                "This repository is missing the starter’s Pages workflow. Fork the Astro Markdown Starter first.".into()
            } else {
                error.message
            }
        })?;
        Ok(connection)
    }

    fn request(&self, method: Method, path: &str, data: Option<Value>) -> Result<Value, Failure> {
        let request = self
            .client
            .request(method, format!("https://api.github.com{path}"))
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10");
        let request = match data {
            Some(data) => request.json(&data),
            None => request,
        };
        let response = request.send().map_err(|_| Failure {
            status: None,
            message: "Could not reach GitHub. Check your connection and retry.".into(),
        })?;
        let status = response.status();
        if status == StatusCode::NO_CONTENT {
            return Ok(Value::Null);
        }
        let value = response.json::<Value>().unwrap_or(Value::Null);
        if !status.is_success() {
            let detail = value["message"]
                .as_str()
                .unwrap_or("Request failed.")
                .replace(&self.token, "[redacted]");
            return Err(Failure {
                status: Some(status),
                message: format!("GitHub HTTP {status}: {detail}"),
            });
        }
        Ok(value)
    }

    fn repo_request(
        &self,
        method: Method,
        path: &str,
        data: Option<Value>,
    ) -> Result<Value, Failure> {
        self.request(method, &format!("/repos/{}{path}", self.repo), data)
    }

    fn get(&self, path: &str) -> Result<Value, Failure> {
        self.repo_request(Method::GET, path, None)
    }

    pub fn setup_pages(&self) -> Result<SiteSetup, String> {
        let pages = match self.get("/pages") {
            Ok(pages) => pages,
            Err(error) if error.status == Some(StatusCode::NOT_FOUND) => self
                .repo_request(
                    Method::POST,
                    "/pages",
                    Some(json!({ "build_type": "workflow" })),
                )
                .map_err(|e| format!("Open Pages settings to finish setup. {}", e.message))?,
            Err(error) => {
                return Err(format!(
                    "Open Pages settings to finish setup. {}",
                    error.message
                ))
            }
        };
        if field(&pages, "/build_type")? != "workflow" {
            return Err(
                "Set the Pages source to GitHub Actions in Pages settings, then retry.".into(),
            );
        }
        let site_url = pages["html_url"].as_str().map(str::to_owned);
        let deploy = (|| -> Result<(), String> {
            let workflow = self.get(&format!("/{WORKFLOW}"))?;
            if field(&workflow, "/state")? != "active" {
                self.repo_request(Method::PUT, &format!("/{WORKFLOW}/enable"), None)?;
            }
            self.repo_request(
                Method::POST,
                &format!("/{WORKFLOW}/dispatches"),
                Some(json!({"ref": self.branch})),
            )?;
            Ok(())
        })();
        let message = match deploy {
            Ok(()) => "GitHub Pages is configured and the deployment was queued. Follow it in Deployment runs.".into(),
            Err(error) => format!("GitHub Pages is configured. Open Deployment runs to enable or run the workflow. {error}"),
        };
        Ok(SiteSetup { site_url, message })
    }

    pub fn publish(&self, post: Post) -> Result<Publication, String> {
        let source = markdown(&post)?;
        let post_dir = format!("{POST_ROOT}/{}", post.slug);
        let path = format!("{post_dir}/index.md");
        let mut images = Vec::new();
        let mut bytes = 0;
        for image in post
            .images
            .iter()
            .filter(|image| source.contains(&format!("](./{})", image.name)))
        {
            let data = image_bytes(image)?;
            bytes += data.len();
            if images.len() == 5 || bytes > IMAGE_LIMIT {
                return Err("Use at most five images with a combined size of 2 MiB.".into());
            }
            if images.iter().any(|(name, _)| name == &image.name) {
                return Err("Two images have the same file name.".into());
            }
            images.push((image.name.clone(), &image.data));
        }
        let branch = url_path_segment(&self.branch);
        let head = self.get(&format!("/git/ref/heads/{branch}"))?;
        let head_sha = field(&head, "/object/sha")?;
        match self.get(&format!("/contents/{path}?ref={head_sha}")) {
            Ok(existing) => {
                let encoded = existing["content"]
                    .as_str()
                    .unwrap_or_default()
                    .replace('\n', "");
                if STANDARD
                    .decode(encoded)
                    .is_ok_and(|data| data == source.as_bytes())
                {
                    return Ok(Publication {
                        commit_url: format!(
                            "https://github.com/{}/blob/{head_sha}/{path}",
                            self.repo
                        ),
                        slug: post.slug,
                    });
                }
                return Err(
                    "That slug already exists on the blog. Choose a different slug.".into(),
                );
            }
            Err(error) if error.status == Some(StatusCode::NOT_FOUND) => {}
            Err(error) => return Err(error.into()),
        }
        let parent = self.get(&format!("/git/commits/{head_sha}"))?;
        let mut entries =
            vec![json!({"path":path, "mode":"100644", "type":"blob", "content":source})];
        for (name, data) in images {
            let blob = self.repo_request(
                Method::POST,
                "/git/blobs",
                Some(json!({
                    "content": data, "encoding": "base64"
                })),
            )?;
            entries.push(json!({
                "path": format!("{post_dir}/{name}"), "mode":"100644", "type":"blob",
                "sha": field(&blob, "/sha")?
            }));
        }
        let tree = self.repo_request(
            Method::POST,
            "/git/trees",
            Some(json!({
                "base_tree": field(&parent, "/tree/sha")?, "tree": entries
            })),
        )?;
        let commit = self.repo_request(
            Method::POST,
            "/git/commits",
            Some(json!({
                "message": format!("Publish {}", post.slug),
                "parents": [head_sha], "tree": field(&tree, "/sha")?
            })),
        )?;
        let sha = field(&commit, "/sha")?;
        self.repo_request(Method::PATCH, &format!("/git/refs/heads/{branch}"), Some(json!({
            "sha": sha, "force": false
        }))).map_err(|e| format!("GitHub could not update the branch. Check write access and branch rules, or retry if the branch just changed. {}", e.message))?;
        Ok(Publication {
            commit_url: format!("https://github.com/{}/commit/{sha}", self.repo),
            slug: post.slug,
        })
    }
}

fn url_path_segment(value: &str) -> String {
    value
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_github_repository_urls() {
        assert_eq!(
            repository_name("https://github.com/Scorpse/my-blog.git/").unwrap(),
            "Scorpse/my-blog"
        );
        for url in [
            "https://github.com.evil.test/x/y",
            "https://github.com/x/y/issues",
            "https://token@github.com/x/y",
        ] {
            assert!(repository_name(url).is_err());
        }
    }

    #[test]
    fn writes_starter_frontmatter_without_yaml_injection() {
        let post = Post {
            slug: "first-post".into(),
            title: "Title: \"quoted\"".into(),
            description: "Line one\nLine two".into(),
            date: "2026-11-11".into(),
            tags: vec!["astro".into()],
            body: "A paragraph.".into(),
            images: vec![],
        };
        let md = markdown(&post).unwrap();
        assert!(md.contains("title: \"Title: \\\"quoted\\\"\""));
        assert!(md.contains("description: \"Line one\\nLine two\""));
        assert!(md.ends_with("---\n\nA paragraph.\n"));
    }

    #[test]
    #[ignore = "creates a deployment and publishes to the dedicated GitHub Pages demo"]
    fn publishes_to_github_pages_demo() {
        let token = std::env::var("ASTRO_WRITER_DEMO_TOKEN")
            .expect("supply a demo token in the environment");
        let connection = Connection::connect(
            "https://github.com/Scorpse/astro-markdown-writer-demo",
            token,
        )
        .expect("connect demo");
        let setup = connection.setup_pages().expect("configure GitHub Pages");
        assert!(
            setup.message.contains("deployment was queued"),
            "{}",
            setup.message
        );
        let image = Attachment {
            name: "demo-pixel.png".into(),
            data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aHn8AAAAASUVORK5CYII=".into(),
        };
        let publication = connection.publish(Post {
            slug: "published-from-writer".into(),
            title: "Published from Astro Markdown Writer".into(),
            description: "A live check of the GitHub Pages publishing workflow.".into(),
            date: "2026-11-11".into(),
            tags: vec!["astro".into(), "markdown".into()],
            body: "This post was committed by [Astro Markdown Writer](https://github.com/Scorpse/astro-markdown-writer). GitHub Actions built it and deployed it to Pages.\n\n![Test image](./demo-pixel.png)".into(),
            images: vec![image],
        }).expect("publish demo post");
        println!("Publication: {}", publication.commit_url);
        println!("Pages: {}", setup.site_url.expect("GitHub Pages URL"));
    }
}
