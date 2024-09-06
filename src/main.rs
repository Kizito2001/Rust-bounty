use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use urlencoding::encode;

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer);
    let request_line = request.lines().next().unwrap_or("");
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    // Strip query parameters if they exist
    let path = path.split('?').next().unwrap_or("/");

    // Join the path to the current directory
    let resource_path = PathBuf::from(".").join(&path.trim_start_matches('/'));

    println!("Resolved path: {:?}", resource_path);

    if resource_path.is_dir() {
        println!("Serving directory: {:?}", resource_path);
        let mut html = String::new();
        html.push_str("<html><body><ul>");

        for entry in fs::read_dir(&resource_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            let encoded_name = encode(file_name_str.as_ref());
            let display_name = file_name_str.clone();
            println!("Found entry: {}", display_name);
            html.push_str(&format!("<li><a href=\"/{}/\">{}</a></li>", encoded_name, display_name));
        }

        html.push_str("</ul></body></html>");
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n")?;
        stream.write_all(html.as_bytes())?;
    } else if resource_path.is_file() {
        println!("Serving file: {:?}", resource_path);
        let mut file = fs::File::open(&resource_path)?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;

        let mime_type = infer::get(&contents).map_or("text/plain", |r| r.mime_type());

        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: ")?;
        stream.write_all(mime_type.as_bytes())?;
        stream.write_all(b"\r\n\r\n")?;
        stream.write_all(&contents)?;
    } else {
        println!("404 Not Found for path: {:?}", resource_path);
        stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\n\r\n")?;
        stream.write_all(b"<html><body><h1>404 Not Found</h1></body></html>")?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    
    // Print the link to the terminal
    println!("Server is running on http://0.0.0.0:8080");
    println!("If using Google Cloud Shell, access your server via the web preview link:");
    println!("https://8080-dot-your-cloudshell-url-abcde.googleusercontent.com/");
    
    for stream in listener.incoming() {
        let stream = stream?;
        std::thread::spawn(move || {
            if let Err(e) = handle_client(stream) {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }

    Ok(())
}
