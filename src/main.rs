use std::env;
use std::io::{self, Write};
use cynic::{MutationBuilder, QueryBuilder};
use cynic::http::SurfExt;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[cynic::schema("hardcover")]
mod schema {}

// Scalar type for citext
#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "citext")]
pub struct Citext(pub String);

// Scalar type for date
#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "date")]
pub struct Date(pub String);

// Scalar type for bigint
#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "bigint")]
pub struct BigInt(pub i64);

// Scalar type for timestamp
#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "timestamp")]
pub struct Timestamp(pub String);

// Scalar type for jsonb
#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "jsonb")]
pub struct JsonB(pub serde_json::Value);

// Query fragments for user details
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "users", schema = "hardcover")]
struct UserDetails {
    username: Option<Citext>,
    id: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "query_root", schema = "hardcover")]
struct UserDetailsQuery {
    me: Vec<UserDetails>,
}

// Query fragments for user books
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "books", schema = "hardcover")]
struct BookDetails {
    id: i32,
    title: Option<String>,
    pages: Option<i32>,
    #[cynic(rename = "release_date")]
    release_date: Option<Date>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "user_book_statuses", schema = "hardcover")]
struct UserBookStatus {
    id: i32,
    status: String,
    #[allow(dead_code)]
    slug: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "user_books", schema = "hardcover")]
struct UserBookDetails {
    book: BookDetails,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "user_books", schema = "hardcover")]
struct UserBookWithOptionalStatus {
    id: i32,
    book: BookDetails,
    #[cynic(rename = "status_id")]
    status_id: Option<i32>,
}


#[derive(cynic::QueryVariables, Debug)]
struct UserBooksVariables {
    user_id: i32,
    limit: i32,
    offset: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "query_root",
    variables = "UserBooksVariables",
    schema = "hardcover"
)]
struct UserBooksQuery {
    #[arguments(where: { user_id: { _eq: $user_id } }, distinct_on: "book_id", limit: $limit, offset: $offset)]
    #[cynic(rename = "user_books")]
    user_books: Vec<UserBookDetails>,
}


#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "query_root", schema = "hardcover")]
struct UserBookStatusesQuery {
    #[cynic(rename = "user_book_statuses")]
    user_book_statuses: Vec<UserBookStatus>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "query_root",
    variables = "UserBooksVariables",
    schema = "hardcover"
)]
struct UserBooksWithOptionalStatusQuery {
    #[arguments(where: { user_id: { _eq: $user_id } }, distinct_on: "book_id", limit: $limit, offset: $offset)]
    #[cynic(rename = "user_books")]
    user_books: Vec<UserBookWithOptionalStatus>,
}

// Query fragments for reading journals summary - just count journals per book
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "books", schema = "hardcover")]
struct BookInfo {
    title: Option<String>,
    id: i32,
}

// Query fragments for individual reading journal entries
#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "reading_journals", schema = "hardcover")]
struct ReadingJournalEntry {
    id: BigInt,
    entry: Option<String>,
    event: Option<String>,
    #[cynic(rename = "created_at")]
    created_at: Timestamp,
    #[cynic(rename = "updated_at")]
    updated_at: Timestamp,
    metadata: JsonB,
    book: Option<BookInfo>,
}

#[derive(cynic::QueryVariables, Debug)]
struct ReadingJournalsVariables {
    user_id: i32,
    limit: i32,
    offset: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "query_root",
    variables = "ReadingJournalsVariables",
    schema = "hardcover"
)]
struct ReadingJournalsQuery {
    #[arguments(where: { user_id: { _eq: $user_id } }, order_by: [{ created_at: desc }], limit: $limit, offset: $offset)]
    #[cynic(rename = "reading_journals")]
    reading_journals: Vec<ReadingJournalEntry>,
}

// Mutation structures
#[derive(cynic::InputObject, Debug)]
#[cynic(schema = "hardcover")]
struct UserBookUpdateInput {
    #[cynic(rename = "status_id")]
    status_id: Option<i32>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(schema = "hardcover")]
struct UserBookCreateInput {
    #[cynic(rename = "book_id")]
    book_id: i32,
    #[cynic(rename = "status_id")]
    status_id: Option<i32>,
    #[cynic(rename = "date_added")]
    date_added: Option<Date>,
}

// Input for read instances
#[derive(cynic::InputObject, Debug)]
#[cynic(schema = "hardcover")]
struct DatesReadInput {
    #[cynic(rename = "edition_id")]
    edition_id: Option<i32>,
    #[cynic(rename = "finished_at")]
    finished_at: Option<Date>,
    id: Option<i32>,
    #[cynic(rename = "progress_pages")]
    progress_pages: Option<i32>,
    #[cynic(rename = "progress_seconds")]
    progress_seconds: Option<i32>,
    #[cynic(rename = "reading_format_id")]
    reading_format_id: Option<i32>,
    #[cynic(rename = "started_at")]
    started_at: Option<Date>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "UserBookIdType", schema = "hardcover")]
struct UserBookIdType {
    id: Option<i32>,
}


// Return type for deleting journal entries
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "DeleteReadingJournalOutput", schema = "hardcover")]
struct DeleteReadingJournalOutput {
    id: i32,
}

#[derive(cynic::QueryVariables, Debug)]
struct UpdateUserBookVariables {
    id: i32,
    object: UserBookUpdateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "mutation_root",
    variables = "UpdateUserBookVariables",
    schema = "hardcover"
)]
struct UpdateUserBookMutation {
    #[arguments(id: $id, object: $object)]
    #[cynic(rename = "update_user_book")]
    #[allow(dead_code)]
    update_user_book: Option<UserBookIdType>,
}

#[derive(cynic::QueryVariables, Debug)]
struct InsertUserBookVariables {
    object: UserBookCreateInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "mutation_root",
    variables = "InsertUserBookVariables",
    schema = "hardcover"
)]
struct InsertUserBookMutation {
    #[arguments(object: $object)]
    #[cynic(rename = "insert_user_book")]
    insert_user_book: Option<UserBookIdType>,
}

// Return type for inserting user book reads
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "UserBookReadIdType", schema = "hardcover")]
struct UserBookReadIdType {
    error: Option<String>,
    id: Option<i32>,
}

// Variables and mutation for inserting read instances
#[derive(cynic::QueryVariables, Debug)]
struct InsertUserBookReadVariables {
    #[cynic(rename = "user_book_id")]
    user_book_id: i32,
    #[cynic(rename = "user_book_read")]
    user_book_read: DatesReadInput,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "mutation_root",
    variables = "InsertUserBookReadVariables",
    schema = "hardcover"
)]
struct InsertUserBookReadMutation {
    #[arguments(user_book_id: $user_book_id, user_book_read: $user_book_read)]
    #[cynic(rename = "insert_user_book_read")]
    insert_user_book_read: Option<UserBookReadIdType>,
}

// Variables and mutation for deleting journal entries
#[derive(cynic::QueryVariables, Debug)]
struct DeleteReadingJournalVariables {
    id: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "mutation_root",
    variables = "DeleteReadingJournalVariables",
    schema = "hardcover"
)]
struct DeleteReadingJournalMutation {
    #[arguments(id: $id)]
    #[cynic(rename = "delete_reading_journal")]
    delete_reading_journal: Option<DeleteReadingJournalOutput>,
}

// Variables and mutation for deleting user book reads
#[derive(cynic::QueryVariables, Debug)]
struct DeleteUserBookReadVariables {
    id: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "mutation_root",
    variables = "DeleteUserBookReadVariables",
    schema = "hardcover"
)]
struct DeleteUserBookReadMutation {
    #[arguments(id: $id)]
    #[cynic(rename = "delete_user_book_read")]
    delete_user_book_read: Option<UserBookReadIdType>,
}

// Query fragment for user book reads
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "user_book_reads", schema = "hardcover")]
struct UserBookRead {
    id: i32,
    #[cynic(rename = "started_at")]
    started_at: Option<Date>,
    #[cynic(rename = "finished_at")]
    finished_at: Option<Date>,
    #[cynic(rename = "user_book_id")]
    #[allow(dead_code)]
    user_book_id: i32,
}

// Variables for querying user book reads
#[derive(cynic::QueryVariables, Debug)]
struct UserBookReadsVariables {
    #[cynic(rename = "user_book_id")]
    user_book_id: i32,
}

// Query for user book reads
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "query_root",
    variables = "UserBookReadsVariables",
    schema = "hardcover"
)]
struct UserBookReadsQuery {
    #[arguments(where: { user_book_id: { _eq: $user_book_id } })]
    #[cynic(rename = "user_book_reads")]
    user_book_reads: Vec<UserBookRead>,
}

struct Cli {
    api_key: String,
    analysis_only: bool,
}

async fn fetch_user_details(api_key: &str) -> Result<UserDetailsQuery, Box<dyn std::error::Error>> {
    use cynic::{QueryBuilder, http::SurfExt};

    let operation = UserDetailsQuery::build(());

    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;

    if let Some(errors) = response.errors {
        eprintln!("GraphQL errors: {:?}", errors);
        return Err("GraphQL query failed".into());
    }

    response.data.ok_or("No data returned".into())
}

async fn fetch_user_books(
    api_key: &str,
    user_id: i32,
    limit: i32,
    offset: i32,
) -> Result<UserBooksQuery, Box<dyn std::error::Error>> {
    use cynic::{QueryBuilder, http::SurfExt};

    let operation = UserBooksQuery::build(UserBooksVariables {
        user_id,
        limit,
        offset,
    });

    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;

    if let Some(errors) = response.errors {
        eprintln!("GraphQL errors: {:?}", errors);
        return Err("GraphQL query failed".into());
    }

    response.data.ok_or("No data returned".into())
}

async fn fetch_user_book_statuses(api_key: &str) -> Result<UserBookStatusesQuery, Box<dyn std::error::Error>> {
    use cynic::{QueryBuilder, http::SurfExt};

    let operation = UserBookStatusesQuery::build(());

    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;

    if let Some(errors) = response.errors {
        eprintln!("GraphQL errors: {:?}", errors);
        return Err("GraphQL query failed".into());
    }

    response.data.ok_or("No data returned".into())
}


async fn fetch_user_books_with_optional_status(
    api_key: &str,
    user_id: i32,
    limit: i32,
    offset: i32,
) -> Result<UserBooksWithOptionalStatusQuery, Box<dyn std::error::Error>> {
    use cynic::{QueryBuilder, http::SurfExt};

    let operation = UserBooksWithOptionalStatusQuery::build(UserBooksVariables {
        user_id,
        limit,
        offset,
    });

    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;

    if let Some(errors) = response.errors {
        eprintln!("GraphQL errors: {:?}", errors);
        return Err("GraphQL query failed".into());
    }

    response.data.ok_or("No data returned".into())
}

async fn insert_user_book(
    api_key: &str,
    book_id: i32,
    status_id: Option<i32>,
) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    let variables = InsertUserBookVariables {
        object: UserBookCreateInput {
            book_id,
            status_id,
            date_added: None, // Let the API set the current date
        },
    };
    
    let operation = InsertUserBookMutation::build(variables);
    
    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;
    
    if let Some(data) = response.data
        && let Some(result) = data.insert_user_book {
            return Ok(result.id);
        }
    
    Ok(None)
}

async fn update_user_book_status(
    api_key: &str,
    user_book_id: i32,
    new_status_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let variables = UpdateUserBookVariables {
        id: user_book_id,
        object: UserBookUpdateInput {
            status_id: Some(new_status_id),
        },
    };
    
    let operation = UpdateUserBookMutation::build(variables);
    
    let _response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;
    
    // The response is handled by surf's run_graphql, so if we get here, it succeeded
    Ok(())
}

// Function to convert timestamp to date format (YYYY-MM-DD)
fn timestamp_to_date(timestamp_str: &str) -> Option<String> {
    // Try to parse the timestamp and extract just the date part
    if let Ok(time) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
        Some(time.format("%Y-%m-%d").to_string())
    } else if let Ok(time) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%.f") {
        Some(time.format("%Y-%m-%d").to_string())
    } else if let Ok(time) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S") {
        Some(time.format("%Y-%m-%d").to_string())
    } else if timestamp_str.len() >= 10 && timestamp_str.chars().nth(4) == Some('-') {
        // If it's already in YYYY-MM-DD format or starts with it
        Some(timestamp_str[..10].to_string())
    } else {
        None
    }
}

// Function to create read instances
async fn insert_user_book_read(
    api_key: &str,
    user_book_id: i32,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert timestamps to date format (YYYY-MM-DD) that the API expects
    let start_date_formatted = start_date.and_then(timestamp_to_date);
    let end_date_formatted = end_date.and_then(timestamp_to_date);
    
    println!("    🔧 Formatted dates - start: {:?}, end: {:?}", start_date_formatted, end_date_formatted);
    
    // Create a read instance with the provided dates (only include essential fields)
    let user_book_read = DatesReadInput {
        edition_id: None,
        finished_at: end_date_formatted.map(Date),
        id: None,
        progress_pages: None,
        progress_seconds: None,
        reading_format_id: None,
        started_at: start_date_formatted.map(Date),
    };
    
    let variables = InsertUserBookReadVariables {
        user_book_id,
        user_book_read,
    };
    
    let operation = InsertUserBookReadMutation::build(variables);
    
    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;
    
    if let Some(data) = response.data
        && let Some(result) = data.insert_user_book_read {
            if let Some(error) = result.error {
                return Err(format!("API error: {}", error).into());
            }
            if let Some(id) = result.id {
                println!("    📖 Created read instance with ID: {}", id);
            }
        }
    
    Ok(())
}

// Function to delete a reading journal entry
async fn delete_reading_journal_entry(
    api_key: &str,
    journal_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let variables = DeleteReadingJournalVariables {
        id: journal_id,
    };
    
    let operation = DeleteReadingJournalMutation::build(variables);
    
    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;
    
    if let Some(data) = response.data
        && let Some(result) = data.delete_reading_journal {
            println!("    🗑️ Deleted journal entry ID: {}", result.id);
        }
    
    Ok(())
}

// Function to query existing user book reads
async fn fetch_user_book_reads(
    api_key: &str,
    user_book_id: i32,
) -> Result<Vec<UserBookRead>, Box<dyn std::error::Error>> {
    let variables = UserBookReadsVariables { user_book_id };
    let operation = UserBookReadsQuery::build(variables);
    
    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;
    
    if let Some(data) = response.data {
        Ok(data.user_book_reads)
    } else {
        Ok(Vec::new())
    }
}

// Function to delete a user book read
async fn delete_user_book_read(
    api_key: &str,
    read_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let variables = DeleteUserBookReadVariables { id: read_id };
    let operation = DeleteUserBookReadMutation::build(variables);
    
    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;
    
    if let Some(data) = response.data
        && let Some(result) = data.delete_user_book_read {
            if let Some(error) = result.error {
                return Err(format!("API error: {}", error).into());
            }
            if let Some(id) = result.id {
                println!("    🗑️ Deleted default read instance with ID: {}", id);
            }
        }
    
    Ok(())
}

// Function to check if a timestamp is within the last 24 hours
fn is_within_last_day(timestamp_str: &str) -> bool {
    // Parse the timestamp string (format: "2024-01-15T10:30:00+00:00" or similar)
    // For safety, we'll be conservative and only delete if we can parse the timestamp
    
    // Try to parse various timestamp formats
    let parsed_time = if let Ok(time) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
        time.timestamp() as u64
    } else if let Ok(time) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S%.f") {
        // Handle microseconds: 2025-10-02T08:44:58.937879
        time.and_utc().timestamp() as u64
    } else if let Ok(time) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S") {
        time.and_utc().timestamp() as u64
    } else if let Ok(time) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
        time.and_utc().timestamp() as u64
    } else {
        // If we can't parse the timestamp, err on the side of caution and don't delete
        println!("    ⚠ Could not parse timestamp '{}' - skipping deletion for safety", timestamp_str);
        return false;
    };
    
    // Get current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    
    // Check if the entry is within the last 24 hours (86400 seconds)
    let time_diff = now.saturating_sub(parsed_time);
    let is_recent = time_diff <= 86400; // 24 hours in seconds
    
    if !is_recent {
        println!("    ⚠ Journal entry is {} hours old - too old to delete safely", time_diff / 3600);
    }
    
    is_recent
}

// Function to get user approval for a book update
fn get_user_approval(book_title: &str, status_name: &str, start_date: Option<&str>, end_date: Option<&str>, current: usize, total: usize) -> bool {
    println!("\n[{}/{}] 📚 Book: '{}'", current, total, book_title);
    println!("📊 Suggested Status: {}", status_name);
    
    if let Some(start) = start_date {
        if let Some(end) = end_date {
            println!("📅 Read Instance: {} to {}", start, end);
        } else {
            println!("📅 Read Instance: {} to ?", start);
        }
    } else if let Some(end) = end_date {
        println!("📅 Read Instance: ? to {}", end);
    }
    
    print!("❓ Apply this update? [y/N]: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    
    matches!(input.as_str(), "y" | "yes")
}

// Function to ask user about date preferences for "Read" status
fn ask_about_read_dates(start_date: Option<&str>, end_date: Option<&str>) -> bool {
    if start_date.is_none() && end_date.is_none() {
        return false; // No dates available
    }
    
    println!("📅 Available dates from journal:");
    if let Some(start) = start_date {
        println!("   Start: {}", start);
    }
    if let Some(end) = end_date {
        println!("   End: {}", end);
    }
    
    print!("❓ Use these dates for the read instance? [y/N] (N = set dates manually later): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    
    matches!(input.as_str(), "y" | "yes")
}


// Function to find the most recent journal entry for a book (to delete after mutation)
async fn find_latest_journal_entry_for_book(
    api_key: &str,
    book_id: i32,
    user_id: i32,
) -> Result<Option<(i32, String)>, Box<dyn std::error::Error>> {
    // Fetch recent journal entries
    let journals = fetch_reading_journals(api_key, user_id, 20, 0).await?;
    
    // Find the most recent entry for this book
    let mut latest_entry_info = None;
    let mut latest_timestamp = String::new();
    
    for entry in journals.reading_journals {
        if let Some(book) = &entry.book
            && book.id == book_id {
                let entry_timestamp = get_priority_timestamp(&entry);
                if latest_timestamp.is_empty() || entry_timestamp > latest_timestamp {
                    latest_timestamp = entry_timestamp.clone();
                    // Convert BigInt to i32 for the delete mutation and include timestamp
                    latest_entry_info = Some((entry.id.0 as i32, entry_timestamp));
                }
            }
    }
    
    Ok(latest_entry_info)
}

async fn fetch_reading_journals(
    api_key: &str,
    user_id: i32,
    limit: i32,
    offset: i32,
) -> Result<ReadingJournalsQuery, Box<dyn std::error::Error>> {
    use cynic::{QueryBuilder, http::SurfExt};

    let operation = ReadingJournalsQuery::build(ReadingJournalsVariables {
        user_id,
        limit,
        offset,
    });

    let response = surf::post("https://api.hardcover.app/v1/graphql")
        .header("Authorization", api_key)
        .run_graphql(operation)
        .await?;

    if let Some(errors) = response.errors {
        eprintln!("GraphQL errors: {:?}", errors);
        return Err("GraphQL query failed".into());
    }

    response.data.ok_or("No data returned".into())
}

async fn fetch_all_reading_journals(
    api_key: &str,
    user_id: i32,
) -> Result<Vec<ReadingJournalEntry>, Box<dyn std::error::Error>> {
    let mut all_journals = Vec::new();
    let mut offset = 0;
    let batch_size = 100; // Use smaller batch size that API likely supports

    println!("Fetching journal entries in batches of {}...", batch_size);

    // Try to fetch journals in batches using offset pagination
    loop {
        println!("Fetching batch starting at offset {}...", offset);

        let journals = fetch_reading_journals(api_key, user_id, batch_size, offset).await?;
        let current_batch_size = journals.reading_journals.len();

        println!("Retrieved {} entries in this batch", current_batch_size);

        all_journals.extend(journals.reading_journals);

        // If we got fewer than the batch size, we've reached the end
        if current_batch_size < batch_size as usize {
            println!(
                "Reached end of results (got {} < {} expected)",
                current_batch_size, batch_size
            );
            break;
        }

        offset += batch_size;

        // Safety limit to prevent infinite loops
        if offset >= 10000 {
            eprintln!("Warning: Reached safety limit of 10,000 journal entries");
            break;
        }

        // Small delay to be respectful to the API
        async_std::task::sleep(std::time::Duration::from_millis(100)).await;
    }

    println!(
        "Fetched {} total journal entries across {} batches",
        all_journals.len(),
        (offset / batch_size) + 1
    );
    Ok(all_journals)
}

#[derive(Debug, Clone)]
struct BookStatusAnalysis {
    suggested_status_id: Option<i32>,
    start_date: Option<String>,
    end_date: Option<String>,
    reasoning: String,
}

fn get_priority_timestamp(entry: &ReadingJournalEntry) -> String {
    // Priority: metadata.action_at > updated_at > created_at
    if let Some(action_at_value) = entry.metadata.0.get("action_at")
        && let Some(action_at_str) = action_at_value.as_str() {
            return action_at_str.to_string();
        }
    
    // Second priority: updated_at (if different from created_at)
    // Special case: use created_at for user_book_read_started events
    if let Some(event) = &entry.event {
        if event == "user_book_read_started" || event == "status_want_to_read" || event == "user_book_read_finished" {
            entry.created_at.0.clone()
        } else if entry.updated_at.0 != entry.created_at.0 {
            entry.updated_at.0.clone()
        } else {
            entry.created_at.0.clone()
        }
    } else if entry.updated_at.0 != entry.created_at.0 {
        entry.updated_at.0.clone()
    } else {
        entry.created_at.0.clone()
    }
}

fn analyze_journal_entries(
    entries: &[ReadingJournalEntry],
    available_statuses: &[UserBookStatus],
) -> BookStatusAnalysis {
    if entries.is_empty() {
        return BookStatusAnalysis {
            suggested_status_id: None,
            start_date: None,
            end_date: None,
            reasoning: "No journal entries found".to_string(),
        };
    }

    // Sort entries by date (oldest first for analysis) using priority timestamp
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by(|a, b| {
        let timestamp_a = get_priority_timestamp(a);
        let timestamp_b = get_priority_timestamp(b);
        timestamp_a.cmp(&timestamp_b)
    });

    let mut start_date = None;
    let mut end_date = None;

    // Find status IDs for common statuses
    let currently_reading_id = available_statuses
        .iter()
        .find(|s| s.status.to_lowercase().contains("currently reading") || s.status.to_lowercase().contains("reading"))
        .map(|s| s.id);
    
    let read_id = available_statuses
        .iter()
        .find(|s| s.status.to_lowercase() == "read" || s.status.to_lowercase().contains("finished"))
        .map(|s| s.id);
        
    let want_to_read_id = available_statuses
        .iter()
        .find(|s| s.status.to_lowercase().contains("want to read") || s.status.to_lowercase().contains("to read"))
        .map(|s| s.id);

    use std::collections::HashMap;

    // Process entries chronologically, applying priority rules for same timestamps
    let mut latest_status = None;
    let mut has_finished_reading = false; // Track if we've seen user_book_read_finished
    let mut reasoning_parts = Vec::new();

    // Extract actual timestamps from entries, preferring metadata if available
    let mut entries_with_timestamps: Vec<(&ReadingJournalEntry, String)> = Vec::new();
    
    for entry in &sorted_entries {
        let actual_timestamp = get_priority_timestamp(entry);
        
        // Debug output to show event and timestamp source (only if verbose debugging needed)
        // This is commented out to avoid cluttering the analysis output
        // if let Some(action_at_value) = entry.metadata.0.get("action_at") {
        //     if let Some(action_at_str) = action_at_value.as_str() {
        //         println!("    Event '{}' at metadata.action_at: {} (vs updated_at: {}, created_at: {})", 
        //                 entry.event.as_deref().unwrap_or("unknown"), action_at_str, entry.updated_at.0, entry.created_at.0);
        //     }
        // } else if entry.updated_at.0 != entry.created_at.0 {
        //     println!("    Event '{}' at updated_at: {} (vs created_at: {}) - no action_at in metadata", 
        //             entry.event.as_deref().unwrap_or("unknown"), entry.updated_at.0, entry.created_at.0);
        // } else {
        //     println!("    Event '{}' at created_at: {} (updated_at same, no action_at in metadata)", 
        //             entry.event.as_deref().unwrap_or("unknown"), entry.created_at.0);
        // }
        
        entries_with_timestamps.push((entry, actual_timestamp));
    }
    
    // Sort by actual timestamps
    entries_with_timestamps.sort_by(|a, b| a.1.cmp(&b.1));
    
    // Group by timestamp
    let mut entries_by_timestamp: HashMap<String, Vec<&ReadingJournalEntry>> = HashMap::new();
    for (entry, timestamp) in &entries_with_timestamps {
        entries_by_timestamp
            .entry(timestamp.clone())
            .or_default()
            .push(entry);
    }

    for timestamp in entries_with_timestamps.iter().map(|(_, ts)| ts).collect::<std::collections::BTreeSet<_>>() {
        if let Some(timestamp_entries) = entries_by_timestamp.get(timestamp) {
            // Determine the highest priority status for this timestamp
            let mut timestamp_status = None;
            let mut status_priority = 0; // 0 = lowest, 3 = highest (Read)
            
            for entry in timestamp_entries {
                if let Some(event) = &entry.event {
                    // Use exact event names from the API
                    match event.as_str() {
                        // Priority 3: Finished Reading (Read) - HIGHEST priority, always wins
                        "user_book_read_finished" => {
                            timestamp_status = read_id;
                            status_priority = 3;
                            end_date = Some(timestamp.clone());
                            has_finished_reading = true; // Mark that we've seen finished reading
                        }
                        
                        // Priority 2: Started Reading (Currently Reading)
                        "user_book_read_started" => {
                            if status_priority < 2 {
                                timestamp_status = currently_reading_id;
                                status_priority = 2;
                                if start_date.is_none() {
                                    start_date = Some(timestamp.clone());
                                }
                            }
                        }
                        
                        // Priority 1: Want to Read
                        "status_want_to_read" => {
                            if status_priority < 1 {
                                timestamp_status = want_to_read_id;
                                status_priority = 1;
                            }
                        }
                        
                        // Ignore other events like "progress_updated", "list_book", "rated", etc.
                        _ => {}
                    }
                }
            }
            
            // Update latest status if we found one for this timestamp
            // BUT: if we've already seen user_book_read_finished, don't override it
            if let Some(status) = timestamp_status {
                if !has_finished_reading || Some(status) == read_id {
                    latest_status = Some(status);
                }
                
                // Add reasoning for this timestamp showing actual events
                if timestamp_entries.len() > 1 {
                    let events: Vec<String> = timestamp_entries
                        .iter()
                        .filter_map(|e| e.event.as_ref())
                        .map(|event| format!("'{}'", event))
                        .collect();
                    let status_name = available_statuses
                        .iter()
                        .find(|s| s.id == status)
                        .map(|s| s.status.as_str())
                        .unwrap_or("unknown");
                    
                    if has_finished_reading && Some(status) != read_id {
                        reasoning_parts.push(format!("At {}: events [{}], but 'Read' status already locked in from earlier 'user_book_read_finished'", 
                                                    timestamp, events.join(", ")));
                    } else {
                        reasoning_parts.push(format!("At {}: events [{}], '{}' status takes priority", 
                                                    timestamp, events.join(", "), status_name));
                    }
                } else if let Some(event) = timestamp_entries.first().and_then(|e| e.event.as_ref()) {
                    if has_finished_reading && Some(status) != read_id {
                        reasoning_parts.push(format!("At {}: '{}', but 'Read' status already locked in from earlier 'user_book_read_finished'", timestamp, event));
                    } else {
                        reasoning_parts.push(format!("At {}: '{}'", timestamp, event));
                    }
                }
            }
        }
    }

    // Use the latest status determined by priority analysis
    // The priority system already handled the correct status based on journal entries
    let suggested_status_id = latest_status;

    let reasoning = if reasoning_parts.is_empty() {
        "Could not determine clear status from journal entries".to_string()
    } else {
        let mut final_reasoning = reasoning_parts.join("; ");
        
        // Add summary based on what we found
        if start_date.is_some() && end_date.is_some() {
            final_reasoning.push_str(". Found both start and end reading dates");
        } else if start_date.is_some() {
            final_reasoning.push_str(". Found start reading date");
        } else if end_date.is_some() {
            final_reasoning.push_str(". Found end reading date");
        }
        
        if let Some(status_id) = suggested_status_id {
            let final_status_name = available_statuses
                .iter()
                .find(|s| s.id == status_id)
                .map(|s| s.status.as_str())
                .unwrap_or("unknown");
            final_reasoning.push_str(&format!(". Final suggestion: '{}'", final_status_name));
        }
        
        final_reasoning
    };

    BookStatusAnalysis {
        suggested_status_id,
        start_date,
        end_date,
        reasoning,
    }
}

async fn process_book_status_update(
    api_key: &str,
    user_id: i32,
    analysis_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Book Status Update Process ===");
    
    // Fetch available statuses
    println!("Fetching available book statuses...");
    let statuses_result = fetch_user_book_statuses(api_key).await?;
    let available_statuses = &statuses_result.user_book_statuses;
    
    println!("Available statuses:");
    for status in available_statuses {
        println!("  {} (ID: {}): {}", status.status, status.id, status.description.as_deref().unwrap_or(""));
    }

    // Find status IDs for exclusion
    let excluded_status_names = ["currently reading", "read", "want to read"];
    let excluded_status_ids: Vec<i32> = available_statuses
        .iter()
        .filter(|s| {
            let status_lower = s.status.to_lowercase();
            excluded_status_names.iter().any(|&excluded| status_lower.contains(excluded))
        })
        .map(|s| s.id)
        .collect();

    println!("\nExcluded status IDs: {:?}", excluded_status_ids);

    // Get all journal entries first
    println!("\nFetching all journal entries...");
    let all_journal_entries = fetch_all_reading_journals(api_key, user_id).await?;

    // Group journal entries by book and collect unique book IDs
    use std::collections::HashMap;
    let mut journals_by_book: HashMap<i32, Vec<&ReadingJournalEntry>> = HashMap::new();
    
    for journal in &all_journal_entries {
        if let Some(book) = &journal.book {
            journals_by_book
                .entry(book.id)
                .or_default()
                .push(journal);
        }
    }

    let books_with_journals: Vec<i32> = journals_by_book.keys().cloned().collect();
    println!("Found {} unique books with journal entries", books_with_journals.len());

    if books_with_journals.is_empty() {
        println!("No books with journal entries found.");
        return Ok(());
    }

    // Now we need to find user_book records for books that have journal entries
    // The issue is that journal entries might exist for books that aren't in the user's library yet
    println!("\nFetching all user books with status information...");
    let mut all_user_books = Vec::new();
    let mut offset = 0;
    let batch_size = 100;

    loop {
        let books_result = fetch_user_books_with_optional_status(api_key, user_id, batch_size, offset).await?;
        let current_batch_size = books_result.user_books.len();
        
        all_user_books.extend(books_result.user_books);
        
        if current_batch_size < batch_size as usize {
            break;
        }
        offset += batch_size;
    }

    println!("Found {} total user books", all_user_books.len());

    // Create a map of book_id -> user_book for quick lookup
    let user_books_by_book_id: HashMap<i32, &UserBookWithOptionalStatus> = all_user_books
        .iter()
        .map(|ub| (ub.book.id, ub))
        .collect();

    println!("Analyzing books with journal entries against user library...");

    // Find candidate books by checking each book that has journal entries
    let mut candidate_books = Vec::new();
    
    // Track counts for summary
    let mut books_to_add_counts = std::collections::HashMap::new();
    let mut total_books_analyzed = 0;
    let mut books_already_in_library = 0;
    let mut books_with_excluded_status = 0;
    
    // Collect books that need to be added (not in library)
    let mut books_to_add = Vec::new();
    
    for (book_id, journal_entries) in &journals_by_book {
        total_books_analyzed += 1;
        
        // Get book title from journal entries
        let book_title = journal_entries
            .first()
            .and_then(|entry| entry.book.as_ref())
            .and_then(|book| book.title.as_deref())
            .unwrap_or("Unknown");

        if let Some(user_book) = user_books_by_book_id.get(book_id) {
            books_already_in_library += 1;
            // This book exists in the user's library
            let is_candidate = match user_book.status_id {
                None => {
                    println!("  Book '{}' (in library) has no status set - candidate", book_title);
                    true
                }
                Some(status_id) => {
                    if excluded_status_ids.contains(&status_id) {
                        books_with_excluded_status += 1;
                        println!("  Book '{}' (in library) has excluded status ID {} - skipping", book_title, status_id);
                        false
                    } else {
                        println!("  Book '{}' (in library) has non-excluded status ID {} - candidate", book_title, status_id);
                        true
                    }
                }
            };
            
            if is_candidate {
                candidate_books.push(*user_book);
            }
        } else {
            // This book has journal entries but is NOT in the user's library
            // Analyze journal entries to determine appropriate status
            let entries_vec: Vec<ReadingJournalEntry> = journal_entries.iter().map(|e| (*e).clone()).collect();
            let analysis = analyze_journal_entries(&entries_vec, available_statuses);
            
            if let Some(suggested_status_id) = analysis.suggested_status_id {
                let status_name = available_statuses
                    .iter()
                    .find(|s| s.id == suggested_status_id)
                    .map(|s| s.status.as_str())
                    .unwrap_or("unknown");
                
                // Track the count for this status
                *books_to_add_counts.entry(status_name.to_string()).or_insert(0) += 1;
                
                // Print simplified analysis
                let _start_date_str = analysis.start_date.as_deref().unwrap_or("?");
                let _end_date_str = analysis.end_date.as_deref().unwrap_or("?");
                
                // Store book for later processing with user approval
                if !analysis_only {
                    books_to_add.push((book_title.to_string(), *book_id, suggested_status_id, analysis.clone()));
                }
            } else {
                println!("    Could not determine appropriate status from journal entries");
            }
        }
    }

    // Process books to add with progress tracking
    if !books_to_add.is_empty() && !analysis_only {
        println!("\n=== PROCESSING BOOKS TO ADD ===");
        let total_to_add = books_to_add.len();
        
        for (current_idx, (book_title, book_id, suggested_status_id, analysis)) in books_to_add.iter().enumerate() {
            let status_name = available_statuses
                .iter()
                .find(|s| s.id == *suggested_status_id)
                .map(|s| s.status.as_str())
                .unwrap_or("unknown");
            
            // Check if user approves this update with progress tracking
            if get_user_approval(
                book_title,
                status_name,
                analysis.start_date.as_deref(),
                analysis.end_date.as_deref(),
                current_idx + 1,
                total_to_add,
            ) {
                // Add this book to the library with the suggested status
                match insert_user_book(api_key, *book_id, Some(*suggested_status_id)).await {
                    Ok(Some(new_user_book_id)) => {
                        println!("    ✅ Added book to library with user_book ID: {}", new_user_book_id);
                        
                        // Find and delete the journal entry created by the mutation
                        match find_latest_journal_entry_for_book(api_key, *book_id, user_id).await {
                            Ok(Some((journal_id, timestamp))) => {
                                // Safety check: only delete if the entry is within the last day
                                if is_within_last_day(&timestamp) {
                                    match delete_reading_journal_entry(api_key, journal_id).await {
                                        Ok(()) => {
                                            // Journal entry deleted successfully
                                        }
                                        Err(e) => {
                                            println!("    ⚠ Failed to delete journal entry: {}", e);
                                        }
                                    }
                                } else {
                                    println!("    ⚠ Skipping journal deletion - entry too old for safety");
                                }
                            }
                            Ok(None) => {
                                println!("    ⚠ No journal entry found to delete");
                            }
                            Err(e) => {
                                println!("    ⚠ Failed to find journal entry: {}", e);
                            }
                        }
                        
                        // Handle read instance creation based on status
                        if status_name == "Read" {
                            // For "Read" status, ask user about date preferences
                            let use_dates = if analysis.start_date.is_some() || analysis.end_date.is_some() {
                                ask_about_read_dates(analysis.start_date.as_deref(), analysis.end_date.as_deref())
                            } else {
                                false
                            };
                            
                            if use_dates {
                                // Create read instance with dates
                                println!("    📅 Creating read instance with start: {:?}, end: {:?}", 
                                         analysis.start_date, analysis.end_date);
                                match insert_user_book_read(
                                    api_key,
                                    new_user_book_id,
                                    analysis.start_date.as_deref(),
                                    analysis.end_date.as_deref(),
                                ).await {
                                    Ok(()) => {
                                        println!("    ✅ Created read instance with dates");
                                        
                                        // Delete any default "? - ?" read instances
                                        match fetch_user_book_reads(api_key, new_user_book_id).await {
                                            Ok(existing_reads) => {
                                                for read in existing_reads {
                                                    // Delete reads that have no dates (default "? - ?")
                                                    if read.started_at.is_none() && read.finished_at.is_none() {
                                                        println!("    🗑️ Found default read instance (no dates), deleting ID: {}", read.id);
                                                        if let Err(e) = delete_user_book_read(api_key, read.id).await {
                                                            println!("    ⚠ Failed to delete default read: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("    ⚠ Failed to fetch existing reads: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("    ⚠ Failed to create read instance: {}", e);
                                    }
                                }
                            } else {
                                println!("    📝 User chose to set read dates manually later");
                            }
                        } else if status_name == "Currently Reading" && (analysis.start_date.is_some() || analysis.end_date.is_some()) {
                            // For "Currently Reading", always create if we have dates
                            println!("    📅 Creating read instance with start: {:?}, end: {:?}", 
                                     analysis.start_date, analysis.end_date);
                            match insert_user_book_read(
                                api_key,
                                new_user_book_id,
                                analysis.start_date.as_deref(),
                                analysis.end_date.as_deref(),
                            ).await {
                                Ok(()) => {
                                    println!("    ✅ Created read instance with dates");
                                    
                                    // Delete any automatically created read instances
                                    match fetch_user_book_reads(api_key, new_user_book_id).await {
                                        Ok(existing_reads) => {
                                            for read in existing_reads {
                                                // Delete reads that are either:
                                                // 1. Default "? - ?" (no dates)
                                                // 2. Auto-created "Currently Reading" (start date = today, no end date)
                                                let should_delete = if read.started_at.is_none() && read.finished_at.is_none() {
                                                    // Default read instance
                                                    true
                                                } else if read.finished_at.is_none() && read.started_at.is_some() {
                                                    // Check if start date is today (auto-created)
                                                    if let Some(start_date) = &read.started_at {
                                                        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                                        start_date.0 == today
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    false
                                                };
                                                
                                                if should_delete {
                                                    println!("    🗑️ Found auto-created read instance, deleting ID: {}", read.id);
                                                    if let Err(e) = delete_user_book_read(api_key, read.id).await {
                                                        println!("    ⚠ Failed to delete auto-created read: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("    ⚠ Failed to fetch existing reads: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("    ⚠ Failed to create read instance: {}", e);
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        println!("    ⚠ Book added but no ID returned");
                    }
                    Err(e) => {
                        println!("    ✗ Failed to add book to library: {}", e);
                    }
                }
            } else {
                println!("    ⏭ Skipped by user");
            }
        }
    }

    println!("Found {} candidate books (books with journal entries, in library, and no excluded status)", candidate_books.len());

    // Process candidate books (existing books that need status updates)
    if !candidate_books.is_empty() && !analysis_only {
        println!("\n=== PROCESSING EXISTING BOOKS ===");
        let total_candidates = candidate_books.len();
        
        for (current_idx, user_book) in candidate_books.iter().enumerate() {
            let book_title = user_book.book.title.as_deref().unwrap_or("Unknown Book");
            let book_id = user_book.book.id;
            
            // Get journal entries for this book
            if let Some(journal_entries) = journals_by_book.get(&book_id) {
                let entries_vec: Vec<ReadingJournalEntry> = journal_entries.iter().map(|e| (*e).clone()).collect();
                let analysis = analyze_journal_entries(&entries_vec, available_statuses);
                
                if let Some(suggested_status_id) = analysis.suggested_status_id {
                    let status_name = available_statuses
                        .iter()
                        .find(|s| s.id == suggested_status_id)
                        .map(|s| s.status.as_str())
                        .unwrap_or("unknown");
                    
                    // Check if user approves this update
                    if get_user_approval(
                        book_title,
                        status_name,
                        analysis.start_date.as_deref(),
                        analysis.end_date.as_deref(),
                        current_idx + 1,
                        total_candidates,
                    ) {
                        // Note: Removed automatic recent processing check to avoid false positives
                        // User confirmation is sufficient protection against double-processing
                        
                        // Update the existing book's status
                        match update_user_book_status(api_key, user_book.id, suggested_status_id).await {
                            Ok(()) => {
                                println!("    ✅ Updated book status to '{}'", status_name);
                                
                                // Find and delete the journal entry created by the mutation
                                match find_latest_journal_entry_for_book(api_key, book_id, user_id).await {
                                    Ok(Some((journal_id, timestamp))) => {
                                        // Safety check: only delete if the entry is within the last day
                                        if is_within_last_day(&timestamp) {
                                            match delete_reading_journal_entry(api_key, journal_id).await {
                                                Ok(()) => {
                                                    // Journal entry deleted successfully
                                                }
                                                Err(e) => {
                                                    println!("    ⚠ Failed to delete journal entry: {}", e);
                                                }
                                            }
                                        } else {
                                            println!("    ⚠ Skipping journal deletion - entry too old for safety");
                                        }
                                    }
                                    Ok(None) => {
                                        println!("    ⚠ No journal entry found to delete");
                                    }
                                    Err(e) => {
                                        println!("    ⚠ Failed to find journal entry: {}", e);
                                    }
                                }
                                
                                // Handle read instance creation based on status
                                if status_name == "Read" {
                                    // For "Read" status, ask user about date preferences
                                    let use_dates = if analysis.start_date.is_some() || analysis.end_date.is_some() {
                                        ask_about_read_dates(analysis.start_date.as_deref(), analysis.end_date.as_deref())
                                    } else {
                                        false
                                    };
                                    
                                    if use_dates {
                                        // Create read instance with dates
                                        println!("    📅 Creating read instance with start: {:?}, end: {:?}", 
                                                 analysis.start_date, analysis.end_date);
                                        match insert_user_book_read(
                                            api_key,
                                            user_book.id,
                                            analysis.start_date.as_deref(),
                                            analysis.end_date.as_deref(),
                                        ).await {
                                            Ok(()) => {
                                                println!("    ✅ Created read instance with dates");
                                                
                                                // Delete any default "? - ?" read instances
                                                match fetch_user_book_reads(api_key, user_book.id).await {
                                                    Ok(existing_reads) => {
                                                        for read in existing_reads {
                                                            // Delete reads that have no dates (default "? - ?")
                                                            if read.started_at.is_none() && read.finished_at.is_none() {
                                                                println!("    🗑️ Found default read instance (no dates), deleting ID: {}", read.id);
                                                                if let Err(e) = delete_user_book_read(api_key, read.id).await {
                                                                    println!("    ⚠ Failed to delete default read: {}", e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("    ⚠ Failed to fetch existing reads: {}", e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("    ⚠ Failed to create read instance: {}", e);
                                            }
                                        }
                                    } else {
                                        println!("    📝 User chose to set read dates manually later");
                                    }
                                } else if status_name == "Currently Reading" && (analysis.start_date.is_some() || analysis.end_date.is_some()) {
                                    // For "Currently Reading", always create if we have dates
                                    println!("    📅 Creating read instance with start: {:?}, end: {:?}", 
                                             analysis.start_date, analysis.end_date);
                                    match insert_user_book_read(
                                        api_key,
                                        user_book.id,
                                        analysis.start_date.as_deref(),
                                        analysis.end_date.as_deref(),
                                    ).await {
                                        Ok(()) => {
                                            println!("    ✅ Created read instance with dates");
                                            
                                            // Delete any automatically created read instances
                                            match fetch_user_book_reads(api_key, user_book.id).await {
                                                Ok(existing_reads) => {
                                                    for read in existing_reads {
                                                        // Delete reads that are either:
                                                        // 1. Default "? - ?" (no dates)
                                                        // 2. Auto-created "Currently Reading" (start date = today, no end date)
                                                        let should_delete = if read.started_at.is_none() && read.finished_at.is_none() {
                                                            // Default read instance
                                                            true
                                                        } else if read.finished_at.is_none() && read.started_at.is_some() {
                                                            // Check if start date is today (auto-created)
                                                            if let Some(start_date) = &read.started_at {
                                                                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                                                start_date.0 == today
                                                            } else {
                                                                false
                                                            }
                                                        } else {
                                                            false
                                                        };
                                                        
                                                        if should_delete {
                                                            println!("    🗑️ Found auto-created read instance, deleting ID: {}", read.id);
                                                            if let Err(e) = delete_user_book_read(api_key, read.id).await {
                                                                println!("    ⚠ Failed to delete auto-created read: {}", e);
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("    ⚠ Failed to fetch existing reads: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("    ⚠ Failed to create read instance: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("    ✗ Failed to update book status: {}", e);
                            }
                        }
                    } else {
                        println!("    ⏭ Skipped by user");
                    }
                }
            }
        }
    }

    // Print summary statistics
    println!("\n=== SUMMARY ===");
    println!("Total books with journal entries analyzed: {}", total_books_analyzed);
    println!("Books already in library: {}", books_already_in_library);
    println!("  - With excluded status (skipped): {}", books_with_excluded_status);
    println!("  - Available for processing: {}", books_already_in_library - books_with_excluded_status);
    
    let books_not_in_library = total_books_analyzed - books_already_in_library;
    println!("Books NOT in library (to be added): {}", books_not_in_library);
    
    if !books_to_add_counts.is_empty() {
        println!("\nBooks to be added by status:");
        let mut sorted_counts: Vec<_> = books_to_add_counts.iter().collect();
        sorted_counts.sort_by_key(|(status, _)| {
            // Sort by priority: Read, Currently Reading, Want to Read, Others
            match status.to_lowercase().as_str() {
                s if s.contains("read") && !s.contains("want") && !s.contains("currently") => 0, // "Read"
                s if s.contains("currently") || s.contains("reading") => 1, // "Currently Reading"  
                s if s.contains("want") => 2, // "Want to Read"
                _ => 3, // Others
            }
        });
        
        for (status, count) in sorted_counts {
            println!("  - {}: {} books", status, count);
        }
        
        let total_to_add: i32 = books_to_add_counts.values().sum();
        println!("  Total: {} books", total_to_add);
    }

    if candidate_books.is_empty() {
        println!("\nNo existing candidate books found in library for status updates.");
        return Ok(());
    }

    // Select the first book for processing
    let selected_user_book = candidate_books[0];
    let book_id = selected_user_book.book.id;
    let book_title = selected_user_book.book.title.as_deref().unwrap_or("Unknown");
    
    println!("\nSelected book for update: {} (ID: {})", book_title, book_id);
    match selected_user_book.status_id {
        Some(status_id) => println!("Current status ID: {}", status_id),
        None => println!("Current status: None (no status set)"),
    }

    // Analyze journal entries for this book
    if let Some(entries) = journals_by_book.get(&book_id) {
        let entries_vec: Vec<ReadingJournalEntry> = entries.iter().map(|e| (*e).clone()).collect();
        let analysis = analyze_journal_entries(&entries_vec, available_statuses);
        
        println!("\nJournal Analysis:");
        println!("  Reasoning: {}", analysis.reasoning);
        if let Some(start) = &analysis.start_date {
            println!("  Start date: {}", start);
        }
        if let Some(end) = &analysis.end_date {
            println!("  End date: {}", end);
        }
        
        if let Some(new_status_id) = analysis.suggested_status_id {
            let new_status = available_statuses.iter().find(|s| s.id == new_status_id);
            if let Some(status) = new_status {
                println!("  Suggested status: {} (ID: {})", status.status, new_status_id);
                
                // Perform the update using the user_book ID
                println!("\nUpdating book status...");
                match update_user_book_status(api_key, selected_user_book.id, new_status_id).await {
                    Ok(()) => {
                        println!("✓ Book status update process completed!");
                        println!("  Would update user_book ID {} to status '{}' (ID: {})", 
                                selected_user_book.id, status.status, new_status_id);
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to update book status: {}", e);
                    }
                }
            } else {
                println!("  Could not find status details for ID: {}", new_status_id);
            }
        } else {
            println!("  No clear status suggestion from journal analysis");
        }
    }

    Ok(())
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <API_KEY> [--analysis-only]", args[0]);
        eprintln!("  --analysis-only: Only fetch and analyze journal entries, don't process book status updates");
        std::process::exit(1);
    }
    
    let api_key = args[1].clone();
    let analysis_only = args.len() > 2 && args[2] == "--analysis-only";

    let cli_args = Cli {
        api_key: api_key.clone(),
        analysis_only,
    };

    println!(
        "api_key: {:?}",
        cli_args.api_key.get(0..10).expect("API key must be longer")
    );
    
    if cli_args.analysis_only {
        println!("Running in analysis-only mode (no status updates will be performed)");
    }

    // Fetch user details
    println!("\nFetching user details...");
    let user_details = fetch_user_details(&cli_args.api_key).await?;

    if let Some(user) = user_details.me.first() {
        println!("User: {:?}", user);
        let username_str = user
            .username
            .as_ref()
            .map(|c| c.0.as_str())
            .unwrap_or("None");
        println!("Username: {}", username_str);
        println!("User ID: {}", user.id);

        // Fetch user's books
        println!("\nFetching user books...");
        let user_books = fetch_user_books(&cli_args.api_key, user.id, 5, 0).await?;

        println!("Found {} books:", user_books.user_books.len());
        for (idx, user_book) in user_books.user_books.iter().enumerate() {
            println!(
                "\n{}. {}",
                idx + 1,
                user_book.book.title.as_deref().unwrap_or("Unknown")
            );
            println!("   Pages: {:?}", user_book.book.pages);
            if let Some(date) = &user_book.book.release_date {
                println!("   Release Date: {}", date.0);
            } else {
                println!("   Release Date: None");
            }
        }
        // Fetch all journal entries
        println!("\n\n=== Reading Journals ===");
        println!(
            "Starting to fetch all journal entries for user ID: {}",
            user.id
        );
        let all_journal_entries = fetch_all_reading_journals(&cli_args.api_key, user.id).await?;

        if all_journal_entries.is_empty() {
            println!("No journal entries found.");
        } else {
            // Group journals by book
            use std::collections::HashMap;
            let mut books_with_journals: HashMap<i32, (String, Vec<&ReadingJournalEntry>)> =
                HashMap::new();

            for journal in &all_journal_entries {
                if let Some(book) = &journal.book {
                    let book_title = book.title.as_deref().unwrap_or("Unknown Book").to_string();
                    books_with_journals
                        .entry(book.id)
                        .or_insert_with(|| (book_title, Vec::new()))
                        .1
                        .push(journal);
                }
            }

            println!(
                "Books with journal entries ({} books, {} total entries):",
                books_with_journals.len(),
                all_journal_entries.len()
            );

            let mut sorted_books: Vec<_> = books_with_journals.iter().collect();
            sorted_books.sort_by(|a, b| {
                // Sort by most recent entry in each book using priority timestamp
                let a_latest = a.1.1.iter().map(|j| get_priority_timestamp(j)).max();
                let b_latest = b.1.1.iter().map(|j| get_priority_timestamp(j)).max();
                b_latest.cmp(&a_latest)
            });

            for (idx, (_book_id, (book_title, entries))) in sorted_books.iter().enumerate() {
                println!("\n{}. {} ({} entries)", idx + 1, book_title, entries.len());

                // Sort entries by date (most recent first) using priority timestamp
                let mut sorted_entries = entries.clone();
                sorted_entries.sort_by(|a, b| {
                    let timestamp_a = get_priority_timestamp(a);
                    let timestamp_b = get_priority_timestamp(b);
                    timestamp_b.cmp(&timestamp_a) // Most recent first
                });

                // Show entry count breakdown by event type
                let event_counts: HashMap<String, usize> = sorted_entries
                    .iter()
                    .filter_map(|e| e.event.as_ref())
                    .fold(HashMap::new(), |mut acc, event| {
                        *acc.entry(event.clone()).or_insert(0) += 1;
                        acc
                    });

                if !event_counts.is_empty() {
                    let mut events: Vec<_> = event_counts.iter().collect();
                    events.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
                    let event_summary: Vec<String> = events
                        .iter()
                        .map(|(event, count)| format!("{}: {}", event, count))
                        .collect();
                    println!("   Events: {}", event_summary.join(", "));
                }

                // Show date range using priority timestamps
                if let (Some(first), Some(last)) = (sorted_entries.last(), sorted_entries.first()) {
                    if sorted_entries.len() > 1 {
                        println!(
                            "   Date range: {} to {}",
                            get_priority_timestamp(first), get_priority_timestamp(last)
                        );
                    } else {
                        println!("   Date: {}", get_priority_timestamp(last));
                    }
                }

                // Show all entries for this book (with limit for readability)
                let display_limit = if sorted_entries.len() > 10 {
                    10
                } else {
                    sorted_entries.len()
                };
                println!("   Recent entries:");

                for (entry_idx, entry) in sorted_entries.iter().take(display_limit).enumerate() {
                    println!(
                        "     {}. {} - {} (created: {} updated: {})",
                        entry_idx + 1,
                        get_priority_timestamp(entry),
                        entry.event.as_deref().unwrap_or("No event"),
                        entry.created_at.0,
                        entry.updated_at.0
                    );

                    if let Some(entry_text) = &entry.entry {
                        let preview = if entry_text.len() > 100 {
                            format!("{}...", &entry_text[..100])
                        } else {
                            entry_text.clone()
                        };
                        println!("        \"{}\"", preview);
                    }
                }

                if sorted_entries.len() > display_limit {
                    println!(
                        "     ... and {} more entries",
                        sorted_entries.len() - display_limit
                    );
                }
            }

            // Summary statistics
            println!("\n=== Journal Summary ===");
            let total_entries = all_journal_entries.len();
            let books_with_entries = books_with_journals.len();

            // Overall event statistics
            let all_event_counts: HashMap<String, usize> = all_journal_entries
                .iter()
                .filter_map(|e| e.event.as_ref())
                .fold(HashMap::new(), |mut acc, event| {
                    *acc.entry(event.clone()).or_insert(0) += 1;
                    acc
                });

            println!("Total entries: {}", total_entries);
            println!("Books with entries: {}", books_with_entries);

            if !all_event_counts.is_empty() {
                println!("Overall event breakdown:");
                let mut sorted_events: Vec<_> = all_event_counts.iter().collect();
                sorted_events.sort_by(|a, b| b.1.cmp(a.1));
                for (event, count) in sorted_events {
                    println!("  {}: {} entries", event, count);
                }
            }

            // Show entries with actual text content
            let entries_with_text: Vec<_> = all_journal_entries
                .iter()
                .filter(|j| j.entry.as_ref().is_some_and(|e| !e.trim().is_empty()))
                .collect();

            println!(
                "\nEntries with text content: {} out of {}",
                entries_with_text.len(),
                total_entries
            );

            if !entries_with_text.is_empty() {
                println!("\n=== Recent Entries with Text Content ===");
                for (idx, journal) in entries_with_text.iter().take(5).enumerate() {
                    let book_title = journal
                        .book
                        .as_ref()
                        .and_then(|b| b.title.as_deref())
                        .unwrap_or("Unknown Book");

                    println!("\n{}. {} ({})", idx + 1, book_title, get_priority_timestamp(journal));
                    if let Some(event) = &journal.event {
                        println!("   Event: {}", event);
                    }
                    if let Some(entry) = &journal.entry {
                        let preview = if entry.len() > 200 {
                            format!("{}...", &entry[..200])
                        } else {
                            entry.clone()
                        };
                        println!("   Entry: \"{}\"", preview);
                    }
                }

                if entries_with_text.len() > 5 {
                    println!(
                        "\n   ... and {} more entries with text content",
                        entries_with_text.len() - 5
                    );
                }
            }
        }

        // Run the book status update process
        println!("\n{}", "=".repeat(50));
        match process_book_status_update(&cli_args.api_key, user.id, cli_args.analysis_only).await {
            Ok(()) => {
                if cli_args.analysis_only {
                    println!("Analysis completed successfully.");
                } else {
                    println!("Book status update process completed successfully.");
                }
            },
            Err(e) => eprintln!("Process failed: {}", e),
        }
        
        if cli_args.analysis_only {
            println!("\n{}", "=".repeat(50));
            println!("Skipping book status update process (analysis-only mode)");
            println!("To run the full process including status updates, run without --analysis-only flag");
        }
    } else {
        println!("No user found");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cynic::QueryBuilder;

    #[test]
    fn test_user_details_query() {
        let operation = UserDetailsQuery::build(());
        println!("User Details Query:\n{}\n", operation.query);

        // Verify the query contains expected elements
        assert!(operation.query.contains("me"));
        assert!(operation.query.contains("username"));
        assert!(operation.query.contains("id"));
    }

    #[test]
    fn test_user_books_query() {
        let operation = UserBooksQuery::build(UserBooksVariables {
            user_id: 12345,
            limit: 5,
            offset: 0,
        });
        println!("User Books Query:\n{}\n", operation.query);

        // Verify the query contains expected elements
        assert!(operation.query.contains("user_books"));
        assert!(operation.query.contains("book"));
        assert!(operation.query.contains("title"));
        assert!(operation.query.contains("pages"));
        assert!(operation.query.contains("release_date"));
        assert!(operation.query.contains("where"));
        assert!(operation.query.contains("distinct_on"));
    }

    #[test]
    fn test_reading_journals_query() {
        let operation = ReadingJournalsQuery::build(ReadingJournalsVariables {
            user_id: 12345,
            limit: 10,
            offset: 0,
        });
        println!("Reading Journals Query:\n{}\n", operation.query);

        // Verify the query contains expected elements
        assert!(operation.query.contains("reading_journals"));
        assert!(operation.query.contains("entry"));
        assert!(operation.query.contains("event"));
        assert!(operation.query.contains("created_at"));
        assert!(operation.query.contains("book"));
        assert!(operation.query.contains("id"));
        assert!(operation.query.contains("offset"));
    }

    #[test]
    fn test_pagination_queries() {
        // Test multiple pagination queries
        for offset in [0, 100, 200] {
            let operation = ReadingJournalsQuery::build(ReadingJournalsVariables {
                user_id: 12345,
                limit: 100,
                offset,
            });

            println!(
                "Pagination Query (offset: {}):\n{}\n",
                offset, operation.query
            );

            // Verify pagination parameters are included in the query structure
            assert!(operation.query.contains("offset: $offset"));
            assert!(operation.query.contains("limit: $limit"));
            assert!(operation.query.contains("$offset: Int!"));

            // Verify the variables are set correctly
            println!(
                "Variables: user_id={}, limit={}, offset={}",
                12345, 100, offset
            );
        }
    }
}
