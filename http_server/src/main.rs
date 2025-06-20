// use std::convert::Infallible;
// use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;
use std::{
    env,
    // io::{self, Write},
    // fs::{self, File},
};

// use http_body_util::Full;
// use hyper::body::{/*Body,*/ Bytes};
// use hyper::client::conn::http2::SendRequest;
use hyper::server::conn::http1;
use hyper::service::service_fn;
// use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
// use regex::Regex;
// use dotenv::dotenv;
// use dotenvy;
// use tokio_rustls::TlsAcceptor;
// use tokio_rustls::rustls::{self, ServerConfig};
// use tokio_rustls::server::TlsStream;

mod handlers;
use handlers::handler;


// async fn readfile(path: &str) -> Result<Vec<u8>, std::io::Error> {
//     let mut file = File::open(path)?;
//     let info = file.metadata()?;
//     let mut buffer = vec![0; info.len() as usize];
//     file.read_exact(&mut buffer)?;
//     Ok(buffer)
// }


// const path_cleanup1: Regex = Regex::new(r"/+").unwrap();
// const path_cleanup2: Regex = Regex::new(r"/$").unwrap();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // let dotenv_successfull = dotenv().ok();
    let dotenvy_successfull = dotenvy::from_path(Path::new(".env"));

    let mut serve_dir: String = env::var("serve_dir").unwrap_or("./public".to_string());
    let mut port: u16 = env::var("port").unwrap_or("3000".to_string()).parse().unwrap_or(3000);
    let mut host: [u8; 4] = { 
        let mut host: [u8; 4] = [0, 0, 0, 0];
        let host_str = env::var("host").unwrap_or("0.0.0.0".to_string());
        let parts: Vec<&str> = host_str.split('.').collect();
        if parts.len() == 4 {
            for (i, part) in parts.iter().enumerate() {
                host[i] = part.parse::<u8>().unwrap_or(0);
            }
        }

        host
    };
    let args: Vec<String> = env::args().collect();

    // Check arguments that the user provided
    if args.len() > 1 {
        serve_dir = args[1].clone();
    } if args.len() > 2 {
        port = args[2].parse::<u16>().unwrap_or(3000);
    } if args.len() > 3 {
        let host_str = args[3].clone();
        let parts: Vec<&str> = host_str.split('.').collect();
        if parts.len() == 4 {
            for (i, part) in parts.iter().enumerate() {
                host[i] = part.parse::<u8>().unwrap_or(0);
            }
        } else {
            eprintln!("Invalid host address. Using default");
        }
    }

    // dbg!(dotenv_successfull);
    dbg!(dotenvy_successfull)?;
    // dbg!(env::var("serve_dir"));


    println!(
        "Parameters of the server are\n\x1b[32mport = {}\n\x1b[33mhost = {:?}\n\x1b[34mdirectory = {}\x1b[0m",
        port, host, serve_dir
    );


    let addr = SocketAddr::from((host, port));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let serve_dir = serve_dir.clone();
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(async|req|handler(&serve_dir, req).await))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
