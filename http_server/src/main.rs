use std::convert::Infallible;
use std::io::Read;
use std::net::SocketAddr;
use std::{
    env,
    // io::{self, Write},
    fs::{self, File},
};

use http_body_util::Full;
use hyper::body::{Body, Bytes};
use hyper::client::conn::http2::SendRequest;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use regex::Regex;
// use dotenv::dotenv;
use std::path::Path;
// use dotenvy;


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


async fn handler(serve_dir: &str, req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("Serving connection");

    let path_cleanup1: Regex = Regex::new(r"/+").unwrap();
    let path_cleanup2: Regex = Regex::new(r"/$").unwrap();
    let path_cleanup3: Regex = Regex::new(r"\?*.").unwrap();

    let full_path: String = { 
        let full_path: String = req.uri().to_string();
        let full_path   =full_path.replace("\\", "/");
        let full_path = path_cleanup1.replace_all(&full_path, "/");
        let full_path = if full_path.len() > 1 {
            path_cleanup2.replace(&full_path, "")
        } else {
            full_path
        };
        let full_path = if full_path.len() > 1 { path_cleanup3.replace_all(&full_path, "") } else { full_path };

        serve_dir.to_owned() + &full_path.to_string()
    };
    println!("Full path: {}", full_path);

    
    
    let info_res = fs::metadata(&full_path);
    match info_res{
        Ok(info) => {
            if info.is_file() {
                file_handler(&full_path).await
                // let mut file = File::open(&full_path).unwrap();
                // let mut buffer = vec![0; info.len() as usize];
                // file.read_exact(&mut buffer).unwrap();
                // Ok(Response::new(Full::new(Bytes::from(buffer))))
            } else if info.is_dir(){
                dir_handler(req, &full_path).await
                // let dir: fs::ReadDir = fs::read_dir(&full_path).unwrap();
                // let mut file: String = "".to_string();
                //
                // for entry in dir{
                //     let entry = entry.unwrap();
                //     let file_name = entry.file_name().into_string().unwrap();
                //     if file_name.starts_with("index.") || file_name == "index" {
                //         file = full_path.to_owned() + "/" + &file_name;
                //         break;
                //     }
                // }
                //
                // if file==""{
                //     error_handler(409, std::io::Error::new(std::io::ErrorKind::IsADirectory,"Cannot find index file in directory"), req).await
                // } else {
                //     file_handler(&file).await
                // }
            } else {
                error_handler(409, std::io::Error::new(std::io::ErrorKind::Unsupported, "File is unusable"), req).await
            }
        },
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                error_handler(404, err, req).await
            } else {
                error_handler(500, err, req).await
            }
        },
    }

    // Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

async fn error_handler(code: u16, err: std::io::Error, req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible>{
    eprintln!("Error of status {} occoured\n\x1b[31m{}\x1b[0m",code,err);
    match code {
        404 => {
            println!("404 Not Found: {}", req.uri());
            Ok(Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("404 Not Found")))
                .unwrap())
        },
        409 => {
            println!("409 Conflict: {}", req.uri());
            Ok(Response::builder()
                .status(409)
                .body(Full::new(Bytes::from("409 Conflict")))
                .unwrap())
        },
        500 => {
            println!("500 Internal Server Error: {}", req.uri());
            Ok(Response::builder()
                .status(500)
                .body(Full::new(Bytes::from("500 Internal Server Error")))
                .unwrap())
        },
        _ => {
            println!("{}: {}", code, req.uri());
            Ok(Response::builder()
                .status(code)
                .body(Full::new(Bytes::from(format!("{}: {}", code, err))))
                .unwrap())
        }
        
    }
}

async fn file_handler(path: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    let mut res=Response::new(Full::new(Bytes::from(buffer)));
    if path.ends_with(".html"){
        res.headers_mut().insert("Content-Type", "text/html".parse().unwrap());
    } else if path.ends_with(".css") {
        res.headers_mut().insert("Content-Type", "text/css".parse().unwrap());
    } else if path.ends_with(".js") {
        res.headers_mut().insert("Content-Type", "application/javascript".parse().unwrap());
    } else if path.ends_with(".json") {
        res.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
    } else if path.ends_with(".png") {
        res.headers_mut().insert("Content-Type", "image/png".parse().unwrap());
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        res.headers_mut().insert("Content-Type", "image/jpeg".parse().unwrap());
    } else if path.ends_with(".gif") {
        res.headers_mut().insert("Content-Type", "image/gif".parse().unwrap());
    } else {
        res.headers_mut().insert("Content-Type", "application/octet-stream".parse().unwrap());
    }
    Ok(res)
}

async fn dir_handler(req: Request<hyper::body::Incoming>,path: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let dir: fs::ReadDir = fs::read_dir(&path).unwrap();
    let mut file: String = "".to_string();

    println!("Path is dir {}", path);

    for entry in dir{
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        let file_parts: Vec<&str> = (&path).to_owned().split('/').collect();
        let last_dir = file_parts.last().unwrap_or(&".");
        let meta = entry.metadata().unwrap();
        if !meta.is_file() {
            continue; // Mitigate dirs treated as files
        }

        println!("Directory entry {}\n{:?}",file_name,entry);

        if file_name.starts_with("index.") || file_name == "index" || file_name.starts_with(last_dir) {
            // file = path.to_owned() + "/" + &file_name;
            file = entry.path().to_string_lossy().to_string();
            break;
        }
    }

    println!("File found {}", file);

    if fs::metadata(&file).unwrap().is_file(){
        file_handler(&file).await
    } else {
        error_handler(409, std::io::Error::new(std::io::ErrorKind::IsADirectory,"Cannot find index file in directory"), req).await
    }
}


