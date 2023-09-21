use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}
// 연결하기
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512]; // 크기가 512이고 모든 요소가 0으로 초기화
    stream.read(&mut buffer).unwrap();

    // &[u8]을 전달받고 String으로 바꿔서 제공. lossy의 의미는 유효하지 않은 UTF-8 배열을 만났을 때의 행동을 나타냄.
    // 유효하지 않은 배열은 U+FFFD REPLACEMENT CHARACTER 로 교체
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // 응답하기
    // let response = "HTTP/1.1 200 OK\r\n\r\n";

    let get = b"GET / HTTP/1.1\r\n";

    // GET 요청 처리하기
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let mut file = File::open(filename).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap(); // 파일에서 내용을 읽어 문자열로 변환 후 contents 변수에 저장.

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap(); // 응답메시지를 저장할 response 변수 선언. 그리고 as_bytes를 호출해 문자열 데이터를 바이트 배열로 변환.
    stream.flush().unwrap(); // flush를 통해 버퍼를 비운다.
}
