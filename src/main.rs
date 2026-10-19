use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!("프록시가 127.0.0.1:8080 에서 연결을 기다립니다.");

    for connection in listener.incoming() {
        let mut stream = connection?;
        println!("새 연결: {}", stream.peer_addr()?);

        let mut buffer = [0u8; 4096];
        let bytes_read = stream.read(&mut buffer)?;

        println!(
            "받은 요청:\n{}",
            String::from_utf8_lossy(&buffer[..bytes_read])
        );

        let request_text = String::from_utf8_lossy(&buffer[..bytes_read]);

        if let Some(request_line) = request_text.lines().next() {
            let mut parts = request_line.split_whitespace();

            let method = parts.next();
            let target = parts.next();
            let version = parts.next();

            match (method, target, version) {
                (Some(method), Some(target), Some(version)) => {
                    match target.strip_prefix("http://") {
                        Some(destination) => {
                            let (authority, path) = match destination.split_once('/') {
                                Some((authority, path)) => (authority, path),
                                None => (destination, ""),
                            };
                            println!("서버 주소 부분: {}", authority);
                            println!("경로: /{}", path);

                            match TcpStream::connect((authority, 80)) {
                                Ok(mut upstream) => {
                                    println!("외부 서버 연결 성공: {}", upstream.peer_addr()?);
                                    let upstream_request = format!("GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", path, authority);
                                    upstream.write_all(upstream_request.as_bytes())?;
                                    println!("외부 서버에 요청을 보냈습니다.");

                                    let copied = std::io::copy(&mut upstream, &mut stream)?;
                                    println!("클라이언트에 전달한 응답: {}바이트", copied);
                                }
                                Err(error) => {
                                    println!("외부 서버 연결 실패: {}", error)
                                }
                            }
                        }
                        None => {
                            println!("현재는 http:// 요청만 처리합니다.")
                        }
                    }
                    
                    println!("메서드: {}", method);
                    println!("HTTP 버전: {}", version);
                }
                _ => {
                    println!("요청줄에 필요한 항목이 부족합니다.");
                }
            }
        }
    }

    Ok(())
}
