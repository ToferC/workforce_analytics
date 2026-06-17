# Security Review Triage -- Workforce Analytics & Workforce Frontend

**Date:** 2026-06-17
**Scope:** `workforce_analytics` (Rust GraphQL API) + `workforce-frontend` (Rust web app)

---

## Executive Summary

A comprehensive security review of both applications identified **7 CRITICAL**, **17 HIGH**, **15 MEDIUM**, and **9 LOW** severity findings across authentication, authorization, input validation, secrets management, API security, infrastructure, and dependency health.

The most urgent issues are: fully permissive CORS on the API, insecure session cookies on the frontend, hardcoded/committed secrets, a static password salt, unauthenticated GraphQL queries exposing PII, and missing CSRF protection on the login form.

---

## Triage: Implementable Options by Priority

### TIER 1 -- Stop the Bleeding (Fix Immediately)

These are actively exploitable vulnerabilities that could lead to data breach or account takeover in a production deployment.

| # | Severity | App | Finding | File(s) | Effort |
|---|----------|-----|---------|---------|--------|
| 1 | CRITICAL | API | **Fully permissive CORS** -- `Cors::permissive()` disables same-origin policy; any website can make authenticated API calls on behalf of logged-in users | `graphql_api/src/main.rs:53` | Small |
| 2 | CRITICAL | Frontend | **Session cookies not marked Secure** -- `cookie_secure(false)` transmits session cookies (containing JWT) over plain HTTP | `src/main.rs:100` | Small |
| 3 | CRITICAL | Frontend | **JWT leaked in HTTP response header** -- Bearer token sent as custom response header on login redirect, visible to proxies/browser history | `src/handlers/authentication_hander.rs:135` | Small |
| 4 | CRITICAL | API | **Static salt for all password hashes** -- `PASSWORD_SECRET_KEY` env var used as salt for every Argon2 hash; identical passwords produce identical hashes | `graphql_api/src/models/auth.rs:104-118` | Medium |
| 5 | CRITICAL | API | **Hardcoded secrets committed to VCS** -- Base64 K8s secrets, plaintext ConfigMap with DB passwords, JWT keys, admin credentials all in the repo | `kubernetes/secrets.yaml`, `gke-k8s/api-config.yaml`, `docker-compose.yml` | Medium |
| 6 | CRITICAL | Frontend | **Login form has no CSRF protection** -- No token in form, no validation in handler; enables login CSRF attacks | `templates/authentication/log_in.html`, `src/handlers/authentication_hander.rs:33-99` | Medium |
| 7 | CRITICAL | API | **All GraphQL queries are unauthenticated** -- `allPeople`, `allOrganizations`, etc. have no guards; anyone can exfiltrate all data including PII (email, phone, address) | All files in `graphql_api/src/graphql/query/` | Large |

**Implementation plan for Tier 1:**

**1. Fix CORS (API)** -- Replace `Cors::permissive()` with explicit origin allowlist:
```rust
let cors = Cors::default()
    .allowed_origin("https://your-frontend-domain.com")
    .allowed_methods(vec!["GET", "POST"])
    .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
    .max_age(3600);
```
Consider loading allowed origins from an env var for flexibility.

**2. Secure session cookies (Frontend)** -- Change `cookie_secure(false)` to `cookie_secure(true)` and add SameSite:
```rust
.cookie_secure(true)
.cookie_same_site(actix_web::cookie::SameSite::Lax)
```

**3. Remove JWT from response header (Frontend)** -- Delete the `.append_header(("Bearer", login_data.bearer))` line from the login handler. The token is already stored server-side in the session.

**4. Fix password hashing salt (API)** -- Generate a unique random salt per password:
```rust
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
```
Existing hashes need a migration strategy (re-hash on next login).

**5. Remove committed secrets (API)** -- Delete `kubernetes/secrets.yaml` and `gke-k8s/api-config.yaml` from VCS. Add to `.gitignore`. Use Kubernetes Secrets (not ConfigMaps) with external secrets management. Rotate all exposed credentials.

**6. Add CSRF to login form (Frontend)** -- Add `csrf_token: String` to `LoginForm`, add hidden field to template, validate with `security::verify_csrf_token()` in the POST handler.

**7. Add auth guards to queries (API)** -- Add `#[graphql(guard = "RoleGuard::new(UserRole::User)")]` to all query resolvers. PII-heavy queries (persons) should require `Analyst` or higher.

---

### TIER 2 -- Harden Authentication & Session Security (This Sprint)

| # | Severity | App | Finding | File(s) | Effort |
|---|----------|-----|---------|---------|--------|
| 8 | HIGH | API | **Weak/placeholder JWT secret** -- `32CHARSECRETKEY` is easily guessable; attacker can forge tokens | `graphql_api/src/models/auth.rs:19-22`, K8s configs | Small |
| 9 | HIGH | API | **No rate limiting** -- Unlimited login attempts; no brute-force protection | No middleware configured | Medium |
| 10 | HIGH | Frontend | **No session regeneration after login** -- Session fixation possible | `src/handlers/authentication_hander.rs:100-136` | Small |
| 11 | HIGH | Frontend | **Logout via GET** -- CSRF-vulnerable; `<img>` tag can force logout | `src/handlers/authentication_hander.rs:139-155` | Small |
| 12 | HIGH | Frontend | **Panic on logout when identity is None** -- `unwrap()` on None crashes handler thread | `src/handlers/authentication_hander.rs:153` | Small |
| 13 | HIGH | API | **Outdated `jsonwebtoken` crate (v7.2)** -- Known algorithm confusion issues; current is v9.x | `graphql_api/Cargo.toml:22` | Medium |
| 14 | HIGH | Frontend | **Wrong auth header in affiliation.rs** -- Uses `.header("Bearer", bearer)` instead of `Authorization: Bearer` | `src/graphql/affiliation.rs:26,63` | Small |

**Implementation plan for Tier 2:**

**8.** Generate a cryptographically random 256-bit JWT secret and document that `32CHARSECRETKEY` must never be used in production.

**9.** Add `actix-governor` to `Cargo.toml` and configure per-IP rate limits, especially on the `sign_in` mutation.

**10.** Call `session.purge()` before storing login data, then re-insert CSRF token.

**11.** Change logout to POST with CSRF validation; update navbar template to use a form.

**12.** Replace `id.unwrap().logout()` with `if let Some(identity) = id { identity.logout(); }`.

**13.** Upgrade `jsonwebtoken` from `7.2.0` to `9.x`; update API calls for breaking changes; explicitly set `Algorithm::HS256` in validation.

**14.** Refactor `affiliation.rs` to use the centralized `post_graphql` client which correctly sets the `Authorization` header.

---

### TIER 3 -- Reduce Attack Surface (Next Sprint)

| # | Severity | App | Finding | File(s) | Effort |
|---|----------|-----|---------|---------|--------|
| 15 | HIGH | API | **GraphQL Playground exposed unconditionally** -- Interactive schema explorer available in production | `graphql_api/src/handlers/routes.rs:21` | Small |
| 16 | HIGH | API | **GraphQL introspection not disabled** -- Full schema discoverable by any client | `graphql_api/src/graphql/utilities.rs:37-50` | Small |
| 17 | HIGH | API | **No query complexity/depth limits** -- Recursive queries can exhaust server resources (DoS) | `graphql_api/src/graphql/utilities.rs:22-51` | Small |
| 18 | HIGH | API | **PII fields exposed without guards** -- email, phone, address, name accessible without auth | `graphql_api/src/models/person.rs:47-54,249,260` | Medium |
| 19 | HIGH | Frontend | **XSS in org_chart error handler** -- API error messages interpolated into raw HTML without escaping | `src/handlers/org_chart.rs:124,147` | Small |
| 20 | HIGH | Frontend | **No Content-Security-Policy header** -- XSS has unrestricted script execution | `src/main.rs` | Medium |
| 21 | HIGH | Frontend | **No clickjacking protection** -- No X-Frame-Options or frame-ancestors directive | `src/main.rs` | Small |
| 22 | HIGH | API | **Docker runs as root** -- Container escape gives root access | `Dockerfile`, `Dockerfile.simple` | Small |
| 23 | HIGH | API | **No TLS on K8s ingress** -- JWT tokens travel in plaintext | `kubernetes/ingress.yaml`, `gke-k8s/ingress.yaml` | Medium |

**Implementation plan for Tier 3:**

**15-16.** Gate playground and introspection behind an `ENVIRONMENT` check:
```rust
if std::env::var("ENVIRONMENT").unwrap_or_default() != "production" {
    config.route("/playground", web::get().to(playground_handler));
}
// In schema build:
.disable_introspection() // or conditionally based on environment
```

**17.** Add to `Schema::build()`: `.limit_depth(10).limit_complexity(1000)`

**18.** Add `#[graphql(guard = "RoleGuard::new(UserRole::Analyst)", visible = "is_analyst")]` to PII fields on `Person`. Uncomment guards on `family_name()` and `given_name()`.

**19.** HTML-escape error messages: `html_escape::encode_text(&e.to_string())` or use Tera template for error rendering.

**20.** Add CSP header via Actix middleware with script-src, style-src allowlists. Move inline scripts to external files or use per-request nonces.

**21.** Add `X-Frame-Options: DENY` and `Content-Security-Policy: frame-ancestors 'none'` headers.

**22.** Add `USER rusty` to `Dockerfile` and `Dockerfile.simple`, matching `Dockerfile.slim`.

**23.** Add TLS sections to ingress manifests; use cert-manager for automatic certificate provisioning.

---

### TIER 4 -- Defense in Depth & Hygiene (Backlog)

| # | Severity | App | Finding | File(s) | Effort |
|---|----------|-----|---------|---------|--------|
| 24 | MEDIUM | API | **JWT tokens logged to stdout** -- Authorization headers and decoded tokens printed | `graphql_api/src/models/auth.rs:57,73`, `graphql_api/src/graphql/mutation/user_mutation.rs:160` | Small |
| 25 | MEDIUM | API | **Admin password hash in startup logs** -- Debug output of User struct | `graphql_api/src/database.rs:48,63` | Small |
| 26 | MEDIUM | Frontend | **Debug println! leaks session data** -- Role, user_id, full HTTP requests, API errors logged | Multiple files (see review) | Small |
| 27 | MEDIUM | API | **sslmode=disable on DB connections** -- Database traffic unencrypted | `docker-compose.yml`, K8s configs | Small |
| 28 | MEDIUM | API | **Unbounded list queries** -- `allPeople` etc. return full datasets; `get_count` accepts unbounded i64 | All query resolvers | Medium |
| 29 | MEDIUM | API | **No input length validation** -- String fields accept unlimited length | All InputObject structs | Medium |
| 30 | MEDIUM | API | **No K8s resource limits** -- Runaway queries can consume all node resources | K8s deployment manifests | Small |
| 31 | MEDIUM | API | **PostgreSQL uses `latest` tag** -- Version can change without notice | `docker-compose.yml:5` | Small |
| 32 | MEDIUM | Frontend | **CSRF token never rotates** -- Same token valid for entire session | `src/security.rs:104-122` | Small |
| 33 | MEDIUM | Frontend | **Bearer token in capability search response** -- JWT leaked in response header | `src/handlers/capability.rs:75` | Small |
| 34 | MEDIUM | Frontend | **Timezone mismatch in session expiry** -- NaiveDateTime compared against local clock | `src/security.rs:90-99` | Small |
| 35 | MEDIUM | Frontend | **CDN resources without SRI** -- Bootstrap Icons and ECharts loaded without integrity checks | `templates/base.html:18,33` | Small |
| 36 | MEDIUM | Frontend | **No HSTS header** -- First-time visitors vulnerable to SSL stripping | `src/main.rs` | Small |
| 37 | MEDIUM | Frontend | **Wildcard dependency versions** -- `chrono = "*"`, `fluent-templates = "*"` | `Cargo.toml:31,49` | Small |
| 38 | MEDIUM | Frontend | **`.expect()` panics in affiliation.rs** -- API null data crashes handler threads | `src/graphql/affiliation.rs:42-43,78-79` | Small |

---

### TIER 5 -- Low Priority / Informational

| # | Severity | App | Finding | File(s) | Effort |
|---|----------|-----|---------|---------|--------|
| 39 | LOW | API | **Unused `alcoholic_jwt` dependency** -- Increases attack surface | `graphql_api/Cargo.toml:51` | Small |
| 40 | LOW | API | **Database dump in repo** -- `mydb.dump` may contain PII | `mydb.dump` | Small |
| 41 | LOW | API | **`imagePullPolicy: Always` with `latest` tag** -- Deployed image can change without explicit deployment | `kubernetes/deployment.yaml:35` | Small |
| 42 | LOW | Frontend | **CSRF comparison not timing-safe** -- Standard `==` used | `src/security.rs:126-131` | Small |
| 43 | LOW | Frontend | **Hardcoded reCAPTCHA site key** -- Not environment-configurable | `templates/authentication/log_in.html:40` | Small |
| 44 | LOW | Frontend | **Unmaintained `dotenv` crate** -- Should migrate to `dotenvy` | `Cargo.toml:30` | Small |
| 45 | LOW | Frontend | **`not_authorized` route never registered** -- Users get 404 instead of 403 page | `src/handlers/routes.rs:324-327` | Small |
| 46 | LOW | Frontend | **Missing X-Content-Type-Options header** | `src/main.rs` | Small |
| 47 | LOW | Frontend | **Outdated `rand` crate** | `Cargo.toml:37` | Small |

---

## Effort Estimates

| Effort | Description | Approx. Time |
|--------|-------------|--------------|
| Small | Config change, one-line fix, or flag toggle | < 1 hour |
| Medium | Code changes across 2-5 files, possible API changes | 2-8 hours |
| Large | Architectural change, touches many files, needs testing | 1-3 days |

## Recommended Implementation Order

1. **Tier 1** (items 1-7): Address immediately. These represent active exploitability.
2. **Tier 2** (items 8-14): Complete within the current sprint. These harden auth flows.
3. **Tier 3** (items 15-23): Next sprint. Reduces attack surface and adds defense in depth.
4. **Tier 4** (items 24-38): Backlog. Important hygiene items that reduce risk over time.
5. **Tier 5** (items 39-47): Low priority. Address opportunistically during related work.

## Positive Findings

- **No SQL injection risk**: All database queries use Diesel's parameterized query builder. No raw SQL found.
- **Password hash correctly excluded from GraphQL**: `#[graphql(skip)]` on `User.hash`.
- **Chart JSON sanitization**: `chart_json()` properly escapes `<` to `<` to prevent script injection.
- **Tera auto-escaping enabled by default**: Most template outputs are properly escaped.
- **CSRF infrastructure exists**: Token generation and verification functions are implemented; they just need consistent application.
- **Role-based access control infrastructure exists**: `RoleGuard`, `require_role()`, and role hierarchy are implemented; they need to be applied more consistently.
