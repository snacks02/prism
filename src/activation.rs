use {
    smol::{
        Async,
        io::{
            AsyncBufReadExt,
            BufReader,
        },
        stream,
        stream::{
            Stream,
            StreamExt,
        },
    },
    std::{
        env,
        fs,
        io::Write,
        os::unix::net::{
            UnixListener,
            UnixStream,
        },
        path::PathBuf,
    },
};

fn socket_path() -> PathBuf {
    env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir())
        .join("prism.sock")
}

pub fn forward() -> bool {
    let mut arguments = env::args().skip(1).peekable();
    if arguments.peek().is_none() {
        return false;
    }
    let Ok(mut stream) = UnixStream::connect(socket_path()) else {
        return false;
    };
    for argument in arguments {
        let _ = writeln!(stream, "{argument}");
    }
    true
}

pub fn incoming() -> impl Stream<Item = Vec<PathBuf>> {
    let socket_path = socket_path();
    let _ = fs::remove_file(&socket_path);
    stream::unfold(
        UnixListener::bind(&socket_path).and_then(Async::new).ok(),
        |unix_listener| async move {
            let unix_listener = unix_listener?;
            let (unix_stream, _) = unix_listener.accept().await.unwrap();
            Some((
                BufReader::new(unix_stream)
                    .lines()
                    .filter_map(Result::ok)
                    .map(PathBuf::from)
                    .collect()
                    .await,
                Some(unix_listener),
            ))
        },
    )
}
