# Rehardcover

A Rust project that fetches user data from [hardcover.app](https://hardcover.app) to parse reading journals and match books with their corresponding entries.

## What it does

This tool connects to the Hardcover API to:
- Fetch user reading data
- Parse reading journals
- Match books with their journal entries

## Building the Project

### Prerequisites

- Rust and Cargo installed
- A Hardcover API token (Bearer token)

### Build Steps

1. **Clone and navigate to the project:**
   ```bash
   git clone <repository-url>
   cd rehardcover
   ```

2. **Generate the GraphQL schema:**
   
   Before building, you need to introspect the Hardcover API to generate the schema file:
   ```bash
   cynic introspect --header "Authorization: Bearer ey..." https://api.hardcover.app/v1/graphql -o schemas/hardcover.schema
   ```
   
   Replace `ey...` with your actual Hardcover API Bearer token.

3. **Build the project:**
   ```bash
   cargo build
   ```

4. **Run the project:**
   ```bash
   cargo run -- "Bearer ey..."
   ```

## API Token

To get your Hardcover API token:
1. Log into your Hardcover account
2. Navigate to your API settings
3. Generate a new Bearer token
4. Use this token in the introspect and run commands above

## Project Structure

- `src/` - Main Rust source code
- `schemas/` - GraphQL schema files (generated)
- `Cargo.toml` - Project dependencies and configuration
- `build.rs` - Custom build code to register the schema for Hardcover
