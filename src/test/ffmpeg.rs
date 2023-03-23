use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

extern crate ffmpeg_the_third as ffmpeg;

#[test]
fn test_blake3() {
    let f = File::open("src/test/static/text.jpg").unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer).unwrap();

    let mut hasher = blake3::Hasher::new();
    hasher.update(buffer.as_slice());

    let hash = hasher.finalize();
    assert_eq!(
        hash.to_string(),
        "8770325564caf30faf3fcfc0ff5be1a6e06afb405b8d1ce647b7dd4a694f6754".to_string()
    );
}

#[test]
fn test_ffmpeg() {
    ffmpeg::init().unwrap();

    let vec = vec![
        (Path::new("src/test/static/text.gif"), "gif"),
        (Path::new("src/test/static/text.jpg"), "image2"),
        (Path::new("src/test/static/text.png"), "png_pipe"),
        (Path::new("src/test/static/text.webp"), "webp_pipe"),
    ];

    for (file, format_name) in vec {
        let file_format_name = ffmpeg::format::input(&file.to_owned())
            .expect("No file?")
            .format()
            .name()
            .to_owned();

        assert_eq!(format_name, file_format_name);
    }
}

#[test]
fn test_infer() {
    let vec = vec![
        (
            infer::get_from_path("src/test/static/text.gif"),
            "gif",
            "image/gif",
        ),
        (
            infer::get_from_path("src/test/static/text.jpg"),
            "jpg",
            "image/jpeg",
        ),
        (
            infer::get_from_path("src/test/static/text.png"),
            "png",
            "image/png",
        ),
        (
            infer::get_from_path("src/test/static/text.webp"),
            "webp",
            "image/webp",
        ),
    ];

    for (file, ext, mime) in vec {
        let res = file.unwrap().unwrap();
        assert_eq!(res.mime_type(), mime);
        assert_eq!(res.extension(), ext);
    }
}
