use std::path::Path;
extern crate ffmpeg_the_third as ffmpeg;

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

        println!("Testing file: {:?}", file);
        assert_eq!(format_name, file_format_name);
    }
}
