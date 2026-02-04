# CloudSubmarine

A Rust port of [osteele/subdomain-router](https://github.com/osteele/subdomain-router) focused on traffic proxying for Cloudflare Workers. It does not support redirects, it only proxies requests.

## Configuration

The worker requires a `ROUTES` environment variable (or secret) containing a JSON map of path prefixes to downstream URLs.

Example `wrangler.toml` configuration (for local dev):

```toml
[vars]
ROUTES = '{"/api/": "https://api.example.com/", "/blog/": "https://blog.example.com/"}'
```

## Deployment

1. **Install Dependencies**: Ensure you have `cargo` and `npm` installed.
2. **Build & Deploy**:
   ```bash
   npx wrangler deploy
   ```

## Development

Run locally:
```bash
npx wrangler dev
```
