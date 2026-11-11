# Astro Markdown Writer

A small Windows desktop app for publishing to a blog you own. It writes Markdown and images to your GitHub repository. [Astro Markdown Starter](https://github.com/Scorpse/astro-markdown-starter) builds the blog through GitHub Actions and serves it on GitHub Pages.

[Download the Windows installer](https://github.com/Scorpse/astro-markdown-writer/releases/latest) · [View the demo blog](https://scorpse.github.io/astro-markdown-writer-demo/)

## Start a blog

1. [Fork the starter](https://github.com/Scorpse/astro-markdown-starter/fork) into your account. You can also use the template's **Use this template** button to create an independent copy. A public repository works with GitHub Free. GitHub's Pages eligibility applies to private repositories.
2. Install and open Astro Markdown Writer. Paste your new blog repository URL and a [GitHub personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens). Select **Connect**. The app holds the token in memory until it closes.
3. Select **Set up and deploy site**. Writer enables Pages with GitHub Actions, enables the starter workflow if needed, and queues the first deployment when your token and repository settings allow it. Select **Deployment runs** to see the build and **Open site** to visit your blog.
4. Write your first post and select **Publish post**. Writer adds the Markdown and referenced images in one commit on your default branch. A successful GitHub Actions run makes the post live. The app links to the commit and workflow run list.

To rename the blog or change its description, edit `src/site.ts` in your blog repository. You can edit the sample post or remove it after you have published your own.

## Token permissions

- **Fine-grained**. Select your blog repository and grant **Contents** read and write. To automate setup and deployment, also grant **Pages**, **Administration**, and **Actions** read and write.
- **Classic**. Use `public_repo` for publishing to a public repository. The Pages and Actions setup APIs require the `repo` scope. A classic token with `repo` has broader repository access. GitHub organization policies and approval requirements still apply.

You can use a publishing-only token and configure hosting in the browser. In your blog repo, go to **Settings → Pages**, select **GitHub Actions** as the source, and enable workflows from **Actions** if GitHub shows an enable button. Run **Build and publish** once. If a protected default branch requires a pull request, follow that repository's rules for adding the post. [GitHub Pages setup](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)

## Writing

The app keeps one working draft per connected repository on the device. It previews Markdown in the editor, and you can attach up to five PNG, JPEG, WebP, or GIF files with a combined size of 2 MiB. Add alt text in the Markdown. A post lives at `src/content/posts/<slug>/index.md` in your blog repo, alongside its images. To revise an existing published post, edit that file in GitHub. Writer v0.1 publishes new posts and protects existing slugs.

## Build from source

The desktop app uses Tauri 2, Rust, and a plain HTML, CSS, and JavaScript frontend. Follow [Tauri's Windows prerequisites](https://v2.tauri.app/start/prerequisites/) to install Rust, the Visual Studio C++ build tools, and WebView2. Install Node.js with npm for the Tauri CLI.

```sh
npm ci
npm run dev
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run build -- -- --locked
```

The Windows installer is under `src-tauri/target/release/bundle/nsis/`. The app uses GitHub's API, so publishing from an installed copy needs neither a local checkout nor a local Git, Node, or Astro installation.

The ignored GitHub smoke test publishes to the project's dedicated demo. It requires an authorized token in the `ASTRO_WRITER_DEMO_TOKEN` process environment. The ordinary test command runs without credentials.
