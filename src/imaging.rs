use crate::dal::DbConnection;
use crate::{dal, Error};
use actix_web::web::Bytes;
use diesel::r2d2::ConnectionManager;
use diesel::Connection;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageReader};
use log::info;
use r2d2::PooledConnection;
use std::fs::OpenOptions;
use std::io::Cursor;

pub fn handle_upload_member_picture(
    conn: &mut PooledConnection<ConnectionManager<DbConnection>>,
    member_id: &i32,
    data: &Bytes,
) -> crate::Result<String> {
    let result = conn.transaction::<String, Error, _>(|conn| {
        let member = dal::members::get_member_by_id(conn, &member_id)?;
        // Mark the already existing picture for deletion, if it exists
        let mark_for_deletion = member.picture_asset_id.clone();

        let dynamic_image = load_alien_member_picture(&data)?;

        // Create a new asset identifier
        let asset_id = crate::generate_asset_id();
        let pb = crate::path_for_asset_id(&asset_id)?;
        let w = OpenOptions::new().write(true).create_new(true).open(&pb)?;
        dynamic_image.write_with_encoder(PngEncoder::new(w))?;

        dal::members::store_member_picture_asset_id(conn, &member.id, &asset_id)?;
        info!(
            "Stored asset into: {} for member: {member_id}",
            pb.to_string_lossy()
        );

        if let Some(asset_id) = mark_for_deletion {
            let pb = crate::path_for_asset_id(&asset_id)?;
            let _ = std::fs::remove_file(pb); // Ignore if this failed
        }

        Ok(asset_id)
    })?;
    Ok(result)
}

fn load_alien_member_picture(data: &Bytes) -> crate::Result<DynamicImage> {
    let reader = ImageReader::new(Cursor::new(&data)).with_guessed_format()?;
    let dynamic_image = reader.decode()?;

    // Create passport size image of 3.5 x 4.5 cm
    let dynamic_image = crop_as_passport_image(dynamic_image);
    let dynamic_image = resize_passport_image(dynamic_image);
    Ok(dynamic_image)
}

fn crop_as_passport_image(dynamic_image: DynamicImage) -> DynamicImage {
    let mut dynamic_image = dynamic_image;
    let width = dynamic_image.width() as f64;
    let height = dynamic_image.height() as f64;

    let width_ratio_to_height = width / 3.5 * 4.5;

    if width_ratio_to_height > height {
        // Image must be cropped width-wise
        let new_width = height * 4.5 / 3.5;
        dynamic_image = dynamic_image.crop(
            ((width - new_width) / 2.0) as u32,
            0,
            new_width as u32,
            height as u32,
        );
    } else if width_ratio_to_height < width {
        // Image must be cropped height wise
        let new_height = width / 4.5 * 3.5;
        dynamic_image = dynamic_image.crop(
            0,
            ((height - new_height) / 2.0) as u32,
            width as u32,
            new_height as u32,
        );
    }

    dynamic_image
}

// Reinterpret image @ 300 dpi => 413 dots x 531 dots for a 3.5 x 4.5 cm passport image
fn resize_passport_image(dynamic_image: DynamicImage) -> DynamicImage {
    let dynamic_image = dynamic_image.resize(413, 531, FilterType::Triangle);
    dynamic_image
}
