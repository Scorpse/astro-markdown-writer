const tauri = window.__TAURI__;
const invoke = tauri.core.invoke;
const openUrl = (url) => invoke("plugin:opener|open_url", { url }).catch(showError);
const byId = (id) => document.getElementById(id);
const starter = "https://github.com/Scorpse/astro-markdown-starter";
const fields = ["title", "slug", "description", "date", "tags", "body"];
const repoKey = "writer:repository";
let repo = localStorage.getItem(repoKey) || "";
let connected = null;
let images = [];
let busy = false;
let slugEdited = false;

const text = (id, value) => { byId(id).textContent = value; };
const showError = (error) => text("status", String(error));
const draftKey = () => "writer:draft:" + (repo || "local");
const dateToday = () => {
  const date = new Date();
  return [date.getFullYear(), date.getMonth() + 1, date.getDate()]
    .map((number) => String(number).padStart(2, "0")).join("-");
};
const makeSlug = (value) => value.toLowerCase().normalize("NFKD")
  .replace(/\p{M}/gu, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 100).replace(/-$/, "");

function saveDraft() {
  const values = Object.fromEntries(fields.map((id) => [id, byId(id).value]));
  try {
    localStorage.setItem(draftKey(), JSON.stringify({ values, images, slugEdited }));
    text("draft-state", "Draft saved on this device.");
  } catch {
    text("draft-state", "The draft exceeded local storage. Keep this window open and remove unused images or copy the text.");
  }
}

function loadDraft() {
  const draft = JSON.parse(localStorage.getItem(draftKey()) || "{}");
  for (const id of fields) byId(id).value = draft.values?.[id] || (id === "date" ? dateToday() : "");
  slugEdited = draft.slugEdited || false;
  images = draft.images || [];
  byId("preview-pane").hidden = true;
  text("draft-state", draft.values ? "Draft saved on this device." : "");
}

function applyControls() {
  byId("publish").disabled = busy || !connected;
  byId("deploy").disabled = busy || !connected;
  byId("connect").querySelector("button[type=submit]").disabled = busy;
  byId("site-controls").hidden = !connected;
}

async function run(message, action) {
  if (busy) return;
  busy = true;
  applyControls();
  text("status", message);
  try { await action(); }
  catch (error) { showError(error); }
  finally {
    busy = false;
    applyControls();
  }
}

function result(message, label, url) {
  text("status", message);
  if (label && url) {
    const link = document.createElement("button");
    link.type = "button";
    link.textContent = label + " ↗";
    link.onclick = () => openUrl(url);
    byId("status").append(" ", link);
  }
}

const githubRepo = () => "https://github.com/" + connected.repo;
byId("repo").value = repo;
byId("template").onclick = () => openUrl(starter + "/fork");
byId("token-guide").onclick = () => openUrl("https://github.com/Scorpse/astro-markdown-writer#token-permissions");
byId("pages").onclick = () => openUrl(githubRepo() + "/settings/pages");
byId("runs").onclick = () => openUrl(githubRepo() + "/actions/workflows/site.yml");
loadDraft();

byId("connect").addEventListener("submit", (event) => {
  event.preventDefault();
  const token = byId("token").value;
  const url = byId("repo").value;
  byId("token").value = "";
  run("Connecting to GitHub…", async () => {
    const connection = await invoke("connect", { url, token });
    connected = connection;
    const nextRepo = githubRepo();
    if (repo !== nextRepo) {
      const firstDraft = !repo && byId("title").value && !localStorage.getItem("writer:draft:" + nextRepo);
      repo = nextRepo;
      if (firstDraft) saveDraft();
      else loadDraft();
    }
    localStorage.setItem(repoKey, repo);
    byId("repo").value = repo;
    text("connected", connected.repo + " · " + connected.branch);
    text("status", "Connected. Set up the site or write your first post.");
  });
});

byId("deploy").onclick = () => run("Configuring GitHub Pages…", async () => {
  const site = await invoke("setup_pages");
  result(site.message, "Open site", site.siteUrl);
});

byId("new").onclick = () => {
  if (byId("title").value || byId("body").value) {
    if (!confirm("Replace the current local draft with a blank one?")) return;
  }
  localStorage.removeItem(draftKey());
  loadDraft();
  byId("title").focus();
};

byId("post").addEventListener("input", (event) => {
  if (event.target.id === "slug") slugEdited = !!byId("slug").value;
  if (event.target.id === "title" && !slugEdited) byId("slug").value = makeSlug(byId("title").value);
  saveDraft();
});

const extensions = {
  "image/png": "png", "image/jpeg": "jpg", "image/webp": "webp", "image/gif": "gif"
};
function readData(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result.split(",")[1]);
    reader.onerror = () => reject("Could not read " + file.name);
    reader.readAsDataURL(file);
  });
}
const references = (image) => byId("body").value.includes("](./" + image.name + ")");
byId("images").addEventListener("change", async (event) => {
  const input = event.target;
  const files = Array.from(input.files);
  input.value = "";
  if (!files.length) return;
  const existing = images.filter(references);
  if (files.length + existing.length > 5
    || files.reduce((size, file) => size + file.size, 0) + existing.reduce((size, image) => size + image.size, 0) > 2 * 1024 * 1024) {
    showError("Use at most five images with a combined size of 2 MiB.");
    return;
  }
  if (files.some((file) => !extensions[file.type])) {
    showError("Choose a PNG, JPEG, WebP, or GIF image.");
    return;
  }
  try {
    const added = await Promise.all(files.map(async (file) => ({
      name: crypto.randomUUID() + "." + extensions[file.type],
      data: await readData(file), type: file.type, size: file.size
    })));
    images = [...existing, ...added];
    byId("body").value += "\n\n" + added.map((image) => "![Describe this image](./" + image.name + ")").join("\n\n");
    saveDraft();
  } catch (error) { showError(error); }
});

byId("preview").onclick = async () => {
  try {
    const markup = await invoke("preview", { source: byId("body").value });
    const doc = new DOMParser().parseFromString(markup, "text/html");
    for (const link of doc.querySelectorAll("a")) link.removeAttribute("href");
    for (const img of doc.querySelectorAll("img")) {
      const src = img.getAttribute("src");
      const image = images.find((candidate) => "./" + candidate.name === src);
      if (image && extensions[image.type]) img.src = "data:" + image.type + ";base64," + image.data;
      else img.remove();
    }
    byId("preview-pane").replaceChildren(...doc.body.childNodes);
    byId("preview-pane").hidden = false;
    byId("preview-pane").scrollIntoView({ block: "nearest" });
  } catch (error) { showError(error); }
};

byId("post").addEventListener("submit", (event) => {
  event.preventDefault();
  if (!connected) return;
  saveDraft();
  const post = Object.fromEntries(fields.map((id) => [id, byId(id).value]));
  post.tags = post.tags.split(",").map((tag) => tag.trim()).filter(Boolean);
  post.images = images.filter(references).map(({ name, data }) => ({ name, data }));
  run("Committing your post to GitHub…", async () => {
    const published = await invoke("publish", { post });
    result("Post committed. Check Deployment runs for the GitHub Pages build.", "View on GitHub", published.commitUrl);
  });
});
