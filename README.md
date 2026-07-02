# Frenzy IAM (Identity & Access Management)

Frenzy IAM is a production-grade backend authorization and
authentication platform built in Rust. Inspired by systems like Auth0
and Keycloak, it answers three fundamental questions for any connected
application: _Who is the user? Which organization do they belong to?
What are they allowed to do?_

---

## 🚀 Features

- **Secure Authentication:** Argon2 password hashing and JWT-based
  access tokens.
- **Stateful Sessions:** Refresh token rotation and active session
  revocation.
- **Granular RBAC Engine:** Dynamic Role-Based Access Control allowing
  custom roles and permissions.
- **Multi-Tenant Organizations:** Users can create and manage isolated
  organizations.
- **Atomic Transactions:** Organization and Owner role bootstrapping
  via strict SQL transactions.
- **Audit Logging:** Internal tracking of critical user lifecycle
  events.

---

## 🛠️ Tech Stack

- **Language:** Rust
- **Web Framework:** Axum
- **Database:** PostgreSQL
- **Database Driver / Queries:** SQLx (Compile-time query
  verification)
- **Authentication:** jsonwebtoken (JWT), Argon2
- **Testing / API Client:** Bruno

---

## 📂 Project Architecture

The codebase enforces a strict separation of concerns, ensuring
handlers, middleware, and database access are decoupled and
maintainable.

```text
src/
├── config.rs              # Environment variable loading & DB connection pooling
├── errors.rs              # Centralized AppError enum with Axum IntoResponse mappings
├── main.rs                # Application entry point and server binding
├── state.rs               # Shared application state (DB Pool, Config)
├── handlers/
│   ├── audit.rs
│   ├── auth.rs
│   ├── memberships.rs
│   ├── organizations.rs
│   ├── permissions.rs
│   ├── roles.rs
│   ├── sessions.rs
│   └── users.rs
├── middleware/
│   └── auth.rs
├── models/
│   └── models.rs
├── repositories/
│   └── rbac.rs
└── services/
    └── jwt.rs
```

---

## ⚙️ Setup & Installation

### 1. Prerequisites

- **Rust** (latest stable)
- **PostgreSQL** running locally or via Docker
- **sqlx-cli** installed (`cargo install sqlx-cli`)

### 2. Environment Variables

```env
DATABASE_URL=postgres://postgres:password@localhost/frenzy_iam
JWT_SECRET=your_super_secret_cryptographic_key_here
JWT_ACCESS_EXPIRY_MINUTES=15
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

### 3. Database Migrations

```bash
sqlx database create
sqlx migrate run
```

### 4. Run the Server

```bash
cargo run
```

The server will start on `http://localhost:3000`.

---

## 🔐 Authorization Engine Details

The core of the application relies on the `has_permission` function
located in `repositories/rbac.rs`. Rather than hardcoding roles, the
system dynamically joins users → memberships → roles → permissions to
evaluate access in real-time.

When a user creates an Organization, a database transaction is opened.
The system seamlessly creates the organization, creates an Owner role,
binds wildcard permissions to that role, and maps the user to the
organization---all atomically.

---

## 🗺️ Roadmap & Future Enhancements

- **Machine-to-Machine API Keys:** Implementation of a hashed API key
  generation system for programmatic organization access, verified via
  an `X-API-Key` middleware.
- **Automated Testing Suite:**
  - **Unit Tests:** Validation of JWT cryptographic functions and
    Argon2 hashing.
  - **Integration Tests:** Using `sqlx::test` to spin up isolated
    Postgres instances for end-to-end router testing.
- **Redis Integration:** Caching `has_permission` queries in Redis to
  reduce database join load on highly trafficked protected routes.
