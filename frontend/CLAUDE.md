# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Epifront** is a Rust web application for organizational management in epidemiology/public health contexts. It manages people, roles, capabilities, teams, organizations, and related data within public health workflows.

## Technology Stack

- **Rust** (Edition 2021) with **Actix-Web 3.3.3** web framework
- **Diesel ORM** with **PostgreSQL** database
- **Tera** templating engine for HTML rendering
- **GraphQL** API integration with client queries
- **Fluent Templates** for bilingual support (English/French)
- **Bootstrap** + **jQuery** for frontend styling and interactions
- **SendGrid** for email services

## Development Commands

### Setup
```bash
# Create .env file with required environment variables (see README.md)
diesel migration run    # Setup database schema
cargo run              # Start development server (http://127.0.0.1:8088)
```

### Development
```bash
cargo build            # Build the application
cargo check            # Quick syntax/type checking
cargo clippy           # Linting (if available)
cargo test             # Run tests (if any exist)
cargo run              # Run development server
```

### Database
```bash
diesel migration run            # Apply all pending migrations
diesel migration generate NAME  # Create new migration
diesel print-schema            # Print current database schema
```

## Architecture

### Core Structure
- **`src/handlers/`** - HTTP request handlers for each entity (person, role, organization, etc.)
- **`src/models/`** - Diesel ORM data models
- **`src/graphql/`** - GraphQL resolvers and type definitions
- **`templates/`** - Tera HTML templates organized by feature
- **`static/`** - CSS, JavaScript, and static assets
- **`migrations/`** - Database schema migrations
- **`queries/`** - GraphQL query definitions
- **`i18n/`** - Internationalization files for EN/FR support

### Key Entry Points
- **`src/main.rs`** - Application bootstrap and server startup
- **`src/lib.rs`** - Library definitions and app-wide utilities
- **`src/handlers/routes.rs`** - HTTP route configuration
- **`schema.graphql`** - Complete GraphQL API schema

### Domain Model
The application centers around organizational management with entities:
- **Person** (individuals with capabilities)
- **Organization** and **OrgTier** (hierarchical structures)
- **Role** and **Team** (positions and groups)
- **Capability** and **Skill** (competencies)
- **Work** and **Task** (assignments and projects)
- **Publication** (research outputs)

### Authentication & Sessions
- Uses **Actix-Identity** for session management
- Email verification workflow via SendGrid
- Role-based access control throughout the application

## Environment Setup

Required `.env` variables:
- `COOKIE_SECRET_KEY` (minimum 32 characters)
- `DATABASE_URL` (PostgreSQL connection string)
- `SENDGRID_API_KEY`
- `ADMIN_NAME`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`
- `ENVIRONMENT=test`

## Code Patterns

- **Handler Pattern**: Each entity has dedicated handlers in `src/handlers/`
- **Template Organization**: Templates mirror the handler structure
- **GraphQL Integration**: Queries defined in `queries/` directory
- **Bilingual Support**: All user-facing strings use Fluent i18n system
- **Static File Compilation**: Build-time asset bundling via `build.rs`

## Development Notes

- Database changes require new Diesel migrations
- Templates use Tera syntax with Fluent filters for i18n
- GraphQL schema changes need corresponding resolver updates
- Static files are compiled at build time - restart after changes
- Email templates are in `templates/emails/` directory