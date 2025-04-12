use std::fs::File;
use std::io::Read;
use std::thread;

use anyhow::Result;
use yaml_rust::{Yaml, YamlLoader};

mod gen_site;
mod serve;
mod watcher;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "init", alias = "initialize")]
    Init {
        #[arg()]
        title: Option<String>,
    },
    Generate,
    #[command(name = "serve", alias = "serve")]
    Serve {
        #[arg(long)]
        watch: bool,
    },
}

#[derive(Debug, Clone)]
struct Post {
    title: String,
    date: String,
    path: String,
}

#[derive(Debug, Clone)]
struct Config {
    posts: Vec<Post>,
}

fn parse_config(yaml_doc: &Yaml) -> Config {
    let mut posts = Vec::new();
    let posts_node = &yaml_doc["posts"];
    for post in posts_node.as_vec().unwrap() {
        let title = post["title"].as_str().unwrap().to_string();
        let date = post["date"].as_str().unwrap().to_string();
        let path = post["path"].as_str().unwrap().to_string();
        posts.push(Post { title, date, path });
    }
    Config { posts }
}

fn gen() -> Result<()> {
    let mut config_file = File::open("PUSHPIN.yaml")?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;

    let yaml_loader = YamlLoader::load_from_str(&contents).unwrap();
    let yaml_doc = &yaml_loader[0];
    let config = parse_config(yaml_doc);

    match gen_site::generate(&config) {
        Ok(num_files_generated) => {
            println!("📌 success: generated site; created {num_files_generated} files")
        }
        Err(err) => println!("Error: {err}"),
    };

    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { title } => match gen_site::initialize(title.clone()) {
            Ok(_) => {
                if let Some(t) = title {
                    println!("📌 success: initialized new site '{t}'");
                } else {
                    println!("📌 success: initialized new site");
                }
            }
            Err(err) => println!("Error: {err}"),
        },
        Commands::Generate => {
            let _ = gen();
        }
        Commands::Serve { watch } => {
            let _ = gen();

            if !(*watch) {
                println!(
                    "📌 local server available at http://127.0.0.1:7878 (type Ctrl+C to stop)"
                );
                serve::serve();
                return Ok(());
            }

            println!("📌 local server available at http://127.0.0.1:7878");
            let server_handle = thread::spawn(|| {
                serve::serve();
            });

            println!("🔍 Watching for changes in 'pages/', 'templates/', (type Ctrl+C to stop):");
            let pages_watcher_handle = watcher::start_file_watcher(
                "pages",
                |_| {
                    let _ = gen();
                },
                true,
            );
            let watcher_handle = watcher::start_file_watcher(
                "templates",
                |_| {
                    let _ = gen();
                },
                true,
            );

            if let Err(e) = pages_watcher_handle.join() {
                eprintln!("😥 internal error: {:?}. Please file a bug at https://github.com/tedbauer/pushpin/issues.", e);
            }

            if let Err(e) = watcher_handle.join() {
                eprintln!("😥 internal error: {:?}. Please file a bug at https://github.com/tedbauer/pushpin/issues.", e);
            }

            // Join the server handle
            if let Err(e) = server_handle.join() {
                eprintln!("😥 internal error: {:?}. Please file a bug at https://github.com/tedbauer/pushpin/issues.", e);
            }
        }
    }
    Ok(())
}
