use std::fs::File;
use clap::Parser;
use ffmpeg_next::{Error, format, Packet};
use ffmpeg_next::format::context::Input;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type;
use ffmpeg_next::rescale::TIME_BASE;
use ffmpeg_next::software::scaling::{Context, Flags};
use image::{ImageEncoder, Rgb, RgbImage};
use image::codecs::jpeg::JpegEncoder;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    quality: u8,

    #[arg(short, long)]
    time: u32,

    #[arg(short, long)]
    size: u32,

    #[arg(short, long)]
    codec: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let i_temp = args.input;
    let o_temp = args.output;
    println!("input: {:?} {:?}", i_temp, o_temp);

    ffmpeg_next::init()?;
    let mut ictx = format::input(&i_temp)?;
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(Error::StreamNotFound)?;
    let stream_index = input.index();

    if input.duration() == 0 {
        eprintln!("Error: The duration of the input file is 0.");
        return Err(Error::InvalidData);
    }

    println!("duration: {:?}", input.duration());

    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let scalar = create_scalar(args.size, &mut decoder)?;

    let seek_percent = args.time as f64 * 0.01;
    seek_to_position(&mut ictx, seek_percent)?;

    let rgb_frame = get_frame(ictx, stream_index, decoder, scalar)?;

    if rgb_frame != Video::empty() {
        if let Err(e) = write_frame_to_jpeg(&rgb_frame, &o_temp, args.quality) {
            eprintln!("Error writing file {:?}", e);
        }
    } else {
        eprintln!("Could not find frame");
    }

    Ok(())
}

fn seek_to_position(ictx: &mut Input, seek_percent: f64) -> Result<(), Error> {
    let seek_pos_in_seconds = ((ictx.duration() as f64 * seek_percent) / f64::from(TIME_BASE.denominator())) as i32;
    let seek_pos = seek_pos_in_seconds * TIME_BASE.denominator();
    ictx.seek(seek_pos as i64, ..seek_pos as i64)?;
    Ok(())
}

fn create_scalar(size: u32, decoder: &mut ffmpeg_next::codec::decoder::video::Video) -> Result<Context, Error> {
    let w = decoder.width();
    let p = size as f32 / w as f32;
    let h = decoder.height();
    let new_h = (h as f32 * p) as u32;
    let scalar = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        size,
        new_h,
        Flags::BILINEAR,
    )?;
    Ok(scalar)
}

fn get_frame(mut ictx: Input,
             stream_index: usize,
             mut decoder: ffmpeg_next::codec::decoder::video::Video,
             mut scaler: Context) -> Result<Video, Error> {
    let mut rgb_frame = Video::empty();
    for (stream, packet) in ictx.packets() {
        if stream.index() == stream_index {
            if let Ok(frame_decoded) = process_packet(&mut decoder, &mut scaler, &packet, &mut rgb_frame) {
                if frame_decoded {
                    break;
                }
            }
        }
    }

    Ok(rgb_frame)
}

fn write_frame_to_jpeg(frame: &Video, filename: &str, quality: u8) -> Result<(), Box<dyn std::error::Error>> {
    if frame.format() != Pixel::RGB24 {
        return Err("Frame format is not RGB24".into());
    }

    let width = frame.width();
    let height = frame.height();

    let mut img_buffer = RgbImage::new(width, height);

    let data = frame.data(0);
    let line_size = frame.stride(0);

    for y in 0..height {
        for x in 0..width {
            let offset = (y * line_size as u32 + x * 3) as usize;
            let rgb = Rgb([data[offset], data[offset + 1], data[offset + 2]]);
            img_buffer.put_pixel(x, y, rgb);
        }
    }

    let file = File::create(filename)?;
    let encoder = JpegEncoder::new_with_quality(file, quality);
    encoder.write_image(&img_buffer, width, height, image::ExtendedColorType::Rgb8)?;

    Ok(())
}

fn process_packet(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut Context,
    packet: &Packet,
    rgb_frame: &mut Video,
) -> Result<bool, Error> {
    decoder.send_packet(packet)?;
    let mut decoded = Video::empty();
    match decoder.receive_frame(&mut decoded) {
        Ok(_) => {
            scaler.run(&decoded, rgb_frame)?;
            Ok(true)
        }
        Err(err) => {
            eprintln!("Failed to receive frame: {:?}", err);
            Ok(false)
        },
    }
}
