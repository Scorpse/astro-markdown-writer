# Astro Markdown Writer

A small desktop app for publishing to a blog you own. It writes Markdown and images to your GitHub repository. [Astro Markdown Starter](https://github.com/Scorpse/astro-markdown-starter) builds the blog through GitHub Actions and serves it on GitHub Pages.

[Download the Windows installer](https://github.com/Scorpse/astro-markdown-writer/releases/latest) · [Download the Debian package](https://github.com/Scorpse/astro-markdown-writer/releases/download/v0.1.0/astro-markdown-writer_0.1.0_amd64.deb) · [Build on Linux](#linux) · [View the demo blog](https://scorpse.github.io/astro-markdown-writer-demo/)

## Start a blog

1. [Fork the starter](https://github.com/Scorpse/astro-markdown-starter/fork) into your account. You can also use the template's **Use this template** button to create an independent copy. A public repository works with GitHub Free. GitHub's Pages eligibility applies to private repositories.
2. Install and open Writer. Paste your new blog repository URL and a GitHub personal access token. Select **Connect**. The app holds the token in memory until it closes. [Windows installer](https://github.com/Scorpse/astro-markdown-writer/releases/latest) · [Linux installation](#install-on-linux) · [GitHub token guide](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
3. Select **Set up and deploy site**. Writer enables Pages with GitHub Actions, enables the starter workflow if needed, and queues the first deployment when your token and repository settings allow it. Select **Deployment runs** to see the build and **Open site** to visit your blog.
4. Write your first post and select **Publish post**. Writer adds the Markdown and referenced images in one commit on your default branch. A successful GitHub Actions run makes the post live. The app links to the commit and workflow run list.

To rename the blog or change its description, edit `src/site.ts` in your blog repository. You can edit the sample post or remove it after you have published your own.

## Token permissions

- **Fine-grained**. Select your blog repository and grant **Contents** read and write. To automate setup and deployment, also grant **Pages**, **Administration**, and **Actions** read and write.
- **Classic**. Use `public_repo` for publishing to a public repository. The Pages and Actions setup APIs require the `repo` scope. A classic token with `repo` has broader repository access. GitHub organization policies and approval requirements still apply.

You can use a publishing-only token and configure hosting in the browser. In your blog repo, go to **Settings → Pages**, select **GitHub Actions** as the source, and enable workflows from **Actions** if GitHub shows an enable button. Run **Build and publish** once. If a protected default branch requires a pull request, follow that repository's rules for adding the post. [GitHub Pages setup](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)

## Writing

The app keeps one working draft per connected repository on the device. It previews Markdown in the editor, and you can attach up to five PNG, JPEG, WebP, or GIF files with a combined size of 2 MiB. Add alt text in the Markdown. A post lives at `src/content/posts/<slug>/index.md` in your blog repo, alongside its images. To revise an existing published post, edit that file in GitHub. Writer v0.1 publishes new posts and protects existing slugs.

## Install on Linux

The amd64 package was tested on Debian 12 and Ubuntu 24.04. Download it,
then run this command in the download directory. [Download the Debian package](https://github.com/Scorpse/astro-markdown-writer/releases/download/v0.1.0/astro-markdown-writer_0.1.0_amd64.deb)

~~~sh
sudo apt install ./astro-markdown-writer_0.1.0_amd64.deb
~~~

Open **Astro Markdown Writer** from your application menu.

## Build from source

### Windows

The desktop app uses Tauri 2, Rust, and a plain HTML, CSS, and JavaScript frontend. Follow [Tauri's Windows prerequisites](https://v2.tauri.app/start/prerequisites/) to install Rust, the Visual Studio C++ build tools, and WebView2. Install Node.js with npm for the Tauri CLI.

```sh
npm ci
npm run dev
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run build -- -- --locked
```

The Windows installer is under `src-tauri/target/release/bundle/nsis/`. The app uses GitHub's API, so publishing from an installed copy needs neither a local checkout nor a local Git, Node, or Astro installation.

### Linux

Build on a Linux desktop with a current stable Rust toolchain and Node.js with
npm. On Debian 12 or Ubuntu 24.04, install Tauri's system dependencies. For
other distributions, use the package list for your distribution.
[Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux) ·
[Rust installation](https://www.rust-lang.org/tools/install) ·
[Node.js downloads](https://nodejs.org/en/download)

```sh
sudo apt update
sudo apt install build-essential pkg-config git curl wget file \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev

git clone https://github.com/Scorpse/astro-markdown-writer.git
cd astro-markdown-writer
npm ci
npm run build -- -- --locked
```

The Linux build creates a `.deb` package under
`src-tauri/target/release/bundle/deb/`. From the repository root, install
the package and open **Astro Markdown Writer** from your application menu.
[Tauri Debian packaging](https://v2.tauri.app/distribute/debian/)

```sh
sudo apt install ./src-tauri/target/release/bundle/deb/*.deb
```

On other Linux distributions, you can build and run the executable directly
after installing the system dependencies and running `npm ci`.

```sh
npm run build -- --no-bundle -- --locked
./src-tauri/target/release/astro-markdown-writer
```

Use `npm run dev` to run Writer while editing its source.

### Tests

The ignored GitHub smoke test publishes to the project's dedicated demo. It requires an authorized token in the `ASTRO_WRITER_DEMO_TOKEN` process environment. The ordinary test command runs without credentials.
