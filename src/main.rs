use std::env;

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
    title: Option<String>,
    pages: Option<i32>,
    #[cynic(rename = "release_date")]
    release_date: Option<Date>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "user_books", schema = "hardcover")]
struct UserBookDetails {
    book: BookDetails,
}

#[derive(cynic::QueryVariables, Debug)]
struct UserBooksVariables {
    user_id: i32,
    limit: i32,
    offset: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "query_root", variables = "UserBooksVariables", schema = "hardcover")]
struct UserBooksQuery {
    #[arguments(where: { user_id: { _eq: $user_id } }, distinct_on: "book_id", limit: $limit, offset: $offset)]
    #[cynic(rename = "user_books")]
    user_books: Vec<UserBookDetails>,
}

// Query fragments for reading journals summary - just count journals per book
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "books", schema = "hardcover")]
struct BookInfo {
    title: Option<String>,
    id: i32,
}

// Query fragments for individual reading journal entries
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "reading_journals", schema = "hardcover")]
struct ReadingJournalEntry {
    entry: Option<String>,
    event: Option<String>,
    #[cynic(rename = "created_at")]
    created_at: Timestamp,
    book: Option<BookInfo>,
}

#[derive(cynic::QueryVariables, Debug)]
struct ReadingJournalsVariables {
    user_id: i32,
    limit: i32,
    offset: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "query_root", variables = "ReadingJournalsVariables", schema = "hardcover")]
struct ReadingJournalsQuery {
    #[arguments(where: { user_id: { _eq: $user_id } }, order_by: [{ created_at: desc }], limit: $limit, offset: $offset)]
    #[cynic(rename = "reading_journals")]
    reading_journals: Vec<ReadingJournalEntry>,
}

struct Cli {
    api_key: String,
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
            println!("Reached end of results (got {} < {} expected)", current_batch_size, batch_size);
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
    
    println!("Fetched {} total journal entries across {} batches", all_journals.len(), (offset / batch_size) + 1);
    Ok(all_journals)
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::args().nth(1).expect("no API key given");

    let args = Cli {
        api_key: api_key.clone(),
    };

    println!("api_key: {:?}", args.api_key.get(0..10).expect("API key must be longer"));

    // Fetch user details
    println!("\nFetching user details...");
    let user_details = fetch_user_details(&args.api_key).await?;
    
    if let Some(user) = user_details.me.first() {
        println!("User: {:?}", user);
        let username_str = user.username.as_ref().map(|c| c.0.as_str()).unwrap_or("None");
        println!("Username: {}", username_str);
        println!("User ID: {}", user.id);
        
        // Fetch user's books
        println!("\nFetching user books...");
        let user_books = fetch_user_books(&args.api_key, user.id, 5, 0).await?;
        
        println!("Found {} books:", user_books.user_books.len());
        for (idx, user_book) in user_books.user_books.iter().enumerate() {
            println!("\n{}. {}", idx + 1, user_book.book.title.as_deref().unwrap_or("Unknown"));
            println!("   Pages: {:?}", user_book.book.pages);
            if let Some(date) = &user_book.book.release_date {
                println!("   Release Date: {}", date.0);
            } else {
                println!("   Release Date: None");
            }
        }
        // Fetch all journal entries
        println!("\n\n=== Reading Journals ===");
        println!("Starting to fetch all journal entries for user ID: {}", user.id);
        let all_journal_entries = fetch_all_reading_journals(&args.api_key, user.id).await?;
        
        if all_journal_entries.is_empty() {
            println!("No journal entries found.");
        } else {
            // Group journals by book
            use std::collections::HashMap;
            let mut books_with_journals: HashMap<i32, (String, Vec<&ReadingJournalEntry>)> = HashMap::new();
            
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
            
            println!("Books with journal entries ({} books, {} total entries):", 
                     books_with_journals.len(), 
                     all_journal_entries.len());
            
            let mut sorted_books: Vec<_> = books_with_journals.iter().collect();
            sorted_books.sort_by(|a, b| {
                // Sort by most recent entry in each book
                let a_latest = a.1.1.iter().map(|j| &j.created_at.0).max();
                let b_latest = b.1.1.iter().map(|j| &j.created_at.0).max();
                b_latest.cmp(&a_latest)
            });
            
            for (idx, (_book_id, (book_title, entries))) in sorted_books.iter().enumerate() {
                println!("\n{}. {} ({} entries)", idx + 1, book_title, entries.len());
                
                // Sort entries by date (most recent first)
                let mut sorted_entries = entries.clone();
                sorted_entries.sort_by(|a, b| b.created_at.0.cmp(&a.created_at.0));
                
                // Show entry count breakdown by event type
                let event_counts: HashMap<String, usize> = sorted_entries.iter()
                    .filter_map(|e| e.event.as_ref())
                    .fold(HashMap::new(), |mut acc, event| {
                        *acc.entry(event.clone()).or_insert(0) += 1;
                        acc
                    });
                
                if !event_counts.is_empty() {
                    let mut events: Vec<_> = event_counts.iter().collect();
                    events.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
                    let event_summary: Vec<String> = events.iter()
                        .map(|(event, count)| format!("{}: {}", event, count))
                        .collect();
                    println!("   Events: {}", event_summary.join(", "));
                }
                
                // Show date range
                if let (Some(first), Some(last)) = (sorted_entries.last(), sorted_entries.first()) {
                    if sorted_entries.len() > 1 {
                        println!("   Date range: {} to {}", first.created_at.0, last.created_at.0);
                    } else {
                        println!("   Date: {}", last.created_at.0);
                    }
                }
                
                // Show all entries for this book (with limit for readability)
                let display_limit = if sorted_entries.len() > 5 { 5 } else { sorted_entries.len() };
                println!("   Recent entries:");
                
                for (entry_idx, entry) in sorted_entries.iter().take(display_limit).enumerate() {
                    println!("     {}. {} - {}", 
                             entry_idx + 1, 
                             entry.created_at.0,
                             entry.event.as_deref().unwrap_or("No event"));
                    
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
                    println!("     ... and {} more entries", sorted_entries.len() - display_limit);
                }
            }
            
            // Summary statistics
            println!("\n=== Journal Summary ===");
            let total_entries = all_journal_entries.len();
            let books_with_entries = books_with_journals.len();
            
            // Overall event statistics
            let all_event_counts: HashMap<String, usize> = all_journal_entries.iter()
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
            let entries_with_text: Vec<_> = all_journal_entries.iter()
                .filter(|j| j.entry.as_ref().map_or(false, |e| !e.trim().is_empty()))
                .collect();
            
            println!("\nEntries with text content: {} out of {}", 
                     entries_with_text.len(), 
                     total_entries);
            
            if !entries_with_text.is_empty() {
                println!("\n=== Recent Entries with Text Content ===");
                for (idx, journal) in entries_with_text.iter().take(5).enumerate() {
                    let book_title = journal.book.as_ref()
                        .and_then(|b| b.title.as_deref())
                        .unwrap_or("Unknown Book");
                    
                    println!("\n{}. {} ({})", idx + 1, book_title, journal.created_at.0);
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
                    println!("\n   ... and {} more entries with text content", 
                             entries_with_text.len() - 5);
                }
            }
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
            
            println!("Pagination Query (offset: {}):\n{}\n", offset, operation.query);
            
            // Verify pagination parameters are included in the query structure
            assert!(operation.query.contains("offset: $offset"));
            assert!(operation.query.contains("limit: $limit"));
            assert!(operation.query.contains("$offset: Int!"));
            
            // Verify the variables are set correctly
            println!("Variables: user_id={}, limit={}, offset={}", 12345, 100, offset);
        }
    }
}
